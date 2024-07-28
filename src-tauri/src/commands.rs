use std::io::Write;
use futures::StreamExt;
use log::{debug, error};
use steam_workshop_api::WorkshopItem;
use tauri::Window;
use crate::{config, Data, ErrorPayload, UpdatePayload, util};

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

#[tauri::command]
pub fn search_workshop(state: tauri::State<Data>, query: &str) -> Result<Vec<WorkshopItem>, String> {
    let ws = &state.workshop.clone();
    ws.search_items(550, query, 25)
        .map_err(|e| e.to_string())
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
    // TODO: get workshop item
    let mut bytes_downloaded: usize = 0;
    debug!("Starting download of id={}", published_file_id);
    match reqwest::Client::new()
        .get(&item.file_url)
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
                                bytes_total: 0,
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
            std::fs::rename(part_file_path, dest_file_path)?;
            window.emit("progress", UpdatePayload {
                publishedfileid: published_file_id,
                bytes_downloaded,
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
