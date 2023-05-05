use anyhow::Result;
use lazy_static::lazy_static;
use log::info;
use polygen_bot::{consts::LEADERBOARD_URL, socket::new_bot, BotData, Config};
use regex::Regex;
use std::{fs, thread};

lazy_static! {
    static ref CONFIG: String = fs::read_to_string("config.toml").unwrap();
    static ref USERNAMES: Vec<String> = {
        (|| -> Result<_> {
            let config: Config = toml::from_str(&CONFIG)?;

            let mut usernames = Vec::new();

            let client = reqwest::blocking::Client::new();

            let re = Regex::new("/user/(.*?)\"")?;

            for (id, bot) in config.bots.iter().enumerate() {
                let res = client
                    .get(LEADERBOARD_URL)
                    .header("cookie", bot.cookie)
                    .send()
                    .and_then(|res| res.text())?;

                match re.captures(&res) {
                    None => panic!("cookie No.{} has expired", id + 1),
                    Some(cap) => {
                        usernames.push(
                            cap.get(1)
                                .map_or(String::from(""), |m| m.as_str().to_string()),
                        );
                    }
                }
            }

            Ok(usernames)
        })()
        .unwrap()
    };
    static ref BOT_DATA: Vec<BotData> = {
        (|| -> Result<_> {
            let config: Config = toml::from_str(&CONFIG)?;

            let mut ans = Vec::new();

            for (id, bot) in config.bots.into_iter().enumerate() {
                ans.push(BotData {
                    bot,
                    room: config.rooms.get(&bot.room).copied(),
                    name: &USERNAMES[id],
                });
            }

            info!("{:?}", ans);

            Ok(ans)
        })()
        .unwrap()
    };
}

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    for bot_data in BOT_DATA.iter() {
        new_bot(bot_data)?;
    }

    loop {
        thread::park();
    }
}
