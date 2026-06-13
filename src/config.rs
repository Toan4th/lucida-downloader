use crate::models::{Cli, DownloadConfig};
use anyhow::Result;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub download: DownloadConfigSection,
    pub cloudflare: CloudflareConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfigSection {
    pub default_output: Option<PathBuf>,
    pub mount_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    pub cf_clearance: Option<String>,
    pub user_agent: Option<String>,
    pub cf_clearance_timestamp: Option<u64>,
    pub cf_clearance_valid: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub show_progress: bool,
    pub colored_output: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download: DownloadConfigSection {
                default_output: None,
                mount_url: None,
            },
            cloudflare: CloudflareConfig {
                cf_clearance: None,
                user_agent: None,
                cf_clearance_timestamp: None,
                cf_clearance_valid: None,
            },
            ui: UiConfig {
                show_progress: true,
                colored_output: true,
            },
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lucida")
}

pub fn get_config_file() -> PathBuf {
    get_config_dir().join("config.toml")
}

pub fn load_config() -> Config {
    let config_file = get_config_file();

    if !config_file.exists() {
        return Config::default();
    }

    let content = match fs::read_to_string(&config_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "Warning: Failed to read config file {}: {}",
                config_file.display(),
                e
            );
            return Config::default();
        }
    };

    match toml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!(
                "Warning: Failed to parse config file {}: {}",
                config_file.display(),
                e
            );
            Config::default()
        }
    }
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir();

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    let config_file = get_config_file();
    let content = toml::to_string_pretty(config)?;
    fs::write(config_file, content)?;

    Ok(())
}

pub fn display_config(config: &Config) {
    println!("Current Lucida Configuration:");
    println!("===========================");

    if let Some(output) = &config.download.default_output {
        println!("Default Output: {}", output.display());
    } else {
        println!("Default Output: (not set - uses current directory)");
    }

    if let Some(mount_url) = &config.download.mount_url {
        println!("Network Share Mount URL: {}", mount_url);
    } else {
        println!("Network Share Mount URL: (not set)");
    }

    if let Some(cf_clearance) = &config.cloudflare.cf_clearance {
        println!("Cloudflare Clearance: {}", cf_clearance);
        if config.cloudflare.cf_clearance_valid.unwrap_or(false) {
            if let Some(timestamp) = config.cloudflare.cf_clearance_timestamp {
                println!("  Status: Valid (timestamp: {})", timestamp);
            } else {
                println!("  Status: Valid");
            }
        } else {
            println!("  Status: Invalid");
        }
    } else {
        println!("Cloudflare Clearance: (not set)");
    }

    if let Some(user_agent) = &config.cloudflare.user_agent {
        println!("User Agent: {}", user_agent);
    } else {
        println!("User Agent: (not set)");
    }

    println!("Show Progress: {}", config.ui.show_progress);
    println!("Colored Output: {}", config.ui.colored_output);
}

pub fn update_config_value(config: &mut Config, set_expr: &str) -> Result<(), String> {
    let parts: Vec<&str> = set_expr.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Invalid format. Use KEY=VALUE".to_string());
    }
    
    let key = parts[0].trim().to_lowercase();
    let value = parts[1].trim();
    
    match key.as_str() {
        "output" | "default_output" | "default-output" => {
            config.download.default_output = Some(PathBuf::from(value));
        }
        "mount" | "mount_url" | "mount-url" => {
            config.download.mount_url = Some(value.to_string());
        }
        "user_agent" | "user-agent" | "ua" => {
            config.cloudflare.user_agent = Some(value.to_string());
        }
        "cf_clearance" | "cf-clearance" | "cf" => {
            config.cloudflare.cf_clearance = Some(value.to_string());
            config.cloudflare.cf_clearance_valid = Some(true);
            config.cloudflare.cf_clearance_timestamp = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            );
        }
        "show_progress" | "show-progress" | "progress" => {
            let val: bool = value.parse().map_err(|_| "Value must be true or false".to_string())?;
            config.ui.show_progress = val;
        }
        "colored_output" | "colored-output" | "color" => {
            let val: bool = value.parse().map_err(|_| "Value must be true or false".to_string())?;
            config.ui.colored_output = val;
        }
        _ => {
            return Err(format!(
                "Unknown configuration key: '{}'. Supported keys: output, mount, user_agent, cf_clearance, show_progress, colored_output",
                key
            ));
        }
    }
    
    Ok(())
}

pub fn merge_cli_with_config(cli: &Cli, config: &Config) -> (DownloadConfig, Option<PathBuf>) {
    let download_config = DownloadConfig {
        country: cli.country.clone(),
        metadata: !cli.no_metadata,
        private: cli.private,
    };

    let output = cli
        .output
        .clone()
        .or_else(|| config.download.default_output.clone());

    (download_config, output)
}

pub fn get_cloudflare_headers(cli: &Cli, config: &Config) -> (Option<String>, Option<String>) {
    let cf_clearance = cli
        .cf_clearance
        .clone()
        .or_else(|| config.cloudflare.cf_clearance.clone());

    let user_agent = cli
        .user_agent
        .clone()
        .or_else(|| config.cloudflare.user_agent.clone());

    (cf_clearance, user_agent)
}

/// Invalidate existing cf-clearance cookies
pub fn invalidate_existing_cookies(config: &mut Config) {
    config.cloudflare.cf_clearance_valid = Some(false);
    config.cloudflare.cf_clearance_timestamp = None;
    println!("Invalidated existing cf-clearance cookies");
}

/// Save fetched cf-clearance cookie with timestamp
pub fn save_fetched_cookie(config: &mut Config, cookie: String) -> Result<()> {
    config.cloudflare.cf_clearance = Some(cookie.clone());
    config.cloudflare.cf_clearance_valid = Some(true);
    config.cloudflare.cf_clearance_timestamp =
        Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());

    // Also save user agent that was used
    config.cloudflare.user_agent = Some("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string());

    save_config(config).map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;
    println!("✓ Successfully saved cf-clearance cookie to configuration");
    Ok(())
}
