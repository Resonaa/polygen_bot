use crate::map::{MaybeLand, MaybeMap, Pos};
use anyhow::Result;
use rust_socketio::{Payload, RawClient};
use serde::Deserialize;

pub fn callback<T, R>(mut input: T) -> impl FnMut(Payload, RawClient) + 'static + Sync + Send
where
    T: FnMut(String, RawClient) -> R + 'static + Sync + Send,
    R: Into<Result<()>>,
{
    move |payload, socket| {
        if let Payload::String(str) = payload {
            if let Err(err) = input(str, socket).into() {
                error!("{:?}", err);
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct GameStart {
    #[serde(rename = "maybeMap")]
    pub maybe_map: MaybeMap,

    #[serde(rename = "myColor")]
    pub my_color: u8,
}

pub type Patch = Vec<(Pos, MaybeLand)>;

pub type UpdateTeams = Vec<(usize, Vec<String>)>;

pub type Rank = Vec<(i32, String, u32, u32)>;
