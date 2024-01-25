extern crate dotenv;
extern crate env_logger;
extern crate reqwest;
extern crate select;
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate hyper;
extern crate serde_json;

use anyhow::Result;
use dotenv::dotenv;
use failure::Error;
use select::document::Document;
use std::env;

mod bot;
mod html;
mod log;
mod parsers;
mod utils;

use crate::parsers::parse_change_log;

fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv().expect("config file .env not found");

    let bot_token = env::var("BOT_TOKEN").expect("Expected BOT_TOKEN env var");
    let chat_id = env::var("CHAT_ID").expect("Expected CHAT_ID env var");

    let html = reqwest::get("https://sidefx.com/changelog")?.text()?;
    let document = Document::from(html.as_str());
    let mut changelog = parse_change_log(&document)?;

    // Uploading the latest logs for analysis.
    let prev_build = changelog.load().expect("load log failed");

    let prev_record = &prev_build.last_record().unwrap();
    let next_record = &changelog.last_record().unwrap();

    // If the previous and next entry do not differ
    // then changing the state of the logs is not required.
    if prev_record == next_record {
        return Ok(());
    }

    // Update logs and save to disk.
    changelog.update().unwrap();
    changelog.store().unwrap();

    let (build, changelog) = changelog.last_record().unwrap();

    let post = format!("<b>Daily Build: {build}</b>\n\n{changelog}");
    let bot = bot::Bot::new(bot_token);
    bot.send_message(&chat_id.clone(), &post)?;
    Ok(())
}
