use super::*;

#[derive(Debug)]
pub struct Player {
    pub moves_left: usize,
    pub turns_left: usize,
    pub hearts: usize,
    /// Extra items player can choose at the end of the turn.
    pub extra_items: usize,
    pub items: Arena<InventoryItem>,
}

#[derive(Debug)]
pub enum PlayerInput {
    Dir(vec2<Coord>),
    Tile(vec2<Coord>),
    SelectItem(usize),
    Reroll,
    Skip,
    Retry,
}

impl Player {
    pub fn new() -> Self {
        Self {
            moves_left: 0,
            turns_left: 0,
            hearts: 0,
            extra_items: 0,
            items: [ItemKind::Sword, ItemKind::Map]
                .into_iter()
                .map(ItemKind::instantiate)
                .collect(),
        }
    }
}
