use super::*;

#[derive(Debug)]
pub struct Player {
    pub moves_left: usize,
    pub turns_left: usize,
    pub hearts: usize,
    pub items: Vec<ItemKind>,
}

#[derive(Debug)]
pub enum PlayerInput {
    Dir(vec2<Coord>),
    Tile(vec2<Coord>),
    SelectItem(usize),
}

impl Player {
    pub fn new() -> Self {
        Self {
            moves_left: 5,
            turns_left: 10,
            hearts: 3,
            items: vec![ItemKind::Sword, ItemKind::Map],
        }
    }
}
