use super::*;

pub struct Grid {
    pub tiles: HashSet<vec2<Coord>>,
    pub fractured: HashSet<vec2<Coord>>,
    /// Positions that are lit up, and the duration (in turns).
    pub lights: HashMap<vec2<Coord>, usize>,
}

impl Grid {
    pub fn new(size: Coord) -> Self {
        let offset = -size / 2;
        Self {
            tiles: (0..size)
                .flat_map(|x| (0..size).map(move |y| vec2(x, y) + vec2::splat(offset)))
                .collect(),
            fractured: HashSet::new(),
            lights: HashMap::new(),
        }
    }

    pub fn is_max(&self) -> bool {
        self.outside_tiles().is_empty()
    }

    pub fn bounds(&self) -> Aabb2<Coord> {
        let mut bounds = Aabb2::<Coord>::ZERO;
        for &pos in &self.tiles {
            bounds = Aabb2 {
                min: vec2(bounds.min.x.min(pos.x), bounds.min.y.min(pos.y)),
                max: vec2(bounds.max.x.max(pos.x), bounds.max.y.max(pos.y)),
            };
        }
        bounds
    }

    pub fn check_pos(&self, pos: vec2<Coord>) -> bool {
        self.tiles.contains(&pos)
    }

    /// Whether the position is inside the possible extension limits.
    pub fn check_in_limits(&self, pos: vec2<Coord>) -> bool {
        // Limit to 5x5
        pos.x.abs().max(pos.y.abs()) < 2
    }

    /// Whether the position is empty, but there is a tile right next to it.
    pub fn check_pos_near(&self, pos: vec2<Coord>) -> bool {
        if self.check_pos(pos) {
            return false;
        }
        if !self.check_in_limits(pos) {
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

    /// Return the outside empty tiles that can be turned into proper tiles.
    pub fn outside_tiles(&self) -> HashSet<vec2<Coord>> {
        let mut outside = HashSet::new();
        for &pos in &self.tiles {
            for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
                let pos = pos + vec2(dx, dy);
                if !self.check_pos(pos) && self.check_in_limits(pos) {
                    outside.insert(pos);
                }
            }
        }
        outside
    }

    pub fn light_up(&mut self, position: vec2<Coord>, radius: Coord, duration: usize) {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let pos = position + vec2(dx, dy);
                if self.check_pos(pos) {
                    self.lights.insert(pos, duration);
                }
            }
        }
    }
}
