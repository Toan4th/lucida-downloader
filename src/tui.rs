use std::path::{Path, PathBuf};
use std::process::Command;
use dialoguer::{Select, Input, Confirm};
use crate::config::{Config, save_config, display_config};

pub async fn run_tui(config: &mut Config) {
    println!("========================================");
    println!("       Lucida Music Downloader          ");
    println!("========================================");

    loop {
        let options = &[
            "1. Download Album(s) from URL",
            "2. Configure Download Location",
            "3. Cloudflare Bypass Settings",
            "4. UI & Performance Settings",
            "5. Exit"
        ];

        let selection = Select::new()
            .with_prompt("Main Menu")
            .items(options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => {
                // Download Album(s)
                let input_url: String = Input::new()
                    .with_prompt("Enter Album URL(s) (space-separated)")
                    .interact_text()
                    .unwrap();

                let urls: Vec<String> = input_url
                    .split_whitespace()
                    .map(|s| s.trim_matches(|c| c == '"' || c == '\'' || c == '[' || c == ']').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if urls.is_empty() {
                    println!("No URLs entered.");
                    continue;
                }

                // Run download
                run_tui_download(urls, config).await;
            }
            1 => {
                // Configure Download Location
                configure_location(config).await;
            }
            2 => {
                // Cloudflare Bypass Settings
                configure_cloudflare(config).await;
            }
            3 => {
                // UI & Performance Settings
                configure_ui_settings(config).await;
            }
            4 => {
                println!("Goodbye!");
                break;
            }
            _ => unreachable!()
        }
    }
}

async fn configure_location(config: &mut Config) {
    loop {
        let current_path = config.download.default_output
            .as_ref()
            .map_or_else(|| "Not set (uses current directory)".to_string(), |p| p.display().to_string());
        
        let current_mount = config.download.mount_url
            .as_ref()
            .map_or_else(|| "Not set".to_string(), |m| m.clone());

        println!("\n--- Location Configuration ---");
        println!("Current Download Location: {}", current_path);
        println!("Network Share Mount URL:   {}", current_mount);
        println!("------------------------------");

        let options = &[
            "1. Set Local Directory",
            "2. Set Network Share Directory",
            "3. Custom (Use Finder folder picker)",
            "4. Go Back"
        ];

        let selection = Select::new()
            .with_prompt("Select Output Type")
            .items(options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => {
                // Local Directory
                let raw_path: String = Input::new()
                    .with_prompt("Paste Local Directory path")
                    .interact_text()
                    .unwrap();
                
                let path = normalize_path(&raw_path);
                config.download.default_output = Some(path.clone());
                config.download.mount_url = None; // Reset mount url
                
                let _ = save_config(config);
                println!("✓ Local download directory set to: {}", path.display());
            }
            1 => {
                // Network Share Directory
                let raw_path: String = Input::new()
                    .with_prompt("Paste Network Share Directory path")
                    .interact_text()
                    .unwrap();
                
                let path = normalize_path(&raw_path);
                config.download.default_output = Some(path.clone());
                
                println!("Normalizing and analyzing mount points...");
                if let Some(discovered_url) = discover_network_share(&path) {
                    println!("✓ Auto-discovered network share URL: {}", discovered_url);
                    let enable_auto = Confirm::new()
                        .with_prompt("Would you like to enable auto-mounting for this share?")
                        .default(true)
                        .interact()
                        .unwrap();
                    
                    if enable_auto {
                        config.download.mount_url = Some(discovered_url);
                    } else {
                        config.download.mount_url = None;
                    }
                } else {
                    println!("Could not auto-discover network share mount URL.");
                    let manual = Confirm::new()
                        .with_prompt("Would you like to enter the mount URL manually?")
                        .default(false)
                        .interact()
                        .unwrap();
                    
                    if manual {
                        let url: String = Input::new()
                            .with_prompt("Enter share URL (e.g. smb://10.10.27.10/Belmont)")
                            .interact_text()
                            .unwrap();
                        config.download.mount_url = Some(url);
                    } else {
                        config.download.mount_url = None;
                    }
                }

                let _ = save_config(config);
                println!("✓ Directory set to: {}", path.display());
            }
            2 => {
                // Finder folder picker
                if let Some(path) = choose_folder_via_finder() {
                    config.download.default_output = Some(path.clone());
                    
                    println!("Analyzing directory mount points...");
                    if let Some(discovered_url) = discover_network_share(&path) {
                        println!("✓ Detected network share: {}", discovered_url);
                        let enable_auto = Confirm::new()
                            .with_prompt("Enable auto-mounting for this share?")
                            .default(true)
                            .interact()
                            .unwrap();
                        
                        if enable_auto {
                            config.download.mount_url = Some(discovered_url);
                        } else {
                            config.download.mount_url = None;
                        }
                    } else {
                        config.download.mount_url = None;
                    }
                    
                    let _ = save_config(config);
                    println!("✓ Location selected: {}", path.display());
                } else {
                    println!("Folder selection cancelled.");
                }
            }
            3 => break,
            _ => unreachable!()
        }
    }
}

async fn configure_cloudflare(config: &mut Config) {
    loop {
        println!("\n--- Cloudflare Cookie Settings ---");
        display_config(config);
        println!("----------------------------------");

        let options = &[
            "1. Auto-Fetch cookie (Bypass Cloudflare using browser)",
            "2. Manually enter cf_clearance cookie",
            "3. Clear saved cookies",
            "4. Go Back"
        ];

        let selection = Select::new()
            .with_prompt("Select Action")
            .items(options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => {
                println!("Starting browser automation to fetch cookie...");
                crate::invalidate_existing_cookies(config);
                match crate::cf_fetcher::fetch_cf_clearance_with_retry().await {
                    Ok(cookie) => {
                        if let Err(e) = crate::save_fetched_cookie(config, cookie) {
                            println!("Warning: Failed to save cookie: {}", e);
                        }
                    }
                    Err(e) => {
                        println!("Failed to fetch cf-clearance: {}", e);
                    }
                }
            }
            1 => {
                let cookie: String = Input::new()
                    .with_prompt("Paste cf_clearance cookie value")
                    .interact_text()
                    .unwrap();
                
                config.cloudflare.cf_clearance = Some(cookie);
                config.cloudflare.cf_clearance_valid = Some(true);
                config.cloudflare.cf_clearance_timestamp = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                );
                
                let _ = save_config(config);
                println!("✓ Manual cookie saved successfully.");
            }
            2 => {
                config.cloudflare.cf_clearance = None;
                config.cloudflare.cf_clearance_valid = Some(false);
                config.cloudflare.cf_clearance_timestamp = None;
                let _ = save_config(config);
                println!("✓ Cookies cleared.");
            }
            3 => break,
            _ => unreachable!()
        }
    }
}

async fn configure_ui_settings(config: &mut Config) {
    loop {
        println!("\n--- UI & Performance Configuration ---");
        println!("Show Progress Bars: {}", config.ui.show_progress);
        println!("Colored Output:     {}", config.ui.colored_output);
        println!("--------------------------------------");

        let options = &[
            "1. Toggle Progress Bars",
            "2. Toggle Colored Output",
            "3. Go Back"
        ];

        let selection = Select::new()
            .with_prompt("Select Setting to Toggle")
            .items(options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => {
                config.ui.show_progress = !config.ui.show_progress;
                let _ = save_config(config);
                println!("✓ Progress bars set to: {}", config.ui.show_progress);
            }
            1 => {
                config.ui.colored_output = !config.ui.colored_output;
                let _ = save_config(config);
                println!("✓ Colored output set to: {}", config.ui.colored_output);
            }
            2 => break,
            _ => unreachable!()
        }
    }
}

pub fn normalize_path(raw: &str) -> PathBuf {
    let mut cleaned = raw.trim().to_string();

    // Strip leading '[' and trailing ']' if present
    if cleaned.starts_with('[') && cleaned.ends_with(']') {
        cleaned = cleaned[1..cleaned.len() - 1].trim().to_string();
    }
    // Strip leading '"' and trailing '"' if present
    if cleaned.starts_with('"') && cleaned.ends_with('"') {
        cleaned = cleaned[1..cleaned.len() - 1].trim().to_string();
    }
    // Strip leading '\'' and trailing '\'' if present
    if cleaned.starts_with('\'') && cleaned.ends_with('\'') {
        cleaned = cleaned[1..cleaned.len() - 1].trim().to_string();
    }

    // Replace escaped spaces '\ ' with regular spaces ' '
    cleaned = cleaned.replace("\\ ", " ");

    PathBuf::from(cleaned)
}

pub fn discover_network_share(path: &Path) -> Option<String> {
    let components: Vec<_> = path.components().collect();
    if components.len() >= 3 && components[1].as_os_str() == "Volumes" {
        let volume_name = components[2].as_os_str().to_str()?;
        
        let output = Command::new("mount").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let target_pattern = format!("on /Volumes/{}", volume_name);
        
        for line in stdout.lines() {
            if line.contains(&target_pattern) {
                // Example: //twan@10.10.27.10/Belmont on /Volumes/Belmont (smbfs, ...)
                if let Some(source) = line.split(" on ").next() {
                    let source = source.trim();
                    if source.starts_with("//") {
                        // Replace leading // with smb://
                        return Some(format!("smb:{}", source));
                    }
                }
            }
        }
    }
    None
}

pub fn choose_folder_via_finder() -> Option<PathBuf> {
    println!("Opening macOS Finder folder picker...");
    let script = "tell application \"System Events\" to activate\n\
                  POSIX path of (choose folder with prompt \"Select Lucida Download Directory\")";
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    Some(PathBuf::from(path_str))
                } else {
                    None
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.contains("User canceled") {
                    eprintln!("Error running Finder folder picker: {}", stderr);
                }
                None
            }
        }
        Err(e) => {
            eprintln!("Failed to execute osascript: {}", e);
            None
        }
    }
}

async fn run_tui_download(urls: Vec<String>, config: &Config) {
    // Check auto-mount before download
    let output = config.download.default_output.clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap());
        
    if !output.exists() {
        if let Some(ref mount_url) = config.download.mount_url {
            if crate::try_mount_share(mount_url) {
                // Poll output directory for up to 5 seconds
                for _ in 0..10 {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    if output.exists() {
                        println!("✓ Output directory is now accessible.");
                        break;
                    }
                }
            }
        }
    }

    if !output.exists() {
        println!("Error: Output directory {} is not accessible.", output.display());
        return;
    }

    println!("Starting download for {} albums...", urls.len());

    let (cf_clearance, user_agent) = if config.cloudflare.cf_clearance_valid.unwrap_or(false) {
        (config.cloudflare.cf_clearance.clone(), config.cloudflare.user_agent.clone())
    } else {
        (None, None)
    };

    let client = {
        let mut client = reqwest::ClientBuilder::new();
        if let Some(ref user_agent) = user_agent {
            client = client.user_agent(user_agent.clone());
        }
        if let Some(cf_clearance) = cf_clearance {
            client = client.default_headers(reqwest::header::HeaderMap::from_iter([(
                reqwest::header::COOKIE,
                format!("cf_clearance={cf_clearance}").try_into().unwrap(),
            )]));
        }
        client.build().unwrap()
    };

    let download_config = crate::models::DownloadConfig {
        country: "auto".to_string(),
        metadata: true,
        private: false,
    };

    let skip_config = crate::models::SkipConfig {
        tracks: false,
        cover: false,
    };

    let urls_len = urls.len();
    let urls_arc = std::sync::Arc::new(std::sync::Mutex::new(urls));
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let worker_count = 1.min(urls_len);

    let mut handles = Vec::new();
    for album_worker in 1..=worker_count {
        let client_clone = client.clone();
        let urls_clone = urls_arc.clone();
        let output_clone = output.clone();
        let dc_clone = download_config.clone();
        let running_clone = running.clone();

        handles.push(tokio::spawn(async move {
            crate::workers::run_album_worker(
                client_clone,
                urls_clone,
                output_clone,
                false,
                false,
                None,
                false,
                dc_clone,
                4,
                skip_config,
                running_clone,
                album_worker,
            ).await;
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    println!("Download workflow completed!");
}
