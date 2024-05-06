use super::*;

use anyhow::Result;
use rune::{
    runtime::RuntimeContext,
    termcolor::{ColorChoice, StandardStream},
    Context, ContextError, Diagnostics, Module, Source, Sources, Unit, Vm,
};

pub struct Engine {
    model_state: Rc<RefCell<ModelState>>,
    side_effects: Rc<RefCell<Vec<Effect>>>,
    context: Context,
    runtime: Arc<RuntimeContext>,
}

impl Engine {
    pub fn new(
        model_state: Rc<RefCell<ModelState>>,
        side_effects: Rc<RefCell<Vec<Effect>>>,
    ) -> Result<Self> {
        let mut context = Context::with_default_modules()?;
        context.install(item::module()?)?;
        let runtime = Arc::new(context.runtime()?);

        Ok(Self {
            model_state,
            side_effects,
            context,
            runtime,
        })
    }

    pub fn compile_items(&self, all_items: &ItemAssets) -> Result<Vec<ItemKind>> {
        let mut items = Vec::with_capacity(all_items.assets.len());
        for item in all_items.assets.values() {
            let mut diagnostics = Diagnostics::new();

            let mut sources = Sources::new();
            let script = item.script.as_deref().unwrap_or("");
            sources.insert(Source::new(&item.config.name, script)?)?;

            let unit = rune::prepare(&mut sources)
                .with_context(&self.context)
                .with_diagnostics(&mut diagnostics)
                .build();

            let script = if diagnostics.is_empty() {
                unit.expect("Something failed but no diagnostics are available")
            } else {
                let mut writer = StandardStream::stdout(ColorChoice::Always);
                writeln!(writer)?;
                writeln!(
                    writer,
                    "Script compilation failed for item {}:",
                    item.config.name
                )?;
                diagnostics.emit(&mut writer, &sources)?;
                writeln!(writer)?;
                Unit::default()
            };

            items.push(ItemKind {
                config: item.config.clone(),
                script: Arc::new(script),
            });
        }

        Ok(items)
    }

    pub fn init_item(&self, kind: ItemKind) -> Result<InventoryItem> {
        // TODO: check if state works
        let vm = Vm::new(Arc::clone(&self.runtime), Arc::clone(&kind.script));
        if let Ok(init) = vm.lookup_function(["init"]) {
            init.call(()).into_result()?;
        }
        let state = ScriptState {
            stack: rune::alloc::prelude::TryClone::try_clone(vm.stack())
                .expect("failed to clone script stack"),
        };

        // let options = CallFnOptions::new().eval_ast(false).rewind_scope(false); // Retain variables
        // call!(&self.inner, options, &mut state, &kind.script, "init", ());

        let base_stats = kind.config.base_stats.clone();

        Ok(InventoryItem {
            model_state: Rc::clone(&self.model_state),
            side_effects: Rc::clone(&self.side_effects),
            on_board: None,
            kind,
            state,
            turns_on_board: 0,
            base_stats,
            perm_stats: ItemStats::default(),
            temp_stats: ItemStats::default(),
        })
    }

    /// Call the item's trigger handler (if it is defined) and return the new item state.
    /// Side effects produced by the script are put into [ModelState].
    ///
    /// *NOTE*: it borrows [ModelState] and mutates `side_effects`.
    pub fn item_trigger(
        &self,
        item: &InventoryItem,
        board_item: &BoardItem,
        method: &str,
    ) -> Result<ScriptState> {
        log::debug!(
            "Item trigger {:?} for {:?}, item: {:?}",
            method,
            item.kind,
            board_item,
        );

        let script_item = item::Item::from_real(item, board_item);

        let vm = Vm::with_stack(
            Arc::clone(&self.runtime),
            Arc::clone(&item.kind.script),
            item.state.clone().stack,
        );
        if let Ok(fun) = vm.lookup_function([method]) {
            fun.call((script_item,)).into_result()?;
        }

        Ok(ScriptState {
            stack: rune::alloc::prelude::TryClone::try_clone(vm.stack())
                .expect("failed to clone script stack"),
        })
    }
}

pub mod item {
    use super::*;

    pub fn module() -> Result<Module, ContextError> {
        let mut module = Module::new();

        module.ty::<Item>()?;
        module.function_meta(Item::damage)?;
        module.function_meta(Item::damage_all_nearby)?;
        module.function_meta(Item::bonus)?;
        module.function_meta(Item::bonus_from)?;
        module.function_meta(Item::bonus_from_nearby)?;
        module.function_meta(Item::bonus_from_connected)?;
        module.function_meta(Item::bonus_to_nearby)?;
        module.function_meta(Item::bonus_to_all)?;
        module.function_meta(Item::open_tiles)?;
        module.function_meta(Item::destroy)?;
        module.function_meta(Item::find_nearby)?;
        module.function_meta(Item::find_random)?;
        module.function_meta(Item::duplicate)?;
        module.function_meta(Item::rng_float)?;
        module.function_meta(Item::gain_moves)?;
        module.function_meta(Item::portal)?;
        module.function_meta(Item::swap_with)?;
        module.function_meta(Item::grid_bounds)?;
        module.function_meta(Item::turn_into)?;
        module.function_meta(Item::emit_light_around)?;
        module.function_meta(Item::is_observed)?;
        module.function_meta(Item::random_kind)?;
        module.function_meta(Item::use_item)?;

        module.ty::<Position>()?;
        module.ty::<Bounds>()?;
        module.ty::<Stats>()?;
        module.ty::<Filter>()?;
        module.ty::<Target>()?;
        module.ty::<Category>()?;

        Ok(module)
    }

    #[derive(Clone, rune::Any)]
    pub struct Item {
        board: BoardItem,
        inventory: InventoryItem,

        #[rune(get)]
        position: Position,
        #[rune(get)]
        turns_on_board: usize,
        #[rune(get)]
        stats: Stats,
    }

    #[derive(Debug, Clone, rune::Any)]
    #[rune(constructor)]
    pub struct Stats {
        #[rune(get)]
        damage: Hp,
    }

    #[derive(Debug, Clone, rune::Any)]
    pub enum Filter {
        #[rune(constructor)]
        This,
        #[rune(constructor)]
        Category(#[rune(get)] Category),
        #[rune(constructor)]
        Named(#[rune(get)] String),
    }

    #[derive(Debug, Clone, Copy, rune::Any)]
    #[rune(constructor)]
    pub struct Position {
        #[rune(get)]
        pub x: Coord,
        #[rune(get)]
        pub y: Coord,
    }

    impl From<vec2<Coord>> for Position {
        fn from(vec2(x, y): vec2<Coord>) -> Self {
            Self { x, y }
        }
    }

    impl From<Position> for vec2<Coord> {
        fn from(value: Position) -> Self {
            vec2(value.x, value.y)
        }
    }

    #[derive(Debug, Clone, Copy, rune::Any)]
    #[rune(constructor)]
    pub struct Bounds {
        #[rune(get)]
        pub low: Position,
        #[rune(get)]
        pub high: Position,
    }

    impl From<Aabb2<Coord>> for Bounds {
        fn from(bounds: Aabb2<Coord>) -> Self {
            Self {
                low: bounds.min.into(),
                high: bounds.max.into(),
            }
        }
    }

    impl Item {
        pub fn from_real(item: &InventoryItem, board_item: &BoardItem) -> Self {
            Self {
                board: board_item.clone(),
                inventory: item.clone(),

                position: board_item.position.into(),
                turns_on_board: item.turns_on_board,
                stats: item.current_stats().into(),
            }
        }

        pub fn as_script(&self) -> ScriptItem<'_> {
            ScriptItem {
                model: self.inventory.model_state.borrow(),
                effects: ScriptEffects(self.inventory.side_effects.borrow_mut()),
                board_item: &self.board,
                item: &self.inventory,
            }
        }

        #[rune::function]
        fn damage(&self, target: Target, damage: ScriptFunction) {
            self.as_script().damage(target, damage)
        }

        #[rune::function]
        fn damage_all_nearby(&self, range: Option<Coord>, damage: ScriptFunction) {
            self.as_script().damage_all_nearby(range, damage)
        }

        #[rune::function]
        fn bonus(&self, stats: Stats, permanent: bool) {
            self.as_script().bonus(stats.into(), permanent)
        }

        #[rune::function]
        fn bonus_from(&self, target: &Item, stats: Stats, permanent: bool) {
            self.as_script()
                .bonus_from(target.board.position, stats.into(), permanent)
        }

        #[rune::function]
        fn bonus_from_nearby(&self, range: Coord, filter: Filter, stats: Stats, permanent: bool) {
            self.as_script().bonus_from_nearby(
                range,
                filter.into_filter(&self.inventory.kind.config.name),
                stats.into(),
                permanent,
            )
        }

        #[rune::function]
        fn bonus_from_connected(&self, filter: Filter, stats: Stats, permanent: bool) {
            self.as_script().bonus_from_connected(
                filter.into_filter(&self.inventory.kind.config.name),
                stats.into(),
                permanent,
            )
        }

        #[rune::function]
        fn bonus_to_nearby(&self, range: Coord, filter: Filter, stats: Stats, permanent: bool) {
            self.as_script().bonus_to_nearby(
                range,
                filter.into_filter(&self.inventory.kind.config.name),
                stats.into(),
                permanent,
            )
        }

        #[rune::function]
        fn bonus_to_all(&self, filter: Filter, stats: Stats, permanent: bool) {
            self.as_script().bonus_to_all(
                filter.into_filter(&self.inventory.kind.config.name),
                stats.into(),
                permanent,
            )
        }

        #[rune::function]
        fn open_tiles(&self, tiles: usize) {
            self.as_script().open_tiles(tiles)
        }

        #[rune::function]
        fn destroy(&self) {
            self.as_script().destroy()
        }

        #[rune::function]
        fn find_nearby(&self, range: Coord, filter: Filter) -> Option<Item> {
            let id = self
                .as_script()
                .find_nearby(range, filter.into_filter(&self.inventory.kind.config.name))?;
            self.get_item_board(id)
        }

        #[rune::function]
        fn find_random(&self, filter: Filter) -> Option<Item> {
            let id = self
                .as_script()
                .find_random(filter.into_filter(&self.inventory.kind.config.name))?;
            self.get_item_board(id)
        }

        fn get_item_board(&self, id: Id) -> Option<Item> {
            let script = self.as_script();
            let (_, inv) = script
                .model
                .player
                .items
                .iter()
                .find(|(_, item)| item.on_board == Some(id))?;
            let board = script.model.items.get(id)?;
            Some(Item::from_real(inv, board))
        }

        #[rune::function]
        fn duplicate(&self) {
            self.as_script().duplicate()
        }

        #[rune::function]
        fn rng_float(&self) -> f32 {
            thread_rng().gen()
        }

        #[rune::function]
        fn gain_moves(&self, moves: usize) {
            self.as_script().gain_moves(moves)
        }

        #[rune::function]
        fn portal(&self) {
            self.as_script().portal()
        }

        #[rune::function]
        fn swap_with(&self, target: &Item) {
            self.as_script()
                .swap_with(target.inventory.on_board.unwrap());
        }

        #[rune::function]
        fn grid_bounds(&self) -> Bounds {
            self.as_script().grid_bounds().into()
        }

        #[rune::function]
        fn turn_into(&self, target: &str) {
            self.as_script().turn_into(target)
        }

        #[rune::function]
        fn emit_light_around(&self, position: Position, radius: Coord, duration: usize) {
            self.as_script()
                .emit_light_around(position.into(), radius, duration)
        }

        #[rune::function]
        fn is_observed(&self) -> bool {
            self.as_script().is_observed()
        }

        /// Excluding kind of the item.
        #[rune::function]
        fn random_kind(&self, category: Category) -> Option<String> {
            let mut rng = thread_rng();
            self.as_script()
                .model
                .all_items
                .iter()
                .filter(|item| {
                    item.config.name != self.inventory.kind.config.name
                        && item.config.categories.contains(&category)
                })
                .choose(&mut rng)
                .map(|kind| kind.config.name.to_string())
        }

        /// Excluding kind of the item.
        #[rune::function]
        fn use_item(&self, target: Item) {
            self.as_script()
                .use_item(target.inventory.on_board.unwrap())
        }
    }

    impl From<ItemStats> for Stats {
        fn from(value: ItemStats) -> Self {
            Self {
                damage: value.damage.unwrap_or_default(),
            }
        }
    }

    impl From<Stats> for ItemStats {
        fn from(value: Stats) -> Self {
            Self {
                damage: Some(value.damage),
            }
        }
    }

    impl Filter {
        fn into_filter(self, this: &Rc<str>) -> ItemFilter {
            match self {
                Filter::This => ItemFilter::Named(Rc::clone(this)),
                Filter::Category(cat) => ItemFilter::Category(cat),
                Filter::Named(name) => ItemFilter::Named(name.into()),
            }
        }
    }
}
