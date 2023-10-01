use super::*;

impl Model {
    pub fn night_phase(&mut self) {
        self.phase = Phase::Night;
        self.shift_items();
        self.spawn_enemies();
        self.spawn_items();
        self.phase = Phase::Resolution;
        // TODO
        for item in &mut self.items {
            item.temp_stats = item.perm_stats.clone();
        }
        self.phase = Phase::Player;
        self.player.moves_left = 5;
    }

    fn shift_items(&mut self) {
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
            self.items.push(kind.instantiate(position));

            available.remove(&position);
            if available.is_empty() {
                break;
            }
        }
    }
}
