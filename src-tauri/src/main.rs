#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod config;
mod util;
mod commands;

use steam_workshop_api::{SteamWorkshop, WorkshopItem};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State, Window};
use futures::{StreamExt};
use std::{io::Write, time::{UNIX_EPOCH}};
use std::fs::{DirEntry, File};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use flexi_logger::{FileSpec, Logger, WriteMode};
use log::{debug, error};
use crate::commands::{get_latest_workshop_info, get_my_addons, get_settings, get_workshop_addons, save_settings};
use crate::util::{WORKSHOP_ID_REGEX};

pub struct Data {
  pub settings: Arc<Mutex<config::SettingsManager>>,
  pub workshop: SteamWorkshop,
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
async fn download_addon(window: Window, state: tauri::State<'_, Data>, item: steam_workshop_api::WorkshopItem) -> Result<(), String> {
  let config = &state.settings;
  let mut dest = {
    let fname = config.lock().unwrap().get().gamedir.as_ref().unwrap().join(format!("{}.vpk", item.publishedfileid));
    std::fs::File::create(fname).expect("Could not create file")
  };
  let mut downloaded: usize = 0;
  debug!("Starting download of file \"{}\" (id {}) ({} bytes)", &item.title, item.publishedfileid, item.file_size);
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
              error!("[{}] Write Error: {}", &item.publishedfileid, err);
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
            error!("Download for {} failed:\n{}", item.title, &err);
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
      debug!("Downloaded file \"{}\" (id {}) ({} bytes)", &item.title, item.publishedfileid, item.file_size);
      return Ok(())
    },
    Err(err) => {
      println!("Download failure for {}: {}", &item, err);
      return Err(err.to_string())
    }
  }
}

fn setup_logging() {
  let _logger = Logger::try_with_str(format!("warn, {}=debug", env!("CARGO_PKG_NAME"))).unwrap()
      .log_to_file(FileSpec::default()
          .directory( PathBuf::from("./logs"))
          .basename("l4d2-addon-manager")
          // .use_timestamp(false)
          // .suppress_timestamp()
      )
      .log_to_stdout()
      .write_mode(WriteMode::BufferAndFlush)
      .start().unwrap();
}

fn main() {
  setup_logging();
  WORKSHOP_ID_REGEX.set(Regex::new(r"[0-9]{4,}").unwrap()).unwrap();
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
    let mut settings = config::SettingsManager::new();
    settings.load().expect("failed to get settings");
    debug!("settings initialized");
    if !settings.get().gamedir.as_ref().unwrap().exists() {
      error!("Specified game directory folder \"{}\" does not exist", settings.get().gamedir.as_ref().unwrap().to_string_lossy());
      std::process::exit(1);
    }

    if settings.get().telemetry {
      // util::send_telemetry(&logger, downloads.size());
    }

    let mut ws = SteamWorkshop::new();
    ws.set_apikey(settings.get().steam_apikey.clone());

    app.manage(Data {
      settings: Arc::new(Mutex::new(settings)),
      workshop: ws,
    });
    debug!("done init.");
    app.get_window("splashscreen").unwrap().hide().ok();
    app.get_window("main").unwrap().show().ok();
    Ok(())
  })
  .invoke_handler(tauri::generate_handler![
    get_latest_workshop_info,
    get_my_addons,
    get_workshop_addons,
    get_settings,
    save_settings,
    close_splashscreen,
  ])
  .run(tauri::generate_context!())
  .expect("error while running tauri application");


}