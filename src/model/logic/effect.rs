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
                if let Some(item) = state.items.get(item_id) {
                    play_animation(AnimationKind::ItemDeath {
                        item: item_id,
                        pos: item.position,
                    });
                }
            }
            Effect::Duplicate { item_id } => {
                if let Some((_, inv)) = state
                    .player
                    .items
                    .iter()
                    .find(|(_, item)| item.on_board == Some(item_id))
                {
                    play_animation(AnimationKind::Dupe {
                        kind: inv.kind.clone(),
                    });
                }
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
