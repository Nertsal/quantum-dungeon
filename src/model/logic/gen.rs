use super::*;

impl Model {
    pub fn night_phase(&mut self) {
        self.shift_items();
        self.spawn_items();

        if self.items.is_empty() {
            self.entities.push(Entity {
                position: vec2(4, 5),
                fraction: Fraction::Enemy,
                health: Health::new_max(5),
                look_dir: vec2(0, -1),
                kind: EntityKind::Dummy,
            });

            self.items.push(Item {
                position: vec2(3, 5),
                kind: ItemKind::Sword,
            });
            self.items.push(Item {
                position: vec2(3, 6),
                kind: ItemKind::Sword,
            });
            self.items.push(Item {
                position: vec2(2, 5),
                kind: ItemKind::Sword,
            });
        }
    }

    fn shift_items(&mut self) {
        let available: HashSet<_> = (0..self.grid.size.x)
            .flat_map(|x| (0..self.grid.size.y).map(move |y| vec2(x, y)))
            .filter(|pos| !self.visible_tiles.contains(pos))
            .collect();
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

    fn spawn_items(&mut self) {}
}
