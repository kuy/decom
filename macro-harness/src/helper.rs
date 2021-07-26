use anyhow::Result;
use std::{fs, path::Path};

pub fn read_expected(path: impl AsRef<Path>) -> Result<String> {
    let copy = path.as_ref().to_owned();
    let content = fs::read_to_string(path).unwrap_or_else(|err| {
        panic!("Failed to read expected file: {:?}\nError: {:?}", copy, err);
    });
    Ok(content)
}
