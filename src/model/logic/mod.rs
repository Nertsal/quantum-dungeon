mod action;
mod animation;
mod gen;
mod item;
mod resolve;

use super::*;

impl Model {
    pub fn update(&mut self, delta_time: Time) {
        self.update_animations(delta_time);
        self.resolve_animations(delta_time);
        if let Phase::LevelFinished { .. } = self.phase {
        } else if self.animations.is_empty() && self.ending_animations.is_empty() {
            self.check_deaths();
            if let Phase::Player = self.phase {
                if self.player.moves_left == 0 {
                    self.vision_phase();
                }
            }
        }
    }

    pub fn get_light_level(&self, position: vec2<Coord>) -> f32 {
        if let Phase::Night {
            fade_time,
            light_time,
        } = self.phase
        {
            if self.visible_tiles.contains(&position) {
                1.0
            } else if fade_time.is_above_min() {
                fade_time.get_ratio().as_f32()
            } else {
                1.0 - light_time.get_ratio().as_f32()
            }
        } else {
            1.0
        }
    }

    pub fn night_phase(&mut self, start_faded: bool) {
        self.phase = Phase::Night {
            fade_time: if start_faded {
                Lifetime::new_zero(r32(1.0))
            } else {
                Lifetime::new_max(r32(1.0))
            },
            light_time: Lifetime::new_max(r32(1.0)),
        };

        self.player.extra_items = self.turn % 2;
        self.grid.fractured.clear();
        for (_, entity) in &self.entities {
            if let EntityKind::Player = entity.kind {
                self.grid.fractured.insert(entity.position);
            }
        }

        // Update light duration
        for duration in self.grid.lights.values_mut() {
            *duration = duration.saturating_sub(1);
        }
        self.grid.lights.retain(|_, duration| *duration > 0);

        self.update_vision();
    }

    pub fn day_phase(&mut self) {
        log::debug!("Day phase");
        self.phase = Phase::Player;
        self.player.moves_left = 3;
    }

    fn player_phase(&mut self) {
        self.phase = Phase::Player;
    }

    fn vision_phase(&mut self) {
        log::debug!("Vision phase");
        self.phase = Phase::Vision;
        for (_, entity) in &mut self.entities {
            entity.look_dir = vec2::ZERO;
        }
        self.update_vision();
    }

    fn select_phase(&mut self, items: usize) {
        log::debug!("Select phase");
        self.update_vision();

        if items > 0 {
            let options: Vec<_> = ItemKind::all()
                .into_iter()
                .filter(|item| *item != ItemKind::KingSkull)
                .collect();
            let mut rng = thread_rng();
            let options = (0..3).map(|_| *options.choose(&mut rng).unwrap()).collect();
            self.phase = Phase::Select {
                options,
                extra_items: items - 1,
            };
        } else {
            self.next_turn();
        }
    }

    fn next_turn(&mut self) {
        self.turn += 1;
        self.player.turns_left = self.player.turns_left.saturating_sub(1);
        if self.player.turns_left == 0 {
            // Damage for every enemy left on the board
            let damage = self
                .entities
                .iter()
                .filter(|(_, e)| e.fraction == Fraction::Enemy)
                .count();
            self.player.hearts = self.player.hearts.saturating_sub(damage);
            if self.player.hearts == 0 {
                self.game_over();
            } else {
                self.finish_level(false);
            }
        } else {
            self.night_phase(false);
        }
    }

    fn finish_level(&mut self, win: bool) {
        log::info!("Level finished, win: {}", win);
        self.phase = Phase::LevelFinished {
            win,
            timer: Lifetime::new_max(r32(2.0)),
        };
    }

    fn game_over(&mut self) {
        log::info!("Game over");
        self.phase = Phase::GameOver;
    }

    fn retry(&mut self) {
        *self = Self::new(self.assets.clone(), self.config.clone());
    }

    fn calculate_empty_space(&self) -> HashSet<vec2<Coord>> {
        let mut available: HashSet<_> = self.grid.tiles.clone();

        for (_, entity) in &self.entities {
            available.remove(&entity.position);
        }
        for (_, item) in &self.items {
            available.remove(&item.position);
        }

        available
    }

    pub fn update_vision(&mut self) {
        log::debug!("Updating vision");
        let mut visible: HashSet<_> = self.grid.lights.keys().copied().collect();
        for (_, entity) in &self.entities {
            if let EntityKind::Player = entity.kind {
                if entity.look_dir == vec2::ZERO {
                    continue;
                }
                let mut pos = entity.position;
                visible.insert(pos);
                loop {
                    let target = pos + entity.look_dir;
                    if !self.grid.check_pos(target) {
                        break;
                    }
                    visible.insert(target);
                    pos = target;
                }
            }
        }

        for (_, board_item) in &self.items {
            let item = &self.player.items[board_item.item_id];
            if let ItemKind::CursedSkull = item.kind {
                visible.insert(board_item.position);
            }
        }

        self.visible_tiles = visible;
    }

    fn check_deaths(&mut self) {
        for (id, entity) in &self.entities {
            if entity.health.is_min() {
                self.animations.insert(Animation::new(
                    self.config.animation_time,
                    AnimationKind::EntityDeath {
                        entity: id,
                        pos: entity.position,
                    },
                ));
            }
        }

        if !self
            .entities
            .iter()
            .any(|(_, e)| e.fraction == Fraction::Enemy)
        {
            // All enemies died -> next level
            self.finish_level(true);
        }
    }

    /// Move the entity to the target position and swap with the entity occupying the target (if any).
    fn move_entity_swap(&mut self, entity_id: Id, target_pos: vec2<Coord>) {
        let Some(entity) = self.entities.get_mut(entity_id) else {
            log::error!("entity does not exist: {:?}", entity_id);
            return;
        };

        let target_pos = if self.grid.check_pos(target_pos) {
            target_pos
        } else {
            log::error!("tried to move to an invalid position: {}", target_pos);
            return;
        };

        let fraction = entity.fraction;

        // Swap with entities
        let mut move_entity = None;
        if let Some((i, _)) = self
            .entities
            .iter_mut()
            .find(|(_, e)| e.position == target_pos)
        {
            move_entity = Some(i);
        }

        // Activate and swap items
        let ids: Vec<_> = self.items.iter().map(|(i, _)| i).collect();
        let mut move_item = None;
        for i in ids {
            if self.items[i].position == target_pos {
                // Activate
                self.resolve_item_active(fraction, i);
                // Swap
                move_item = Some(i);
            }
        }

        self.animations.insert(Animation::new(
            0.0,
            AnimationKind::MovePlayer {
                entity_id,
                move_entity,
                move_item,
                target_pos,
            },
        ));
    }
}

fn distance(a: vec2<Coord>, b: vec2<Coord>) -> Coord {
    let delta = b - a;
    delta.x.abs().max(delta.y.abs())
}
