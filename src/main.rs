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
use std::{env, io::stdin};

mod bot;
mod html;
mod log;
mod parsers;
mod utils;

use crate::parsers::parse_change_log;

fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv()?;

    let bot_token = env::var("BOT_TOKEN")
        .ok()
        .expect("Expected BOT_TOKEN env var");
    let chat_id = env::var("CHAT_ID").ok().expect("Expected CHAT_ID env var");

    let html = reqwest::get("https://sidefx.com/changelog")?.text()?;
    let document = Document::from(html.as_str());
    let mut changelog = parse_change_log(&document)?;

    changelog.update().unwrap();
    changelog.store().unwrap();

    let (build, changelog) = changelog.last_record().unwrap();
    if changelog.is_empty() {
        println!("There have been no updates lately. Come back later.");
        return Ok(());
    }

    let post = format!("<b>Daily Build: {build}</b>\n\n{changelog}");
    println!("Preview Post:\n\n{post}");

    println!("Are we posting this?");
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    if input.trim() == "Y" {
        println!("Sending...");
        let bot = bot::Bot::new(bot_token);
        bot.send_message(&chat_id.clone(), &post)?;
    }

    Ok(())
}
