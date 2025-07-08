use anyhow::{Result, anyhow};
use serde::{Deserialize};
use chrono::{Utc, TimeZone, DateTime, Local};

// Mempool.space API response structures
#[derive(Debug, Deserialize)]
pub struct MempoolBlockchainInfo {
    pub chain: String,
    pub blocks: u32,
    pub headers: u32,
    pub difficulty: f64,
    pub size_on_disk: u64,
    pub verification_progress: f64,
}

#[derive(Debug, Deserialize)]
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
    #[serde(rename = "minimumFee")]
    pub minimum_fee: u32,
}

#[derive(Debug, Deserialize)]
pub struct MempoolStats {
    pub funded_txo_count: u64,
    pub funded_txo_sum: u64,
    pub spent_txo_count: u64,
    pub spent_txo_sum: u64,
    pub tx_count: u64,
}

// Mempool API client for handling all API interactions
pub struct MempoolClient {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl MempoolClient {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10)) // 10 second timeout
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
            
        MempoolClient {
            client,
            base_url: "https://mempool.space/api".to_string(),
        }
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
