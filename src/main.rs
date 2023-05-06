use anyhow::Result;
use lazy_static::lazy_static;
use log::info;
use polygen_bot::{socket::new_bot, BotData, Config};
use std::{fs, thread};

lazy_static! {
    static ref CONFIG: String = fs::read_to_string("config.toml").unwrap();
    static ref BOT_DATA: Vec<BotData> = {
        (|| -> Result<_> {
            let config: Config = toml::from_str(&CONFIG)?;

            let mut ans = Vec::new();

            for bot in config.bots{
                ans.push(BotData {
                    bot,
                    room: config.rooms.get(&bot.room).copied(),
                    base_url: config.base_url,
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
