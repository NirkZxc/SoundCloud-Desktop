use std::env;
use tauri::{
    Manager, WebviewWindowBuilder, WebviewUrl, WindowEvent,
    Position, LogicalPosition
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

mod state;
mod commands;
mod modules;

use state::{AppState, get_app_data_dir, load_settings, save_settings_to_disk};
use commands::*;

fn main() {
    // Set up the WebView2 user data folder in AppData
    let app_data_dir = get_app_data_dir();
    env::set_var("WEBVIEW2_USER_DATA_FOLDER", app_data_dir.clone());

    let app_state = AppState::new(&app_data_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_app_data_dir_cmd,
            get_settings,
            save_settings,
            toggle_fullscreen,
            set_window_size,
            get_window_state,
            open_settings_window,
            close_settings_window,
            tray_get_state,
            tray_set_minimize_to_tray,
            tray_set_start_minimized,
            tray_show_main_window,
        ])
        .setup(|app| {
            println!("[SETUP] App setup started");

            // Initialize tray
            {
                let state = app.state::<AppState>();
                let mut tray_state = state.tray.lock().unwrap();
                modules::tray::setup_tray(app.handle(), &mut tray_state)?;
            }

            // Register global shortcut for F11 (fullscreen toggle)
            app.global_shortcut().on_shortcut("F11", move |_app, _shortcut, event| {
                println!("[SHORTCUT] F11 pressed, state: {:?}", event.state);
                if event.state == ShortcutState::Pressed {
                    let app = _app.clone();
                    tauri::async_runtime::spawn(async move {
                        println!("[SHORTCUT] Calling toggle_fullscreen");
                        let _ = toggle_fullscreen(app).await;
                    });
                }
            })?;

            // Register global shortcut for Ctrl+, (settings)
            app.global_shortcut().on_shortcut("Ctrl+Comma", move |_app, _shortcut, event| {
                println!("[SHORTCUT] Ctrl+, pressed, state: {:?}", event.state);
                if event.state == ShortcutState::Pressed {
                    let app = _app.clone();
                    tauri::async_runtime::spawn(async move {
                        println!("[SHORTCUT] Calling open_settings_window");
                        let _ = open_settings_window(app).await;
                    });
                }
            })?;

            // Load settings
            let state = app.state::<AppState>();
            let mut settings = state.settings.lock().unwrap().clone();
            println!("[SETUP] Loaded settings: {:?}", settings);

            // Validate and fix loaded settings
            const HIDDEN_POS: i32 = -32000;

            // Ensure minimum window size
            if settings.window_width < 800 {
                println!("[SETUP] Fixing invalid window_width: {} -> 1200", settings.window_width);
                settings.window_width = 1200;
            }
            if settings.window_height < 600 {
                println!("[SETUP] Fixing invalid window_height: {} -> 800", settings.window_height);
                settings.window_height = 800;
            }

            // Don't use -32000 position (Windows hidden window position)
            if settings.window_x == Some(HIDDEN_POS) {
                println!("[SETUP] Ignoring invalid window_x: -32000");
                settings.window_x = None;
            }
            if settings.window_y == Some(HIDDEN_POS) {
                println!("[SETUP] Ignoring invalid window_y: -32000");
                settings.window_y = None;
            }

            // Save corrected settings back
            let app_data_dir = get_app_data_dir();
            let _ = save_settings_to_disk(&app_data_dir, &settings);

            // Create main window with initialization script for OAuth popup handling
            let init_script = r#"
                (function() {
                  const originalOpen = window.open;

                  window.open = function(url, target, features) {
                    console.log('[OAUTH] window.open called:', url, target, features);

                    // Create a fake popup object
                    let closed = false;
                    const popup = {
                      closed: false,
                      close: function() {
                        closed = true;
                        this.closed = true;
                        console.log('[OAUTH] popup.close() called');
                      },
                      location: {
                        href: '',
                        set href(value) {
                          console.log('[OAUTH] popup.location.href set to:', value);
                          this.href = value;

                          // Check if this is an OAuth URL
                          if (typeof value === 'string' && (
                            value.includes('accounts.google.com') ||
                            value.includes('oauth') ||
                            value.includes('auth') ||
                            value.includes('signin') ||
                            value.includes('login')
                          )) {
                            console.log('[OAUTH] OAuth URL detected, navigating main window to:', value);
                            // Navigate the MAIN window to the OAuth URL
                            window.location.href = value;
                          }
                        },
                        get href() {
                          return this.href;
                        },
                        toString: function() { return this.href; }
                      },
                      postMessage: function() {},
                      focus: function() {},
                      blur: function() {},
                      name: target || '_blank'
                    };

                    // Make closed a getter/setter
                    Object.defineProperty(popup, 'closed', {
                      get: function() { return closed; },
                      set: function(v) { closed = v; },
                      configurable: true
                    });

                    console.log('[OAUTH] Returning fake popup object');
                    return popup;
                  };

                  console.log('[OAUTH] window.open overridden successfully');
                })();
            "#;

            let main_window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://soundcloud.com".parse().unwrap())
            )
            .title("SoundCloud Desktop")
            .inner_size(settings.window_width as f64, settings.window_height as f64)
            .min_inner_size(800.0, 600.0)
            .resizable(true)
            .decorations(true)
            .center()
            .data_directory(app_data_dir)
            .devtools(true)
            .initialization_script(init_script)
            .build()?;

            println!("[SETUP] Main window created, label: {}", main_window.label());

            // Apply saved window position (only if valid and not hidden position)
            if let (Some(x), Some(y)) = (settings.window_x, settings.window_y) {
                if x != HIDDEN_POS && y != HIDDEN_POS {
                    let _ = main_window.set_position(Position::Logical(LogicalPosition::new(x as f64, y as f64)));
                    println!("[SETUP] Applied window position: ({}, {})", x, y);
                } else {
                    println!("[SETUP] Skipping hidden window position: ({}, {})", x, y);
                }
            }

            // Apply fullscreen state if needed
            if settings.window_mode == "fullscreen" {
                let _ = main_window.set_fullscreen(true);
                println!("[SETUP] Applied fullscreen mode");
            }

            // Check if we should start minimized
            let tray_state = state.tray.lock().unwrap();
            if tray_state.start_minimized {
                let _ = main_window.hide();
                println!("[SETUP] Started minimized to tray");
            }

            println!("[SETUP] App setup completed");
            Ok(())
        })
        .on_window_event(|window, event| {
            // Save window state when it changes (main window only)
            if window.label() == "main" {
                match event {
                    WindowEvent::Resized(size) => {
                        println!("[WINDOW] Resized: {}x{}", size.width, size.height);
                        let app_handle = window.app_handle().clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = save_window_state(app_handle).await;
                        });
                    }
                    WindowEvent::Moved(pos) => {
                        println!("[WINDOW] Moved: ({}, {})", pos.x, pos.y);
                        let app_handle = window.app_handle().clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = save_window_state(app_handle).await;
                        });
                    }
                    WindowEvent::CloseRequested { api, .. } => {
                        println!("[WINDOW] CloseRequested");
                        let state = window.app_handle().state::<AppState>();
                        let tray_state = state.tray.lock().unwrap();
                        if tray_state.minimize_to_tray {
                            println!("[WINDOW] Minimizing to tray instead of closing");
                            api.prevent_close();
                            let _ = window.hide();
                        }
                    }
                    _ => {}
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}