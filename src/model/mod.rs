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
    engine: Engine,
    pub state: Rc<RefCell<ModelState>>,
    pub level: usize,
    pub turn: usize,
    pub score: Score,
    pub phase: Phase,

    pub animations: Arena<Animation>,
    pub ending_animations: Vec<Animation>,

    pub resolving_items: Collection<ItemResolving>,
    pub resolved_items: Collection<ItemResolved>,

    /// The stack of effect queues.
    pub effect_queue_stack: Vec<VecDeque<QueuedEffect>>,
    /// Effects produced by scripts. Should be consumed after the script is executed and moved to the queue.
    pub side_effects: Rc<RefCell<Vec<Effect>>>,
}

#[derive(Debug, HasId)]
pub struct ItemResolving {
    #[has_id(id)]
    pub board_item: Id,
    pub animations: Vec<Id>,
}

#[derive(Debug, HasId)]
pub struct ItemResolved {
    #[has_id(id)]
    pub board_item: Id,
    pub time: Lifetime,
}

/// The stuff accessible from within the scripts.
pub struct ModelState {
    pub all_items: Vec<ItemKind>,
    pub grid: Grid,
    pub player: Player,
    pub items: Arena<BoardItem>,
    pub entities: Arena<Entity>,
    pub visible_tiles: HashSet<vec2<Coord>>,
}

#[derive(Debug, Clone)]
pub enum Phase {
    /// Level transition.
    LevelStarting { timer: Lifetime },
    /// Resolve night effects.
    Night { fade_time: Lifetime },
    /// Shift and spawn items and enemies.
    Dawn { light_time: Lifetime },
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
    },
    /// Place a tile on the map.
    Map {
        tiles_left: usize,
        /// Phase to go to after the tiles have been opened.
        next_phase: Box<Phase>,
    },
    /// Swap position with a magic item.
    Portal {
        /// Phase to go to after the teleport.
        next_phase: Box<Phase>,
    },
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
            all_items: vec![], // Initialized after engine
            grid: Grid::new(3),
            player: Player::new(),
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
            visible_tiles: HashSet::new(),
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

        {
            // Initialize player items
            let player_items = &mut state.borrow_mut().player.items;
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
        }

        state.borrow_mut().all_items = all_items;

        Self::new_compiled(assets, config, engine, state, side_effects)
    }

    fn new_compiled(
        assets: Rc<Assets>,
        config: Config,
        engine: Engine,
        state: Rc<RefCell<ModelState>>,
        side_effects: Rc<RefCell<Vec<Effect>>>,
    ) -> Self {
        let mut model = Self {
            assets,
            config,
            engine,
            state,
            level: 0,
            turn: 0,
            score: 0,
            phase: Phase::Dawn {
                light_time: Lifetime::new_max(r32(0.5)),
            },

            animations: Arena::new(),
            ending_animations: Vec::new(),

            resolving_items: Collection::new(),
            resolved_items: Collection::new(),

            effect_queue_stack: Vec::new(),
            side_effects,
        };
        model.next_level();
        model
    }
}
