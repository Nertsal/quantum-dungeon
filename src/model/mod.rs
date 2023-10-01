mod animation;
mod entity;
mod grid;
mod item;
mod logic;
mod player;

pub use self::{animation::*, entity::*, grid::*, item::*, player::*};

use crate::prelude::*;

pub type Time = R32;
pub type Turn = u64;
pub type Coord = i64;

pub struct Model {
    pub config: Config,
    pub turn: Turn,
    pub phase: Phase,
    pub grid: Grid,
    pub player: Player,
    pub visible_tiles: HashSet<vec2<Coord>>,
    pub items: Vec<Item>,
    pub entities: Vec<Entity>,
    pub animations: Vec<Animation>,
}

#[derive(Debug, Clone)]
pub enum Phase {
    /// Shift and spawn items and enemies.
    Night,
    /// Resolve passive item effects.
    Passive {
        current_item: usize,
        start_delay: Lifetime,
        end_delay: Lifetime,
    },
    /// Player movement.
    Player,
    /// Resolve active item effects.
    Active {
        fraction: Fraction,
        item_id: usize,
        start_delay: Lifetime,
        end_delay: Lifetime,
    },
    /// Place a tile on the map.
    Map,
    /// Player sets their look direction.
    Vision,
    /// Select a new item.
    Select { options: Vec<ItemKind> },
}

impl Model {
    pub fn new(config: Config) -> Self {
        let mut model = Self {
            config,
            turn: 0,
            phase: Phase::Night,
            grid: Grid::new(3),
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
            animations: Vec::new(),
        };
        model.night_phase();
        model.update_vision();
        model
    }
}
