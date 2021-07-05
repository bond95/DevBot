use teloxide::{prelude::*, utils::command::BotCommand};
use rusqlite::{params, Connection};
use std::sync::{Mutex};
use std::error::Error;

use lazy_static::lazy_static;
use if_chain::if_chain;
use regex::Regex;
use std::env;

lazy_static! {
    static ref DB : Mutex<Connection> = Mutex::new(Connection::open(&String::from("./my_db.db3")).unwrap());
}

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "add link to DB from parameter or from replied message")]
    AddLink(String),
    #[command(description = "get all interesting links.")]
    GetLinks,
}

fn get_links() -> Result<String, Box<dyn Error>> {
    let mut resp: String = "".to_owned();
    let db_unwraped = DB.lock().unwrap();
    let mut stmt = db_unwraped.prepare("SELECT id, link FROM interesting_links").unwrap();
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let id: i32 = row.get(0).unwrap();
        let link: String = row.get(1).unwrap();
        resp = format!("{}{:?}. {:?}\n", resp, id, link);
    }
    Ok(resp)
}

async fn answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"((http://www\.|https://www\.|http://|https://)?[a-zA-Z0-9]+([\-\.]{1}[a-zA-Z0-9]+)*\.[a-z]{2,5}(:[0-9]{1,5})?(/.*)?)").unwrap();
    }
    match command {
        Command::Help => cx.answer(Command::descriptions()).await?,
        Command::AddLink(message_text) => {
            if RE.is_match(&message_text) {
                for cap in RE.captures_iter(&message_text) {
                    DB.lock().unwrap().execute(
                        "INSERT INTO interesting_links (link) VALUES (?1)",
                        params![&cap[1]],
                    );
                }
                cx.answer("Link saved").await?
            } else {
                if_chain! {
                    if let Some(reply) = cx.update.reply_to_message();
                    if let Some(reply_text) = reply.text();
                    if RE.is_match(&reply_text);
                    then {
                        for cap in RE.captures_iter(&reply_text) {
                            DB.lock().unwrap().execute(
                                "INSERT INTO interesting_links (link) VALUES (?1)",
                                params![&cap[1]],
                            );
                        }
                        cx.answer("Link saved").await?
                    } else {
                        cx.answer("Link not found").await?
                    }
                }
            }
        },
        Command::GetLinks => {
            let links: String = get_links().unwrap();
            cx.answer(links).await?
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting dev_bot...");
    DB.lock().unwrap().execute(
        "CREATE TABLE interesting_links (
                  id              INTEGER PRIMARY KEY,
                  link            TEXT NOT NULL,
                  type            TEXT
                  )",
        [],
    );

    let bot = Bot::from_env().auto_send();

    let bot_name: String = env::var("TELEGRAM_BOT_NAME")
        .map_err(|_| "telegram bot name is not set")
        .unwrap();
    teloxide::commands_repl(bot, bot_name, answer).await;
}
