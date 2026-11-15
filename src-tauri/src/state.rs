use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::modules::tray::TrayState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub window_mode: String,
    pub window_width: u32,
    pub window_height: u32,
    pub custom_width: Option<u32>,
    pub custom_height: Option<u32>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            window_mode: "windowed".to_string(),
            window_width: 1200,
            window_height: 800,
            custom_width: None,
            custom_height: None,
            window_x: None,
            window_y: None,
        }
    }
}

pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub tray: Arc<Mutex<TrayState>>,
}

impl AppState {
    pub fn new(app_data_dir: &PathBuf) -> Self {
        Self {
            settings: Arc::new(Mutex::new(crate::load_settings(app_data_dir).unwrap_or_default())),
            tray: Arc::new(Mutex::new(TrayState::load(app_data_dir))),
        }
    }
}

pub fn get_app_data_dir() -> PathBuf {
    let mut path = PathBuf::new();
    if let Ok(app_data) = std::env::var("APPDATA") {
        path.push(app_data);
    } else {
        path.push(".");
    }
    path.push("SoundCloud-Desktop");
    path
}

pub fn get_settings_path(app_data_dir: &PathBuf) -> PathBuf {
    let mut path = app_data_dir.clone();
    path.push("settings.json");
    path
}

pub fn load_settings(app_data_dir: &PathBuf) -> Result<AppSettings, String> {
    let path = get_settings_path(app_data_dir);
    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let settings: AppSettings = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(settings)
    } else {
        Ok(AppSettings::default())
    }
}

pub fn save_settings_to_disk(app_data_dir: &PathBuf, settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path(app_data_dir);
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}