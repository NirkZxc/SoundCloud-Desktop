use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent, MouseButton};
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TrayState {
    #[serde(default)]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(skip)]
    pub tray_icon: Option<Arc<Mutex<Option<TrayIcon>>>>,
}

impl TrayState {
    pub fn load(app_data_dir: &PathBuf) -> Self {
        let path = app_data_dir.join("tray_settings.json");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, app_data_dir: &PathBuf) -> Result<(), String> {
        let path = app_data_dir.join("tray_settings.json");
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn setup_tray(app: &tauri::AppHandle, tray_state: &mut TrayState) -> Result<(), String> {
    println!("[TRAY] Setting up system tray");

    // Create tray menu
    let show_item = MenuItemBuilder::new("Show SoundCloud")
        .id("show")
        .build(app)
        .map_err(|e| e.to_string())?;

    let settings_item = MenuItemBuilder::new("Settings")
        .id("settings")
        .build(app)
        .map_err(|e| e.to_string())?;

    let separator = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

    let quit_item = MenuItemBuilder::new("Quit")
        .id("quit")
        .build(app)
        .map_err(|e| e.to_string())?;

    let menu = MenuBuilder::new(app)
        .items(&[&show_item, &settings_item, &separator, &quit_item])
        .build()
        .map_err(|e| e.to_string())?;

    // Create tray icon
    let app_handle = app.clone();
    let tray_icon = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().ok_or("Failed to get default icon")?)
        .menu(&menu)
        .on_menu_event(move |_tray, event| {
            println!("[TRAY] Menu event: {:?}", event.id);
            match event.id.as_ref() {
                "show" => {
                    println!("[TRAY] Show clicked");
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "settings" => {
                    println!("[TRAY] Settings clicked");
                    let app_handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = crate::commands::open_settings_window(app_handle).await;
                    });
                }
                "quit" => {
                    println!("[TRAY] Quit clicked");
                    app_handle.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(move |tray, event| {
            println!("[TRAY] Tray icon event: {:?}", event);
            match event {
                TrayIconEvent::Click { button: MouseButton::Left, .. } => {
                    // Left click - show/hide main window
                    println!("[TRAY] Left click - toggling window visibility");
                    let app_handle = tray.app_handle();
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let is_visible = window.is_visible().unwrap_or(false);
                        if is_visible {
                            println!("[TRAY] Hiding window");
                            let _ = window.hide();
                        } else {
                            println!("[TRAY] Showing window");
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                TrayIconEvent::Click { button: MouseButton::Right, .. } => {
                    // Right click - menu is shown automatically
                    println!("[TRAY] Right click - showing menu");
                }
                _ => {}
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    println!("[TRAY] System tray created successfully");
    tray_state.tray_icon = Some(Arc::new(Mutex::new(Some(tray_icon))));
    Ok(())
}