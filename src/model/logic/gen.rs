use super::*;

impl Model {
    pub fn night_phase(&mut self) {
        self.shift_items();
        self.spawn_enemies();
        self.spawn_items();
    }

    fn shift_items(&mut self) {
        let mut available: HashSet<_> = (0..self.grid.size.x)
            .flat_map(|x| (0..self.grid.size.y).map(move |y| vec2(x, y)))
            .filter(|pos| !self.visible_tiles.contains(pos))
            .collect();
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
            .enumerate()
            .filter(|(_, item)| !self.visible_tiles.contains(&item.position))
            .map(|(i, _)| (i, *available.iter().choose(&mut rng).unwrap()))
            .collect();

        for (item_id, target) in moves {
            let item = &self.items[item_id];
            let from = item.position;

            // Swap
            for item in &mut self.items {
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
        for _ in 0..1 {
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

    fn spawn_items(&mut self) {
        let mut available = self.calculate_empty_space().sub(&self.visible_tiles);
        if available.is_empty() {
            return;
        }

        let mut rng = thread_rng();
        for kind in &self.player.items {
            let position = *available.iter().choose(&mut rng).unwrap();
            self.items.push(Item {
                position,
                kind: kind.clone(),
            });

            available.remove(&position);
            if available.is_empty() {
                break;
            }
        }
    }
}
