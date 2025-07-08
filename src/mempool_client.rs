use anyhow::{Result, anyhow};
use serde::{Deserialize};
use chrono::{Utc, TimeZone, DateTime, Local};
use url::Url;


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MempoolBlockInfo {
    pub id: String,
    pub height: u32,
    pub version: u32,
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
    pub difficulty: f64,
    pub merkle_root: String,
    pub tx_count: u32,
    pub size: u32,
    pub weight: u32,
    pub previousblockhash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MempoolFeeEstimate {
    #[serde(rename = "fastestFee")]
    pub fastest_fee: u32,
    #[serde(rename = "halfHourFee")]
    pub half_hour_fee: u32,
    #[serde(rename = "hourFee")]
    pub hour_fee: u32,
    #[serde(rename = "economyFee")]
    pub economy_fee: u32,
}

// Default mempool.space API URL
pub const DEFAULT_MEMPOOL_API_URL: &str = "https://mempool.space/api";

// Mempool API client for handling all API interactions
pub struct MempoolClient {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl MempoolClient {
    /// Create a new MempoolClient with default mempool.space API URL
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::with_url(DEFAULT_MEMPOOL_API_URL)
    }
    
    /// Create a new MempoolClient with a custom API URL
    pub fn with_url(base_url: &str) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10)) // 10 second timeout
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        
        // Normalize the URL to ensure it doesn't end with a slash
        let normalized_url = Self::normalize_url(base_url);
            
        MempoolClient {
            client,
            base_url: normalized_url,
        }
    }
    
    /// Normalize a URL to ensure it's properly formatted
    /// - Validates that the URL is valid
    /// - Removes trailing slashes
    /// - Ensures the URL has a scheme (http/https)
    pub fn normalize_url(input_url: &str) -> String {
        // Try to parse the URL
        let parsed_url = match Url::parse(input_url) {
            Ok(url) => url,
            Err(_) => {
                // If parsing fails, try adding https:// prefix and parse again
                match Url::parse(&format!("https://{}", input_url)) {
                    Ok(url) => url,
                    Err(_) => {
                        // If still fails, just return the original URL
                        // The API calls will likely fail, but we won't crash here
                        return input_url.to_string();
                    }
                }
            }
        };
        
        // Get the URL without the trailing slash
        let mut normalized = parsed_url.to_string();
        if normalized.ends_with('/') {
            normalized.pop(); // Remove trailing slash
        }
        
        // If the URL doesn't end with "/api", append it
        if !normalized.ends_with("/api") {
            normalized = format!("{}/api", normalized);
        }
        
        normalized
    }
    
    /// Get the current base URL
    #[allow(dead_code)]
    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    pub fn fetch_latest_block(&self) -> Result<MempoolBlockInfo> {
        let url = format!("{}/blocks/tip/height", self.base_url);
        
        println!("Fetching latest block height from: {}", url);
        
        // First get the latest block height
        let response = self.client.get(&url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch block height: {}", e))?;
            
        if !response.status().is_success() {
            return Err(anyhow!("API returned error status: {}", response.status()));
        }
        
        let height: u32 = response.text()
            .map_err(|e| anyhow!("Failed to parse block height: {}", e))?
            .parse()
            .map_err(|e| anyhow!("Failed to parse block height as number: {}", e))?;
            
        // Now get the block details
        let block_url = format!("{}/block-height/{}", self.base_url, height);
        let block_hash_response = self.client.get(&block_url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch block hash: {}", e))?;
            
        if !block_hash_response.status().is_success() {
            return Err(anyhow!("API returned error status: {}", block_hash_response.status()));
        }
        
        let block_hash = block_hash_response.text()
            .map_err(|e| anyhow!("Failed to parse block hash: {}", e))?;
            
        // Finally get the block details
        let block_details_url = format!("{}/block/{}", self.base_url, block_hash);
        let block_details_response = self.client.get(&block_details_url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch block details: {}", e))?;
            
        if !block_details_response.status().is_success() {
            return Err(anyhow!("API returned error status: {}", block_details_response.status()));
        }
        
        let block_info: MempoolBlockInfo = block_details_response.json()
            .map_err(|e| anyhow!("Failed to parse block details: {}", e))?;
            
        Ok(block_info)
    }
    
    pub fn fetch_fee_estimates(&self) -> Result<MempoolFeeEstimate> {
        let url = format!("{}/v1/fees/recommended", self.base_url);
        
        println!("Fetching fee estimates from: {}", url);
        
        let response = self.client.get(&url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch fee estimates: {}", e))?;
            
        if !response.status().is_success() {
            return Err(anyhow!("API returned error status: {}", response.status()));
        }
        
        let fee_estimates: MempoolFeeEstimate = response.json()
            .map_err(|e| anyhow!("Failed to parse fee estimates: {}", e))?;
            
        Ok(fee_estimates)
    }

}

// Helper function to format Unix timestamp to date-time format (YYYY-MM-DD HH:MM)
pub fn format_unix_timestamp(timestamp: u32) -> String {
    if let Some(datetime) = Utc.timestamp_opt(timestamp as i64, 0).single() {
        let local_time: DateTime<Local> = DateTime::from(datetime);
        return local_time.format("%Y-%m-%d %H:%M").to_string();
    }
    "Invalid timestamp".to_string()
}
