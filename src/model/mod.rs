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
pub type Score = u64;

pub struct Model {
    pub config: Config,
    pub level: usize,
    pub turn: usize,
    pub score: Score,
    pub phase: Phase,
    pub grid: Grid,
    pub player: Player,
    pub visible_tiles: HashSet<vec2<Coord>>,
    pub items: Arena<BoardItem>,
    pub entities: Arena<Entity>,
    pub animations: Arena<Animation>,
    pub ending_animations: Vec<Animation>,
}

#[derive(Debug, Clone)]
pub enum Phase {
    /// Level transition.
    LevelStarting { timer: Lifetime },
    /// Shift and spawn items and enemies.
    Night {
        fade_time: Lifetime,
        light_time: Lifetime,
    },
    /// Resolve passive item effects.
    Passive {
        item_queue: Vec<Id>,
        start_delay: Lifetime,
        end_delay: Lifetime,
    },
    /// Player movement.
    Player,
    /// Place a tile on the map.
    Map { tiles_left: usize },
    /// Swap position with a magic item.
    Portal,
    /// Player sets their look direction.
    Vision,
    /// Vision has beet set, visualize.
    PostVision { timer: Lifetime },
    /// Select a new item.
    Select {
        options: Vec<ItemKind>,
        extra_items: usize,
    },
    /// Level has completed: either all enemies were killed (win) or player ran out of turns.
    LevelFinished { win: bool, timer: Lifetime },
    /// Game over, you lost.
    GameOver,
}

impl Model {
    pub fn new(config: Config) -> Self {
        let mut model = Self {
            level: 0,
            turn: 0,
            score: 0,
            config,
            phase: Phase::Night {
                fade_time: Lifetime::new_zero(r32(0.5)),
                light_time: Lifetime::new_max(r32(0.5)),
            },
            grid: Grid::new(3),
            player: Player::new(),
            visible_tiles: HashSet::new(),
            items: Arena::new(),
            entities: [Entity {
                position: vec2(0, 0),
                fraction: Fraction::Player,
                health: Health::new_max(100),
                look_dir: vec2(0, 0),
                kind: EntityKind::Player,
            }]
            .into_iter()
            .collect(),
            animations: Arena::new(),
            ending_animations: Vec::new(),
        };
        model.next_level();
        model
    }
}
