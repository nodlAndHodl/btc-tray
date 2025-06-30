# Tray Icon App

A simple tray icon application built with Rust using the tray-icon and tao crates.

## Features

- System tray icon with tooltip
- Tray menu with About, Show Message, and Quit options
- Event handling for tray icon and menu interactions

## Running the Application

To run the application:

```bash
cargo run
```

The application will show an icon in your system tray. Right-click on the icon to display the menu.

## Menu Options

- **About**: Shows information about the application
- **Show Message**: Prints a message to the console
- **Quit**: Exits the application

## Requirements

- Rust 1.71 or newer
- For Linux: GTK 3 development libraries
