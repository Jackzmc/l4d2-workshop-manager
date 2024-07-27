use log::{debug, error};
use steam_workshop_api::WorkshopItem;
use crate::{config, Data, util};

#[tauri::command]
pub fn get_my_addons(state: tauri::State<'_, Data>) -> Result<Vec<util::AddonEntry>, String> {
    let settings = state.settings.lock().unwrap();
    let path = settings.get().gamedir.as_ref().unwrap().to_owned();
    let ws = state.workshop.clone();
    util::get_addons(&ws, &path)
}

#[tauri::command]
pub fn get_workshop_addons(state: tauri::State<'_, Data>) -> Result<Vec<util::AddonEntry>, String> {
    let settings = state.settings.lock().unwrap();
    let path = settings.get().gamedir.as_ref().unwrap().join("workshop");
    let ws = &state.workshop.clone();
    util::get_addons(&ws, &path)
}

#[tauri::command]
pub fn get_latest_workshop_info(state: tauri::State<'_, Data>, publishedfileid: u32) -> Result<WorkshopItem, String> {
    let ws = &state.workshop.clone();
    let entries = vec![publishedfileid.to_string()];
    debug!("ws.get_published_file_details {:?}", entries);
    let mut latest_info = ws.get_published_file_details(&entries)
        .map_err(|e| e.to_string())?;
    if latest_info.len() == 0 {
        // TODO: mark this in the cache, that it's deleted?
        return Err("Could not find workshop info, may have been deleted or made private".to_string());
    }
    debug!("got latest info for {}", publishedfileid);
    let latest_info = latest_info.remove(0);
    Ok(latest_info)

}

#[tauri::command]
pub fn get_settings(state: tauri::State<Data>) -> config::Settings {
    let settings = state.settings.lock().unwrap();
    settings.get_clone()
}

#[tauri::command]
pub fn save_settings(state: tauri::State<Data>, changed: config::Settings) -> Result<(), String> {
    debug!("saving settings");
    let mut settings = state.settings.lock().unwrap();
    settings.replace(changed);
    if let Err(e) = settings.save() {
        error!("Error saving changes: {}", e);
        return Err(e)
    }
    Ok(())
}