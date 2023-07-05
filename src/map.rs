use serde::Deserialize;
use std::ops::{Index, IndexMut};

pub type Pos = (usize, usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LandType {
    Land,
    General,
    City,
    Mountain,
    UnknownCity,
    Unknown,
    UnknownMountain,
}

impl LandType {
    #[inline]
    pub const fn from(t: u8) -> Self {
        match t {
            0 => Self::Land,
            1 => Self::General,
            2 => Self::City,
            3 => Self::Mountain,
            4 => Self::UnknownCity,
            5 => Self::Unknown,
            _ => Self::UnknownMountain,
        }
    }
}

impl Default for LandType {
    #[inline]
    fn default() -> Self {
        Self::Land
    }
}

#[derive(Deserialize, Clone, Copy)]
pub struct MaybeLand {
    pub c: Option<u8>,
    pub t: Option<u8>,
    pub a: Option<i32>,
}

#[derive(Clone, Copy, Default)]
pub struct Land {
    pub color: u8,
    pub r#type: LandType,
    pub amount: i32,
}

impl Land {
    #[inline]
    pub fn patch(&mut self, maybe_land: MaybeLand) {
        if let Some(color) = maybe_land.c {
            self.color = color;
        }
        if let Some(r#type) = maybe_land.t {
            self.r#type = LandType::from(r#type);
        }
        if let Some(amount) = maybe_land.a {
            self.amount += amount;
        }
    }

    #[inline]
    pub fn from(maybe_land: MaybeLand) -> Self {
        let mut res: Self = Default::default();
        res.patch(maybe_land);
        res
    }
}

#[derive(Deserialize, Clone)]
pub struct MaybeMap {
    pub width: usize,
    pub height: usize,
    pub gm: Vec<Vec<MaybeLand>>,
    pub mode: String,
}

#[derive(Clone)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub gm: Vec<Vec<Land>>,
    pub mode: String,
}

impl Index<Pos> for Map {
    type Output = Land;

    #[inline]
    fn index(&self, index: Pos) -> &Self::Output {
        &self.gm[index.0][index.1]
    }
}

impl IndexMut<Pos> for Map {
    #[inline]
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.gm[index.0][index.1]
    }
}

impl Map {
    #[inline]
    pub fn from(maybe_map: MaybeMap) -> Self {
        let gm = maybe_map
            .gm
            .into_iter()
            .map(|row| row.into_iter().map(Land::from).collect())
            .collect();
        Self {
            width: maybe_map.width,
            height: maybe_map.height,
            gm,
            mode: maybe_map.mode,
        }
    }

    #[inline]
    pub fn new() -> Self {
        Self {
            mode: "六边形".to_string(),
            width: 0,
            height: 0,
            gm: Vec::new(),
        }
    }

    #[inline]
    pub const fn check(&self, (i, j): Pos) -> bool {
        i >= 1 && i <= self.height && j >= 1 && j <= self.width
    }

    #[inline]
    pub fn accessible(&self, pos: Pos) -> bool {
        let land = &self[pos];
        land.r#type != LandType::Mountain && land.r#type != LandType::UnknownMountain
    }

    #[inline]
    pub fn dir(&self, (_, j): Pos) -> Vec<(i8, i8)> {
        match self.mode.as_str() {
            "六边形" if j % 2 == 1 => vec![(-1, -1), (-1, 0), (-1, 1), (0, 1), (1, 0), (0, -1)],
            "六边形" => vec![(0, -1), (-1, 0), (0, 1), (1, 1), (1, 0), (1, -1)],
            _ => vec![(-1, 0), (0, -1), (1, 0), (0, 1)],
        }
    }

    #[inline]
    pub fn neighbours(&self, (i, j): Pos) -> Vec<Pos> {
        self.dir((i, j))
            .into_iter()
            .map(|(dx, dy)| ((i as i8 + dx) as usize, (j as i8 + dy) as usize))
            .filter(|&pos| self.check(pos) && self.accessible(pos))
            .collect()
    }

    #[inline]
    pub fn iter(&self) -> impl IntoIterator<Item = (Pos, &Land)> + '_ {
        (1..=self.height)
            .flat_map(|x| (1..=self.width).map(move |y| (x, y)))
            .map(|pos| (pos, &self[pos]))
    }
}
