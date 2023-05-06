use serde::Deserialize;
use std::collections::HashMap;

mod bot;
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

const fn default_calc_cnt() -> u8 {
    1
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct BotConfig<'a> {
    pub cookie: &'a str,
    pub room: &'a str,
    pub auto_ready: AutoReady,

    pub name: &'a str,

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
    pub base_url: &'a str,
    #[serde(borrow)]
    pub bots: Vec<BotConfig<'a>>,
    pub rooms: HashMap<&'a str, RoomConfig<'a>>,
}

#[derive(Clone, Debug)]
pub struct BotData {
    pub bot: BotConfig<'static>,
    pub room: Option<RoomConfig<'static>>,
    pub base_url: &'static str
}
