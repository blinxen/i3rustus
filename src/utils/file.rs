use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Error};

pub fn read_file(path: &str) -> Option<File> {
    OpenOptions::new().read(true).open(path).ok()
}

pub fn read_first_line_in_file(path: &str) -> Result<String, Error> {
    let mut first_line = String::new();
    let file: Result<File, Error> = OpenOptions::new().read(true).open(path);

    BufReader::new(file?).read_line(&mut first_line)?;
    // Delete the newline character at the end of the file because we don't need it
    first_line.pop();
    Ok(first_line)
}
