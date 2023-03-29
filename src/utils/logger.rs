use log::{Level, Log, Metadata, Record};
use std::{
    fs::{File, OpenOptions},
    io::Write,
};

pub struct Logger {
    pub file: Option<File>,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) && self.file.is_some() {
            // println!();
            self.file
                .as_ref()
                .unwrap()
                .write_all(format!("{} - {}\n", record.level(), record.args()).as_bytes())
                .expect("Could not write to log file");
        }
    }

    fn flush(&self) {}
}

impl Logger {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            file: OpenOptions::new()
                .create(true)
                .append(true)
                .open("/var/log/i3rustus.log")
                .ok(),
        })
    }
}
