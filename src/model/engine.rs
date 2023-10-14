use rhai::{CallFnOptions, EvalAltResult};

use super::*;

pub struct Engine {
    inner: rhai::Engine,
}

type Result<T, E = Box<EvalAltResult>> = std::result::Result<T, E>;

macro_rules! call {
    ($engine:expr, $options:expr, $scope:expr, $script:expr, $name:expr, $args: expr $(,)?) => {{
        let script = $script;
        let name = $name;
        if script.iter_functions().any(|fun| fun.name == name) {
            Some($engine.call_fn_with_options($options, $scope, script, name, $args)?)
        } else {
            None
        }
    }};
}

impl Engine {
    pub fn empty() -> Self {
        Self {
            inner: rhai::Engine::new_raw(),
        }
    }

    pub fn new(state: Rc<RefCell<ModelState>>, side_effects: Rc<RefCell<Vec<Effect>>>) -> Self {
        let mut engine = rhai::Engine::new();

        // Types
        engine.register_type_with_name::<Id>("Id");
        engine
            .register_type_with_name::<InventoryItem>("Item")
            .register_get("turns_on_board", |item: &mut InventoryItem| {
                item.turns_on_board
            })
            .register_get("stats", |item: &mut InventoryItem| item.current_stats());
        engine
            .register_type_with_name::<ItemStats>("ItemStats")
            .register_get("damage", |stats: &mut ItemStats| {
                stats.damage.unwrap_or_default()
            });

        // Effects
        engine.register_fn("damage_nearest", {
            let state = Rc::clone(&state);
            let side_effects = Rc::clone(&side_effects);
            move |item: InventoryItem, damage: Hp| {
                item.damage_nearest(damage, &state.borrow(), &mut side_effects.borrow_mut())
            }
        });

        Self { inner: engine }
    }

    pub fn init_item(&self, kind: ItemKind) -> Result<InventoryItem> {
        // Base damage:
        // Sword => 2,
        // FireScroll => 5,
        // SoulCrystal => 0,
        // RadiationCore => 1,
        // GreedyPot => 1,
        // ElectricRod => 2,
        // Phantom => 1,
        // KingSkull => 3,
        // CharmingStaff => 0,
        // Solitude => 2,

        let mut state = Scope::new();
        let options = CallFnOptions::new().eval_ast(false).rewind_scope(false); // Retain variables
        call!(&self.inner, options, &mut state, &kind.script, "init", ());
        let base_stats = kind.config.base_stats.clone();

        Ok(InventoryItem {
            on_board: None,
            kind,
            state,
            turns_on_board: 0,
            base_stats,
            perm_stats: ItemStats::default(),
            temp_stats: ItemStats::default(),
        })
    }

    /// Call the item's trigger handler (if it is defined).
    /// Side effects produced by the script are put into [ModelState].
    ///
    /// *NOTE*: it borrows [ModelState] and mutates `side_effects`.
    pub fn item_trigger(&self, item: &mut InventoryItem, method: &str) -> Result<()> {
        let mut item_this = rhai::Dynamic::from(item.clone());
        let options = CallFnOptions::new().bind_this_ptr(&mut item_this);

        call!(
            &self.inner,
            options,
            &mut item.state,
            &item.kind.script,
            method,
            (),
        );
        Ok(())
    }

    pub fn compile_items(&self, all_items: &ItemAssets) -> Vec<ItemKind> {
        let mut items = Vec::with_capacity(all_items.assets.len());
        for item in all_items.assets.values() {
            let script = item.script.as_deref().unwrap_or("");
            let script = self.inner.compile(script).unwrap_or_else(|err| {
                println!("Script compilation failed for item {}", item.config.name);
                println!("{}", err);
                self.inner.compile("").unwrap()
            });
            let script = Rc::new(script);

            items.push(ItemKind {
                config: item.config.clone(),
                script,
            });
        }

        items
    }
}
