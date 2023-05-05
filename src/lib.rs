use consts::default_calc_cnt;
use serde::Deserialize;
use std::collections::HashMap;

mod bot;
pub mod consts;
mod event;
mod map;
pub mod socket;

#[macro_use]
extern crate log;

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(untagged)]
pub enum AutoReady {
    Unconditional(bool),
    Conditional { more_than: usize },
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct BotConfig<'a> {
    pub cookie: &'a str,
    pub room: &'a str,
    pub auto_ready: AutoReady,

    #[serde(default = "default_calc_cnt")]
    pub calc_cnt: u8,
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct RoomConfig<'a> {
    pub mode: Option<&'a str>,
    pub map: Option<&'a str>,
    pub speed: Option<f64>,
}

#[derive(Deserialize)]
pub struct Config<'a> {
    #[serde(borrow)]
    pub bots: Vec<BotConfig<'a>>,
    pub rooms: HashMap<&'a str, RoomConfig<'a>>,
}

#[derive(Clone, Debug)]
pub struct BotData {
    pub bot: BotConfig<'static>,
    pub room: Option<RoomConfig<'static>>,
    pub name: &'static str,
}
