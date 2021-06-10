#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use steam_workshop_api::{Workshop, WorkshopItem};
use std::path::PathBuf;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::{UNIX_EPOCH};
use tauri::{Window};
use futures::{stream, StreamExt};
use std::{io::Write};


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

struct Download {
  file: std::fs::File,
  item: steam_workshop_api::WorkshopItem,
  bytes_downloaded: usize,
  chunk: usize
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

const CONCURRENT_REQUESTS: usize = 4;

#[tauri::command]
fn get_config() -> Result<config::Settings, String> {
  config::Settings::load()
}

#[tauri::command]
fn download_addons(window: Window, items: Vec<steam_workshop_api::WorkshopItem>) -> () {
  let mut downloads: Vec<Download> = Vec::with_capacity(items.len());
  let config: config::Settings = match get_config() {
    Ok(config) => config,
    Err(err) => {
      &window.emit("progress", ErrorPayload {
        publishedfileid: None,
        error: err.to_string()
      }).ok();
      return ()
    }
  };
  
  for item in items {
    
    let dest = {
      let fname = config.gamedir.join(format!("{}.vpk", item.publishedfileid));
      std::fs::File::create(fname).expect("Could not create file")
    };
    let download = Download {
      file: dest,
      item: item.clone(),
      bytes_downloaded: 0,
      chunk: 0
    };
    downloads.push(download);
  }
  
  let client = reqwest::Client::new();
  
  tauri::async_runtime::block_on(async {
    stream::iter(downloads)
    .map(|mut download: Download| {
      let client = &client;
      let window = &window;
      async move {
        match client
          .get(&download.item.file_url)
          .header("User-Agent", "L4D2-Workshop-Downloader")
          .send()
          .await
        {
          Ok(response) => {
            let mut stream = response.bytes_stream();
            while let Some(result) = stream.next().await {
              match result {
                Ok(chunk) => {
                  if let Err(err) = download.file.write(&chunk) {
                    println!("[{}] Write Error: {}", &download.item.publishedfileid, err);
                    break;
                  }
                  download.bytes_downloaded += chunk.len();
                  download.chunk += 1;
                  if download.chunk > 100 {
                    download.chunk = 0;
                    window.emit("progress", UpdatePayload {
                      publishedfileid: download.item.publishedfileid.clone(),
                      bytes_downloaded: download.bytes_downloaded,
                      complete: false
                    }).ok();
                  }
                },
                Err(err) => {
                  window.emit("progress", ErrorPayload {
                    publishedfileid: Some(download.item.publishedfileid.clone()),
                    error: err.to_string()
                  }).ok();
                  println!("Download for {} failed:\n{}", &download.item.title, &err); 
                }
              }
            }
            download.file.flush().ok();
          },
          Err(err) => {
            println!("Download failure for {}: {}", &download.item, err);
          }
        }
        download
      }
    })
    .buffer_unordered(CONCURRENT_REQUESTS)
    .for_each(|download| {
      let mut downloads = config::Downloads::load().unwrap();
      let entry = config::DownloadEntry::from_item(&download.item);
      
      match downloads.get_id_index(&entry.publishedfileid) {
        Some(index) => downloads.set_download(index, entry),
        None => downloads.add_download(entry)
      }
      downloads.save().ok();

      let window = &window;
      async move {
        window.emit("progress", UpdatePayload {
          publishedfileid: download.item.publishedfileid.clone(),
          bytes_downloaded: download.bytes_downloaded,
          complete: true
        }).ok();
        println!("Finished: {}", &download.item);
      }
    })
    .await
  })
}


fn main() {
  tauri::Builder::default()
  .invoke_handler(tauri::generate_handler![
    get_items, 
    download_addons,
    get_config,
  ])
  .run(tauri::generate_context!())
  .expect("error while running tauri application");
}