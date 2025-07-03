use anyhow::{Result, anyhow};
use serde::{Deserialize};
use chrono::{Utc, TimeZone, DateTime, Local};

// Bitstamp API response structures
#[derive(Debug, Deserialize)]
pub struct BitstampResponse {
    pub last: String,
}

#[derive(Debug, Deserialize)]
pub struct BitstampOHLC {
    pub timestamp: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
}

#[derive(Debug, Deserialize)]
pub struct BitstampOHLCData {
    pub ohlc: Vec<BitstampOHLC>,
}

#[derive(Debug, Deserialize)]
pub struct BitstampHistoricalData {
    pub data: BitstampOHLCData,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartTimeframe {
    Hours24,
    Week,
    Month,
}

impl ChartTimeframe {
    pub fn description(&self) -> &'static str {
        match self {
            ChartTimeframe::Hours24 => "24 Hours (hourly)",
            ChartTimeframe::Week => "1 Week (4-hour)",
            ChartTimeframe::Month => "1 Month (daily)",
        }
    }
    
    pub fn api_params(&self) -> (u32, u32) {
        match self {
            ChartTimeframe::Hours24 => (3600, 24),      // 1 hour steps, 24 candles
            ChartTimeframe::Week => (14400, 42),        // 4 hour steps, 42 candles (1 week)
            ChartTimeframe::Month => (86400, 30),       // 24 hour steps, 30 candles (1 month)
        }
    }
}

// Bitstamp API client for handling all API interactions
pub struct BitstampClient {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl BitstampClient {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10)) // 10 second timeout
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
            
        BitstampClient {
            client,
            base_url: "https://www.bitstamp.net/api/v2".to_string(),
        }
    }
    
    pub fn fetch_current_price(&self) -> Result<f64> {
        let url = format!("{}/ticker/btcusd/", self.base_url);
        
        println!("Fetching current BTC price from: {}", url);
        
        let response = self.client.get(&url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch price: {}", e))?;
            
        if !response.status().is_success() {
            return Err(anyhow!("API returned error status: {}", response.status()));
        }
        
        let ticker: BitstampResponse = response.json()
            .map_err(|e| anyhow!("Failed to parse price response: {}", e))?;
            
        // Convert the price string to a float
        let price = ticker.last.parse::<f64>()
            .map_err(|e| anyhow!("Failed to parse price value: {}", e))?;
            
        Ok(price)
    }
    
    pub fn fetch_historical_prices(&self, timeframe: ChartTimeframe) -> Result<BitstampHistoricalData> {
        // Get the step (candle interval in seconds) and limit (number of candles) based on timeframe
        let (step, limit) = timeframe.api_params();
        
        // Construct the URL with the appropriate parameters
        let url = format!("{}/ohlc/btcusd/?step={}&limit={}", self.base_url, step, limit);
        println!("Fetching historical data from: {} ({})", url, timeframe.description());
        
        let response = self.client.get(&url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch historical data: {}", e))?;
            
        if !response.status().is_success() {
            return Err(anyhow!("Historical API returned error status: {}", response.status()));
        }
        
        let response_text = response.text()
            .map_err(|e| anyhow!("Failed to get response text: {}", e))?;
            
        if response_text.len() > 200 {
            println!("Response text sample: {}...", &response_text[..200]);
        } else {
            println!("Response text: {}", response_text);
        }
        
        // Try to parse the JSON
        let data = serde_json::from_str::<BitstampHistoricalData>(&response_text)
            .map_err(|e| anyhow!("Failed to parse historical data: {}", e))?;
            
        println!("Successfully parsed historical data for {} ({} candles)", 
                timeframe.description(), data.data.ohlc.len());
                
        // Print a sample of formatted timestamps if available
        if !data.data.ohlc.is_empty() {
            let sample_timestamp = &data.data.ohlc[0].timestamp;
            let formatted = format_unix_timestamp(sample_timestamp);
            println!("Sample timestamp: {} formatted as: {}", sample_timestamp, formatted);
        }
        
        Ok(data)
    }
}

// Helper function to format Unix timestamp to date-time format (YYYY-MM-DD HH:MM)
pub fn format_unix_timestamp(timestamp_str: &str) -> String {
    if let Ok(timestamp) = timestamp_str.parse::<i64>() {
        if let Some(datetime) = Utc.timestamp_opt(timestamp, 0).single() {
            let local_time: DateTime<Local> = DateTime::from(datetime);
            return local_time.format("%Y-%m-%d %H:%M").to_string();
        }
    }
    "Invalid timestamp".to_string()
}
