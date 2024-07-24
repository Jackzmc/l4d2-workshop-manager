use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use sourcepak::common::file::VPKFileReader;
use sourcepak::common::format::VPKTree;
use sourcepak::common::format::PakReader;
use steam_workshop_api::WorkshopItem;
use crate::logger;

pub struct Addons {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>
}

#[derive(Serialize, Deserialize)]
pub struct AddonInfo {
    title: String,
    version: String,
    author: String,
    description: String
}
use sourcepak::pak::v1::format::VPKVersion1;
pub fn get_addon_info(path: &Path) -> Result<AddonInfo, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let vpk = VPKVersion1::try_from(&mut file)?;
    let archive_path = path.parent().unwrap().to_str().unwrap().to_owned();

    let archive_file_name = path.file_stem().unwrap().to_str().unwrap().to_owned();
    debug!("archive_path={} file_name={} v={} tree_size={} #files={}", archive_path, archive_file_name, vpk.header.tree_size, vpk.header.tree_size, vpk.tree.files.len());
    let addoninfo = vpk.tree.files.get(" /addoninfo.txt");
    if let Some(addoninfo) = addoninfo {
        debug!("found addoninfo.txt. offset={} len={} i={}", addoninfo.entry_offset, addoninfo.entry_length, addoninfo.archive_index);
        file.seek(SeekFrom::Start(addoninfo.entry_offset as u64)).expect("seek failed");
        let mut buf: Vec<u8> = Vec::with_capacity(addoninfo.entry_length as usize);
        file.read_exact(&mut buf).unwrap();
        debug!("content = {:?}", buf);
    }
    let content = vpk.read_file(&archive_path, &archive_file_name, &"/addoninfo.txt".to_owned());
    if content.is_none() {
        debug!("no content for {:?}", path);
        return Err("no addoninfo.txt content".to_string());
    }

    Err("test".to_string())
}


pub struct MetadataManager {
    path: PathBuf
}

/* TODO:
Workshop items in workshop/<publishedfileid>.json
All vpk's get parsed, maybe .cache.json in main addons for _all_ files
*/



pub fn prompt_game_dir() -> PathBuf {
    //FIXIME: Figure out why this crashes?
    if let Some(file_path) = tinyfiledialogs::open_file_dialog(
        "Choose where Left 4 Dead 2 is installed", 
        "",
        Some((&["left4dead2.exe"], "left4dead2.exe"))
    ) {
        let path = PathBuf::from(file_path)
            .parent()
            .expect("Invalid folder: No parent")
            .join("left4dead2")
            .join("addons");
        if !path.exists() {
            warn!("left4dead2/addons folder missing, creating..");
            std::fs::create_dir_all(&path).ok();
            let meta_path = path.join(".metadata");
            std::fs::create_dir_all(meta_path).ok();
            return path;
        }
        return path
    } else {
        eprintln!("Could not open file dialog");
        std::process::exit(1);
    }
}
  
pub fn send_telemetry(logger: &logger::Logger, downloads: usize) {
    return;
    // Server doesn't work:
    match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
    {
        Ok(client) => {
        if let Err(err) = client
            .get("https://telemetry.jackz.me/track.php")
            .query(&[
            ("item", "l4d2-workshop-downloader"),
            ("v", env!("CARGO_PKG_VERSION")),
            ("os", std::env::consts::OS),
            ("arch", std::env::consts::ARCH),
            ("downloaded", &downloads.to_string())
            ])
            .send() {
            logger.warn("send_telemetry", &format!("Failed to send telemetry: {}", err.to_string()));
        }
        },
        Err(err) => {
        logger.warn("send_telemetry", &format!("Failed to setup sending telemetry: {}", err.to_string()));
        }
    }
}

pub fn get_addon_files(dir: &std::path::Path) -> Result<Addons, String> {
    let mut entries: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(file) => {
            match file.map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>() {
                Ok(files) => files,
                Err(err) => return Err(err.to_string())
            }
        },
        Err(err) => return Err(err.to_string())
    };
    entries.sort();

    let mut enabled: Vec<String> = Vec::with_capacity(entries.len());
    let mut disabled: Vec<String> = Vec::new();

    for entry in entries {
        if !entry.is_dir() {
        match entry.extension().and_then(std::ffi::OsStr::to_str) {
            Some("vpk") => enabled.push(entry.file_stem().unwrap().to_str().unwrap().to_owned()),
            Some("disabled") => disabled.push(entry.file_stem().unwrap().to_str().unwrap().to_owned()),
            _ => ()
        }
        }
    }

    Ok(Addons {
        enabled,
        disabled
    })
}

