mod gen;

use super::*;

impl Model {
    pub fn update(&mut self, _delta_time: Time) {}

    pub fn player_move(&mut self, player_input: PlayerInput) {
        if self.player.moves_left == 0 {
            // Cannot move yet (probably)
            return;
        }

        let mut moves = Vec::new();
        for (i, entity) in self.entities.iter_mut().enumerate() {
            if let EntityKind::Player = entity.kind {
                // TODO: if there are multiple players, resolve conflicting movement
                moves.push(i);
            }
        }

        let mut moved = false;
        for i in moves {
            let entity = self.entities.get_mut(i).unwrap();
            let target = self.grid.clamp_pos(entity.position + player_input.move_dir);
            if target != entity.position {
                let fraction = entity.fraction;
                self.move_entity_swap(i, target);
                self.collect_item_at(fraction, target);
                self.update_vision();
                moved = true;
            }
        }

        if moved {
            self.check_deaths();
            self.player.moves_left -= 1;
            if self.player.moves_left == 0 {
                self.select_phase();
            }
        }
    }

    fn select_phase(&mut self) {
        // TODO
        self.turn += 1;
        self.night_phase();
        self.player.moves_left = 5;
    }

    fn calculate_empty_space(&self) -> HashSet<vec2<Coord>> {
        let mut available: HashSet<_> = (0..self.grid.size.x)
            .flat_map(|x| (0..self.grid.size.y).map(move |y| vec2(x, y)))
            .collect();

        for entity in &self.entities {
            available.remove(&entity.position);
        }
        for item in &self.items {
            available.remove(&item.position);
        }

        available
    }

    pub fn update_vision(&mut self) {
        let mut visible = HashSet::new();
        for entity in &self.entities {
            if let EntityKind::Player = entity.kind {
                let mut pos = entity.position;
                visible.insert(pos);
                loop {
                    let target = self.grid.clamp_pos(pos + entity.look_dir);
                    if target == pos {
                        break;
                    }
                    visible.insert(target);
                    pos = target;
                }
            }
        }
        self.visible_tiles = visible;
    }

    fn check_deaths(&mut self) {
        self.entities.retain(|e| e.health.is_above_min());
    }

    /// Move the entity to the target position and swap with the entity occupying the target (if any).
    fn move_entity_swap(&mut self, entity_id: usize, target_pos: vec2<Coord>) {
        let Some(entity) = self.entities.get_mut(entity_id) else {
            log::error!("entity does not exist: {}", entity_id);
            return;
        };

        let from_pos = entity.position;
        let target_pos = self.grid.clamp_pos(target_pos);
        if let Some(target) = self.entities.iter_mut().find(|e| e.position == target_pos) {
            target.position = from_pos;
        }

        let entity = self.entities.get_mut(entity_id).unwrap();
        entity.position = target_pos;
    }

    /// Collect an item at the given position.
    fn collect_item_at(&mut self, fraction: Fraction, position: vec2<Coord>) {
        let mut items = Vec::new();
        for i in (0..self.items.len()).rev() {
            if self.items[i].position == position {
                // TODO: use time
                items.push(self.items.swap_remove(i));
            }
        }

        for item in items {
            self.use_item(fraction, item);
        }
    }

    fn use_item(&mut self, fraction: Fraction, item: Item) {
        match item.kind {
            ItemKind::Sword => {
                let bonus = self.count_items_near(item.position, ItemKind::Sword);
                let damage = 2 + bonus as Hp * 2;
                let range = 1;
                self.deal_damage_around(item.position, fraction, damage, range);
            }
        }
    }

    fn deal_damage_around(
        &mut self,
        position: vec2<Coord>,
        source_fraction: Fraction,
        damage: Hp,
        range: Coord,
    ) {
        for entity in &mut self.entities {
            if source_fraction != entity.fraction && distance(entity.position, position) <= range {
                entity.health.change(-damage);
            }
        }
    }

    fn count_items_near(&self, position: vec2<Coord>, kind: ItemKind) -> usize {
        self.items
            .iter()
            .filter(|item| item.kind == kind && distance(position, item.position) <= 1)
            .count()
    }
}

fn distance(a: vec2<Coord>, b: vec2<Coord>) -> Coord {
    let delta = b - a;
    delta.x.abs() + delta.y.abs()
}
