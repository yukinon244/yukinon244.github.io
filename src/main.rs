use clap::Arg;
use futures::{stream, StreamExt};
use reqwest::Client;
use soup::prelude::*;
use std::{
    fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = clap::Command::new("mirrorsite")
        .arg(
            Arg::new("threads")
                .short('n')
                .default_value("5")
                .help("max threads used in download"),
        )
        .arg(
            Arg::new("max_page")
                .long("max")
                .required(true)
                .takes_value(true)
                .help("max popular page number to fetch"),
        )
        .arg(
            Arg::new("min_page")
                .long("min")
                .default_value("1")
                .help("min popular page number to fetch"),
        )
        .get_matches();
    let threads = args.value_of("threads").unwrap().parse::<usize>()?;
    let min_page = args.value_of("min_page").unwrap().parse::<usize>()?;
    let max_page = args.value_of("max_page").unwrap().parse::<usize>()?;
    if min_page > max_page {
        return Ok(());
    }

    let client = Client::new();
    stream::iter(min_page..=max_page)
        .map(|page| download(&client, page))
        .buffer_unordered(threads)
        .collect::<Vec<_>>()
        .await;

    Ok(())
}

async fn download(client: &Client, page: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("Fetch popular page {}", page);
    let request = client
        .get(format!(
            "https://icourse.club/course/popular/?page={}",
            page
        ))
        .build()?;
    let text = client.execute(request).await?.text().await?;
    //info!("finish download popular page {}", page);

    // get course url in popular page
    let courses: Vec<String> = Soup::new(text.as_str())
        .tag("a")
        .class("px16")
        .find_all()
        .map(|tag| tag.attrs().get("href").unwrap().clone())
        .collect();

    for course in courses {
        let num = course
            .strip_prefix("/course/")
            .unwrap()
            .strip_suffix("/")
            .unwrap();
        info!("Fetch course {}", num);
        let request = client
            .get(format!("https://icourse.club{}", course))
            .build()?;
        let text = client.execute(request).await?.bytes().await?;
        //info!("finish download course {}", num);

        fs::create_dir_all(format!("./icourse.club{}", course))?;
        let mut file = File::create(format!("./icourse.club{}index.html", course))?;
        file.write_all(&text)?;

        let images: Vec<String> = Soup::new(std::str::from_utf8(&text).unwrap())
            .tag("img")
            .find_all()
            .map(|img| img.attrs().get("src").unwrap().clone())
            .collect();
        for name in images {
            info!("Fetch image {}", name);
            let mut cache = PathBuf::from(name.clone());
            cache.set_extension("webp");
            let cache = cache.to_str().unwrap();
            if Path::new(format!("./icourse.club{}", cache).as_str()).exists() {
                info!("image already exist");
                continue;
            }
            let request = client
                .get(format!("https://icourse.club{}", name))
                .build()?;
            let image = client.execute(request).await?.bytes().await?;
            let mut file = File::create(format!("./icourse.club{}", name))?;
            file.write_all(&image)?;
        }
    }
    Ok(())
}
