use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::task;
use reqwest::Client;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    url: String,
    #[arg(short, long, default_value_t = 10)]
    duration: u64, // duration in seconds
    #[arg(short, long, default_value_t = 4)]
    threads: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = Client::new();
    let total_requests = Arc::new(Mutex::new(0));
    let latencies = Arc::new(Mutex::new(vec![]));

    let start = Instant::now();
    let mut handles = vec![];

    for _ in 0..args.threads {
        let client = client.clone();
        let url = args.url.clone();
        let total_requests = Arc::clone(&total_requests);
        let latencies = Arc::clone(&latencies);

        handles.push(task::spawn(async move {
            while start.elapsed().as_secs() < args.duration {
                let req_start = Instant::now();
                let resp = client.get(&url).send().await.unwrap();
                resp.text().await.unwrap();
                let req_duration = req_start.elapsed();

                // Update request count and latency
                *total_requests.lock().unwrap() += 1;
                latencies.lock().unwrap().push(req_duration);
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Calculate statistics
    let total_requests = *total_requests.lock().unwrap();
    let total_time = start.elapsed();
    let reqs_per_sec = total_requests as f64 / total_time.as_secs_f64();

    let latencies = latencies.lock().unwrap();
    let avg_latency: Duration = latencies.iter().sum::<Duration>() / total_requests as u32;

    println!("Total requests: {}", total_requests);
    println!("Requests per second: {:.2}", reqs_per_sec);
    println!("Average latency per request: {:?}", avg_latency);
}
