use tauri::{AppHandle, Manager, WebviewWindowBuilder, WebviewUrl, Size, LogicalSize};

use crate::state::{AppState, AppSettings, get_app_data_dir, save_settings_to_disk};
use crate::modules::tray::TrayState;

#[tauri::command]
pub async fn get_app_data_dir_cmd() -> Result<String, String> {
    Ok(get_app_data_dir().to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    Ok(state.settings.lock().unwrap().clone())
}

#[tauri::command]
pub async fn save_settings(state: tauri::State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    let mut s = state.settings.lock().unwrap();
    *s = settings.clone();
    let app_data_dir = get_app_data_dir();
    save_settings_to_disk(&app_data_dir, &settings)
}

#[tauri::command]
pub async fn toggle_fullscreen(app: AppHandle) -> Result<bool, String> {
    println!("[COMMAND] toggle_fullscreen called");
    if let Some(window) = app.get_webview_window("main") {
        let is_fullscreen = window.is_fullscreen().map_err(|e| {
            println!("[COMMAND] toggle_fullscreen is_fullscreen error: {}", e);
            e.to_string()
        })?;
        println!("[COMMAND] Current fullscreen state: {}, toggling to: {}", is_fullscreen, !is_fullscreen);
        window.set_fullscreen(!is_fullscreen).map_err(|e| {
            println!("[COMMAND] toggle_fullscreen set_fullscreen error: {}", e);
            e.to_string()
        })?;
        Ok(!is_fullscreen)
    } else {
        println!("[COMMAND] toggle_fullscreen: Main window not found!");
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
pub async fn set_window_size(app: AppHandle, width: u32, height: u32) -> Result<(), String> {
    println!("[COMMAND] set_window_size called: {}x{}", width, height);
    if let Some(window) = app.get_webview_window("main") {
        if width < 800 || height < 600 {
            return Err("Window size must be at least 800x600".to_string());
        }

        let monitor = window.current_monitor().map_err(|e| e.to_string())?;
        if let Some(monitor) = monitor {
            let monitor_size = monitor.size();
            let scale_factor = monitor.scale_factor();
            let max_width = (monitor_size.width as f64 / scale_factor) as u32;
            let max_height = (monitor_size.height as f64 / scale_factor) as u32;

            let final_width = width.min(max_width);
            let final_height = height.min(max_height);

            window.set_size(Size::Logical(LogicalSize::new(
                final_width as f64,
                final_height as f64,
            ))).map_err(|e| e.to_string())?;

            let state = app.state::<AppState>();
            let mut settings = state.settings.lock().unwrap();
            settings.window_width = final_width;
            settings.window_height = final_height;
            let app_data_dir = get_app_data_dir();
            save_settings_to_disk(&app_data_dir, &settings)?;
        }
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
pub async fn get_window_state(app: AppHandle) -> Result<serde_json::Value, String> {
    if let Some(window) = app.get_webview_window("main") {
        let size = window.inner_size().map_err(|e| e.to_string())?;
        let is_fullscreen = window.is_fullscreen().map_err(|e| e.to_string())?;
        let position = window.outer_position().ok();

        Ok(serde_json::json!({
            "width": size.width,
            "height": size.height,
            "isFullscreen": is_fullscreen,
            "position": position.map(|p| serde_json::json!({ "x": p.x, "y": p.y }))
        }))
    } else {
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
pub async fn open_settings_window(app: AppHandle) -> Result<(), String> {
    println!("[COMMAND] open_settings_window called");
    if let Some(existing) = app.get_webview_window("settings") {
        println!("[COMMAND] Settings window already exists, focusing it");
        let _ = existing.set_focus();
        return Ok(());
    }

    let app_data_dir = get_app_data_dir();

    let settings = WebviewWindowBuilder::new(
        &app,
        "settings",
        WebviewUrl::App("settings.html".into())
    )
    .title("Settings - SoundCloud Desktop")
    .inner_size(480.0, 520.0)
    .min_inner_size(440.0, 480.0)
    .resizable(false)
    .decorations(true)
    .center()
    .data_directory(app_data_dir)
    .build()
    .map_err(|e| {
        println!("[COMMAND] open_settings_window build error: {}", e);
        e.to_string()
    })?;

    println!("[COMMAND] Settings window created successfully: {}", settings.label());
    Ok(())
}

#[tauri::command]
pub async fn close_settings_window(app: AppHandle) -> Result<(), String> {
    println!("[COMMAND] close_settings_window called");
    if let Some(window) = app.get_webview_window("settings") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// Tray commands
#[tauri::command]
pub async fn tray_get_state(state: tauri::State<'_, AppState>) -> Result<TrayState, String> {
    let settings = state.tray.lock().unwrap().clone();
    Ok(settings)
}

#[tauri::command]
pub async fn tray_set_minimize_to_tray(
    state: tauri::State<'_, AppState>,
    minimize: bool,
) -> Result<(), String> {
    let mut settings = state.tray.lock().unwrap();
    settings.minimize_to_tray = minimize;

    let app_data_dir = get_app_data_dir();
    settings.save(&app_data_dir)
}

#[tauri::command]
pub async fn tray_set_start_minimized(
    state: tauri::State<'_, AppState>,
    start_minimized: bool,
) -> Result<(), String> {
    let mut settings = state.tray.lock().unwrap();
    settings.start_minimized = start_minimized;

    let app_data_dir = get_app_data_dir();
    settings.save(&app_data_dir)
}

#[tauri::command]
pub async fn tray_show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

pub async fn save_window_state(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let state = app.state::<AppState>();
        let mut settings = state.settings.lock().unwrap();

        let size = window.inner_size().map_err(|e| e.to_string())?;

        // Don't save zero or below-minimum sizes (WebView2 initialization artifact)
        if size.width < 800 || size.height < 600 {
            println!("[WINDOW] Ignoring invalid size: {}x{}", size.width, size.height);
        } else {
            settings.window_width = size.width;
            settings.window_height = size.height;
        }

        let position = window.outer_position().ok();
        if let Some(pos) = position {
            // Don't save -32000 position (Windows hidden window position)
            const HIDDEN_POS: i32 = -32000;
            if pos.x != HIDDEN_POS && pos.y != HIDDEN_POS {
                settings.window_x = Some(pos.x);
                settings.window_y = Some(pos.y);
            } else {
                println!("[WINDOW] Ignoring hidden window position: ({}, {})", pos.x, pos.y);
            }
        }

        let is_fullscreen = window.is_fullscreen().map_err(|e| e.to_string())?;
        settings.window_mode = if is_fullscreen { "fullscreen" } else { "windowed" }.to_string();

        let app_data_dir = get_app_data_dir();
        save_settings_to_disk(&app_data_dir, &settings)
    } else {
        Err("Main window not found".to_string())
    }
}