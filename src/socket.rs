use crate::{
    bot::Bot,
    event::{self, callback, UpdateTeams},
    map::Map,
    AutoReady, BotData,
};
use anyhow::Result;
use parking_lot::Mutex;
use rust_socketio::{client::Client, ClientBuilder, RawClient};
use serde_json::json;
use std::sync::Arc;

fn ready(socket: &RawClient, config: &BotData) -> Result<()> {
    if let AutoReady::Unconditional(true) = config.bot.auto_ready {
        socket.emit("ready", json!(()))?;
    }

    Ok(())
}

fn vote(socket: &RawClient, config: &BotData) -> Result<()> {
    if let Some(room) = config.room {
        if let Some(mode) = room.mode {
            socket.emit("vote", json!({"item": "mode", "value": mode}))?;
        }
        if let Some(map) = room.map {
            socket.emit("vote", json!({"item": "map", "value": map}))?;
        }
        if let Some(speed) = room.speed {
            socket.emit("vote", json!({"item": "speed", "value": speed}))?;
        }
    }

    Ok(())
}

pub fn new_bot(config: &'static BotData) -> Result<Client> {
    let global_bot = Arc::new(Mutex::new(Bot::new(config)));
    let global_is_ready = Arc::new(Mutex::new(false));
    let global_teams = Arc::new(Mutex::new(UpdateTeams::new()));

    let open = move |_, socket: RawClient| {
        info!("{} connected", config.bot.name);
        socket.emit("joinRoom", json!(config.bot.room))?;
        vote(&socket, config)?;
        ready(&socket, config)
    };

    let bot = global_bot.clone();
    let game_start = move |payload: String, _| {
        use event::GameStart;

        let game_start: GameStart = serde_json::from_str(&payload)?;

        let mut bot = bot.lock();
        bot.target = None;
        bot.gm = Map::from(game_start.maybe_map);
        bot.my_color = if game_start.my_color == -1 {
            0
        } else {
            game_start.my_color as u8
        };

        Ok(())
    };

    let bot = global_bot.clone();
    let teams = global_teams.clone();
    let patch = move |payload: String, socket: RawClient| {
        use event::Patch;

        let data =
            lz_str::decompress_from_utf16(&serde_json::from_str::<String>(&payload)?).unwrap();
        let string = String::from_utf16(data.as_slice())?;
        let patch: Patch = serde_json::from_str(&string)?;

        let mut bot = bot.lock();

        for (id, data) in patch.updates {
            let y = (id - 1) % bot.gm.width + 1;
            let x = (id - y) / bot.gm.width + 1;
            let pos = (x, y);
            bot.gm[pos].patch(data);
        }

        if let Some(movement) = bot.expand() {
            socket.emit("move", json!(movement))?;
        }

        if bot.teammates.is_empty() {
            let teams = teams.lock();
            let bot_name = config.bot.name.to_string();

            for (_, color, username, _, _) in patch.rank {
                if color != -1
                    && (color as u8) != bot.my_color
                    && username != bot_name
                    && teams.iter().any(|(_, players)| {
                        players.contains(&bot_name) && players.contains(&username)
                    })
                {
                    bot.teammates.push(color as u8);
                }
            }
        }

        Ok(())
    };

    let bot = global_bot;
    let is_ready = global_is_ready.clone();
    let teams = global_teams.clone();
    let win = move |payload: String, socket| {
        let winner: &str = serde_json::from_str(&payload)?;

        info!("Room {}: {} won", config.bot.room, winner);

        let mut bot = bot.lock();

        bot.target = None;
        bot.teammates.clear();

        *is_ready.lock() = false;

        ready(&socket, config)?;

        if let AutoReady::Conditional { more_than } = config.bot.auto_ready {
            let teams = teams.lock();
            let count = teams
                .iter()
                .filter(|(id, _)| *id != 0)
                .flat_map(|(_, players)| players)
                .count();

            let mut is_ready = is_ready.lock();

            if count > more_than && !*is_ready {
                *is_ready = true;
                socket.emit("ready", json!(()))?;
            } else if count <= more_than && *is_ready {
                *is_ready = false;
                socket.emit("ready", json!(()))?;
            }
        }

        Ok(())
    };

    let is_ready = global_is_ready;
    let teams = global_teams;
    let update_teams = move |payload: String, socket: RawClient| {
        let mut teams = teams.lock();

        *teams = serde_json::from_str::<UpdateTeams>(&payload)?;

        if let AutoReady::Conditional { more_than } = config.bot.auto_ready {
            let count = teams
                .iter()
                .filter(|(id, _)| *id != 0)
                .flat_map(|(_, players)| players)
                .count();

            let mut is_ready = is_ready.lock();

            if count > more_than && !*is_ready {
                *is_ready = true;
                socket.emit("ready", json!(()))?;
            } else if count <= more_than && *is_ready {
                *is_ready = false;
                socket.emit("ready", json!(()))?;
            }
        }

        Ok(())
    };

    let client = ClientBuilder::new(config.base_url)
        .opening_header("cookie", config.bot.cookie)
        .on("open", callback(open))
        .on("close", move |_, _| {
            error!("{} disconnected", config.bot.name)
        })
        .on("gameStart", callback(game_start))
        .on("patch", callback(patch))
        .on("win", callback(win))
        .on("updateTeams", callback(update_teams))
        .connect()?;

    Ok(client)
}
