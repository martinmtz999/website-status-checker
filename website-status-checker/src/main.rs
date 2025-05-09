use std::env;
use std::fs;
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use std::sync::{mpsc, Arc, Mutex};


struct Opts {
    file:Option<String>,
    urls: Vec<String>,
    workers:usize,
    timeout:u64,
    retries:u64,
}


struct Site {
    url:String,
    status:Result<u16,String>,
    time:Duration,
    at:SystemTime,
}

fn main() {
    let opts = get_opts();
    let list = load_urls(&opts);
    println!("Found {} URLs", list.len());
    let client = make_client(opts.timeout);
    let (job_tx, job_rx) = mpsc::channel::<String>();
    let job_rx = Arc::new(Mutex::new(job_rx));
    let (res_tx, res_rx) = mpsc::channel::<Site>();


    for _ in 0..opts.workers {
        let job_rx = Arc::clone(&job_rx);
        let res_tx = res_tx.clone();
        let cli = client.clone();
        let rep = opts.retries;
        
        thread::spawn(move || {
            loop {
                let url = match job_rx.lock().unwrap().recv() {
                    Ok(u) => u,
                    Err(_) => break,
                };
                let start = Instant::now();
                let mut st = Err("timeout".to_string());
                for _ in 0..=rep {
                    match cli.get(&url).send() {
                        Ok(r) => { st = Ok(r.status().as_u16()); break; }
                        Err(e) => { st = Err(e.to_string()); thread::sleep(Duration::from_millis(100)); }
                    }
                }
                let s = Site {
                    url: url.clone(),
                    status: st,
                    time: start.elapsed(),
                    at: SystemTime::now(),
                };
                res_tx.send(s).unwrap();
            }
        });
    }
    drop(res_tx);

    for u in list { job_tx.send(u).unwrap(); }
    drop(job_tx);


    let mut all = Vec::new();
    for site in res_rx {
        match &site.status {
            Ok(c) => println!("[{}] {} - {}ms", c, site.url, site.time.as_millis()),
            Err(e) => println!("[ERR] {} - {}", site.url, e),
        }
        all.push(site);
    }

    dump_json(all);
}

fn get_opts() -> Opts {
    let mut args = env::args().skip(1);
    let mut file = None;
    let mut urls = Vec::new();
    let mut workers = std::thread::available_parallelism()
        .map(|n| n.get()).unwrap_or(4);
    let mut timeout = 5;
    let mut retries = 0;

    while let Some(a) = args.next() {
        match a.as_str() {
            "--file"    => file    = args.next(),
            "--workers" => workers = args.next().unwrap().parse().unwrap_or(workers),
            "--timeout" => timeout = args.next().unwrap().parse().unwrap_or(timeout),
            "--retries" => retries = args.next().unwrap().parse().unwrap_or(retries),
            other       => urls.push(other.to_string()),
        }
    }

    if file.is_none() && urls.is_empty() {
        eprintln!("Usage: website_checker [--file path] [URL ...]");
        std::process::exit(2);
    }

    Opts { file, urls, workers, timeout, retries }
}

fn load_urls(o: &Opts) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(p) = &o.file {
        let data = fs::read_to_string(p).unwrap_or_default();
        for line in data.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') { continue; }
            v.push(t.to_string());
        }
    }
    v.extend(o.urls.clone());
    v
}

fn make_client(to_secs: u64) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(to_secs))
        .build()
        .unwrap()
}
fn dump_json(sites: Vec<Site>) {
    let mut out = String::from("[\n");
    for (i, s) in sites.iter().enumerate() {
        if i > 0 { out.push_str(",\n"); }
        let code = match s.status {
            Ok(c) => c.to_string(),
            Err(ref e) => format!("\"{}\"", e),
        };
        let ts = s.at.duration_since(SystemTime::UNIX_EPOCH)
                  .unwrap().as_secs();
        out.push_str(&format!(
            "  {{ \"url\": \"{}\", \"status\": {}, \"time_ms\": {}, \"timestamp\": {} }}",
            s.url, code, s.time.as_millis(), ts
        ));
    }
    out.push_str("\n]\n");
    fs::write("status.json", out).unwrap();
}
