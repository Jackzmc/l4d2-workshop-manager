use std::sync::{Arc, Mutex};
use steam_workshop_api::{SteamWorkshop, WorkshopItem};
use crate::config::SettingsManager;

pub struct DownloadManager {
    ws: SteamWorkshop,
    downloads: Vec<WorkshopItem>,
    config: Arc<Mutex<SettingsManager>>
}

impl DownloadManager {
    pub fn new(ws: SteamWorkshop, config: Arc<Mutex<SettingsManager>>) -> Self {
        Self {
            downloads: vec![],
            ws,
            config
        }
    }

    pub fn count(&self) -> usize {
        self.downloads.len()
    }

    pub fn add_item(&mut self, item: WorkshopItem) -> usize {
        self.add_item(item)
    }

    pub fn add_item_by_id(&mut self, id: u32) -> Result<usize, String> {
        // TODO: resolve
        let mut items = self.ws.get_published_file_details(&[id.to_string()])
            .map_err(|e| e.to_string())?;
        if items.len() == 0 {
            return Err("No workshop id found with that id. May be private, or deleted".to_string());
        }
        let item = items.remove(0);
        Ok(self.add_item(item))
    }

    pub fn clear(&mut self) {
        self.downloads.clear()
    }

    pub fn process(&mut self) {
        if self.downloads.len() == 0 { return; }
        let item = self.downloads.pop().unwrap();
        let dest_file_path = self.config.lock().unwrap().get().gamedir.as_ref().unwrap().join(format!("{}.vpk", item.publishedfileid));
        let mut part_file_path = dest_file_path.clone();
        
    }
}