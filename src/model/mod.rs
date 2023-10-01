mod animation;
mod entity;
mod grid;
mod item;
mod logic;
mod player;

pub use self::{animation::*, entity::*, grid::*, item::*, player::*};

use crate::prelude::*;

pub type Time = R32;
pub type Coord = i64;

pub struct Model {
    pub config: Config,
    pub level: usize,
    pub phase: Phase,
    pub grid: Grid,
    pub player: Player,
    pub visible_tiles: HashSet<vec2<Coord>>,
    pub items: Arena<BoardItem>,
    pub entities: Vec<Entity>,
    pub animations: Vec<Animation>,
}

#[derive(Debug, Clone)]
pub enum Phase {
    /// Shift and spawn items and enemies.
    Night,
    /// Resolve passive item effects.
    Passive {
        item_queue: Vec<Id>,
        start_delay: Lifetime,
        end_delay: Lifetime,
    },
    /// Player movement.
    Player,
    /// Resolve active item effects.
    Active {
        fraction: Fraction,
        item_id: Id,
        start_delay: Lifetime,
        end_delay: Lifetime,
    },
    /// Place a tile on the map.
    Map { tiles_left: usize },
    /// Player sets their look direction.
    Vision,
    /// Select a new item.
    Select { options: Vec<ItemKind> },
}

impl Model {
    pub fn new(config: Config) -> Self {
        let mut model = Self {
            level: 0,
            config,
            phase: Phase::Night,
            grid: Grid::new(3),
            player: Player::new(),
            visible_tiles: HashSet::new(),
            items: Arena::new(),
            entities: vec![Entity {
                position: vec2(0, 0),
                fraction: Fraction::Player,
                health: Health::new_max(100),
                look_dir: vec2(0, 1),
                kind: EntityKind::Player,
            }],
            animations: Vec::new(),
        };
        model.next_level();
        model
    }
}
