use super::*;

#[derive(Debug)]
pub struct Player {
    pub position: vec2<Coord>,
}

#[derive(Debug)]
pub struct PlayerInput {
    pub move_dir: vec2<Coord>,
}

impl Player {
    pub fn new(position: vec2<Coord>) -> Self {
        Self { position }
    }
}
