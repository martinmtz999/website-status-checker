# website-status-checker

This is a RUST command-line program to check the status of many websites in parallel. 

Includes threads, channels, error handling, JSON file output, and CLI parsing, only using one third‑party crate a HTTP client  reqwest with the blocking feature.

Build:
1. Must have Rust ≥ 1.78 installed
2. Clone this repo and enter to the directory:
   cd website‑status‑checker
3. Build in release mode:
  cargo build --release
4. The binary will be at target/release/website_checker.

Usage:
From the project root with no arguments:
  cargo run --release
This prints usage and exits with a code 2

Check URLs from a file:
  cargo run --release -- --file sites.txt

Check positional URLs:
  cargo run --release -- https://example.com https://does-not-exist.invalid

Override defaults:
  cargo run --release -- \
    --file sites.txt \
    --workers 8 \
    --timeout 2 \
    --retries 3

Flags:
--file <path>
  Text file with one URL per line (ignores blank lines and # comments)
--workers N
  Number of threads (default = logical CPU cores)
--timeout S
  Per‑request timeout in seconds (default = 5)
--retries N
  Number of extra attempts on failure (default = 0, 100 ms pause between)

Expected output with previous positional URLs:
  Found 2 URLs
  [200] https://example.com - 71ms
  [ERR] https://does-not-exist.invalid - error sending request for url (https://does-not-exist.invalid/)

Expected JSON file output with previous positional URLs:
  [
    { "url": "https://example.com", "status": 200, "time_ms": 71, "timestamp": 1746823581 },
    { "url": "https://does-not-exist.invalid", "status": "error sending request for url (https://does-not-exist.invalid/)", "time_ms": 128, "timestamp": 1746823581 }
  ]






 
