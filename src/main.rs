use anyhow::Result;
use lazy_static::lazy_static;
use log::{info, warn};
use polygen_bot::{socket::new_bot, BotData, Config};
use std::{fs, sync::mpsc::channel};

lazy_static! {
    static ref CONFIG: String = fs::read_to_string("config.toml").unwrap();
    static ref BOT_DATA: Vec<BotData> = {
        (|| -> Result<_> {
            let config: Config = toml::from_str(&CONFIG)?;

            let mut ans = Vec::new();

            for bot in config.bots {
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

    let mut clients = Vec::new();

    for bot_data in BOT_DATA.iter() {
        clients.push(new_bot(bot_data)?);
    }

    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))?;

    rx.recv()?;

    warn!("Shutting down...");

    for client in clients {
        client.disconnect()?;
    }

    Ok(())
}
