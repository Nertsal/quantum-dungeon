use super::*;

#[derive(Debug, Clone)]
pub struct Item {
    pub position: vec2<Coord>,
    pub use_time: usize,
    pub kind: ItemKind,
    pub bonus: i64,
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
        let use_time = match self {
            ItemKind::Boots => 1,
            ItemKind::Forge => 2,
            ItemKind::Sword => 1,
            ItemKind::Map => 3,
        };
        Item {
            position,
            use_time,
            kind: self,
            bonus: 0,
        }
    }
}
