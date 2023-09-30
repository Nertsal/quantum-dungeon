use super::*;

#[derive(Debug)]
pub struct Player {
    pub moves_left: usize,
}

#[derive(Debug)]
pub struct PlayerInput {
    pub move_dir: vec2<Coord>,
}

impl Player {
    pub fn new() -> Self {
        Self { moves_left: 5 }
    }
}
