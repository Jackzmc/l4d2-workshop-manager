#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod config;
mod util;
mod commands;

use steam_workshop_api::{SteamWorkshop};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State, Window};
use futures::{StreamExt};
use std::{io::Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use flexi_logger::{colored_default_format, FileSpec, Logger, WriteMode};
use log::{debug, error, info, log, trace, warn};
use crate::commands::{get_latest_workshop_info, get_my_addons, get_settings, get_workshop_addons, save_settings, search_workshop};
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
  publishedfileid: u32,
  bytes_downloaded: usize,
  bytes_total: usize,
  complete: bool
}

#[derive(Serialize, Deserialize, Clone)]
struct ErrorPayload {
  publishedfileid: u32,
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

fn setup_logging() {
  let _logger = Logger::try_with_env_or_str(format!("warn, l4d2_addon_manager=debug")).unwrap()
      .set_palette("168;226;81;34;38".to_string()) // error, warn, info, debug, trace
      .format_for_stdout(colored_default_format)
      .log_to_file(FileSpec::default()
          .directory( PathBuf::from("./logs"))
          .basename("l4d2-addon-manager")
          // .use_timestamp(false)
          // .suppress_timestamp()
      )
      .log_to_stdout()
      .write_mode(WriteMode::BufferAndFlush)
      .start().unwrap();
  error!("error");
  warn!("warn");
  debug!("debug");
  info!("info");
  trace!("trace");
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
    search_workshop
  ])
  .run(tauri::generate_context!())
  .expect("error while running tauri application");


}