use std::fs::File;
use std::fs::OpenOptions;
use std::io::Error;


pub fn read_file(path: &str) -> Option<File> {
    let meminfo: Result<File, Error> = OpenOptions::new()
                                                    .read(true)
                                                    .open(path);
    match meminfo {
        Ok(file) => Some(file),
        _ => None
    }

}
