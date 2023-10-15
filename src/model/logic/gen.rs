use super::*;

impl Model {
    pub fn next_level(&mut self) {
        {
            let mut state = self.state.borrow_mut();
            if self.level > 0 {
                self.score += self.config.score_per_level;
                self.score += self.config.score_per_turn_left * state.player.turns_left as Score;
            }

            self.level += 1;
            log::info!("Next level {}", self.level);

            let turns = 4 + 2_usize.pow(self.level as u32 / 4);
            let turns = turns.min(10);
            state.player.turns_left = turns;
            // self.player.hearts = 3;
        }

        // self.items.clear();
        // if self.entities.len() == 1 {
        //     for (_, entity) in &mut self.entities {
        //         entity.position = vec2::ZERO;
        //     }
        // }

        self.spawn_enemies();
        self.spawn_items(); // First spawn has to be done manually
        self.phase = Phase::LevelStarting {
            timer: Lifetime::new_max(r32(0.5)),
        };
    }

    pub(super) fn shift_everything(&mut self) {
        let available: HashSet<_> = self.grid.tiles.sub(&self.visible_tiles);
        if available.is_empty() {
            // Cannot shift
            return;
        }

        enum Thing {
            Entity(Id),
            Item(Id),
        }

        let mut state = self.state.borrow_mut();
        let items = state
            .items
            .iter()
            .map(|(i, item)| (Thing::Item(i), item.position));
        let entities = state
            .entities
            .iter()
            .map(|(i, e)| (Thing::Entity(i), e.position));
        let things = items.chain(entities);

        let mut rng = thread_rng();
        let moves: Vec<(Thing, vec2<Coord>)> = things
            .filter(|(_, pos)| !self.visible_tiles.contains(pos))
            .map(|(i, _)| (i, *available.iter().choose(&mut rng).unwrap()))
            .collect();

        for (thing, target) in moves {
            let from = match thing {
                Thing::Entity(i) => state.entities[i].position,
                Thing::Item(i) => state.items[i].position,
            };

            // Swap
            for (_, item) in &mut state.items {
                if item.position == target {
                    item.position = from;
                }
            }
            for (_, entity) in &mut state.entities {
                if entity.position == target {
                    entity.position = from;
                }
            }

            match thing {
                Thing::Entity(i) => state.entities[i].position = target,
                Thing::Item(i) => state.items[i].position = target,
            }
        }
    }

    fn spawn_enemies(&mut self) {
        let mut available = self.calculate_empty_space().sub(&self.visible_tiles);
        if available.is_empty() {
            return;
        }

        let options = [EntityKind::Dummy];
        let mut rng = thread_rng();

        let level = self.level.saturating_sub(1);
        let enemies = level % 4 + level / 4 + 1;
        let enemies = enemies.min(5);

        let health = 5 * 2_usize.pow(self.level as u32 / 4);
        let health = health as i64;

        for _ in 0..enemies {
            let kind = options.choose(&mut rng).unwrap();
            let position = *available.iter().choose(&mut rng).unwrap();

            self.state.borrow_mut().entities.insert(Entity {
                position,
                fraction: Fraction::Enemy,
                health: Health::new_max(health),
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
        //     for kind in [ItemKind::CharmingStaff] {
        //         let position = *available.iter().choose(&mut rng).unwrap();
        //         let item_id = self.player.items.insert(kind.instantiate());
        //         let item = &mut self.player.items[item_id];
        //         let on_board = self.items.insert(BoardItem {
        //             position,
        //             item_id,
        //             turns_alive: 0,
        //             used: false,
        //         });
        //         item.on_board = Some(on_board);

        //         available.remove(&position);
        //         if available.is_empty() {
        //             break;
        //         }
        //     }
        //     return;
        // }

        // What is this trick KEKW
        let mut state = self.state.borrow_mut();
        let state = &mut *state;

        for (item_id, item) in &mut state.player.items {
            if let Some(id) = item.on_board {
                if state.items.contains(id) {
                    // Already on the board
                    continue;
                } else {
                    // It's been a lie all along
                    item.on_board = None;
                }
            }

            let position = *available.iter().choose(&mut rng).unwrap();
            let on_board = state.items.insert(BoardItem {
                position,
                item_id,
                used: false,
            });
            item.on_board = Some(on_board);

            available.remove(&position);
            if available.is_empty() {
                break;
            }
        }
    }
}
