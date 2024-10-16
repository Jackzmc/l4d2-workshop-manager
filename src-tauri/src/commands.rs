use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use futures::StreamExt;
use log::{debug, error};
use regex::Regex;
use steam_workshop_api::{SearchOptions, WorkshopItem};
use tauri::Window;
use crate::{config, Data, ErrorPayload, UpdatePayload, util};
use crate::util::AddonEntry;

#[allow(dead_code)]

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
    match util::get_workshop_info(ws, publishedfileid) {
        Ok(None) => Err("Could not find workshop info, may have been deleted or made private".to_string()),
        Err(e) => Err(e),
        Ok(Some(item)) => Ok(item)
    }
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

pub static WORKSHOP_URL_REGEX: OnceLock<Regex> = OnceLock::new();
#[tauri::command]
pub fn search_workshop(state: tauri::State<Data>, query: &str) -> Result<Vec<WorkshopItem>, String> {
    // TODO: strip out url, and search for publishedfileid directly

    let ws = &state.workshop.clone();
    ws.search_items(&SearchOptions {
        count: 30,
        app_id: 550,
        query: query.to_string(),
        cursor: None,
        required_tags: None,
        excluded_tags: None,
    })
        .map_err(|e| e.to_string())
        .map(|r| r.items)
}

#[tauri::command]
async fn download_addon(state: tauri::State<'_, Data>, window: Window, published_file_id: u32) -> Result<(), String> {
    let config = &state.settings;
    let dest_file_path = config.lock().unwrap().get().gamedir.as_ref().unwrap().join(format!("{}.vpk", published_file_id));
    let mut part_file_path = dest_file_path.clone();
    part_file_path.set_file_name(format!("{}.vpk.part", published_file_id));
    let mut file = {
        std::fs::File::create(&part_file_path).expect("Could not create part file")
    };
    let item = util::get_workshop_info(&state.workshop.clone(), published_file_id)?;
    let item = item.ok_or_else(|| "Could not find workshop item".to_string() )?;
    // TODO: get workshop item
    let mut bytes_downloaded: usize = 0;
    let bytes_total: usize = item.file_size.parse().unwrap();
    debug!("Starting download of id={} title={} bytes_total={}", published_file_id, item.title, bytes_total);
    match reqwest::Client::new()
        .get(&item.file_url.unwrap())
        .header("User-Agent", "L4D2-Workshop-Downloader")
        .send()
        .await
    {
        Ok(response) => {
            let mut stream = response.bytes_stream();
            let mut chunk_index: u8 = 0;
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        if let Err(err) = file.write(&chunk) {
                            error!("[{}] Write Error: {}", published_file_id, err);
                            break;
                        }
                        bytes_downloaded += chunk.len();
                        chunk_index += 1;
                        if chunk_index > 100 {
                            chunk_index = 0;
                            window.emit("progress", UpdatePayload {
                                publishedfileid: published_file_id,
                                bytes_downloaded,
                                bytes_total,
                                complete: false
                            }).ok();
                        }
                    },
                    Err(err) => {
                        window.emit("progress", ErrorPayload {
                            publishedfileid: published_file_id,
                            error: err.to_string()
                        }).ok();
                        error!("Download for {} failed:\n{}", published_file_id, &err);
                        return Err(err.to_string())
                    }
                }
            }
            file.flush().ok();
            std::fs::rename(part_file_path, dest_file_path)
                .map_err(|e| e.to_string())?;
            window.emit("progress", UpdatePayload {
                publishedfileid: published_file_id,
                bytes_downloaded,
                bytes_total,
                complete: true
            }).ok();
            debug!("Downloaded (id {}) ({} bytes)", published_file_id, bytes_downloaded);
            return Ok(())
        },
        Err(err) => {
            println!("Download failure for {}: {}", published_file_id, err);
            return Err(err.to_string())
        }
    }
}
#[tauri::command]
pub(crate) fn delete_addon(path: &str) -> Result<(), String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("File does not exist at {:?}", path).to_string());
    } else if path.is_dir() {
        return Err(format!("File path {:?} provided is a folder", path).to_string());
    } else {
        debug!("deleting {:?}", path);
        std::fs::remove_file(path).map_err(|e| e.to_string())
    }
}
#[tauri::command]
pub(crate) fn toggle_addon(state: tauri::State<'_, Data>, path: &str) -> Result<AddonEntry, String> {
    let path = PathBuf::from(path);
    if path.is_dir() {
        return Err(format!("File path {:?} provided is a folder", path).to_string());
    }
    let file_name = path.file_name().unwrap().to_string_lossy();
    let new_path;
    if file_name.ends_with(".disabled") {
        // Remove the .disabled:
        new_path = path.clone().with_extension("");
    } else if file_name.ends_with(".vpk") {
        // Add on .disabled:
        new_path = path.clone().with_file_name(format!("{}.disabled", file_name));
    } else {
        return Err("Filename does not end with .disabled or .vpk, cannot toggle".to_string());
    }
    debug!("toggle_addon {:?} -> {:?}", &path, &new_path);
    std::fs::rename(&path, &new_path).map_err(|e| e.to_string())?;
    util::get_addon_info(&new_path)
}

#[tauri::command]
pub(crate) fn migrate_addon(state: tauri::State<'_, Data>, path: &str) -> Result<AddonEntry, String> {
    let path = PathBuf::from(path);
    if path.is_dir() {
        return Err(format!("File path {:?} provided is a folder", path).to_string());
    }
    let parent_name = path.parent().unwrap().file_name().unwrap().to_string_lossy();
    if parent_name != "workshop" {
        return Err(format!("migrate_addon called on file that is not in a workshop folder.").to_string());
    }
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let new_path = path.parent().unwrap().with_file_name(file_name);
    debug!("migrate_addon {:?} -> {:?}", &path, &new_path);

    std::fs::copy(&path, &new_path).map_err(|e| e.to_string())?;
    util::get_addon_info(&new_path)
}


