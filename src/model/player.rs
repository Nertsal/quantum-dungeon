use super::*;

#[derive(Debug)]
pub struct Player {
    pub moves_left: usize,
    pub items: Vec<ItemKind>,
}

#[derive(Debug)]
pub enum PlayerInput {
    Dir(vec2<Coord>),
    Tile(vec2<Coord>),
}

impl Player {
    pub fn new() -> Self {
        Self {
            moves_left: 5,
            items: vec![ItemKind::Sword, ItemKind::Map],
        }
    }
}
