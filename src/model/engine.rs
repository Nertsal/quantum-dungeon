use super::*;

use anyhow::Result;
use rune::{
    runtime::RuntimeContext,
    termcolor::{ColorChoice, StandardStream},
    Context, ContextError, Diagnostics, Module, Source, Sources, Unit, Vm,
};

pub struct Engine {
    context: Context,
    runtime: Arc<RuntimeContext>,
}

// macro_rules! call {
//     ($engine:expr, $options:expr, $scope:expr, $script:expr, $name:expr, $args: expr $(,)?) => {{
//         let script = $script;
//         let name = $name;
//         if script.iter_functions().any(|fun| fun.name == name) {
//             Some($engine.call_fn_with_options($options, $scope, script, name, $args)?)
//         } else {
//             None
//         }
//     }};
// }

impl Engine {
    pub fn empty() -> Self {
        Self {
            context: Context::default(),
            runtime: Arc::new(RuntimeContext::default()),
        }
    }

    pub fn new(
        state: Rc<RefCell<ModelState>>,
        side_effects: Rc<RefCell<Vec<Effect>>>,
    ) -> Result<Self> {
        let mut context = Context::with_default_modules()?;
        context.install(item::module()?)?;
        let runtime = Arc::new(context.runtime()?);

        // // Types
        // engine.register_type_with_name::<Id>("Id");
        // engine
        //     .register_type_with_name::<InventoryItem>("Item")
        //     .register_get("turns_on_board", |item: &mut InventoryItem| {
        //         item.turns_on_board
        //     })
        //     .register_get("stats", |item: &mut InventoryItem| item.current_stats());
        // engine
        //     .register_type_with_name::<ItemStats>("ItemStats")
        //     .register_get("damage", |stats: &mut ItemStats| {
        //         stats.damage.unwrap_or_default()
        //     });

        // // Effects
        // engine.register_fn("damage_nearest", {
        //     let state = Rc::clone(&state);
        //     let side_effects = Rc::clone(&side_effects);
        //     move |item: InventoryItem, damage: Hp| {
        //         item.damage_nearest(damage, &state.borrow(), &mut side_effects.borrow_mut())
        //     }
        // });

        Ok(Self { context, runtime })
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
        let script_item = item::Item::from(&*item);

        let vm = Vm::with_stack(
            Arc::clone(&self.runtime),
            Arc::clone(&item.kind.script),
            item.state.clone().stack,
        );
        if let Ok(fun) = vm.lookup_function([method]) {
            fun.call((script_item,)).into_result()?;
        }
        item.state = ScriptState {
            stack: rune::alloc::prelude::TryClone::try_clone(vm.stack())
                .expect("failed to clone script stack"),
        };
        Ok(())
    }
}

mod item {
    use super::*;

    pub fn module() -> Result<Module, ContextError> {
        let mut module = Module::new();

        module.ty::<Item>()?;
        module.ty::<Stats>()?;

        Ok(module)
    }

    #[derive(Debug, Clone, rune::Any)]
    pub struct Item {
        #[rune(get)]
        pub turns_on_board: usize,
        #[rune(get)]
        pub stats: Stats,
    }

    #[derive(Debug, Clone, rune::Any)]
    pub struct Stats {
        #[rune(get)]
        pub damage: Hp,
    }

    impl From<&InventoryItem> for Item {
        fn from(value: &InventoryItem) -> Self {
            Self {
                turns_on_board: value.turns_on_board,
                stats: value.current_stats().into(),
            }
        }
    }

    impl From<ItemStats> for Stats {
        fn from(value: ItemStats) -> Self {
            Self {
                damage: value.damage.unwrap_or_default(),
            }
        }
    }
}
