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
            animation.time.change(-delta_time);
            if animation.time.is_min() {
                if let AnimationKind::UseActive { .. } = animation.kind {
                    if self.animations.len() > 2 {
                        // Wait for other animations
                        continue;
                    }
                }
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
                    self.state.borrow_mut().entities[*entity_id].position = *target_pos;
                }
                AnimationKind::MoveItem {
                    item_id,
                    target_pos,
                } => {
                    if let Some(item) = self.state.borrow_mut().items.get_mut(*item_id) {
                        item.position = *target_pos;
                    }
                }
                AnimationKind::UseActive { fraction, item_id } => {
                    // Activate item
                    let _fraction = *fraction; // TODO: maybe
                    let item_id = *item_id;
                    self.resolve_trigger(Trigger::Active, Some(item_id));
                }
                AnimationKind::EntityDeath { entity, .. } => {
                    self.state.borrow_mut().entities.remove(*entity);
                    self.assets.sounds.enemy_death.play();
                }
                AnimationKind::ItemDeath { item, .. } => {
                    let item = self.state.borrow_mut().items.remove(*item).unwrap();
                    self.state.borrow_mut().player.items.remove(item.item_id);
                }
                AnimationKind::Dupe { kind } => {
                    self.new_item_and_spawn(kind.clone());
                }
                AnimationKind::Damage { target, damage, .. } => {
                    self.state.borrow_mut().entities[*target]
                        .health
                        .change(-damage);
                    self.assets.sounds.damage.play();
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

                    let board_item = &state.items[*target];
                    let item = &mut state.player.items[board_item.item_id];
                    if *permanent {
                        item.perm_stats = item.perm_stats.combine(bonus);
                    } else {
                        item.temp_stats = item.temp_stats.combine(bonus);
                    }
                }
            }
            animation.time.set_ratio(R32::ONE);
            self.ending_animations.push(animation);
        }
    }

    fn new_item_and_spawn(&mut self, kind: ItemKind) {
        let item = self
            .engine
            .init_item(kind)
            .expect("Item initialization failed"); // TODO: handle error
        let mut state = self.state.borrow_mut();
        let item_id = state.player.items.insert(item);

        let available = self.calculate_empty_space().sub(&self.visible_tiles);
        if !available.is_empty() {
            let mut rng = thread_rng();
            let &position = available.iter().choose(&mut rng).unwrap();

            let item = &mut state.player.items[item_id];
            let on_board = self.state.borrow_mut().items.insert(BoardItem {
                position,
                item_id,
                used: false,
            });
            item.on_board = Some(on_board);
        }
    }
}
