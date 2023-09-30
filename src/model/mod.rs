mod entity;
mod item;
mod logic;
mod player;

pub use self::{entity::*, item::*, player::*};

use crate::prelude::*;

pub type Time = R32;
pub type Turn = u64;
pub type Coord = i64;

pub struct Model {
    pub config: Config,
    pub turn: Turn,
    pub grid: Grid,
    pub player: Player,
    pub visible_tiles: HashSet<vec2<Coord>>,
    pub items: Vec<Item>,
    pub entities: Vec<Entity>,
}

pub struct Grid {
    pub size: vec2<Coord>,
}

impl Model {
    pub fn new(config: Config) -> Self {
        let mut model = Self {
            config,
            turn: 0,
            grid: Grid { size: vec2(10, 10) },
            player: Player::new(),
            visible_tiles: HashSet::new(),
            items: Vec::new(),
            entities: vec![Entity {
                position: vec2(0, 0),
                fraction: Fraction::Player,
                health: Health::new_max(100),
                look_dir: vec2(0, 1),
                kind: EntityKind::Player,
            }],
        };
        model.night_phase();
        model.update_vision();
        model
    }
}

impl Grid {
    pub fn clamp_pos(&self, pos: vec2<Coord>) -> vec2<Coord> {
        vec2(
            pos.x.clamp(0, self.size.x - 1),
            pos.y.clamp(0, self.size.y - 1),
        )
    }
}
