use super::*;

impl Model {
    pub(super) fn update_effects(&mut self) {
        // Clear empty queues
        while let Some(queue) = self.effect_queue_stack.pop() {
            if !queue.is_empty() {
                self.effect_queue_stack.push(queue);
                break;
            }
        }

        // if self.wait_for_animations() {
        if self.animations.is_empty() {
            self.resolve_next_effect();
        }
    }

    fn resolve_next_effect(&mut self) {
        if let Some(queue) = self.effect_queue_stack.last_mut() {
            if let Some(effect) = queue.pop_front() {
                self.resolve_effect(effect);
            }
        }
    }

    pub fn resolve_effect(&mut self, effect: QueuedEffect) {
        log::debug!("Resolving effect {:?}", effect);
        let mut state = self.state.borrow_mut();
        let Some(proc_item) = state.items.get(effect.proc_item) else {
            log::error!("invalid item {:?}", effect.proc_item);
            return;
        };
        let stats = state.player.items[proc_item.item_id].current_stats();
        let stats = crate::model::engine::item::Stats::from(stats);

        let mut animations = Vec::new();

        let animation_delay = self.animations.insert(Animation::new(
            self.config.animation_time,
            AnimationKind::ItemEffect {
                item: effect.proc_item,
            },
        ));
        animations.push(animation_delay);

        let mut play_animation =
            |kind| {
                animations.push(self.animations.insert(
                    Animation::new(self.config.animation_time, kind).after([animation_delay]),
                ));
            };

        match effect.effect {
            Effect::SetUsed { item_id } => {
                if let Some(item) = state.items.get_mut(item_id) {
                    item.used = true;
                }
            }
            Effect::Damage { target, damage } => {
                let damage: Hp = damage.call((stats,)).expect("failed to call rune function"); // TODO: handle error
                play_animation(AnimationKind::Damage {
                    from: proc_item.position,
                    target,
                    damage,
                });
            }
            Effect::Bonus {
                from,
                target,
                bonus,
                permanent,
            } => {
                play_animation(AnimationKind::Bonus {
                    from,
                    target,
                    bonus,
                    permanent,
                });
            }
            Effect::OpenTiles { tiles } => {
                let mut next_phase = Phase::Vision;
                std::mem::swap(&mut self.phase, &mut next_phase);
                self.phase = Phase::Map {
                    tiles_left: tiles,
                    next_phase: Box::new(next_phase),
                };
            }
            Effect::Destroy { item_id } => {
                // TODO: error log
                if let Some(item) = state.player.items.get(item_id) {
                    if let Some(board) = item.on_board.and_then(|id| state.items.get(id)) {
                        play_animation(AnimationKind::ItemDeath {
                            item: item_id,
                            pos: board.position,
                        });
                    } else {
                        state.player.items.remove(item_id);
                    }
                }
            }
            Effect::Duplicate { item_id } => {
                if let Some(inv) = state.player.items.get(item_id) {
                    play_animation(AnimationKind::Dupe {
                        kind: inv.kind.clone(),
                    });
                }
            }
            Effect::GainMoves { moves } => {
                state.player.moves_left += moves;
            }
            Effect::Portal => {
                if state
                    .player
                    .items
                    .iter()
                    .any(|(_, item)| item.kind.config.categories.contains(&Category::Magic))
                {
                    let mut next_phase = Phase::Vision;
                    std::mem::swap(&mut self.phase, &mut next_phase);
                    self.phase = Phase::Portal {
                        next_phase: Box::new(next_phase),
                    };
                } else {
                    log::debug!("Tried activating portal state but there are no magic items");
                }
            }
            Effect::SwapItems { board_a, board_b } => {
                if let Some(a) = state.items.get(board_a) {
                    if let Some(b) = state.items.get(board_b) {
                        play_animation(AnimationKind::MoveItem {
                            item_id: board_a,
                            target_pos: b.position,
                        });
                        play_animation(AnimationKind::MoveItem {
                            item_id: board_b,
                            target_pos: a.position,
                        });
                    }
                }
            }
            Effect::TransformItem {
                item_id,
                target_name,
            } => {
                let state = &mut *state;
                if let Some(item) = state.player.items.get_mut(item_id) {
                    if let Some(target) = state
                        .all_items
                        .iter()
                        .find(|kind| *kind.config.name == target_name)
                    {
                        let new_item = self
                            .engine
                            .init_item(target.clone())
                            .expect("Item initialization failed");
                        *item = new_item;
                    } else {
                        log::error!(
                            "Tried transforming an item into an unknown kind: {:?}",
                            target_name
                        );
                    }
                }
            }
            Effect::EmitLight {
                position,
                radius,
                duration,
            } => {
                state.grid.light_up(position, radius, duration);
                drop(state);
                self.update_vision();
            }
        }

        let board_item = effect.proc_item;
        if self.resolving_items.get(&board_item).is_none() {
            // Set wind up animation
            let down = self
                .resolved_items
                .get(&board_item)
                .map_or(Time::ZERO, |item| item.time.get_ratio());
            let t = Time::ONE - down;
            if let Some(anim) = self.animations.get_mut(animation_delay) {
                anim.time.set_ratio(t);
            }
        }
        self.resolving_items.insert(ItemResolving {
            board_item,
            animations,
        });
    }
}
