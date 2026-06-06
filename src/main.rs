use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use clap::Parser;
use dialoguer::{Input, Confirm};
use futures::future;
use models::{Cli, SkipConfig};
use reqwest::ClientBuilder;
use reqwest::header::{COOKIE, HeaderMap};
use tokio::signal;

mod config;
mod cf_fetcher;
mod downloaders;
mod models;
mod requests;
mod text_utils;
mod ui;
mod workers;

use config::{load_config, save_config, display_config, merge_cli_with_config, get_cloudflare_headers, Config, invalidate_existing_cookies, save_fetched_cookie};
use ui::DownloadProgress;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();
    let mut config = load_config();

    // Handle configuration and setup commands first
    if cli.config {
        display_config(&config);
        return;
    }

    if cli.setup {
        run_interactive_setup(&mut config).await;
        return;
    }

    if cli.set_output.is_some() || cli.set_user_agent.is_some() || cli.update_cf.is_some() {
        handle_config_updates(&cli, &mut config);
        return;
    }

    // Handle cf-clearance fetching
    if cli.fetch_cf || cli.refresh_cf {
        if cli.refresh_cf {
            invalidate_existing_cookies(&mut config);
        }
        
        println!("Starting cf-clearance fetch process for lucida.to...");
        match cf_fetcher::fetch_cf_clearance_with_retry().await {
            Ok(cookie) => {
                if let Err(e) = save_fetched_cookie(&mut config, cookie) {
                    eprintln!("Warning: Failed to save cookie: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch cf-clearance: {}", e);
                return;
            }
        }
    }

    // Clone URLs before moving cli values
    let mut urls = cli.urls.clone();

    for file in &cli.file {
        urls.extend(
            BufReader::new(File::open(file).unwrap())
                .lines()
                .map(|line| line.unwrap()),
        );
    }

    urls.reverse();

    if urls.is_empty() {
        eprintln!("no URLs to download");
        eprintln!("Use --help to see available commands");
        return;
    }

    let urls_len = urls.len();
    
    let ui = DownloadProgress::new(config.ui.show_progress, config.ui.colored_output);
    ui.print_info(&format!("Downloading {} albums", urls_len));

    let (cf_clearance, user_agent) = if cli.fetch_cf || cli.refresh_cf {
        // After fetch, use the newly saved cookie
        (config.cloudflare.cf_clearance.clone(), config.cloudflare.user_agent.clone())
    } else if config.cloudflare.cf_clearance_valid.unwrap_or(false) {
        // Use existing valid cookies
        (config.cloudflare.cf_clearance.clone(), config.cloudflare.user_agent.clone())
    } else {
        // Fall back to manual configuration
        get_cloudflare_headers(&cli, &config)
    };
    
    let (download_config, output_path) = merge_cli_with_config(&cli, &config);

    let client = {
        let mut client = ClientBuilder::new();

        if let Some(ref user_agent) = user_agent {
            client = client.user_agent(user_agent.clone());
            ui.print_info(&format!("Using User-Agent: {}", user_agent));
        }

        if let Some(cf_clearance) = cf_clearance {
            client = client.default_headers(HeaderMap::from_iter([(
                COOKIE,
                format!("cf_clearance={cf_clearance}").try_into().unwrap(),
            )]));
            ui.print_info("Using Cloudflare clearance cookie");
        }

        client.build().unwrap()
    };

    let output = output_path.unwrap_or_else(|| env::current_dir().unwrap());
    ui.print_info(&format!("Output directory: {}", output.display()));

    let urls = Arc::new(Mutex::new(urls));
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let worker_count = cli.album_workers.min(urls_len);

    ui.print_info(&format!("Spawning {} album workers", worker_count));

    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        running_clone.store(false, Ordering::Relaxed);
        eprintln!("Stopping gracefully");
    });

    for result in future::join_all((1..=worker_count).map(|album_worker| {
        tokio::spawn(workers::run_album_worker(
            client.clone(),
            urls.clone(),
            output.clone(),
            cli.force,
            cli.group_singles,
            cli.album_year,
            cli.flatten_directories,
            download_config.clone(),
            cli.track_workers,
            SkipConfig {
                tracks: cli.skip_tracks,
                cover: cli.skip_cover,
            },
            running.clone(),
            album_worker,
        ))
    }))
    .await
    {
        result.unwrap();
    }

    ui.print_success("Download finished!");
}

async fn run_interactive_setup(config: &mut Config) {
    println!("Welcome to Lucida Downloader Setup!");
    println!("==================================");
    
    // Set up output directory
    let current_output = config.download.default_output
        .as_ref()
        .map_or_else(|| env::current_dir().unwrap().display().to_string(), |p| p.display().to_string());
    
    let output = Input::new()
        .with_prompt("Default output directory")
        .default(current_output)
        .interact_text()
        .unwrap();
    
    config.download.default_output = Some(output.into());

    // Set up user agent
    let default_agent = config.cloudflare.user_agent.clone().unwrap_or_else(|| {
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
    });
    
    let user_agent = Input::new()
        .with_prompt("User-Agent")
        .default(default_agent)
        .interact_text()
        .unwrap();
    
    config.cloudflare.user_agent = Some(user_agent);

    // Ask about progress bars
    let show_progress = Confirm::new()
        .with_prompt("Enable progress bars?")
        .default(true)
        .interact()
        .unwrap();
    
    config.ui.show_progress = show_progress;

    // Ask about colored output
    let colored_output = Confirm::new()
        .with_prompt("Enable colored output?")
        .default(true)
        .interact()
        .unwrap();
    
    config.ui.colored_output = colored_output;

    if let Err(e) = save_config(config) {
        eprintln!("Error saving configuration: {}", e);
    } else {
        println!("Configuration saved successfully!");
        display_config(config);
    }
}

fn handle_config_updates(cli: &Cli, config: &mut Config) {
    if let Some(output) = &cli.set_output {
        config.download.default_output = Some(output.clone());
        println!("Updated default output directory to: {}", output.display());
    }

    if let Some(user_agent) = &cli.set_user_agent {
        config.cloudflare.user_agent = Some(user_agent.clone());
        println!("Updated User-Agent to: {}", user_agent);
    }

    if let Some(cf_clearance) = &cli.update_cf {
        config.cloudflare.cf_clearance = Some(cf_clearance.clone());
        println!("Updated Cloudflare clearance cookie");
    }

    if let Err(e) = save_config(config) {
        eprintln!("Error saving configuration: {}", e);
    } else {
        println!("Configuration updated successfully!");
    }
}
