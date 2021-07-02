use std::{error::Error, result::Result};

mod docker;
mod docker_compose;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ids = docker_compose::containers().await?;
    let mapping = docker::names(ids).await?;
    println!("ids: {:?}", mapping);
    Ok(())
}
