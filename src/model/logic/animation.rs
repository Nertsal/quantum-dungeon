use super::*;

impl Model {
    pub(super) fn update_animations(&mut self, delta_time: Time) {
        for animation in &mut self.ending_animations {
            animation.time.change(-delta_time);
        }
        self.ending_animations
            .retain(|anim| anim.time.is_above_min());

        let mut finished = Vec::new();

        let ids: Vec<Id> = self.animations.iter().map(|(i, _)| i).collect();
        'anim: for i in ids {
            for &id in &self.animations[i].dependent_on {
                if self.animations.contains(id) {
                    // Wait for the animation to finish
                    continue 'anim;
                }
            }

            let animation = &mut self.animations[i];

            // TODO: independent start and end times
            if let AnimationKind::EntityDeath { .. } = animation.kind {
                animation.time.set_ratio(Time::ZERO);
            } else {
                animation.time.change(-delta_time);
            }

            if animation.time.is_min() {
                finished.push(i);
            }
        }

        for i in finished {
            let mut animation = self.animations.remove(i).unwrap();
            match &animation.kind {
                AnimationKind::MoveEntity {
                    entity_id,
                    target_pos,
                } => {
                    if let Some(entity) = self.state.borrow_mut().entities.get_mut(*entity_id) {
                        entity.position = *target_pos;
                    }
                }
                AnimationKind::MoveItem {
                    item_id,
                    target_pos,
                } => {
                    if let Some(item) = self.state.borrow_mut().items.get_mut(*item_id) {
                        item.position = *target_pos;
                    }
                }
                AnimationKind::ItemEffect { .. } => {}
                AnimationKind::EntityDeath { entity, .. } => {
                    self.state.borrow_mut().entities.remove(*entity);
                    self.assets.sounds.enemy_death.play();
                }
                AnimationKind::ItemDeath { item, .. } => {
                    let mut state = self.state.borrow_mut();
                    if let Some(item) = state.player.items.remove(*item) {
                        if let Some(id) = item.on_board {
                            state.items.remove(id);
                        }
                    }
                }
                AnimationKind::Dupe { kind } => {
                    self.new_item_and_spawn(kind.clone());
                }
                AnimationKind::Damage { target, damage, .. } => {
                    if let Some(target) = self.state.borrow_mut().entities.get_mut(*target) {
                        target.health.change(-damage);
                        self.assets.sounds.damage.play();
                    }
                }
                AnimationKind::Bonus {
                    target,
                    bonus,
                    permanent,
                    ..
                } => {
                    // What is this trick KEKW
                    let mut state = self.state.borrow_mut();
                    let state = &mut *state;

                    if let Some(board_item) = state.items.get(*target) {
                        let item = &mut state.player.items[board_item.item_id];
                        if *permanent {
                            item.perm_stats = item.perm_stats.combine(bonus);
                        } else {
                            item.temp_stats = item.temp_stats.combine(bonus);
                        }
                    }
                }
            }
            animation.time = Lifetime::new_max(animation.ending_time);
            self.ending_animations.push(animation);
        }
    }

    fn new_item_and_spawn(&mut self, kind: ItemKind) {
        let item = self
            .engine
            .init_item(kind)
            .expect("Item initialization failed"); // TODO: handle error
        let item_id = self.state.borrow_mut().player.items.insert(item);

        let available = self
            .calculate_empty_space()
            .sub(&self.state.borrow().visible_tiles);
        if !available.is_empty() {
            let mut rng = thread_rng();
            let &position = available.iter().choose(&mut rng).unwrap();

            let mut state = self.state.borrow_mut();
            let on_board = state.items.insert(BoardItem {
                position,
                item_id,
                used: false,
            });
            state.player.items[item_id].on_board = Some(on_board);
        }
    }
}
