use super::*;

#[derive(Debug, Clone)]
pub struct Item {
    pub position: vec2<Coord>,
    pub use_time: usize,
    pub kind: ItemKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    // Boots,
    // Forge,
    Sword,
    Map,
}

impl ItemKind {
    pub fn instantiate(self, position: vec2<Coord>) -> Item {
        let use_time = match self {
            ItemKind::Sword => 1,
            ItemKind::Map => 3,
        };
        Item {
            position,
            use_time,
            kind: self,
        }
    }
}
