use std::path::PathBuf;
use crate::logger;

pub struct Addons {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>
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
            std::fs::create_dir_all(&path).ok();
            println!("Warn: left4dead2/addons folder missing, creating..");
            return path;
        }
        return path
    } else {
        eprintln!("Could not open file dialog");
        std::process::exit(1);
    }
}
  
pub fn send_telemetry(logger: &logger::Logger, downloads: usize) {
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

