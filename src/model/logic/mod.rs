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
        self.effect_queue_stack.is_empty() && self.wait_for_animations()
    }

    /// Returns `true` when all animations are done.
    fn wait_for_animations(&self) -> bool {
        self.animations.is_empty() && self.ending_animations.is_empty()
    }

    pub fn get_light_level(&self, position: vec2<Coord>) -> f32 {
        match self.phase {
            Phase::LevelStarting { .. } => 0.0,
            Phase::Night {
                fade_time,
                light_time,
            } => {
                if self.visible_tiles.contains(&position) {
                    1.0
                } else if fade_time.is_above_min() {
                    fade_time.get_ratio().as_f32()
                } else {
                    1.0 - light_time.get_ratio().as_f32()
                }
            }
            _ => 1.0,
        }
    }

    pub fn night_phase(&mut self, start_faded: bool) {
        log::debug!("Night phase");
        self.phase = Phase::Night {
            fade_time: if start_faded {
                Lifetime::new_zero(r32(1.0))
            } else {
                Lifetime::new_max(r32(1.0))
            },
            light_time: Lifetime::new_max(r32(1.0)),
        };

        self.state.borrow_mut().player.extra_items = self.turn % 2;
        self.grid.fractured.clear();
        for (_, entity) in &self.state.borrow().entities {
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
            let options: Vec<_> = self
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
            let damage = self
                .state
                .borrow()
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
            self.night_phase(false);
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
            timer: Lifetime::new_max(r32(2.0)),
        };
    }

    fn game_over(&mut self) {
        log::info!("Game over");
        self.phase = Phase::GameOver;
    }

    fn retry(&mut self) {
        log::debug!("Retry");
        *self = Self::new_compiled(
            self.assets.clone(),
            self.config.clone(),
            std::mem::replace(
                &mut self.engine,
                Engine::new(Rc::clone(&self.state), Rc::clone(&self.side_effects)).unwrap(),
            ),
            self.all_items.clone(),
            Rc::clone(&self.state),
            Rc::clone(&self.side_effects),
        );
    }

    fn calculate_empty_space(&self) -> HashSet<vec2<Coord>> {
        let mut available: HashSet<_> = self.grid.tiles.clone();

        for (_, entity) in &self.state.borrow().entities {
            available.remove(&entity.position);
        }
        for (_, item) in &self.state.borrow().items {
            available.remove(&item.position);
        }

        available
    }

    pub fn update_vision(&mut self) {
        let mut visible: HashSet<_> = self.grid.lights.keys().copied().collect();
        for (_, entity) in &self.state.borrow().entities {
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

        self.visible_tiles = visible;
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

        if !state
            .entities
            .iter()
            .any(|(_, e)| e.fraction == Fraction::Enemy)
        {
            // All enemies died -> next level
            drop(state);
            self.finish_level(true);
        }
    }

    /// Move the entity to the target position and swap with the entity occupying the target (if any).
    fn move_entity_swap(&mut self, entity_id: Id, target_pos: vec2<Coord>) {
        let Some(_entity) = self.state.borrow_mut().entities.get_mut(entity_id) else {
            log::error!("entity does not exist: {:?}", entity_id);
            return;
        };

        let target_pos = if self.grid.check_pos(target_pos) {
            target_pos
        } else {
            log::error!("tried to move to an invalid position: {}", target_pos);
            return;
        };

        // Activate items
        let ids: Vec<_> = self.state.borrow().items.iter().map(|(i, _)| i).collect();
        for item_id in ids {
            if self.state.borrow().items[item_id].position == target_pos {
                // Activate
                self.resolve_trigger(Trigger::Active, Some(item_id));
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
