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

        if self.wait_for_animations() {
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

    fn resolve_effect(&mut self, effect: QueuedEffect) {
        log::debug!("Resolving effect {:?}", effect);
        let mut state = self.state.borrow_mut();
        let Some(proc_item) = state.items.get(effect.proc_item) else {
            log::error!("invalid item {:?}", effect.proc_item);
            return;
        };
        let stats = state.player.items[proc_item.item_id].current_stats();
        let stats = engine::item::Stats::from(stats);

        match effect.effect {
            Effect::SetUsed { item_id } => {
                if let Some(item) = state.items.get_mut(item_id) {
                    item.used = true;
                }
            }
            Effect::Damage { target, damage } => {
                let damage: Hp = damage.call((stats,)).expect("failed to call rune function"); // TODO: handle error
                self.animations.insert(Animation::new(
                    self.config.animation_time,
                    AnimationKind::Damage {
                        from: proc_item.position,
                        target,
                        damage,
                    },
                ));
            }
            Effect::Bonus {
                from,
                target,
                bonus,
                permanent,
            } => {
                self.animations.insert(Animation::new(
                    self.config.animation_time,
                    AnimationKind::Bonus {
                        from,
                        target,
                        bonus,
                        permanent,
                    },
                ));
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
                if let Some(item) = state.items.get(item_id) {
                    self.animations.insert(Animation::new(
                        self.config.animation_time,
                        AnimationKind::ItemDeath {
                            item: item_id,
                            pos: item.position,
                        },
                    ));
                }
            }
        }
    }
}
