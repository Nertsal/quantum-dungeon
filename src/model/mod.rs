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
    pub grid_size: vec2<Coord>,
    pub player: Player,
}

impl Model {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            turn: 0,
            grid_size: vec2(10, 10),
            player: Player::new(vec2(0, 0)),
        }
    }
}
