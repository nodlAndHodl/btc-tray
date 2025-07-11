# Bitcoin Tray App

A Bitcoin price monitoring application that lives in your system tray. Built with Rust using `egui`, `eframe`, and `tray-icon` crates.

## Features

- **Live Bitcoin Price Updates**: Real-time BTC/USD price displayed directly in your system tray
- **Interactive Candlestick Chart**: Visualize price movements with multiple timeframes:
  - 24 Hours (hourly candles)
  - 1 Week (4-hour candles)
  - 1 Month (daily candles)
  - 1 Year (daily candles)
- **Current Price Indicator**: Horizontal line marker showing the current price on the chart
- **Bitcoin Network Data**: Real-time mempool information including:
  - Latest block height and timestamp
  - Transaction fee estimates (fastest, half-hour, hour, economy)
- **Data Sources**:
  - Price data from Bitstamp API
  - Network data from mempool.space API (configurable)
- **Local Time Display**: All timestamps converted to your local timezone
- **Customizable Configuration**: Ability to use custom mempool API endpoints

## Installation

### Prerequisites

- Rust 1.71 or newer
- For Linux: GTK 3 development libraries (`libgtk-3-dev` package on Debian/Ubuntu)
- For macOS: Xcode command line tools

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/btc-tray-app.git
   cd btc-tray-app
   ```

2. Build and run the application:
   ```bash
   cargo build --release
   cargo run --release
   ```

The compiled binary will be available in `target/release/btc-ticker`.

## Running the Application

To run the application:

```bash
cargo run
```

The application will show a Bitcoin icon in your system tray. Right-click on the icon to display the menu.

## Menu Options

- **Refresh BTC Price**: Manually refreshes the Bitcoin price and chart data
- **Chart Timeframe Options**:
  - **24 Hours (hourly)**: Shows hourly candles for the past 24 hours
  - **1 Week (4-hour)**: Shows 4-hour candles for the past week
  - **1 Month (daily)**: Shows daily candles for the past month
  - **1 Year (daily)**: Shows daily candles for the past year
- **Settings**: Configure application settings
  - **Custom Mempool API**: Set a custom mempool API endpoint
- **Quit**: Exits the application

## Configuration

The application stores its configuration in:
- Linux/macOS: `~/.config/btc-ticker/config.json`
- Windows: `%APPDATA%\btc-ticker\config.json`

Configuration options include:
- `mempool_custom_url_enabled`: Whether to use a custom mempool API URL
- `mempool_api_url`: The custom mempool API URL when enabled

## Data Sources

- **Price Data**: Fetched from Bitstamp's public API (`https://www.bitstamp.net/api/v2`)
- **Mempool Data**: By default, fetched from mempool.space API (`https://mempool.space/api`)
  - Can be configured to use any compatible mempool API endpoint

## Requirements

- Rust 1.71 or newer
- For Linux: GTK 3 development libraries
- For macOS: Xcode command line tools

## License

This project is open source and available under the [MIT License](LICENSE).
