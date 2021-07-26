use anyhow::Result;
use std::{
    path::Path,
    process::{Command, Output},
};

pub fn expand(manifest_path: impl AsRef<Path>) -> Result<Output> {
    let manifest_path = manifest_path.as_ref().to_owned();
    let output = Command::new("cargo")
        .args(&[
            "expand",
            "--manifest-path",
            manifest_path.as_path().to_str().unwrap(),
        ])
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "Failed to run 'cargo expand': {:?}\n  Error: {:?}",
                manifest_path, err
            );
        });
    if !output.status.success() {
        let msg = String::from_utf8(output.stderr).expect("should be convert");
        panic!("Failed to expand\n  Error: {}", msg);
    }
    Ok(output)
}
