#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::Result;
use egui_plot::{Plot, BoxPlot, BoxElem, BoxSpread, Corner, Legend};
use chrono::{DateTime, Utc, TimeZone, Timelike, Local};

use tray_icon::{
    menu::{MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};

use eframe::egui;

mod bitstamp_client;
use bitstamp_client::{BitstampClient, ChartTimeframe};

// For debugging
fn print_historical_data(data: &bitstamp_client::BitstampHistoricalData) {
    for (i, point) in data.data.ohlc.iter().enumerate().take(5) {
        println!("Point {}: timestamp={}, close={}", i, point.timestamp, point.close);
    }
}

// Shared state between the tray icon and the egui app
struct BitcoinState {
    price: f64,
    last_updated: String,
    updating: bool,
    // Add a flag to indicate when a new price has been fetched
    new_price_fetched: bool,
    // Store historical data as OHLC (Open, High, Low, Close)
    historical_data: Vec<(TimeInfo, CandleData)>,
    // Current chart timeframe
    chart_timeframe: ChartTimeframe,
}

// Structure to hold candlestick data
#[derive(Debug, Clone, Copy)]
struct CandleData {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

// Structure to hold formatted timestamp info
#[derive(Debug, Clone)]
struct TimeInfo {
    raw_timestamp: i64,     
    formatted_time: String, 
    rfc3339: String,        
}
impl std::fmt::Display for TimeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.formatted_time)
    }
}

impl BitcoinState {
    fn new() -> Self {
        BitcoinState {
            price: 0.0,
            last_updated: String::new(),
            updating: false,
            new_price_fetched: false,
            historical_data: Vec::new(),
            chart_timeframe: ChartTimeframe::Hours24, 
        }
    }
}
struct BitcoinApp {
    state: Arc<Mutex<BitcoinState>>,
    price_history: Vec<(TimeInfo, CandleData)>,
}

impl BitcoinApp {
    fn new(state: Arc<Mutex<BitcoinState>>) -> Self {
        Self {
            state,
            price_history: Vec::new(),
        }
    }

    fn update_price_history(&mut self) {
        let mut state = self.state.lock().unwrap();  
        // First check if we need to load initial historical data
        if self.price_history.is_empty() && !state.historical_data.is_empty() {
            self.price_history = state.historical_data.clone();
        } else if !state.historical_data.is_empty() {
            self.price_history = state.historical_data.clone();
        }
        
        if state.new_price_fetched {
            let candle = CandleData {
                open: state.price,
                high: state.price,
                low: state.price,
                close: state.price,
            };
            
            let now = chrono::Utc::now();
            let timestamp = now.timestamp();
            let time_info = TimeInfo {
                raw_timestamp: timestamp,
                formatted_time: state.last_updated.clone(),
                rfc3339: now.to_rfc3339(),
            };
            self.price_history.push((time_info, candle));
            
            // Keep only last 100 entries
            if self.price_history.len() > 100 {
                self.price_history.remove(0);
            }
            
            // Reset the flag
            state.new_price_fetched = false;
        }
    }

}

impl eframe::App for BitcoinApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_price_history();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            
            let state = self.state.lock().unwrap();
            let price_text = if state.price > 0.0 {
                // Calculate satoshis per dollar (1 BTC = 100,000,000 satoshis)
                let sats_per_dollar = 100_000_000.0 / state.price;
                format!("${:.2} | {:.0} sats/$", state.price, sats_per_dollar)
            } else {
                "Loading...".to_string()
            };
            
            ui.add_space(20.0);
            // Center the price and last updated information
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading(price_text);
                ui.add_space(5.0);
                ui.label(format!("Last updated: {}", state.last_updated));
            });
            
            ui.add_space(10.0);
            
            if !self.price_history.is_empty() {
                ui.add_space(10.0);
                
                // Center the chart and its label
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    // Get the appropriate chart title based on current timeframe
                    let chart_title = match state.chart_timeframe {
                        ChartTimeframe::Hours24 => "BTC Price (24 hours - hourly):",
                        ChartTimeframe::Week => "BTC Price (1 week - 4-hour):",
                        ChartTimeframe::Month => "BTC Price (1 month - daily):",
                    };
                    ui.label(chart_title);
                    ui.add_space(5.0);
                    
                    // Create plot data
                    if !self.price_history.is_empty() {
                        // Create candlestick elements for the chart
                        let mut candles = Vec::with_capacity(self.price_history.len());
                        
                        // Calculate x-axis values (time elapsed in minutes from first data point)
                        if let Some((first_time_info, _)) = self.price_history.first() {
                            if let Ok(_first_time) = DateTime::parse_from_rfc3339(&first_time_info.rfc3339) {
                                for (time_info, candle_data) in self.price_history.iter() {
                                    if let Ok(_timestamp) = DateTime::parse_from_rfc3339(&time_info.rfc3339) {
                                       // Use timestamp directly (as seconds since epoch) for x-axis position
                                        // Convert i64 timestamp to f64 for plotting
                                        let plot_x = time_info.raw_timestamp as f64;
                                        
                                        // For proper candlestick chart, the x-position is time and y values are price
                                        let box_elem = BoxElem::new(
                                            plot_x,  // x position (timestamp as f64)
                                            BoxSpread::new(
                                                candle_data.low,       // lowest price (bottom whisker)
                                                candle_data.open,      // box bottom - ALWAYS the open price
                                                (candle_data.open + candle_data.close) / 2.0, // median - midpoint between open and close
                                                candle_data.close,     // box top - ALWAYS the close price
                                                candle_data.high       // highest price (top whisker)
                                            )
                                        )
                                        .whisker_width(0.8)  // Width of the whiskers relative to the box
                                        .box_width(2.2)      // Width of the box/body
                                        // Color the candle based on whether price went up or down
                                        .fill(if candle_data.close >= candle_data.open {
                                            egui::Color32::from_rgb(0, 200, 0) // Brighter green for price up
                                        } else {
                                            egui::Color32::from_rgb(200, 0, 0)  // Brighter red for price down
                                        })
                                        .stroke(if candle_data.close >= candle_data.open {
                                            egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 0)) // Green stroke for price up
                                        } else {
                                            egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 0, 0)) // Red stroke for price down
                                        }); // Outline color matches fill color
                                        
                                        candles.push(box_elem);
                                    }
                                }
                            }
                        }
                        
                        // Only display chart if we have valid candles
                        if !candles.is_empty() {
                            //println!("Created {} candles for chart", candles.len());
                            
                            // Create a named box plot with the candles
                            let box_plot = BoxPlot::new("BTC/USD", candles);
                            
                            // Calculate the min and max y values for better scaling
                            let mut min_price = f64::MAX;
                            let mut max_price = f64::MIN;
                            
                            for (_, candle) in &self.price_history {
                                min_price = min_price.min(candle.low);
                                max_price = max_price.max(candle.high);
                            }
                            
                            // Add some padding to the min/max for better visual appearance
                            let price_range = max_price - min_price;
                            let min_y = (min_price - (price_range * 0.05)).max(0.0); // 5% padding below, but not below 0
                            let max_y = max_price + (price_range * 0.05); // 5% padding above
                            
                            // Create a custom formatter for the x-axis to show time
                            let time_formatter = |_name: &str, value: &egui_plot::PlotPoint| -> String {
                                // Try to convert the timestamp back to a readable format
                                if let Some(utc_dt) = Utc.timestamp_opt(value.x as i64, 0).single() {
                                    // Convert UTC to local time
                                    let local_time = Local.from_utc_datetime(&utc_dt.naive_utc());
                                    // Format as HH:MM in local time
                                    format!("{:02}  {:02}:{:02}", local_time.date_naive(), local_time.hour(), local_time.minute())
                                } else {
                                    format!("{:.1}", value.x) // Fallback
                                }
                            };
                            
                            // Get available width from UI
                            let available_width = ui.available_width();
                            
                            // Calculate height based on width (maintain aspect ratio)
                            let chart_height = (available_width / 2.5).min(300.0).max(150.0);
                            
                            // Display the plot using available width
                            Plot::new("btc_price_history")
                                .view_aspect(2.5)  // Wider aspect ratio
                                .height(chart_height)     // Dynamic height based on width
                                .width(available_width.min(1200.0))      // Use available width with maximum cap
                                .allow_zoom(true)
                                .allow_scroll(true)
                                .allow_drag(true)
                                .min_size(egui::vec2(300.0, 150.0)) // Set reasonable minimum size
                                // .include_y(0.0)    // Always include zero on y-axis
                                .y_axis_min_width(0.5)   // Make y-axis more visible
                                .y_axis_label("Price ($)")
                                .x_axis_label("Time (Local)")
                                .label_formatter(time_formatter)
                                .legend(Legend::default().position(Corner::RightTop))
                                // Set custom bounds for better scaling
                                .include_y(min_y) // Include minimum y value
                                .include_y(max_y) // Include maximum y value
                                .show(ui, |plot_ui| {
                                    // Add the candlestick chart
                                    plot_ui.box_plot(box_plot);
                                    
                                    // Add an orange horizontal line at the current Bitcoin price
                                    if state.price > 0.0 {
                                        let orange_line_color = egui::Color32::from_rgb(255, 140, 0); // Orange color
                                        let line_stroke = egui::Stroke::new(2.0, orange_line_color); // Thicker line for visibility
                                        
                                        // Create a horizontal line across the entire plot at the current price
                                        // Get the first and last timestamps from our data for the line endpoints
                                        if let (Some((first_time, _)), Some((last_time, _))) = (self.price_history.first(), self.price_history.last()) {
                                            let start_x = first_time.raw_timestamp as f64;
                                            let end_x = last_time.raw_timestamp as f64;
                                            
                                            // Add the horizontal line using a line with two points
                                            let points: Vec<[f64; 2]> = vec![
                                                [start_x, state.price],
                                                [end_x, state.price],
                                            ];
                                            let line = egui_plot::Line::new(format!("Current Price: ${:.2}", state.price), points)
                                            .stroke(line_stroke);
                                            
                                            plot_ui.line(line);
                                        }
                                    }
                                });
                        }
                    }
                });
            }
        });
        
        // Request repaint every second to keep the UI updated
        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

fn main() -> Result<(), eframe::Error> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");
    let icon = load_icon(std::path::Path::new(path));

    // Create shared state
    let bitcoin_state = Arc::new(Mutex::new(BitcoinState::new()));
    
    // Fetch historical data and initial price
    let init_state = bitcoin_state.clone();
    
    thread::spawn(move || {
        // First try to get historical data
        if let Ok(historical_data) = fetch_historical_bitcoin_prices(ChartTimeframe::Hours24) {
            // Print debug info about the data
            print_historical_data(&historical_data);
            
            // Process historical data
            let mut history = Vec::new();
            
            // Convert historical data to our format
            for point in historical_data.data.ohlc.iter() {
                // Debug print point
                println!("Processing point: timestamp={}, close={}", point.timestamp, point.close);
                
                if let (Ok(timestamp), Ok(open), Ok(high), Ok(low), Ok(close)) = (
                    point.timestamp.parse::<i64>(),
                    point.open.parse::<f64>(),
                    point.high.parse::<f64>(),
                    point.low.parse::<f64>(),
                    point.close.parse::<f64>()
                ) {
                    // Debug timestamp parsing
                    println!("  Parsed timestamp: {}", timestamp);
                    
                    // Create candle data
                    let candle = CandleData {
                        open,
                        high,
                        low,
                        close
                    };
                    
                    // Convert unix timestamp to ISO 8601
                    if let Some(datetime) = Utc.timestamp_opt(timestamp, 0).single() {
                        let rfc3339 = datetime.to_rfc3339();
                        let formatted_time = format_unix_timestamp(&point.timestamp);
                        println!("  Formatted time: {}", formatted_time);
                        history.push((TimeInfo {
                            raw_timestamp: timestamp,
                            formatted_time,
                            rfc3339,
                        }, candle));
                    } else {
                        println!("  Invalid timestamp: {}", timestamp);
                    }
                } else {
                    println!("  Failed to parse timestamp or price values");
                }
            }
            
            println!("Total processed history points: {}", history.len());
            
            // Store the historical data in the app state
            if !history.is_empty() {
                let mut state = init_state.lock().unwrap();
                state.historical_data = history;
                
                // Set the current price from the latest historical data point
                if let Some((_, latest_candle)) = state.historical_data.last() {
                    state.price = latest_candle.close;
                    state.last_updated = get_current_timestamp();
                }
            }
        }
        
        // Then get the current price
        refresh_bitcoin_price(init_state);
    });

    // Set up a periodic timer for price updates
    let timer_state = bitcoin_state.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(60)); // Update every minute
            refresh_bitcoin_price(timer_state.clone());
        }
    });

    // Create the tray icon (platform specific)
    #[cfg(target_os = "linux")]
    {
        let linux_state = bitcoin_state.clone();
        
        // Spawn the GTK thread for the tray icon
        let _tray_thread = thread::spawn(move || {
            use tray_icon::menu::Menu;
            gtk::init().unwrap();
            
            // Create menu items with unique IDs
            let tray_menu = Menu::new();
            
            // Create menu items with unique identifiers
            // The third parameter is for keyboard shortcuts (Accelerator), not callbacks
            let refresh_i = MenuItem::with_id("refresh-btc", "Refresh BTC Price", true, None);
                
            // Add chart timeframe selection options
            let timeframe_24h = MenuItem::with_id("timeframe-24h", "24 Hours (hourly)", true, None);
            let timeframe_week = MenuItem::with_id("timeframe-week", "1 Week (4-hour)", true, None);
            let timeframe_month = MenuItem::with_id("timeframe-month", "1 Month (daily)", true, None);
                
            let quit_i = MenuItem::with_id("quit-app", "Quit", true, None);
                
            // Create a clone of the state for the menu event handler thread
            let state_for_menu_events = linux_state.clone();
                
            // Add items to the menu
            let _ =tray_menu.append_items(&[
                &refresh_i,
                &PredefinedMenuItem::separator(),
                &timeframe_24h,
                &timeframe_week,
                &timeframe_month,
                &PredefinedMenuItem::separator(),
                &quit_i,
            ]);
            
            // Create the tray icon
            let _tray_icon = TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_icon(icon)
                .with_tooltip("BTC Ticker")
                .build()
                .unwrap();
            
            // Start a thread to handle menu events
            thread::spawn(move || {
                // Use the built-in event receiver from tray-icon
                use tray_icon::menu::MenuEvent;
                let receiver = MenuEvent::receiver();
                
                while let Ok(event) = receiver.recv() {
                    // Get the string representation of the MenuId
                    let id = event.id.0.to_string();  // Access the inner value with .0
                    match id.as_str() {
                        "refresh-btc" => {
                            refresh_bitcoin_price(state_for_menu_events.clone());
                        },
                        "timeframe-24h" => {
                            let mut state = state_for_menu_events.lock().unwrap();
                            if state.chart_timeframe != ChartTimeframe::Hours24 {
                                state.chart_timeframe = ChartTimeframe::Hours24;
                                // Drop lock before refreshing
                                drop(state);
                                refresh_bitcoin_price(state_for_menu_events.clone());
                            }
                        },
                        "timeframe-week" => {
                            let mut state = state_for_menu_events.lock().unwrap();
                            if state.chart_timeframe != ChartTimeframe::Week {
                                state.chart_timeframe = ChartTimeframe::Week;
                                // Drop lock before refreshing
                                drop(state);
                                refresh_bitcoin_price(state_for_menu_events.clone());
                            }
                        },
                        "timeframe-month" => {
                            let mut state = state_for_menu_events.lock().unwrap();
                            if state.chart_timeframe != ChartTimeframe::Month {
                                state.chart_timeframe = ChartTimeframe::Month;
                                // Drop lock before refreshing
                                drop(state);
                                refresh_bitcoin_price(state_for_menu_events.clone());
                            }
                        },
                        "quit-app" => {
                            std::process::exit(0);
                        },
                        _ => {}
                    }
                }
            });
            
            gtk::main();
        });
    }

    #[cfg(not(target_os = "linux"))]
    {
        let mut _tray_icon = Rc::new(RefCell::new(None));
        let tray_c = _tray_icon.clone();
        let refresh_state = bitcoin_state.clone();
    }

    // Run the egui application
    let app_state = bitcoin_state.clone();
    eframe::run_native(
        "Bitcoin Metrics",
        eframe::NativeOptions {
            // Use viewport to set the window size in newer eframe versions
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([700.0, 480.0]), // Match chart width (250px height * 2.5 aspect ratio)
            ..Default::default()
        },
        Box::new(move |_cc| {
            #[cfg(not(target_os = "linux"))]
            {
                // Create menu items
                let mut menu = Menu::new();
                
                // Create refresh menu item with direct callback
                let refresh_state = app_state.clone();
                let refresh_i = MenuItem::new("Refresh BTC Price", true, Some(Box::new(move || {
                    let state = refresh_state.clone();
                    thread::spawn(move || {
                        refresh_bitcoin_price(state);
                    });
                })));
                
                // Create quit menu item with direct callback
                let quit_i = MenuItem::new("Quit", true, Some(Box::new(|| {
                    std::process::exit(0);
                })));
                
                let _ = menu.append_items(&[&refresh_i, &quit_i]);
                
                // Create the tray icon
                tray_c
                    .borrow_mut()
                    .replace(TrayIconBuilder::new()
                        .with_menu(Box::new(menu))
                        .with_tooltip("BTC Ticker")
                        .with_icon(icon)
                        .build()
                        .unwrap());
            }
            Ok(Box::new(BitcoinApp::new(app_state)) as Box<dyn eframe::App>)
        }),
    )
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

// Helper function to get formatted timestamp
fn get_current_timestamp() -> String {
    let dt = chrono::Local::now();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

// Helper function to refresh Bitcoin price
fn refresh_bitcoin_price(state: Arc<Mutex<BitcoinState>>) {
    println!("Refreshing Bitcoin price and historical data...");
    // Mark as updating
    {
        let mut state = state.lock().unwrap();
        state.updating = true;
    }
    
    // Create a reusable API client
    let client = BitstampClient::new();
    
    // First fetch the current price
    match client.fetch_current_price() {
        Ok(price) => {
            println!("Updated BTC price: ${:.2}", price);
            let mut state = state.lock().unwrap();
            
            // Set flag if price changed
            if state.price != price {
                state.new_price_fetched = true;
            }
            
            state.price = price;
            state.last_updated = get_current_timestamp();
            state.updating = false;
        },
        Err(e) => {
            eprintln!("Failed to fetch BTC price: {}", e);
            let mut state = state.lock().unwrap();
            state.updating = false;
            
            // If we have historical data, we can use the latest price as a fallback
            if !state.historical_data.is_empty() {
                if let Some((_, latest_candle)) = state.historical_data.last() {
                    println!("Using last historical price as fallback: ${:.2}", latest_candle.close);
                    state.price = latest_candle.close;
                    state.last_updated = format!("{}* (fallback)", get_current_timestamp());
                }
            }
            
            return; // Exit early if we couldn't fetch the current price
        }
    }
    
    // Then fetch historical data to update the chart
    let timeframe;
    {
        // Temporarily lock state to get current timeframe
        let locked_state = state.lock().unwrap();
        timeframe = locked_state.chart_timeframe;
    }
    
    match BitstampClient::new().fetch_historical_prices(timeframe) {
        Ok(historical_data) => {
            let mut history = Vec::new();
            
            // Convert historical data to our format
            for point in historical_data.data.ohlc.iter() {
                // Parse the timestamp
                if let Ok(timestamp) = point.timestamp.parse::<i64>() {
                    if let Some(datetime) = Utc.timestamp_opt(timestamp, 0).single() {
                        // Create candle data structure using values from BitstampOHLC
                        let candle = CandleData {
                            open: point.open.parse::<f64>().unwrap_or(0.0),
                            high: point.high.parse::<f64>().unwrap_or(0.0),
                            low: point.low.parse::<f64>().unwrap_or(0.0),
                            close: point.close.parse::<f64>().unwrap_or(0.0)
                        };
                        
                        // Create human-readable time for debugging
                        let human_time: String = format_unix_timestamp(&point.timestamp);
                        
                        history.push((TimeInfo {
                            raw_timestamp: timestamp,
                            formatted_time: human_time,
                            rfc3339: datetime.to_rfc3339(),
                        }, candle));
                    }
                }
            }
            
            // Update historical data in state
            if !history.is_empty() {
                let mut state = state.lock().unwrap();
                state.historical_data = history;
                state.new_price_fetched = true; // Force chart update
            }
        },
        Err(e) => {
            eprintln!("Failed to fetch historical data: {}", e);
        }
    }
}

// Helper function to format Unix timestamp to date-time format (YYYY-MM-DD HH:MM)
fn format_unix_timestamp(unix_timestamp_str: &str) -> String {
    bitstamp_client::format_unix_timestamp(unix_timestamp_str)
}

// Wrapper function for backward compatibility
fn fetch_historical_bitcoin_prices(timeframe: ChartTimeframe) -> Result<bitstamp_client::BitstampHistoricalData> {
    BitstampClient::new().fetch_historical_prices(timeframe)
}