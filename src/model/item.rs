use super::*;

#[derive(Debug, Clone)]
pub struct Item {
    pub position: vec2<Coord>,
    pub use_time: usize,
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
    Weapon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Sword,
    Forge,
    Boots,
    Map,
}

impl ItemRef {
    pub fn check(&self, item: ItemKind) -> bool {
        match self {
            ItemRef::Category(category) => item.category() == Some(*category),
            ItemRef::Specific(_) => todo!(),
        }
    }
}

impl ItemKind {
    pub fn category(&self) -> Option<ItemCategory> {
        match self {
            ItemKind::Boots => None,
            ItemKind::Forge => None,
            ItemKind::Sword => Some(ItemCategory::Weapon),
            ItemKind::Map => None,
        }
    }

    pub fn instantiate(self, position: vec2<Coord>) -> Item {
        let (use_time, damage) = match self {
            ItemKind::Boots => (1, None),
            ItemKind::Forge => (2, None),
            ItemKind::Sword => (1, Some(2)),
            ItemKind::Map => (3, None),
        };
        Item {
            position,
            use_time,
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
