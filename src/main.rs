extern crate dotenv;
extern crate env_logger;
extern crate reqwest;
extern crate select;
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate hyper;
extern crate serde_json;

use dotenv::dotenv;
use failure::Error;
use select::document::Document;

use std::env;
use std::fs;

mod bot;
mod html;
mod parsers;
mod types;

use crate::parsers::*;

const FILE_NAME: &'static str = "houdini_main_changelogs.last_id";

fn file_path() -> std::path::PathBuf {
    let config_dir: Option<std::path::PathBuf> = env::var("CONFIG_DIR")
        .ok()
        .and_then(|value| value.parse().ok())
        .or_else(dirs::config_dir);

    config_dir.unwrap_or("/tmp/".into()).join(FILE_NAME)
}

fn read_last_id() -> Result<i32, Error> {
    println!("Reading last-id at: {:?}", file_path().to_str());
    let src = fs::read(file_path())?;
    let src = String::from_utf8_lossy(&src);
    let value: i32 = src.trim().parse()?;

    Ok(value)
}

fn save_last_id(id: i32) -> std::io::Result<()> {
    println!("Writing last-id {} to: {:?}", id, file_path().to_str());
    fs::write(file_path(), id.to_string())
}

fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv()?;

    let bot_token = env::var("BOT_TOKEN")
        .ok()
        .expect("Expected BOT_TOKEN env var");
    let chat_id = env::var("CHAT_ID").ok().expect("Expected CHAT_ID env var");
    let dev = env::var("DEV").ok().is_some();

    let last_id = read_last_id().unwrap_or(0);
    let mut last_id_to_be_saved = last_id;

    println!("Last ID: {}", last_id);
    println!("Starting fetch articles list...");

    let html = reqwest::get("https://sidefx.com/changelog")?.text()?;
    let document = Document::from(html.as_str());
    let change_log = parse_change_log(&document, last_id)?;

    println!("{:#?}", change_log.last_log());

    //let bot = bot::Bot::new(bot_token);
    //bot.send_message(chat_id.clone(), "Houdini Change Logs".to_owned());

    //if id > last_id_to_be_saved {
    //    last_id_to_be_saved = id;
    //}

    //if !dev {
    //    println!("Last ID: {}", last_id_to_be_saved);
    //    save_last_id(last_id_to_be_saved)?;
    //}

    Ok(())
}
