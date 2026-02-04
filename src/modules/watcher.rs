use anyhow::Result;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::UnixStream;

pub async fn run_watcher(socket_path: &Path) -> Result<()> {
    // Retry connection loop
    loop {
        // Connection loop
        let mut stream = loop {
            match UnixStream::connect(socket_path).await {
                Ok(s) => break s,
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
            }
        };

        let mut buf = [0u8; 4096];
        loop {
            // Read loop
            match stream.read(&mut buf).await {
                Ok(0) => {
                    // EOF - Server restart?
                    break;
                }
                Ok(n) => {
                    let s = String::from_utf8_lossy(&buf[..n]);
                    print!("{}", s);
                    io::stdout().flush().ok();
                }
                Err(_) => {
                    break;
                }
            }
        }

        // Wait before reconnecting
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
