use super::*;

pub struct Player {
    pub moves_left: usize,
    pub turns_left: usize,
    pub hearts: usize,
    /// Extra items player can choose at the end of the turn.
    pub extra_items: usize,
    /// Number of refreshes available in the select menu.
    pub refreshes: usize,
    pub items: Arena<InventoryItem>,
}

#[derive(Debug)]
pub enum PlayerInput {
    Dir(vec2<Coord>),
    Tile(vec2<Coord>),
    Vision { pos: vec2<Coord>, commit: bool },
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
            hearts: 3,
            extra_items: 0,
            refreshes: 0,
            items: Arena::new(),
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
