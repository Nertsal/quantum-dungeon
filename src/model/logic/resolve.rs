use super::*;

impl Model {
    pub(super) fn resolve_animations(&mut self, delta_time: Time) {
        for item in &mut self.resolved_items {
            item.time.change(-delta_time);
        }
        self.resolved_items.retain(|item| item.time.is_above_min());

        for item in &mut self.resolving_items {
            item.animations.retain(|id| self.animations.contains(*id));
            if item.animations.is_empty() {
                self.resolved_items.insert(ItemResolved {
                    board_item: item.board_item,
                    time: Lifetime::new_max(self.config.animation_time),
                });
            }
        }
        self.resolving_items
            .retain(|item| !item.animations.is_empty());

        let wait_for_effects = self.wait_for_effects() && self.resolved_items.is_empty();
        match &mut self.phase {
            Phase::LevelStarting { timer } => {
                timer.change(-delta_time);
                if timer.is_min() {
                    log::info!("level start dawn");
                    self.dawn_phase();
                }
            }
            Phase::LevelFinished { timer, .. } => {
                timer.change(-delta_time);
                if timer.is_min() {
                    self.shift_everything();
                    self.next_level(false);
                }
            }
            Phase::PostVision { timer } => {
                timer.change(-delta_time);
                if timer.is_min() {
                    let mut state = self.state.borrow_mut();
                    state.player.refreshes = 2;
                    let extra = state.player.extra_items;
                    drop(state);
                    self.select_phase(extra);
                }
            }
            Phase::Night { fade_time } => {
                if wait_for_effects {
                    if fade_time.is_above_min() {
                        fade_time.change(-delta_time);
                    } else if !self
                        .state
                        .borrow()
                        .entities
                        .iter()
                        .any(|(_, e)| e.fraction == Fraction::Enemy)
                    {
                        // Win -> next level
                        self.finish_level(true);
                    } else {
                        // Next turn
                        self.shift_everything();
                        self.spawn_items();
                        self.dawn_phase();
                    }
                }
            }
            Phase::Dawn { light_time } => {
                light_time.change(-delta_time);
                if light_time.is_min() {
                    self.resolution_phase();
                }
            }
            Phase::DayBonus { start_delay } => {
                if start_delay.is_above_min() {
                    // Start animation
                    start_delay.change(-delta_time);
                    if start_delay.is_min() {
                        self.resolve_all(Trigger::DayBonus);
                    }
                } else if self.wait_for_effects() {
                    self.resolve_all(Trigger::DayAction);
                    self.phase = Phase::DayAction {
                        end_delay: Lifetime::new_max(r32(0.2)),
                    };
                }
            }
            Phase::DayAction { .. } => {
                if self.wait_for_effects() {
                    // End animation
                    if let Phase::DayAction { end_delay } = &mut self.phase {
                        end_delay.change(-delta_time);
                        if end_delay.is_min() {
                            self.day_end_phase();
                        }
                    }
                }
            }
            Phase::Active {
                entity_id,
                position,
            } if wait_for_effects => {
                let entity_id = *entity_id;
                let target_pos = *position;
                self.resolve_active_phase(entity_id, target_pos)
            }
            _ => (),
        }
    }

    pub fn resolve_active_phase(&mut self, entity_id: Id, target_pos: vec2<Coord>) {
        self.animations.insert(Animation::new(
            self.config.animation_time,
            AnimationKind::MoveEntity {
                entity_id,
                target_pos,
            },
        ));

        let state = self.state.borrow();
        let from_pos = target_pos;
        let target_pos = state.entities[entity_id].position;
        if let Some((entity_id, _)) = state.entities.iter().find(|(_, e)| e.position == from_pos) {
            self.animations.insert(Animation::new(
                self.config.animation_time,
                AnimationKind::MoveEntity {
                    entity_id,
                    target_pos,
                },
            ));
        }
        if let Some((item_id, _)) = state.items.iter().find(|(_, i)| i.position == from_pos) {
            self.animations.insert(Animation::new(
                self.config.animation_time,
                AnimationKind::MoveItem {
                    item_id,
                    target_pos,
                },
            ));
        }
        drop(state);

        self.player_phase();
    }

    pub fn resolution_phase(&mut self) {
        log::debug!("Day resolution phase");
        self.phase = Phase::DayBonus {
            start_delay: Lifetime::new_max(r32(0.2)),
        };

        // What is this trick KEKW
        let mut state = self.state.borrow_mut();
        let state = &mut *state;

        // Reset vision
        state.visible_tiles.clear();
    }

    pub(super) fn resolve_trigger(&mut self, trigger: Trigger, id: Id) {
        let effects = self
            .resolve_item(id, trigger)
            .into_iter()
            .map(move |effect| QueuedEffect {
                trigger,
                proc_item: id,
                effect,
            })
            .collect();
        self.effect_queue_stack.push(effects);
    }

    pub(super) fn resolve_all(&mut self, trigger: Trigger) {
        let mut ids: Vec<_> = self.state.borrow().items.iter().map(|(i, _)| i).collect();

        // Sort by item position
        ids.sort_by_key(|&id| {
            let pos = self.state.borrow().items[id].position;
            // Left -> Right
            // Top -> Bottom
            (pos.x, -pos.y)
        });

        self.resolution_queue
            .extend(ids.into_iter().map(|id| (id, trigger)));
    }

    /// Resolve the item's response to the trigger.
    fn resolve_item(&mut self, item_id: Id, trigger: Trigger) -> Vec<Effect> {
        log::debug!("Resolving item {:?} trigger {:?}", item_id, trigger);

        let state = self.state.borrow();
        let Some(board_item) = state.items.get(item_id) else {
            log::debug!("Item {:?} not found on the board", item_id);
            return vec![];
        };

        if let Trigger::Active = trigger {
            if board_item.used {
                log::debug!("Item {:?} has already been activated", item_id);
                return vec![];
            }
        }

        let item = &state.player.items[board_item.item_id];

        // Execute
        // NOTE: requires immutable access to [ModelState]
        let item_state = match self
            .engine
            .item_trigger(item, board_item, trigger.method_name())
        {
            Ok(state) => state,
            Err(err) => {
                log::error!("Trigger handler failed: {:?}", err);
                return vec![];
            }
        };

        // Update item state
        drop(state);
        // What is this trick KEKW
        let mut state = self.state.borrow_mut();
        let state = &mut *state;

        let mut effects = std::mem::take(&mut *self.side_effects.borrow_mut());
        let board_item = state.items.get(item_id).unwrap();
        if !effects.is_empty()
            && matches!(trigger, Trigger::Active)
            && !effects.iter().any(|e| matches!(e, Effect::Destroy { .. }))
        {
            effects.push(Effect::SetUsed { item_id });
        }
        state.player.items[board_item.item_id].state = item_state;

        log::debug!("Item {:?} resolved: {:?}", item_id, effects);
        effects
    }
}
