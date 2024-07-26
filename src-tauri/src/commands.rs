use log::{debug, error};
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