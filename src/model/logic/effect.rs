use super::*;

impl InventoryItem {
    pub fn damage_nearest(self, damage: Hp, state: &ModelState, effects: &mut Vec<Effect>) {
        let Some(board_item) = self.on_board.and_then(|id| state.items.get(id)) else {
            log::error!(
                "called an effect from an item that is not on the board: {:?}",
                self
            );
            return;
        };

        let source_fraction = Fraction::Player;
        let nearest = state
            .entities
            .iter()
            .filter(|(_, entity)| source_fraction != entity.fraction)
            .min_by_key(|(_, entity)| distance(entity.position, board_item.position));
        if let Some((target, _)) = nearest {
            effects.push(Effect::Damage { target, damage });
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

        match effect.effect {
            Effect::Damage { target, damage } => {
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
