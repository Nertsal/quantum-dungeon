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
                    if self.animations.len() > 1 {
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
                AnimationKind::UseActive { fraction, item_id } => {
                    // Activate item
                    let fraction = *fraction;
                    let item_id = *item_id;
                    self.active_effect(fraction, item_id);
                }
                AnimationKind::EntityDeath { entity, .. } => {
                    self.entities.remove(*entity);
                }
                AnimationKind::ItemDeath { item, .. } => {
                    let item = self.items.remove(*item).unwrap();
                    self.player.items.remove(item.item_id);
                }
                AnimationKind::Dupe { kind } => {
                    self.new_item_and_spawn(*kind);
                }
                AnimationKind::Damage { target, damage, .. } => {
                    self.entities[*target].health.change(-damage);
                }
                AnimationKind::Bonus {
                    target,
                    bonus,
                    permanent,
                    ..
                } => {
                    let board_item = &self.items[*target];
                    let item = &mut self.player.items[board_item.item_id];
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
        let item_id = self.player.items.insert(kind.instantiate());

        let available = self.calculate_empty_space().sub(&self.visible_tiles);
        if !available.is_empty() {
            let mut rng = thread_rng();
            let &position = available.iter().choose(&mut rng).unwrap();

            let item = &mut self.player.items[item_id];
            let on_board = self.items.insert(BoardItem {
                position,
                item_id,
                turns_alive: 0,
            });
            item.on_board = Some(on_board);
        }
    }
}
