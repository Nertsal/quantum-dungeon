use super::*;

#[derive(Debug, Clone)]
pub struct BoardItem {
    pub position: vec2<Coord>,
    pub item_id: Id,
    /// Whether the item was used this turn.
    pub used: bool,
}

#[derive(Debug, Clone)]
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

/// A representation on the item used temporarily for scripts.
pub struct ScriptItem<'a> {
    pub model: Ref<'a, ModelState>,
    pub effects: RefMut<'a, Vec<Effect>>,
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

/// A reference to an item kind or a category of items.
/// Used in synergies.
#[derive(Debug, Clone)]
pub enum ItemRef {
    Category(ItemCategory),
    Specific { name: Rc<str> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemCategory {
    Spooky,
    Tech,
    Weapon,
    Treasure,
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

impl ItemRef {
    pub fn check(&self, item: &ItemKind) -> bool {
        match self {
            ItemRef::Category(category) => item.config.categories.contains(category),
            ItemRef::Specific { name } => item.config.name == *name,
        }
    }
}

impl ItemKind {
    // pub fn categories(&self) -> Vec<ItemCategory> {
    //     use ItemCategory::*;
    //     match self {
    //         ItemKind::Boots => vec![Tech],
    //         ItemKind::Forge => vec![Magic],
    //         ItemKind::Sword => vec![Weapon],
    //         ItemKind::Map => vec![Treasure],
    //         ItemKind::Camera => vec![Tech],
    //         ItemKind::Ghost => vec![Spooky],
    //         ItemKind::FireScroll => vec![Magic, Weapon],
    //         ItemKind::SoulCrystal => vec![Spooky],
    //         ItemKind::RadiationCore => vec![Tech, Weapon],
    //         ItemKind::GreedyPot => vec![Treasure],
    //         ItemKind::SpiritCoin => vec![Spooky, Treasure],
    //         ItemKind::Chest => vec![Treasure],
    //         ItemKind::MagicTreasureBag => vec![Treasure, Magic],
    //         ItemKind::ElectricRod => vec![Tech, Weapon],
    //         ItemKind::MagicWire => vec![Magic, Tech],
    //         ItemKind::Melter => vec![Tech],
    //         ItemKind::Phantom => vec![Spooky, Weapon],
    //         ItemKind::CursedSkull => vec![Spooky],
    //         ItemKind::KingSkull => vec![Treasure, Weapon],
    //         ItemKind::GoldenLantern => vec![Treasure],
    //         ItemKind::CharmingStaff => vec![Magic, Weapon],
    //         ItemKind::WarpPortal => vec![Magic],
    //         ItemKind::Solitude => vec![Weapon],
    //     }
    // }
}

impl ItemStats {
    pub fn combine(&self, other: &Self) -> Self {
        fn combine<T: Add<T, Output = T>>(value: Option<T>, other: Option<T>) -> Option<T> {
            match value {
                Some(a) => match other {
                    Some(b) => Some(a + b),
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
