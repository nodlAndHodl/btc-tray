#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::Result;
use serde::Deserialize;

use tray_icon::{
    menu::{AboutMetadata, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};

use eframe::egui;

#[derive(Debug, Deserialize)]
struct BitstampResponse {
    last: String,      // Price is returned as a string in Bitstamp API
    high: String,      // Highest price in last 24h
    low: String,       // Lowest price in last 24h
    volume: String,    // Volume in last 24h
    timestamp: String, // Server timestamp
}

// Used for the original event system
enum UserEvent {
    TrayIconEvent(tray_icon::TrayIconEvent),
    MenuEvent(tray_icon::menu::MenuEvent),
    RefreshPrice,
}

// Shared state between the tray icon and the egui app
struct BitcoinState {
    price: f64,
    last_updated: String,
    updating: bool,
}

// The egui application
struct BitcoinApp {
    state: Arc<Mutex<BitcoinState>>,
    show_history: bool,
    price_history: Vec<(String, f64)>,
}

impl BitcoinApp {
    fn new(state: Arc<Mutex<BitcoinState>>) -> Self {
        Self {
            state,
            show_history: false,
            price_history: Vec::new(),
        }
    }

    fn update_price_history(&mut self) {
        let state = self.state.lock().unwrap();
        if state.price > 0.0 && !state.updating {
            // Only add to history if we have a valid price and not currently updating
            self.price_history.push((state.last_updated.clone(), state.price));
            
            // Keep only last 100 entries
            if self.price_history.len() > 100 {
                self.price_history.remove(0);
            }
        }
    }
}

impl eframe::App for BitcoinApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_price_history();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Bitcoin Price Tracker");
            
            let state = self.state.lock().unwrap();
            let price_text = if state.price > 0.0 {
                format!("${:.2}", state.price)
            } else {
                "Loading...".to_string()
            };
            
            ui.add_space(20.0);
            ui.heading(price_text);
            ui.add_space(5.0);
            ui.label(format!("Last updated: {}", state.last_updated));
            
            ui.add_space(10.0);
            if ui.button("Refresh Price").clicked() {
                // We'll handle the actual refresh from outside the egui app
                drop(state); // Release the mutex before the thread operation
                let state_clone = self.state.clone();
                thread::spawn(move || {
                    refresh_bitcoin_price(state_clone);
                });
            }
            
            ui.add_space(20.0);
            ui.checkbox(&mut self.show_history, "Show Price History");
            
            if self.show_history && !self.price_history.is_empty() {
                ui.add_space(10.0);
                ui.label("Price History:");
                ui.add_space(5.0);
                
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for (time, price) in self.price_history.iter().rev() {
                        ui.label(format!("{}: ${:.2}", time, price));
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
    let bitcoin_state = Arc::new(Mutex::new(BitcoinState {
        price: 0.0,
        last_updated: "Not yet updated".to_string(),
        updating: false,
    }));
    
    // Get initial price
    let init_state = bitcoin_state.clone();
    thread::spawn(move || {
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
            let show_i = MenuItem::new("Show Window", true, None);
            let quit_i = MenuItem::with_id("quit-app", "Quit", true, None);
            
            // Create a clone of the state for the menu event handler thread
            let state_for_menu_events = linux_state.clone();
            
            // Add items to the menu
            tray_menu.append_items(&[
                &PredefinedMenuItem::about(
                    None,
                    Some(AboutMetadata {
                        name: Some("BTC Ticker".to_string()),
                        copyright: Some("Copyright BTC Ticker".to_string()),
                        ..Default::default()
                    }),
                ),
                &refresh_i,
                &show_i,
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
        "Bitcoin Price Tracker",
        eframe::NativeOptions {
            // Use viewport to set the window size in newer eframe versions
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([360.0, 480.0]),
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
            Box::new(BitcoinApp::new(app_state))
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
    // Mark as updating
    {
        let mut state = state.lock().unwrap();
        state.updating = true;
    }
    
    match fetch_bitcoin_price() {
        Ok(price) => {
            println!("Updated BTC price: ${:.2}", price);
            let mut state = state.lock().unwrap();
            state.price = price;
            state.last_updated = get_current_timestamp();
            state.updating = false;
        },
        Err(e) => {
            eprintln!("Failed to fetch BTC price: {}", e);
            let mut state = state.lock().unwrap();
            state.updating = false;
        }
    }
}

fn fetch_bitcoin_price() -> Result<f64> {
    // Use Bitstamp API to get BTC/USD ticker
    let url = "https://www.bitstamp.net/api/v2/ticker/btcusd/";
    let response = reqwest::blocking::get(url)?
        .json::<BitstampResponse>()?;
        
    // Convert the price string to a float
    let price = response.last.parse::<f64>()?;
    Ok(price)
}