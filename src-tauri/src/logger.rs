use std::{fs,path};
use chrono::prelude::*;
use console::style;
use std::fs::OpenOptions;
use std::io::Write;

const MAX_LOG_SIZE: u64 = 1000000; //10 MB

pub struct Logger {
    file: fs::File
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum LogLevel {
    NORMAL,
    ERROR,
    WARN,
    INFO,
    SUCCESS,
    DEBUG
}

#[allow(dead_code)]
impl Logger {
    pub fn new(filepath: path::PathBuf) -> Logger {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&filepath)
            .unwrap();
        

        if file.metadata().unwrap().len() > MAX_LOG_SIZE {
            file.set_len(0).expect("Could not clear log file, file size is over purge threshold");
        }
        
        let logger = Logger {
            file
        };
        logger.log(LogLevel::INFO, &format!("Start of logging session v{}", env!("CARGO_PKG_VERSION")));
        logger
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%F %H:%M:%S %Z");
        writeln!(&self.file, "[{}] [{:?}] {}", 
            timestamp,
            level,
            msg
        ).ok();
    }

    pub fn logp(&self, level: LogLevel, prefix: &'static str, msg: &str) {
        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%F %H:%M:%S %Z");
        writeln!(&self.file, "[{}] [{}/{:?}] {}", 
            timestamp,
            prefix,
            level,
            msg
        ).ok();
    }

    pub fn info(&self, prefix: &'static str, msg: &str) {
        self.logp(LogLevel::INFO, prefix, &msg);
    }

    pub fn warn(&self, prefix: &'static str, msg: &str) {
        self.logp(LogLevel::WARN, prefix, &msg);
        println!("{}", style(msg).yellow());
    }

    pub fn success(&self, prefix: &'static str, msg: &str) {
        self.logp(LogLevel::WARN, prefix, &msg);
        println!("{}", style(msg).green());
    }

    pub fn error(&self, prefix: &'static str, msg: &str) {
        self.logp(LogLevel::ERROR, prefix, &msg);
        eprintln!("{} {}", style("Error: ").red().bold(), style(msg).red());
    }

    pub fn debug(&self, prefix: &'static str, msg: &str) {
        self.logp(LogLevel::DEBUG, prefix, &msg);
        println!("{}", style(msg).magenta());
    }
}