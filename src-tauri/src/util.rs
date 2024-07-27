use std::collections::HashMap;
use std::fs::{DirEntry, File};
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::UNIX_EPOCH;
use log::{debug, error, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sourcepak::common::file::VPKFileReader;
use sourcepak::common::format::VPKTree;
use steam_workshop_api::{SteamWorkshop, WorkshopItem};
use crate::{logger, util};

pub struct Addons {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct AddonEntry {
    file_name: String,
    file_size: u64,
    last_update_time: Option<u64>,
    create_time: Option<u64>,

    addon_data: Option<AddonData>,

    workshop_info: Option<WorkshopItem>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AddonData {
    info: Option<AddonInfo>,
    mission_info: Option<MissionInfo>
}

#[derive(Serialize, Deserialize, Clone)]
// TODO: remove renames, use serde-aux to make field names case insensitive
// https://developer.valvesoftware.com/wiki/Addoninfo.txt
pub struct AddonInfo {
    #[serde(rename(serialize = "title", deserialize = "addontitle"), alias = "addonTitle", alias = "AddonTitle")]
    pub title: String,
    #[serde(rename(serialize = "version", deserialize = "addonversion"), alias = "addonVersion", alias = "AddonVersion")]
    pub version: Option<String>,
    #[serde(rename(serialize = "author", deserialize = "addonauthor"), alias = "addonAuthor", alias = "AddonAuthor")]
    pub author: Option<String>,
    #[serde(rename(serialize = "description", deserialize = "addondescription"), alias = "addonDescription", alias = "AddonDescription")]
    pub description: Option<String>,

    #[serde(rename(serialize = "content_script", deserialize = "addoncontent_script"), alias = "AddonContent_Script", default = "bool::default")]
    pub content_script: bool,

    #[serde(rename(serialize = "content_campaign", deserialize = "addoncontent_campaign"), alias = "addonContent_Campaign", default = "bool::default")]
    /** Has multiple maps that form a campaign? **/
    pub content_campaign: bool,

    #[serde(rename(serialize = "content_map", deserialize = "addoncontent_map"), alias = "addonContent_Map", default = "bool::default")]
    /** Has at least one map file? **/
    pub content_map: bool,

    #[serde(rename(serialize = "content_music", deserialize = "addoncontent_music"), alias = "addonContent_Music", default = "bool::default")]
    pub content_music: bool,

    #[serde(rename(serialize = "content_sound", deserialize = "addoncontent_sound"), alias = "addonContent_Sound", default = "bool::default")]
    pub content_sound: bool,

    #[serde(rename(serialize = "content_skin", deserialize = "addoncontent_skin"), alias = "addonContent_Skin", default = "bool::default")]
    pub content_skin: bool,

    #[serde(rename(serialize = "content_weapon", deserialize = "addoncontent_weapon"), alias = "addonContent_Weapon", default = "bool::default")]
    pub content_weapon: bool

    // TODO: add other categories
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MissionInfo {
    modes: Option<MissionModes>
}
#[derive(Serialize, Deserialize, Clone)]
pub struct MissionModes {
    coop: Option<HashMap<u64, MissionChapter>>,
    versus: Option<HashMap<u64, MissionChapter>>,
    survival: Option<HashMap<u64, MissionChapter>>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct MissionChapter {
    #[serde(rename(serialize = "map", deserialize = "Map"))]
    map: String,
    #[serde(rename(serialize = "display_name", deserialize = "DisplayName"))]
    display_name: String,
    #[serde(rename(serialize = "image", deserialize = "Image"))]
    image: String,
}

use sourcepak::pak::v1::format::VPKVersion1;
pub fn get_addon_data(path: &Path) -> Result<AddonData, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let vpk = VPKVersion1::try_from(&mut file)?;
    // debug!("archive_path={} file_name={} v={} tree_size={} #files={}", archive_path, archive_file_name, vpk.header.tree_size, vpk.header.tree_size, vpk.tree.files.len());
    let addoninfo = vpk.tree.files.get(" /addoninfo.txt");
    if let Some(addoninfo) = addoninfo {
        let offset_start = file.stream_position().unwrap();
        // debug!("found addoninfo.txt. offset={} len={} i={}", addoninfo.entry_offset, addoninfo.entry_length, addoninfo.archive_index);
        file.seek(SeekFrom::Current(addoninfo.entry_offset as i64)).expect("seek failed");

        let buf = file.read_bytes(addoninfo.entry_length as usize).unwrap();
        let content = String::from_utf8_lossy(&buf);
        let addon_info: AddonInfo = keyvalues_serde::from_str(&content)
            .map_err(|e| format!("failed to parse addoninfo.txt: {}", e))?;

        file.seek(SeekFrom::Start(offset_start)).unwrap();
        let mission_info = get_mission_data(&mut file, &vpk);

        Ok(AddonData {
            info: Some(addon_info),
            mission_info,
        })
    } else {
        // TODO: make just a warning, return Option<?>
        Err("No addoninfo.txt found".to_string())
    }
}

pub fn get_mission_data(file: &mut File, vpk: &VPKVersion1) -> Option<MissionInfo> {
    for (path, entry) in &vpk.tree.files {
        if path.starts_with("missions/") && path.ends_with(".txt") {
            file.seek(SeekFrom::Current(entry.entry_offset as i64)).unwrap();
            let buf = file.read_bytes(entry.entry_length as usize).unwrap();
            let content = String::from_utf8_lossy(&buf);
            debug!("found mission file = {:?}", path);
            return match keyvalues_serde::from_str::<MissionInfo>(&content) {
                Ok(mi) => Some(mi),
                Err(e) => {
                    error!("Failed to parse mission file = {}", e);
                    None
                }
            }
        }
    }
    None
}

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

fn get_vpks_in_folder(path: &Path) -> Result<Vec<DirEntry>, String> {
    let entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut files: Vec<DirEntry> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_name = entry.file_name();
        let file_name = file_name.to_str().expect("bad filename");
        if file_name.ends_with(".vpk") {
            files.push(entry)
        }
    }
    debug!("found {} vpks in {:?}", files.len(), path);
    return Ok(files);
}

struct WorkshopResult {
    item: WorkshopItem,
    cached: bool
}

pub fn get_workshop_data(ws: &SteamWorkshop, entries: &[DirEntry]) -> HashMap<u32, WorkshopResult> {
    let mut pending_workshop_ids: Vec<u32> = vec![];
    let mut results: HashMap<u32, WorkshopResult> = HashMap::with_capacity(entries.len());
    for entry in entries {
        let path = entry.path();

        let workshop_id = find_workshop_id_in_str(&path.file_stem().unwrap().to_string_lossy());
        if workshop_id.is_none() { continue; }
        let workshop_id = workshop_id.unwrap();
        if let Some(item) = get_cached_workshop_info(&path, workshop_id) {
            results.insert(workshop_id, WorkshopResult { item, cached: true });
        } else {
            // Queue up for bulk fetching
            pending_workshop_ids.push(workshop_id);
        }
    }

    // Steam API only accepts 100 entries at a time
    while(pending_workshop_ids.len() > 0) {
        // let items = steam_workshop_api::Workshop::
        let end = pending_workshop_ids.len().min(100);
        let slice: Vec<u32> = pending_workshop_ids.drain(0..end).collect();
        let slice: Vec<String> = slice.iter().map(|d| d.to_string()).collect();
        debug!("slice = {}", slice.join(" "));
        match ws.get_published_file_details(&slice) {
            Ok(items) => {
                for item in items {
                    results.insert(item.publishedfileid.parse().unwrap(), WorkshopResult { item, cached: false });
                }
            },
            Err(e) => {
                error!("get_workshop_data error: {}", e)
            }
        }

    }

    results
}
pub fn get_addons(workshop: &SteamWorkshop, dir: &Path) -> Result<Vec<AddonEntry>, String> {
    let entries = get_vpks_in_folder(dir)?;
    let mut workshop_record = get_workshop_data(workshop, &entries);
    let mut files: Vec<AddonEntry> = vec![];


    for entry in entries {
        let meta = entry.metadata().unwrap();
        let path = entry.path();


        let addon_data: Option<AddonData> = get_addon_data(&path).ok();
        let workshop_info = find_workshop_id_in_str(&path.file_stem().unwrap().to_string_lossy())
            .and_then(|id| workshop_record.remove(&id));
        // If item was not cached, then save to file
        if let Some(data) = &workshop_info {
            if !data.cached {
                let content = serde_json::to_string(&data.item).unwrap();
                let mut path = path.clone();
                path.set_file_name(format!("{}_ws.json", data.item.publishedfileid));
                std::fs::write(path, content).ok();
            }
        }
        let workshop_info = workshop_info.map(|data| data.item);
        let file = AddonEntry {
            file_name: entry.file_name().to_str().unwrap().to_string(),
            file_size: meta.size(),
            last_update_time: meta.modified().ok().and_then(|s| Some(s.duration_since(UNIX_EPOCH).unwrap().as_secs())),
            create_time: meta.created().ok().and_then(|s| Some(s.duration_since(UNIX_EPOCH).unwrap().as_secs())),

            workshop_info,
            addon_data
        };

        files.push(file);
    }
    Ok(files)
}
pub static WORKSHOP_ID_REGEX: OnceLock<Regex> = OnceLock::new();
pub fn find_workshop_id_in_str(file_name: &str) -> Option<u32> {
    WORKSHOP_ID_REGEX.get().unwrap().captures(file_name)
        .map(|c| c[0].parse().unwrap())
}
pub fn get_cached_workshop_info(path: &Path, workshop_id: u32) -> Option<WorkshopItem> {
    let path = path.with_file_name(format!("{}_ws.json", workshop_id));
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).ok()
        },
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                warn!("Could not read cached workshop info at {:?}: {}", path, e);
            }
            None
        }
    }
}