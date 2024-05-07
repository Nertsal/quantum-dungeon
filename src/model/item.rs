use super::*;

#[derive(Debug, Clone)]
pub struct BoardItem {
    pub position: vec2<Coord>,
    pub item_id: Id,
    /// Whether the item was used this turn.
    pub used: bool,
}

#[derive(Clone)]
pub struct InventoryItem {
    pub model_state: Rc<RefCell<ModelState>>,
    pub side_effects: Rc<RefCell<Vec<Effect>>>,
    /// The id of the board item, if it is present on the board.
    pub on_board: Option<Id>,
    pub kind: ItemKind,
    /// The number of turns this item has been present on the board so far.
    pub turns_on_board: usize,
    /// State of the item script (local variables).
    pub state: ScriptState,
    /// Base stats that might be used to reset all modifications.
    pub base_stats: ItemStats,
    /// Permanent stats that persist through turns.
    pub perm_stats: ItemStats,
    /// Resolution stats that reset every turn, act as a modifier on the perm_stats.
    /// Call `current_stats()` to get relevant stats for the time.
    pub temp_stats: ItemStats,
}

pub struct ScriptEffects<'a>(pub RefMut<'a, Vec<Effect>>);

/// A representation on the item used temporarily for scripts.
pub struct ScriptItem<'a> {
    pub model: Ref<'a, ModelState>,
    pub effects: ScriptEffects<'a>,
    pub board_item: &'a BoardItem,
    pub item: &'a InventoryItem,
}

#[derive(Debug, Default)]
pub struct ScriptState {
    pub stack: rune::runtime::Stack,
}

impl Clone for ScriptState {
    fn clone(&self) -> Self {
        Self {
            stack: rune::alloc::prelude::TryClone::try_clone(&self.stack)
                .expect("failed to clone item script state"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ItemStats {
    pub damage: Option<i64>,
}

/// A filter for item kind or category of items.
#[derive(Debug, Clone, rune::Any)]
pub enum ItemFilter {
    Category(Category),
    Named(Rc<str>),
}

/// A filter for item kind or category of items.
#[derive(Debug, Clone, rune::Any)]
pub enum Target {
    #[rune(constructor)]
    Nearest,
    #[rune(constructor)]
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, rune::Any)]
pub enum Category {
    #[rune(constructor)]
    Spooky,
    #[rune(constructor)]
    Tech,
    #[rune(constructor)]
    Weapon,
    #[rune(constructor)]
    Treasure,
    #[rune(constructor)]
    Magic,
}

#[derive(Clone)]
pub struct ItemKind {
    pub config: ItemConfig,
    pub script: Arc<Script>,
}

impl InventoryItem {
    pub fn current_stats(&self) -> ItemStats {
        self.base_stats
            .combine(&self.temp_stats.combine(&self.perm_stats))
    }
}

impl ItemFilter {
    pub fn check(&self, item: &ItemKind) -> bool {
        match self {
            ItemFilter::Category(category) => item.config.categories.contains(category),
            ItemFilter::Named(name) => item.config.name == *name,
        }
    }
}

impl ItemStats {
    pub fn combine(&self, other: &Self) -> Self {
        fn combine<T: Num + Ord>(value: Option<T>, other: Option<T>) -> Option<T> {
            match value {
                Some(a) => match other {
                    Some(b) => Some((a + b).max(T::ZERO)),
                    None => Some(a),
                },
                None => other,
            }
        }

        Self {
            damage: combine(self.damage, other.damage),
        }
    }
}

impl std::fmt::Debug for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ItemKind")
            .field("config", &self.config)
            .field("script", &"<hidden>")
            .finish()
    }
}
