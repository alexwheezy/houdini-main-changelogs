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

mod bot;
mod html;
mod log;
mod parsers;
mod utils;

use crate::parsers::*;

fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv()?;

    let bot_token = env::var("BOT_TOKEN")
        .ok()
        .expect("Expected BOT_TOKEN env var");
    let chat_id = env::var("CHAT_ID").ok().expect("Expected CHAT_ID env var");

    //let html = Document::from(include_str!("sidefx_changelogs.html"));
    //let document = Document::from(html);
    let html = reqwest::get("https://sidefx.com/changelog")?.text()?;
    let document = Document::from(html.as_str());
    let mut changelog = parse_change_log(&document)?;

    changelog.update().unwrap();
    changelog.store().unwrap();

    let (build, changelog) = changelog.last_record().unwrap();
    let publish = format!("<b>Daily Build: {build}</b>\n\n{changelog}");
    println!("{publish}");

    //let bot = bot::Bot::new(bot_token);
    //bot.send_message(&chat_id.clone(), &publish)?;

    Ok(())
}
