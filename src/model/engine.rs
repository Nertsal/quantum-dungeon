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
        module.function_meta(Item::bonus)?;
        module.function_meta(Item::bonus_from_nearby)?;
        module.function_meta(Item::bonus_to_nearby)?;
        module.function_meta(Item::open_tiles)?;
        module.function_meta(Item::destroy)?;

        module.ty::<Position>()?;
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
        Named(Rc<str>),
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
        fn bonus(&self, stats: Stats, permanent: bool) {
            self.as_script().bonus(stats.into(), permanent)
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
        fn bonus_to_nearby(&self, range: Coord, filter: Filter, stats: Stats, permanent: bool) {
            self.as_script().bonus_to_nearby(
                range,
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
                Filter::Named(name) => ItemFilter::Named(name),
            }
        }
    }
}
