use super::*;

pub struct Grid {
    pub tiles: HashSet<vec2<Coord>>,
}

impl Grid {
    pub fn new(size: Coord) -> Self {
        let offset = -size / 2;
        Self {
            tiles: (0..size)
                .flat_map(|x| (0..size).map(move |y| vec2(x, y) + vec2::splat(offset)))
                .collect(),
        }
    }

    pub fn check_pos(&self, pos: vec2<Coord>) -> bool {
        self.tiles.contains(&pos)
    }

    pub fn expand(&mut self, pos: vec2<Coord>) {
        self.tiles.insert(pos);
    }
}
