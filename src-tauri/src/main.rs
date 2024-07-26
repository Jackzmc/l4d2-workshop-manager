#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod config;
mod logger;
mod util;
mod commands;

use steam_workshop_api::{SteamWorkshop, WorkshopItem};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State, Window};
use futures::{StreamExt};
use std::{io::Write, time::{UNIX_EPOCH}};
use std::fs::DirEntry;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use log::{debug, error};
use crate::commands::{get_my_addons, get_settings, get_workshop_addons, save_settings};
use crate::util::{AddonInfo, WORKSHOP_ID_REGEX};

pub struct Data {
  pub settings: Arc<Mutex<config::SettingsManager>>,
  pub workshop: SteamWorkshop,
  pub downloads: Arc<Mutex<config::Downloads>>,
  pub logger: logger::Logger
}

struct SplashscreenWindow(Arc<Mutex<Window>>);
struct MainWindow(Arc<Mutex<Window>>);



#[derive(Serialize, Deserialize, Clone)]
enum ItemType {
  Updateable,
  Managed,
  Unmanaged,
  Unknown,
  Workshop
}

#[derive(Serialize, Deserialize, Clone)]
struct UpdatePayload {
  publishedfileid: String,
  bytes_downloaded: usize,
  complete: bool
}

#[derive(Serialize, Deserialize, Clone)]
struct ErrorPayload {
  publishedfileid: Option<String>,
  error: String
}

#[tauri::command]
fn close_splashscreen(
  splashscreen: State<SplashscreenWindow>,
  main: State<MainWindow>,
) {
  // Close splashscreen
  splashscreen.0.lock().expect("splashscreen lock fail").close().expect("splash close fail");
  // Show main window
  main.0.lock().expect("main lock fail").show().expect("main close fail");
}

#[tauri::command]
fn get_install_info(
  state: tauri::State<'_, Data>,
  id: String
) -> Option<config::DownloadEntry> {
  match state.downloads.lock().expect("get_install_info: Could not get downloads lock").get_download(&id) {
    Some(download) => Some(download.clone()),
    None => None
  }

}

#[tauri::command]
async fn download_addon(window: Window, state: tauri::State<'_, Data>, item: steam_workshop_api::WorkshopItem) -> Result<(), String> {
  let config = &state.settings;
  let mut dest = {
    let fname = config.lock().unwrap().get().gamedir.as_ref().unwrap().join(format!("{}.vpk", item.publishedfileid));
    std::fs::File::create(fname).expect("Could not create file")
  };
  let mut downloaded: usize = 0;
  state.logger.logp(logger::LogLevel::NORMAL, "download_addons", &format!("Starting download of file \"{}\" (id {}) ({} bytes)", &item.title, item.publishedfileid, item.file_size));
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
            if let Err(err) = dest.write(&chunk) {
              state.logger.error("download_addon", &format!("Write error for ID {}: {}", item.publishedfileid, err));
              println!("[{}] Write Error: {}", &item.publishedfileid, err);
              break;
            }
            downloaded += chunk.len();
            chunk_index += 1;
            if chunk_index > 100 {
              chunk_index = 0;
              window.emit("progress", UpdatePayload {
                publishedfileid: item.publishedfileid.clone(),
                bytes_downloaded: downloaded,
                complete: false
              }).ok();
            }
          },
          Err(err) => {
            window.emit("progress", ErrorPayload {
              publishedfileid: Some(item.publishedfileid.clone()),
              error: err.to_string()
            }).ok();
            state.logger.error("download_addon", &format!("Chunk failure for ID {}: {}", item.publishedfileid, err));
            println!("Download for {} failed:\n{}", item.title, &err); 
            return Err(err.to_string())
          }
        }
      }
      dest.flush().ok();
      window.emit("progress", UpdatePayload {
        publishedfileid: item.publishedfileid.clone(),
        bytes_downloaded: downloaded,
        complete: true
      }).ok();
      let entry = config::DownloadEntry::from_item(&item);
      let mut downloads = state.downloads.lock().expect("download_addon: Could not get downloads lock");
      match downloads.get_id_index(&item.publishedfileid) {
        Some(index) => downloads.set_download(index, entry),
        None => downloads.add_download(entry)
      }
      state.logger.logp(logger::LogLevel::NORMAL, "download_addon", &format!("Downloaded file \"{}\" (id {}) ({} bytes)", &item.title, item.publishedfileid, item.file_size));
      return Ok(())
    },
    Err(err) => {
      println!("Download failure for {}: {}", &item, err);
      return Err(err.to_string())
    }
  }
}

fn main() {
  env_logger::init();
  WORKSHOP_ID_REGEX.set(Regex::new(r"[0-9]+").unwrap()).unwrap();
  let mut settings = config::SettingsManager::new();
  if let Ok(false) = settings.load() {
    let gamedir = util::prompt_game_dir();
    let mut settings = config::SettingsManager::new();
    settings.get_mut().gamedir = Some(gamedir);
    if let Err(err) = settings.save() {
      panic!("Could not save settings: {}", err);
    }
  };

  tauri::Builder::default()
  .setup(|app| {
    // set the splashscreen and main windows to be globally available with the tauri state API
    app.manage(SplashscreenWindow(Arc::new(Mutex::new(
      app.get_window("splashscreen").expect("splash window fail")
    ))));
    let main = app.get_window("main").expect("main window fail");
    main.hide().ok();
    app.manage(MainWindow(Arc::new(Mutex::new(
     main
    ))));
    //TODO: Check if settings exists, if not, create new. exit on error (or send err)
    let logger = logger::Logger::new(config::get_appdir().join("downloader.log"));
    let mut settings = config::SettingsManager::new();
    settings.load().expect("failed to get settings");
    debug!("settings initialized");
    if !settings.get().gamedir.as_ref().unwrap().exists() {
      logger.error("setup", &format!("Specified game directory folder \"{}\" does not exist", settings.get().gamedir.as_ref().unwrap().to_string_lossy()));
      std::process::exit(1);
    }
    let downloads = match config::Downloads::load() {
      Ok(downloads) => downloads,
      Err(_e) => {
        config::Downloads::new()
      }
    };
    debug!("downloads initialized");

    if settings.get().telemetry {
      util::send_telemetry(&logger, downloads.size());
    }

    let mut ws = SteamWorkshop::new();
    ws.set_apikey(settings.get().steam_apikey.clone());

    app.manage(Data {
      settings: Arc::new(Mutex::new(settings)),
      downloads: Arc::new(Mutex::new(downloads)),
      workshop: ws,
      logger
    });
    debug!("done init.");
    app.get_window("splashscreen").unwrap().hide().ok();
    app.get_window("main").unwrap().show().ok();
    Ok(())
  })
  .invoke_handler(tauri::generate_handler![
    get_my_addons,
    get_workshop_addons,
    get_settings,
    save_settings,
    close_splashscreen,
    get_install_info,
  ])
  .run(tauri::generate_context!())
  .expect("error while running tauri application");


}