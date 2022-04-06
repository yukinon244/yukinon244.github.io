package main

import "fmt"
import 	"os/exec"
import "bytes"
import "net/http"

// 这里是 worker，我们将并发执行多个 worker。
// worker 将从 `jobs` 通道接收任务，并且通过 `results` 发送对应的结果。
// 我们将让每个任务间隔 1s 来模仿一个耗时的任务
func worker(id int, jobs <-chan int, results chan<- int) {
    for j := range jobs {
        fmt.Println("worker", id, "processing job", j)
	    r, err := http.Get("https://api.github.com/events")
    	if err != nil {
        	panic(err)
    	}
    	defer func() { _ = r.Body.Close() }()
		if r.StatusCode >= 400 {
			return
		}
		fmt.Println(r.StatusCode)
var out bytes.Buffer
var stderr bytes.Buffer
	var url_str = fmt.Sprintf("https://icourse.club/course/%d/", j)
	cmd := exec.Command("/usr/bin/wget", "--mirror", "--page-requisites", "--adjust-extension", "--no-parent", "--convert-links", "--tries=0", url_str)
    	
cmd.Stdout = &out
cmd.Stderr = &stderr
	cmd.Run()
	    fmt.Println("Result: " + out.String())
	fmt.Println("Error: " + stderr.String())
	results <- j * 2
    }
}

func main() {

    // 为了使用 worker 线程池并且收集他们的结果，我们需要 2 个通道。
    jobs := make(chan int, 50000)
    results := make(chan int, 50000)
	

    // 这里启动了 3 个 worker，初始是阻塞的，因为还没有传递任务。
    for w := 1; w <= 100; w++ {
        go worker(w, jobs, results)
    }

    // 这里我们发送 9 个 `jobs`，然后 `close` 这些通道
    // 来表示这些就是所有的任务了。
    for j := 1; j <= 50000; j++ {
        jobs <- j
    }
    close(jobs)

    // 最后，我们收集所有这些任务的返回值。
    for a := 1; a <= 50000; a++ {
        <-results
    }
}
