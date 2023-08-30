#![allow(dead_code)]

use anyhow;
use std::env;
use std::fs;
use std::path::PathBuf;

pub const FILE_NAME: &'static str = "houdini_main_changelogs.last_id";

pub fn file_path() -> PathBuf {
    let config_dir = env::var("CONFIG_DIR")
        .ok()
        .and_then(|value| value.parse().ok())
        .or_else(dirs::config_dir);
    config_dir.unwrap_or("/tmp/".into()).join(FILE_NAME)
}

pub fn read_last_id() -> anyhow::Result<i32> {
    println!("Reading last-id at: {:?}", file_path().to_str());
    let src = fs::read(file_path())?;
    let src = String::from_utf8_lossy(&src);
    let value: i32 = src.trim().parse()?;
    Ok(value)
}

pub fn save_last_id(id: i32) -> anyhow::Result<()> {
    println!("Writing last-id {} to: {:?}", id, file_path().to_str());
    fs::write(file_path(), id.to_string())?;
    Ok(())
}
