use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Error, BufReader, BufRead};


pub fn read_file(path: &str) -> Option<File> {
    let meminfo: Result<File, Error> = OpenOptions::new()
                                            .read(true)
                                            .open(path);
    match meminfo {
        Ok(file) => Some(file),
        _ => None
    }

}

pub fn read_first_line_in_file(path: &str) -> String {
    let first_line = String::new();
    let file: Result<File, Error> = OpenOptions::new()
                                        .read(true)
                                        .open(path);

    BufReader::new(file.unwrap()).read_line(&mut first_line);
    first_line
}
