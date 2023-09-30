mod logic;
mod player;

pub use self::player::*;

use crate::prelude::*;

pub type Time = R32;
pub type Turn = u64;
pub type Coord = i64;

pub struct Model {
    pub config: Config,
    pub turn: Turn,
    pub grid: Grid,
    pub player: Player,
}

pub struct Grid {
    pub size: vec2<Coord>,
}

impl Model {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            turn: 0,
            grid: Grid { size: vec2(10, 10) },
            player: Player::new(vec2(0, 0)),
        }
    }
}

impl Grid {
    pub fn clamp_pos(&self, pos: vec2<Coord>) -> vec2<Coord> {
        vec2(pos.x.clamp(0, self.size.x), pos.y.clamp(0, self.size.y))
    }
}
