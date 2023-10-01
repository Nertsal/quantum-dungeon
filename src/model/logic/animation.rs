use super::*;

impl Model {
    pub(super) fn update_animations(&mut self, delta_time: Time) {
        for animation in &mut self.ending_animations {
            animation.time.change(-delta_time);
        }
        self.ending_animations
            .retain(|anim| anim.time.is_above_min());

        let mut finished = Vec::new();
        for (i, animation) in self.animations.iter_mut().enumerate() {
            animation.time.change(-delta_time);
            if animation.time.is_min() {
                finished.push(i);
            }
        }

        for i in finished.into_iter().rev() {
            let mut animation = self.animations.swap_remove(i);
            match &animation.kind {
                AnimationKind::UseActive { fraction, item_id } => {
                    // Activate item
                    let fraction = *fraction;
                    let item_id = *item_id;
                    self.active_effect(fraction, item_id);
                }
                AnimationKind::Death { item } => {
                    self.items.remove(*item);
                }
                AnimationKind::CameraDupe { item } => {
                    // Duplicate an item
                    let item = self.items.get(*item).unwrap();
                    let item = &self.player.items[item.item_id];
                    self.new_item_and_spawn(item.kind);
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
            let on_board = self.items.insert(BoardItem { position, item_id });
            item.on_board = Some(on_board);
        }
    }
}
