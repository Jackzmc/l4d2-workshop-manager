#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use steam_workshop_api::{Workshop, WorkshopItem};
use std::path::PathBuf;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State, Window};
use futures::{StreamExt};
use std::{io::Write, time::{UNIX_EPOCH}};
use std::sync::{Arc, Mutex};

struct Data {
  settings: config::Settings,
  downloads: Arc<Mutex<config::Downloads>>
}

struct SplashscreenWindow(Arc<Mutex<Window>>);
struct MainWindow(Arc<Mutex<Window>>);

mod config;

#[derive(Serialize, Deserialize)]
enum ItemType {
  Updateable,
  Managed,
  Unmanaged,
  Unknown,
  Workshop
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum File {
  Item { 
    item: WorkshopItem,
    item_type: ItemType
  },
  Unknown {
    item: UnknownFile,
    item_type: ItemType
  }
}

#[derive(Serialize, Deserialize)]
struct UnknownFile {
  publishedfileid: String,
  file_size: Option<u64>,
  time_updated: Option<u64>,
}

#[tauri::command]
fn get_items() -> Result<Vec<File>, String> {
  let regex = Regex::new(r"([0-9]{7,})").unwrap();
  let mut unknown_ids = Vec::new();
  let fileids = match Workshop::get_vpks_in_folder(&PathBuf::from(r"D:\_temp\rust_ws_test")) {
    Ok(results) => {
      //Tries to find an ID to parse
      let mut fileids: Vec<String> = Vec::with_capacity(results.len());
      for filename in results.iter() {
        if let Some(mat) = regex.find(&filename) {
          fileids.push(filename[mat.start()..mat.end()].to_string());
        } else {
          //ItemType::Unknown
          let full_file = format!("{}.vpk", filename);
          if let Ok(metadata) = std::fs::metadata(&PathBuf::from(r"D:\_temp\rust_ws_test").join(full_file)) {
            unknown_ids.push(UnknownFile {
              publishedfileid: filename.clone(), 
              file_size: Some(metadata.len()),
              time_updated: Some(metadata.modified().unwrap().duration_since(UNIX_EPOCH).expect("time went backwards").as_millis() as u64)
            });
          } else {
            unknown_ids.push(UnknownFile {
              publishedfileid: filename.clone(), 
              file_size: None,
              time_updated: None
            });
          }
        }
      }
      fileids
    },
    Err(err) => {
      println!("Error1: {}", err);
      return Err(err)
    }
  };
  let mut files: Vec<File> = Vec::with_capacity(fileids.len());
  let details: Vec<WorkshopItem> = match Workshop::new(None).get_published_file_details(&fileids) {
    Ok(details) => details,
    Err(err) => { 
      println!("Error2: {}", err);
      return Err(err.to_string())
    }
  };
  
  let downloads = config::Downloads::load()?;
  
  for detail in details {
    //TODO: 1. Check if file is in downloads list
    //2. Check if file has an update
    match downloads.get_download(&detail.publishedfileid) {
      Some(download) => {
        let item_type = 
        if detail.time_updated >= download.time_updated {
          ItemType::Updateable
        } else {
          ItemType::Managed
        };
        files.push(File::Item {
          item: detail,
          item_type
        });
      },
      None => {
        files.push(File::Item {
          item: detail,
          item_type: ItemType::Unmanaged,
        });
      }
    }
    
    
  }
  
  for unknown in unknown_ids {
    files.push(File::Unknown {
      item: unknown,
      item_type: ItemType::Unknown,
    });
  }
  Ok(files)
}


#[derive(Serialize, Deserialize)]
struct UpdatePayload {
  publishedfileid: String,
  bytes_downloaded: usize,
  complete: bool
}

#[derive(Serialize, Deserialize)]
struct ErrorPayload {
  publishedfileid: Option<String>,
  error: String
}

#[tauri::command]
fn get_settings(state: tauri::State<Data>) -> config::Settings {
  state.settings.clone()
}

#[tauri::command]
fn close_splashscreen(
  splashscreen: State<SplashscreenWindow>,
  main: State<MainWindow>,
) {
  // Close splashscreen
  splashscreen.0.lock().unwrap().close().unwrap();
  // Show main window
  main.0.lock().unwrap().show().unwrap();
}

#[tauri::command]
async fn download_addon(window: Window, state: tauri::State<'_, Data>, item: steam_workshop_api::WorkshopItem) -> Result<(), String> {
  let config = &state.settings;
  let mut dest = {
    let fname = config.gamedir.join(format!("{}.vpk", item.publishedfileid));
    std::fs::File::create(fname).expect("Could not create file")
  };
  let mut downloaded: usize = 0;
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
      println!("Writing to file... {}", item.publishedfileid);
      let entry = config::DownloadEntry::from_item(&item);
      let mut downloads = state.downloads.lock().unwrap();
      match downloads.get_id_index(&item.publishedfileid) {
        Some(index) => downloads.set_download(index, entry),
        None => downloads.add_download(entry)
      }
      return Ok(())
    },
    Err(err) => {
      println!("Download failure for {}: {}", &item, err);
      return Err(err.to_string())
    }
  }
}

fn main() {
  tauri::Builder::default()
  .setup(|app| {
    // set the splashscreen and main windows to be globally available with the tauri state API
    app.manage(SplashscreenWindow(Arc::new(Mutex::new(
      app.get_window("splashscreen").unwrap(),
    ))));
    app.manage(MainWindow(Arc::new(Mutex::new(
      app.get_window("main").unwrap(),
    ))));
    //TODO: Check if settings exists, if not, create new. exit on error (or send err)
    let settings = match config::Settings::load() {
      Ok(config) => config,
      Err(_e) => {
        let gamedir = prompt_game_dir();
        config::Settings {
          gamedir
        }
      }
    };
    let downloads = match config::Downloads::load() {
      Ok(downloads) => downloads,
      Err(_e) => {
        config::Downloads::new()
      }
    };
    app.manage(Data {
      settings,
      downloads: Arc::new(Mutex::new(downloads))
    });
    Ok(())
  })
  .invoke_handler(tauri::generate_handler![
    get_items, 
    download_addon,
    get_settings,
    close_splashscreen
  ])
  .run(tauri::generate_context!())
  .expect("error while running tauri application");
}

fn prompt_game_dir() -> PathBuf {
  if let Some(file_path) = tinyfiledialogs::open_file_dialog(
    "Choose where Left 4 Dead 2 is installed", 
    "",
    Some((&["left4dead2.exe"], "left4dead2.exe"))
  ) {
    let path = PathBuf::from(file_path)
    .parent()
    .unwrap()
    .join("left4dead2")
    .join("addons");
    if !path.exists() {
      eprintln!("A valid directory was not specified. Exiting.");
    }
    return path
  } else {
    eprintln!("A valid directory was not specified. Exiting.");
  }
  return prompt_game_dir();
}