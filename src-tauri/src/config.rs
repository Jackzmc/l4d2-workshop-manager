use std::{path::PathBuf, io, fs};
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

#[derive(Serialize, Deserialize, Clone)]
pub struct DownloadEntry {
    pub title: String,
    pub publishedfileid: String,
    pub time_updated: usize
}
pub struct Downloads {
    downloads: Vec<DownloadEntry>
}

impl DownloadEntry {
    pub fn from_item(item: &steam_workshop_api::WorkshopItem) -> DownloadEntry {
        DownloadEntry {
            title: item.title.clone(),
            publishedfileid: item.publishedfileid.clone(),
            time_updated: item.time_updated
        }
    }
}

#[allow(dead_code)]
impl Downloads {
    pub fn new() -> Downloads {
        Downloads {
            downloads: Vec::new()
        }
    }
    pub fn load() -> Result<Downloads, String> {
        let path = Downloads::get_path();
        if !path.exists() {
            let dl = Downloads::new();
            dl.save().ok();
            return Ok(dl)
        }
        match fs::File::open(path) {
            Ok(file) => {
                let reader = io::BufReader::new(file);
                match serde_json::from_reader(reader) {
                    Ok(json) => {
                        return Ok(Downloads {
                            downloads: json
                        });
                    },
                    Err(_e) => Err("Could not parse JSON".to_owned())
                }
            },
            Err(e) => return Err(e.to_string())
        }
    }
    pub fn get(&self, index: usize) -> Option<&DownloadEntry> {
        self.downloads.get(index)
    }
    pub fn get_id_index(&self, id: &str) -> Option<usize> {
        for (i, item) in self.downloads.iter().enumerate() {
            if item.publishedfileid == id {
                return Some(i)
            }
        }
        return None
    }
    pub fn get_download(&self, id: &str) -> Option<&DownloadEntry> {
        for item in &self.downloads {
            if item.publishedfileid == id {
                return Some(item)
            }
        }
        return None
    }

    pub fn find_download(&self, item: &DownloadEntry) -> Option<usize> {
        for (i, itm) in self.downloads.iter().enumerate() {
            if itm.publishedfileid == item.publishedfileid {
                return Some(i);
            }
        }
        return None
    }

    pub fn update_download(&mut self, item: DownloadEntry) {
        for (i, itm) in self.downloads.iter().enumerate() {
            if itm.publishedfileid == item.publishedfileid {
                self.downloads[i] = item;
                break;
            }
        }
    }

    pub fn size(&self) -> usize {
        self.downloads.len()
    }

    pub fn set_download(&mut self, index: usize, item: DownloadEntry) {
        self.downloads[index] = item;
    }

    pub fn add_download(&mut self, item: DownloadEntry) {
        self.downloads.push(item);
    }

    pub fn save(&self) -> Result<(), String>{
        fs::write(Downloads::get_path(), serde_json::to_string(&self.downloads).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_path() -> PathBuf {
        get_appdir().join("downloads.json")
    }
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

