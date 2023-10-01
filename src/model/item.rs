use super::*;

#[derive(Debug, Clone)]
pub struct BoardItem {
    pub position: vec2<Coord>,
    pub item_id: Id,
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    /// The id of the board item, if it is present on the board.
    pub on_board: Option<Id>,
    pub kind: ItemKind,
    /// Permanent stats that persist through turns.
    pub perm_stats: ItemStats,
    /// Resolution stats that reset every turn.
    pub temp_stats: ItemStats,
}

#[derive(Debug, Clone, Default)]
pub struct ItemStats {
    pub damage: Option<i64>,
}

/// A reference to an item kind or a category of items.
/// Used in synergies.
#[derive(Debug, Clone, Copy)]
pub enum ItemRef {
    Category(ItemCategory),
    Specific(ItemKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCategory {
    Spooky,
    Tech,
    Weapon,
    Treasure,
    Magic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Sword,
    Forge,
    Boots,
    Map,
    Camera,
}

impl ItemRef {
    pub fn check(&self, item: ItemKind) -> bool {
        match self {
            ItemRef::Category(category) => item.categories().contains(category),
            ItemRef::Specific(_) => todo!(),
        }
    }
}

impl ItemKind {
    pub fn categories(&self) -> Vec<ItemCategory> {
        match self {
            ItemKind::Boots => vec![ItemCategory::Tech],
            ItemKind::Forge => vec![ItemCategory::Magic],
            ItemKind::Sword => vec![ItemCategory::Weapon],
            ItemKind::Map => vec![ItemCategory::Treasure],
            ItemKind::Camera => vec![ItemCategory::Tech],
        }
    }

    pub fn instantiate(self) -> InventoryItem {
        let damage = match self {
            ItemKind::Boots => None,
            ItemKind::Forge => None,
            ItemKind::Sword => Some(2),
            ItemKind::Map => None,
            ItemKind::Camera => None,
        };
        InventoryItem {
            on_board: None,
            kind: self,
            perm_stats: ItemStats { damage },
            temp_stats: default(),
        }
    }
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

impl Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ItemKind::Sword => "Sword",
            ItemKind::Forge => "Ancient forge",
            ItemKind::Boots => "Ultra speed boots",
            ItemKind::Map => "Grand map",
            ItemKind::Camera => "Camera",
        };
        write!(f, "{}", name)
    }
}
