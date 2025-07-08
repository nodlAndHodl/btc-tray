# Bitcoin Tray App

A Bitcoin price monitoring application that lives in your system tray. Built with Rust using `egui`, `eframe`, and `tray-icon` crates.

## Features

- Live Bitcoin price updates in the system tray
- Interactive candlestick chart with multiple timeframes:
  - 24 Hours (hourly candles)
  - 1 Week (4-hour candles)
  - 1 Month (daily candles)
- Current price horizontal line marker on the chart
- Automatic data fetching from Bitstamp API
- Local time display for all chart timestamps

## Running the Application

To run the application:

```bash
cargo run
```

The application will show an icon in your system tray. Right-click on the icon to display the menu.

## Menu Options
- **Refresh BTC Price**: Manually refreshes the Bitcoin price and chart data
- **Chart Timeframe Options**:
  - **24 Hours (hourly)**: Shows hourly candles for the past 24 hours
  - **1 Week (4-hour)**: Shows 4-hour candles for the past week
  - **1 Month (daily)**: Shows daily candles for the past month
- **Quit**: Exits the application

## Requirements

- Rust 1.71 or newer
- For Linux: GTK 3 development libraries
