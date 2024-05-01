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

    pub fn bonus_from_nearby(
        &mut self,
        range: Coord,
        filter: ItemFilter,
        bonus: ItemStats,
        permanent: bool,
    ) {
        for (_, board_item) in &self.model.items {
            let item = &self.model.player.items[board_item.item_id];
            let dist = distance(board_item.position, self.board_item.position);
            if (1..=range).contains(&dist) && filter.check(&item.kind) {
                self.effects.push(Effect::Bonus {
                    from: board_item.position,
                    target: self.item.on_board.unwrap(),
                    bonus: bonus.clone(),
                    permanent,
                });
            }
        }
    }
    // /// Give a temporary bonus to nearby items.
    // pub(super) fn bonus_near_temporary(
    //     &mut self,
    //     position: vec2<Coord>,
    //     range: Coord,
    //     item_ref: ItemFilter,
    //     bonus: ItemStats,
    // ) {
    //     for (target, board_item) in &self.state.borrow().items {
    //         let item = &mut self.player.items[board_item.item_id];
    //         if distance(board_item.position, position) <= range && item_ref.check(&item.kind) {
    //             self.animations.insert(Animation::new(
    //                 self.config.animation_time,
    //                 AnimationKind::Bonus {
    //                     from: position,
    //                     target,
    //                     bonus: bonus.clone(),
    //                     permanent: false,
    //                 },
    //             ));
    //         }
    //     }
    // }
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
        }
    }
}
