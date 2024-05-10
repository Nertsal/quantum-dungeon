mod action;
mod animation;
pub mod effect;
mod engine;
mod gen;
mod item;
mod resolve;

use super::*;

impl Model {
    pub fn update(&mut self, delta_time: Time) {
        // TODO: unhardcode
        if let Phase::Map { .. } = self.phase {
        } else {
            self.update_animations(delta_time);
        }
        self.resolve_animations(delta_time);
        self.update_effects();

        if let Phase::LevelFinished { .. } = self.phase {
        } else {
            self.check_deaths();
            if self.animations.is_empty() && self.ending_animations.is_empty() {
                if let Phase::Player = self.phase {
                    if self.state.borrow().player.moves_left == 0 {
                        self.vision_phase();
                    }
                }
            }
        }
    }

    /// Returns `true` when all effects are processed and executed.
    fn wait_for_effects(&self) -> bool {
        self.resolution_queue.is_empty()
            && self.effect_queue_stack.is_empty()
            && self.wait_for_animations()
    }

    /// Returns `true` when all animations are done.
    fn wait_for_animations(&self) -> bool {
        self.animations.is_empty() && self.ending_animations.is_empty()
    }

    pub fn get_light_level(&self, position: vec2<Coord>) -> f32 {
        let state = self.state.borrow();
        match self.phase {
            Phase::LevelStarting { .. } => 0.0,
            Phase::Night { fade_time } => {
                if state.visible_tiles.contains(&position) {
                    1.0
                } else {
                    fade_time.get_ratio().as_f32()
                }
            }
            Phase::Dawn { light_time } => {
                if state.visible_tiles.contains(&position) {
                    1.0
                } else {
                    1.0 - light_time.get_ratio().as_f32()
                }
            }
            _ => 1.0,
        }
    }

    pub fn night_phase(&mut self) {
        log::debug!("Night phase");
        self.phase = Phase::Night {
            fade_time: Lifetime::new_max(r32(1.0)),
        };

        self.resolve_all(Trigger::Night);
    }

    pub fn dawn_phase(&mut self) {
        log::debug!("Dawn phase");
        self.phase = Phase::Dawn {
            light_time: Lifetime::new_max(r32(1.0)),
        };

        let mut state = self.state.borrow_mut();
        let state = &mut *state;

        state.player.extra_items = self.turn % 2;
        state.grid.fractured.clear();
        for (_, entity) in &state.entities {
            if let EntityKind::Player = entity.kind {
                state.grid.fractured.insert(entity.position);
            }
        }

        // Update light duration
        for duration in state.grid.lights.values_mut() {
            *duration = duration.saturating_sub(1);
        }
        state.grid.lights.retain(|_, duration| *duration > 0);

        // Clear temp stats
        for (_, item) in &mut state.player.items {
            item.temp_stats = ItemStats::default();
        }
        // Update turn counter
        for (_, item) in &mut state.items {
            item.used = false;
            state.player.items[item.item_id].turns_on_board += 1;
        }
    }

    pub fn day_end_phase(&mut self) {
        log::debug!("Day phase");
        self.phase = Phase::Player;
        self.state.borrow_mut().player.moves_left = 3;
    }

    fn player_phase(&mut self) {
        log::debug!("Player can move again");
        self.phase = Phase::Player;
    }

    fn vision_phase(&mut self) {
        log::debug!("Vision phase");
        self.phase = Phase::Vision;
        for (_, entity) in &mut self.state.borrow_mut().entities {
            entity.look_dir = vec2::ZERO;
        }
        self.update_vision();
    }

    fn select_phase(&mut self, items: usize) {
        log::debug!("Select phase");
        self.update_vision();

        if items > 0 {
            let state = self.state.borrow();
            let options: Vec<_> = state
                .all_items
                .iter()
                .filter(|item| item.config.appears_in_shop)
                .collect();
            let mut rng = thread_rng();
            let options = (0..3)
                .map(|_| {
                    let item = options.choose(&mut rng).unwrap();
                    (*item).clone()
                })
                .collect();
            self.phase = Phase::Select {
                options,
                extra_items: items - 1,
            };
        } else {
            self.next_turn();
        }
    }

    fn next_turn(&mut self) {
        log::debug!("Next turn");
        self.turn += 1;
        let mut state = self.state.borrow_mut();
        state.player.turns_left = state.player.turns_left.saturating_sub(1);
        if state.player.turns_left == 0 {
            // Damage for every enemy left on the board
            let damage = state
                .entities
                .iter()
                .filter(|(_, e)| e.fraction == Fraction::Enemy)
                .count();
            state.player.hearts = state.player.hearts.saturating_sub(damage);
            let hearts = state.player.hearts;
            drop(state);
            if hearts == 0 {
                self.game_over();
            } else {
                self.finish_level(false);
            }
        } else {
            drop(state);
            self.night_phase();
        }
    }

    fn finish_level(&mut self, win: bool) {
        if win && !self.wait_for_effects() {
            // Cant win until all effects are done
            return;
        }

        log::info!("Level finished, win: {}", win);
        self.phase = Phase::LevelFinished {
            win,
            timer: Lifetime::new_max(r32(0.0)),
        };
    }

    fn game_over(&mut self) {
        log::info!("Game over");
        self.phase = Phase::GameOver;
    }

    fn retry(&mut self) {
        log::debug!("Retry");
        *self = Self::new(
            self.assets.clone(),
            self.config.clone(),
            self.item_assets.clone(),
        );
    }

    fn calculate_empty_space(&self) -> HashSet<vec2<Coord>> {
        let state = self.state.borrow();
        let mut available: HashSet<_> = state.grid.tiles.clone();

        for (_, entity) in &state.entities {
            available.remove(&entity.position);
        }
        for (_, item) in &state.items {
            available.remove(&item.position);
        }

        available
    }

    pub fn update_vision(&mut self) {
        let mut state = self.state.borrow_mut();
        let mut visible: HashSet<_> = state.grid.lights.keys().copied().collect();
        for (_, entity) in &state.entities {
            if let EntityKind::Player = entity.kind {
                if entity.look_dir == vec2::ZERO {
                    continue;
                }
                let mut pos = entity.position;
                visible.insert(pos);
                loop {
                    let target = pos + entity.look_dir;
                    if !state.grid.check_pos(target) {
                        break;
                    }
                    visible.insert(target);
                    pos = target;
                }
            }
        }

        state.visible_tiles = visible;
    }

    fn check_deaths(&mut self) {
        let state = self.state.borrow();

        for (id, entity) in &state.entities {
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
    }

    /// Move the entity to the target position and swap with the entity occupying the target (if any).
    fn move_entity_swap(&mut self, entity_id: Id, target_pos: vec2<Coord>) {
        let mut state = self.state.borrow_mut();
        let Some(_entity) = state.entities.get_mut(entity_id) else {
            log::error!("entity does not exist: {:?}", entity_id);
            return;
        };

        let target_pos = if state.grid.check_pos(target_pos) {
            target_pos
        } else {
            log::error!("tried to move to an invalid position: {}", target_pos);
            return;
        };
        drop(state);

        // Activate items
        let ids: Vec<_> = self.state.borrow().items.iter().map(|(i, _)| i).collect();
        for item_id in ids {
            if self.state.borrow().items[item_id].position == target_pos {
                // Activate
                self.resolve_trigger(Trigger::Active, item_id);
            }
        }

        // NOTE: swapping items/entities is resolve on phase end

        self.phase = Phase::Active {
            entity_id,
            position: target_pos,
        };
    }
}

fn distance(a: vec2<Coord>, b: vec2<Coord>) -> Coord {
    let delta = b - a;
    delta.x.abs().max(delta.y.abs())
}

fn distance_manhattan(a: vec2<Coord>, b: vec2<Coord>) -> Coord {
    let delta = b - a;
    delta.x.abs() + delta.y.abs()
}
