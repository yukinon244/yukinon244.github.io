#!/bin/python3

import os
import subprocess
import argparse
from multiprocessing import Process
from utils import convert_link, convert_image


def convert_links():
    for root, dirs, files in os.walk("./icourse.club/course"):
        for name in files:
            print("Convert links in {}".format(os.path.join(root, name)))
            convert_link(os.path.join(root, name))


def convert_images():
    for root, dirs, files in os.walk("./icourse.club/uploads/images"):
        for file in files:
            _, ext = os.path.splitext(file)
            if ext != ".webp":
                convert_image(root, file)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--min", help="min popular page to fetch", action="store")
    parser.add_argument(
        "--max", help="max popular page to fetch", action="store", required=True
    )
    parser.add_argument("-n", help="threads to use in download", action="store")
    parser.add_argument(
        "--convert-links",
        help="convert links in html file",
        action="store_const",
        const=True,
    )
    parser.add_argument(
        "--convert-images",
        help="convert images downloaded to webp",
        action="store_const",
        const=True,
    )
    args = parser.parse_args()

    if not os.path.exists("./icourse.club/uploads/images"):
        os.makedirs("./icourse.club/uploads/images")
    if not os.path.exists("./icourse.club/static/image"):
        os.makedirs("./icourse.club/static/image")
    if not os.path.exists("./icourse.club/course"):
        os.makedirs("./icourse.club/course")

    env = os.environ.copy()
    env["RUST_LOG"] = "info"
    subprocess.run(
        [
            "cargo",
            "run",
            "--release",
            "--",
            "--min={}".format(args.min if args.min is not None else 1),
            "--max={}".format(args.max),
            "-n={}".format(args.n if args.n is not None else 5),
        ],
        check=True,
        env=env,
    )

    handles = []

    if args.convert_links:
        p = Process(target=convert_links)
        p.start()
        handles.append(p)

    if args.convert_images:
        p = Process(target=convert_images)
        p.start()
        handles.append(p)

    for handle in handles:
        handle.join()
