use anyhow::Result;
use std::{fs, path::PathBuf};

pub fn read_expected(path: PathBuf) -> Result<String> {
    let copy = path.clone();
    let content = fs::read_to_string(path).unwrap_or_else(|err| {
        panic!("Failed to read expected file: {:?}\nError: {:?}", copy, err);
    });
    Ok(content)
}
