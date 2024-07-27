use std::{path::PathBuf, fs};
use log::debug;
use serde::{Deserialize, Serialize};

#[cfg(debug_assertions)]
const APPDATA_FOLDER_NAME: &str = "l4d2-workshop-dev";
#[cfg(not(debug_assertions))]
const APPDATA_FOLDER_NAME: &str = "l4d2-workshop-downloader";

pub fn get_appdir() -> PathBuf {

    let folder = dirs::config_dir().expect("Could not find a valid config folder").join(APPDATA_FOLDER_NAME);
    if !folder.exists() {
        fs::create_dir_all(&folder).expect("Could not create config folder");
    }
    return folder;
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Settings {
    pub gamedir: Option<PathBuf>,
    pub version: Option<String>,
    pub steam_apikey: Option<String>,
    pub telemetry: bool
}
pub struct SettingsManager {
    config_path: PathBuf,
    settings: Settings
}

impl SettingsManager {
    pub fn new() -> Self {
        Self {
            config_path: get_appdir().join("config.json"),
            settings: Settings::default(),
        }
    }
    pub fn load(&mut self) -> Result<bool, String> {
        debug!("Loading settings from {:?}", self.config_path);
        if !self.config_path.exists() {
            return Ok(false)
        }
        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| e.to_string())?;
        self.settings = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;
        Ok(true)
    }

    pub fn replace(&mut self, settings: Settings) {
        self.settings = settings;
    }

    pub fn get_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    pub fn get(&self) -> &Settings {
        &self.settings
    }

    pub fn get_clone(&self) -> Settings {
        self.settings.clone()
    }

    pub fn save(&mut self) -> Result<(), String> {
        self.settings.version = Some(env!("CARGO_PKG_VERSION").to_string());
        let content = serde_json::to_string(&self.settings)
            .map_err(|e| e.to_string())?;
        debug!("saving:\n{}", content);
        fs::write(&self.config_path, content)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

