use std::{error::Error, result::Result};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let output = Command::new("docker-compose")
        .args(&["logs", "penguin"])
        .current_dir("/home/kodama/work/initial/workspaces/e2e/environments")
        .output()
        .await?;
    let output = std::str::from_utf8(output.stdout.as_slice())?;
    print!("[output]\n{}", output);
    Ok(())
}
