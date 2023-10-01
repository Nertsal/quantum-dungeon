use super::*;

pub struct Grid {
    pub tiles: HashSet<vec2<Coord>>,
    pub fractured: HashSet<vec2<Coord>>,
}

impl Grid {
    pub fn new(size: Coord) -> Self {
        let offset = -size / 2;
        Self {
            tiles: (0..size)
                .flat_map(|x| (0..size).map(move |y| vec2(x, y) + vec2::splat(offset)))
                .collect(),
            fractured: HashSet::new(),
        }
    }

    pub fn check_pos(&self, pos: vec2<Coord>) -> bool {
        self.tiles.contains(&pos)
    }

    /// Whether the position is empty, but there is a tile right next to it.
    pub fn check_pos_near(&self, pos: vec2<Coord>) -> bool {
        if self.check_pos(pos) {
            return false;
        }

        for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let pos = pos + vec2(dx, dy);
            if self.check_pos(pos) {
                return true;
            }
        }
        false
    }

    pub fn expand(&mut self, pos: vec2<Coord>) {
        self.tiles.insert(pos);
    }
}
