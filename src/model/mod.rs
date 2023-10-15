mod animation;
mod effect;
mod engine;
mod entity;
mod grid;
mod item;
mod logic;
mod player;

pub use self::{animation::*, entity::*, grid::*, item::*, player::*};
use self::{effect::*, engine::Engine};

use crate::prelude::*;

use std::collections::VecDeque;

pub type Script = rune::Unit;
pub type ScriptFunction = rune::runtime::Function;
pub type Time = R32;
pub type Coord = i64;
pub type Score = u64;

pub struct Model {
    pub assets: Rc<Assets>,
    pub config: Config,
    pub all_items: Vec<ItemKind>,
    engine: Engine,
    pub state: Rc<RefCell<ModelState>>,
    pub level: usize,
    pub turn: usize,
    pub score: Score,
    pub phase: Phase,
    pub grid: Grid,
    pub player: Player,
    pub visible_tiles: HashSet<vec2<Coord>>,
    pub animations: Arena<Animation>,
    pub ending_animations: Vec<Animation>,
    /// The stack of effect queues.
    pub effect_queue_stack: Vec<VecDeque<QueuedEffect>>,
    /// Effects produced by scripts. Should be consumed after the script is executed and moved to the queue.
    pub side_effects: Rc<RefCell<Vec<Effect>>>,
}

/// The stuff accessible from within the scripts.
#[derive(Debug)]
pub struct ModelState {
    pub items: Arena<BoardItem>,
    pub entities: Arena<Entity>,
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
        start_delay: Lifetime,
        end_delay: Lifetime,
    },
    /// Player movement.
    Player,
    /// Player moving and activating an item.
    Active {
        /// Id of the player.
        entity_id: Id,
        /// Target movement position
        position: vec2<Coord>,
        /// Item to swap with.
        item: Option<Id>,
        /// Entity to swap with.
        entity: Option<Id>,
    },
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
    pub fn new(assets: Rc<Assets>, config: Config, all_items: &ItemAssets) -> Self {
        let state = ModelState {
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
        };
        let state = Rc::new(RefCell::new(state));

        // TODO: maybe mpsc or smth
        let side_effects = Rc::new(RefCell::new(Vec::new()));

        // TODO: handle errors maybe
        let engine = Engine::new(Rc::clone(&state), Rc::clone(&side_effects))
            .expect("Script engine initialization failed");
        let all_items = engine
            .compile_items(all_items)
            .expect("Item compilation failed");

        Self::new_compiled(assets, config, engine, all_items, state, side_effects)
    }

    fn new_compiled(
        assets: Rc<Assets>,
        config: Config,
        engine: Engine,
        all_items: Vec<ItemKind>,
        state: Rc<RefCell<ModelState>>,
        side_effects: Rc<RefCell<Vec<Effect>>>,
    ) -> Self {
        let mut player_items = Arena::new();
        for item in &config.starting_items {
            match all_items.iter().find(|kind| *kind.config.name == **item) {
                Some(item) => {
                    let item = engine
                        .init_item(item.clone())
                        .expect("Item initialization failed");
                    player_items.insert(item);
                }
                None => {
                    panic!("Unknown item {}", item);
                }
            };
        }

        let mut model = Self {
            assets,
            config,
            all_items,
            engine,
            state,
            level: 0,
            turn: 0,
            score: 0,
            phase: Phase::Night {
                fade_time: Lifetime::new_zero(r32(0.5)),
                light_time: Lifetime::new_max(r32(0.5)),
            },
            grid: Grid::new(3),
            player: Player::new(player_items),
            visible_tiles: HashSet::new(),
            animations: Arena::new(),
            ending_animations: Vec::new(),
            effect_queue_stack: Vec::new(),
            side_effects,
        };
        model.next_level();
        model
    }
}
