use std::fs;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};

// Default configuration values
pub const DEFAULT_MEMPOOL_API_URL: &str = "https://mempool.space/api";

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    // Mempool configuration
    pub mempool_custom_url_enabled: bool,
    pub mempool_api_url: String,
    
    // Can add more configuration options here in the future
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            mempool_custom_url_enabled: false,
            mempool_api_url: DEFAULT_MEMPOOL_API_URL.to_string(),
        }
    }
}

impl AppConfig {
    // Get the config file path in the user's config directory
    pub fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("btc-ticker");
        fs::create_dir_all(&path).ok(); // Create directory if it doesn't exist
        path.push("config.json");
        path
    }
    
    // Load configuration from file, or create default if it doesn't exist
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(config) => return config,
                        Err(e) => {
                            eprintln!("Error parsing config file: {}", e);
                            // Fall back to default config
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading config file: {}", e);
                    // Fall back to default config
                }
            }
        }
        
        // If we get here, either the file doesn't exist or there was an error
        // Create default config and save it
        let default_config = AppConfig::default();
        default_config.save().ok(); // Ignore errors on first save
        default_config
    }
    
    // Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
            
        fs::write(&config_path, content)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;
            
        Ok(())
    }
}
