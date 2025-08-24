use crate::file_manager::FileSort;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub output_dir: Option<PathBuf>,
    pub skip_noop: bool,
    pub theme_dark: bool,
    pub worker_count: usize,
    pub file_sort: FileSort,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            skip_noop: true,
            theme_dark: true,
            worker_count: num_cpus::get().max(1),
            file_sort: FileSort::DateAdded,
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn load() -> AppConfig {
        match Self::load_config() {
            Ok(config) => config,
            Err(_) => {
                // Config doesn't exist or is invalid, create default and save it
                let default_config = AppConfig::default();
                let _ = Self::save(&default_config); // Ignore save errors on first run
                default_config
            }
        }
    }

    pub fn save(config: &AppConfig) -> Result<(), String> {
        let config_path = Self::get_config_path()?;
        let config_json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&config_path, config_json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    fn load_config() -> Result<AppConfig, String> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Err("Config file does not exist".to_string());
        }

        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: AppConfig = serde_json::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

    fn get_config_path() -> Result<PathBuf, String> {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;
        let exe_dir = exe_path
            .parent()
            .ok_or("Failed to get executable directory")?;
        Ok(exe_dir.join("config.json"))
    }
}
