use std::process::Command;
use std::time::Duration;
use anyhow::{Result, anyhow};

pub async fn fetch_cf_clearance() -> Result<String> {
    let script_path = std::path::PathBuf::from("/Users/twan/.cargo/bin/lucida-downloader/scripts/fetch-cf.js");
    
    let output = Command::new("node")
        .arg(&script_path)
        .arg("https://lucida.to/")
        .output()
        .map_err(|e| anyhow!("Failed to execute Node.js script: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Node.js script failed: {}", stderr));
    }
    
    let cookie = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();
    
    if cookie.is_empty() {
        return Err(anyhow!("No cf_clearance cookie returned"));
    }
    
    Ok(cookie)
}

pub async fn fetch_cf_clearance_with_retry() -> Result<String> {
    let mut delay = Duration::from_secs(5);
    let mut attempt = 1;
    
    loop {
        println!("Attempt {} to fetch cf-clearance from lucida.to...", attempt);
        
        match fetch_cf_clearance().await {
            Ok(cookie) => {
                println!("✓ Successfully fetched cf-clearance from lucida.to on attempt {}", attempt);
                return Ok(cookie);
            }
            Err(e) => {
                eprintln!("✗ Failed to fetch from lucida.to (attempt {}): {}", attempt, e);
                eprintln!("  Retrying in {} seconds (Ctrl+C to cancel)...", delay.as_secs());
                
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, Duration::from_secs(30));
                attempt += 1;
            }
        }
    }
}
