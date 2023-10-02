use super::*;

impl Model {
    pub fn next_level(&mut self) {
        self.level += 1;
        self.player.turns_left = 10;
        self.player.hearts = 3;
        log::info!("Next level {}", self.level);

        // TODO: animation
        self.items.clear();
        if self.entities.len() == 1 {
            for entity in &mut self.entities {
                entity.position = vec2::ZERO;
            }
        }
        self.spawn_enemies();
        self.spawn_items(); // First spawn has to be done manually
        self.night_phase(true);
    }

    pub(super) fn shift_items(&mut self) {
        let mut available: HashSet<_> = self.grid.tiles.sub(&self.visible_tiles);
        for entity in &self.entities {
            available.remove(&entity.position);
        }
        if available.is_empty() {
            // Cannot shift items
            return;
        }

        let mut rng = thread_rng();
        let moves: Vec<_> = self
            .items
            .iter()
            .filter(|(_, item)| !self.visible_tiles.contains(&item.position))
            .map(|(i, _)| (i, *available.iter().choose(&mut rng).unwrap()))
            .collect();

        for (item_id, target) in moves {
            let item = &self.items[item_id];
            let from = item.position;

            // Swap
            for (_, item) in &mut self.items {
                if item.position == target {
                    item.position = from;
                }
            }

            let item = &mut self.items[item_id];
            item.position = target;
        }
    }

    fn spawn_enemies(&mut self) {
        let mut available = self.calculate_empty_space().sub(&self.visible_tiles);
        if available.is_empty() {
            return;
        }

        let options = [EntityKind::Dummy];
        let mut rng = thread_rng();

        let enemies = self.level.min(5);
        for _ in 0..enemies {
            let kind = options.choose(&mut rng).unwrap();
            let position = *available.iter().choose(&mut rng).unwrap();

            self.entities.push(Entity {
                position,
                fraction: Fraction::Enemy,
                health: Health::new_max(5),
                look_dir: vec2(0, -1),
                kind: kind.clone(),
            });

            available.remove(&position);
            if available.is_empty() {
                break;
            }
        }
    }

    pub(super) fn spawn_items(&mut self) {
        let mut available = self.calculate_empty_space().sub(&self.visible_tiles);
        if available.is_empty() {
            return;
        }

        let mut rng = thread_rng();

        // For testing
        // if self.items.is_empty() {
        //     for kind in [ItemKind::Ghost, ItemKind::RadiationCore, ItemKind::Sword] {
        //         let position = *available.iter().choose(&mut rng).unwrap();
        //         let item_id = self.player.items.insert(kind.instantiate());
        //         let item = &mut self.player.items[item_id];
        //         let on_board = self.items.insert(BoardItem { position, item_id });
        //         item.on_board = Some(on_board);

        //         available.remove(&position);
        //         if available.is_empty() {
        //             break;
        //         }
        //     }
        //     return;
        // }

        for (item_id, item) in &mut self.player.items {
            if let Some(id) = item.on_board {
                if self.items.contains(id) {
                    // Already on the board
                    continue;
                } else {
                    // It's been a lie all along
                    item.on_board = None;
                }
            }

            let position = *available.iter().choose(&mut rng).unwrap();
            let on_board = self.items.insert(BoardItem { position, item_id });
            item.on_board = Some(on_board);

            available.remove(&position);
            if available.is_empty() {
                break;
            }
        }
    }
}
