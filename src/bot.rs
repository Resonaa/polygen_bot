use crate::{
    map::{LandType, Map, Pos},
    BotData,
};
use fastrand::Rng;
use std::collections::{HashMap, VecDeque};

type Movement = (Pos, Pos, bool);

pub struct Bot {
    pub gm: Map,
    pub my_color: u8,
    pub target: Option<Pos>,
    pub teammates: Vec<u8>,
    from: Option<Pos>,
    config: &'static BotData,
    rng: Rng,
}

impl Bot {
    pub fn new(config: &'static BotData) -> Self {
        Self {
            config,
            rng: Rng::new(),
            my_color: 0,
            target: None,
            from: None,
            gm: Map::new(),
            teammates: Vec::new(),
        }
    }

    fn move_to(&self, from: Pos, to: Pos) -> Movement {
        let from_land = &self.gm[from];
        let to_land = &self.gm[to];

        let mut half_tag = false;

        if self.gm.accessible(to)
            && to_land.color != self.my_color
            && to_land.color != 0
            && (from_land.amount as i32 - 1) / 2 > to_land.amount as i32
            && !self.teammates.contains(&to_land.color)
        {
            for neighbour in self.gm.neighbours(from) {
                let land = &self.gm[neighbour];

                if land.color != self.my_color
                    && land.color != 0
                    && matches!(land.r#type, LandType::City | LandType::Land)
                    && neighbour != to
                    && !self.teammates.contains(&land.color)
                {
                    half_tag = true;
                    break;
                }
            }
        }

        if to_land.r#type == LandType::City
            && to_land.color == 0
            && from_land.amount > 25
            && !half_tag
        {
            for neighbour in self.gm.neighbours(from) {
                let land = &self.gm[neighbour];

                if land.color != self.my_color
                    && !self.teammates.contains(&land.color)
                    && matches!(land.r#type, LandType::City | LandType::Land)
                    && neighbour != to
                {
                    half_tag = true;
                    break;
                }
            }
        }

        (from, to, half_tag)
    }

    fn new_target(&self) -> Option<Pos> {
        let mut targets = Vec::new();

        for (pos, land) in self.gm.iter() {
            if self.gm.accessible(pos)
                && !matches!(land.r#type, LandType::Unknown | LandType::UnknownCity)
                && land.color != self.my_color
                && !self.teammates.contains(&land.color)
            {
                targets.push(pos);
            }
        }

        self.rng.shuffle(&mut targets);

        let get_score = |&pos: &Pos| {
            let land = &self.gm[pos];
            match land.r#type {
                LandType::General => 1,
                LandType::City if land.color != 0 => 1,
                LandType::City => 5,
                LandType::Land if land.color != 0 => 2,
                LandType::Land => 3,
                _ => 9,
            }
        };

        targets.sort_unstable_by_key(|target| get_score(target));

        targets.first().copied()
    }

    pub fn expand(&mut self) -> Option<Movement> {
        let mut moves = Vec::new();

        for (from, from_land) in self.gm.iter() {
            if from_land.color == self.my_color {
                for to in self.gm.neighbours(from) {
                    let to_land = &self.gm[to];

                    let delta = if to_land.r#type == LandType::City && to_land.color != 0 {
                        2
                    } else {
                        1
                    };

                    if to_land.color != self.my_color
                        && from_land.amount > to_land.amount + delta
                        && !self.teammates.contains(&to_land.color)
                    {
                        moves.push((from, to));
                    }
                }
            }
        }

        self.rng.shuffle(&mut moves);

        let get_score = |&from: &Pos, &to: &Pos| {
            let from_land = &self.gm[from];
            let to_land = &self.gm[to];

            let mut score = match to_land.r#type {
                LandType::General => 1,
                LandType::City if to_land.color != 0 => 2,
                LandType::City => 4,
                LandType::Land if to_land.color != 0 => 3,
                LandType::Land => 5,
                _ => 9,
            };

            if from_land.r#type == LandType::Land
                && matches!(to_land.r#type, LandType::General | LandType::City)
            {
                score -= 20 - (from_land.amount - to_land.amount).min(10) as i8;
            }

            let (_, _, half_tag) = self.move_to(from, to);

            let from_remain = if half_tag { from_land.amount / 2 } else { 1 };

            for neighbour in self.gm.neighbours(from) {
                if self.gm[neighbour].color != self.my_color
                    && self.gm[neighbour].amount > from_remain + 1
                    && neighbour != to
                {
                    score += 10;
                    break;
                }
            }

            score
        };

        moves.sort_unstable_by_key(|(from, to)| get_score(from, to));

        match moves.first() {
            Some(&(from, to)) => {
                if Some(from) == self.from && Some(to) != self.target {
                    self.target = None;
                }

                Some(self.move_to(from, to))
            }
            None => self.move_to_target(0),
        }
    }

    fn move_to_target(&mut self, try_time: u8) -> Option<Movement> {
        if try_time >= self.config.bot.calc_cnt {
            return None;
        }

        if self.target.is_none()
            || matches!(&self.target, Some(target) if self.gm[*target].color == self.my_color
                                                   || self.teammates.contains(&self.gm[*target].color))
        {
            self.target = self.new_target();
            self.from = None;
        }

        let target = self.target?;

        let get_score = |pos: Pos| {
            let land = &self.gm[pos];

            if land.color == self.my_color || self.teammates.contains(&land.color) {
                land.amount as i32 - 1
            } else {
                -(land.amount as i32) - 1
            }
        };

        let mut max_ans = None;
        let mut max_score = f64::MIN;
        let mut new_from = None;

        let mut q = VecDeque::new();
        let mut vis = HashMap::new();

        let mut found_enemy = false;

        for (_, land) in self.gm.iter() {
            if land.color != self.my_color
                && land.color != 0
                && !self.teammates.contains(&land.color)
                && matches!(
                    land.r#type,
                    LandType::General | LandType::City | LandType::Land
                )
            {
                found_enemy = true;
                break;
            }
        }

        let mut bfs = |from: Pos| {
            let mut tmp_ans = None;
            let mut tmp_score = f64::MIN;
            let mut tmp_from = None;

            for try_time in 0..self.config.bot.calc_cnt {
                q.clear();
                vis.clear();

                q.push_back((from, get_score(from), 0, None));
                vis.insert(from, ());

                while let Some((cur, amount, length, ans)) = q.pop_front() {
                    if cur == target {
                        let score = amount as f64 / (length as f64);

                        if score > tmp_score && !(amount < 0 && length < 2) {
                            tmp_score = score;
                            tmp_ans = ans;

                            tmp_from = Some(from);

                            continue;
                        }
                    }

                    if !found_enemy && length > 6 {
                        continue;
                    }

                    let mut neighbours = self.gm.neighbours(cur);
                    self.rng.shuffle(&mut neighbours);

                    for nxt in neighbours {
                        if self.gm[nxt].r#type == LandType::General
                            && self.teammates.contains(&self.gm[nxt].color)
                        {
                            continue;
                        }

                        vis.entry(nxt).or_insert_with(|| {
                            if cur == from {
                                q.push_back((nxt, amount + get_score(nxt), length + 1, Some(nxt)));
                            } else {
                                q.push_back((nxt, amount + get_score(nxt), length + 1, ans));
                            }
                        });
                    }
                }

                if try_time == 2 && tmp_score < max_score / 2.0 {
                    break;
                }
            }

            if tmp_score > max_score {
                max_score = tmp_score;
                max_ans = tmp_ans;
                new_from = tmp_from;
            }
        };

        match &self.from {
            Some(from) => bfs(*from),
            _ => {
                'outer: for (pos, land) in self.gm.iter() {
                    if land.color == self.my_color && land.amount > 1 {
                        for neighbour in self.gm.neighbours(pos) {
                            let land = &self.gm[neighbour];

                            if land.color != self.my_color
                                && land.color != 0
                                && !self.teammates.contains(&land.color)
                                && matches!(land.r#type, LandType::Land | LandType::City)
                            {
                                continue 'outer;
                            }
                        }

                        bfs(pos);
                    }
                }
            }
        }

        if max_ans.is_none() {
            self.target = None;
            return self.move_to_target(try_time + 1);
        }

        let max_ans = max_ans.unwrap();

        if max_ans == target {
            self.target = None;
        }

        if self.from.is_none() {
            self.from = new_from;
        }

        let ans = self.move_to(self.from.unwrap(), max_ans);
        self.from = Some(max_ans);
        Some(ans)
    }
}
