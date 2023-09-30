use super::*;

#[derive(Debug)]
pub struct Player {}

#[derive(Debug)]
pub struct PlayerInput {
    pub move_dir: vec2<Coord>,
}

impl Player {
    pub fn new() -> Self {
        Self {}
    }
}
