use super::*;

#[derive(Debug, Clone)]
pub struct Item {
    pub position: vec2<Coord>,
    pub kind: ItemKind,
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    // Boots,
    // Forge,
    // Sword,
}
