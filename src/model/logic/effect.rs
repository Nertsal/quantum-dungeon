use super::*;

impl ScriptItem<'_> {
    pub fn damage_nearest(&mut self, damage: ScriptFunction) {
        let source_fraction = Fraction::Player;
        let nearest = self
            .model
            .entities
            .iter()
            .filter(|(_, entity)| source_fraction != entity.fraction)
            .min_by_key(|(_, entity)| distance(entity.position, self.board_item.position));
        if let Some((target, _)) = nearest {
            self.effects.push(Effect::Damage { target, damage });
        }
    }
}

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
        let state = self.state.borrow();
        let Some(proc_item) = state.items.get(effect.proc_item) else {
            log::error!("invalid item {:?}", effect.proc_item);
            return;
        };
        let stats = self.player.items[proc_item.item_id].current_stats();
        let stats = engine::item::Stats::from(stats);

        match effect.effect {
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
        }
    }
}
