use super::*;

impl Model {
    /// Give a temporary bonus to nearby items.
    pub(super) fn bonus_near_temporary(
        &mut self,
        position: vec2<Coord>,
        range: Coord,
        item_ref: ItemRef,
        bonus: ItemStats,
    ) {
        for (target, board_item) in &self.items {
            let item = &mut self.player.items[board_item.item_id];
            if distance(board_item.position, position) <= range && item_ref.check(&item.kind) {
                self.animations.insert(Animation::new(
                    self.config.animation_time,
                    AnimationKind::Bonus {
                        from: position,
                        target,
                        bonus: bonus.clone(),
                        permanent: false,
                    },
                ));
            }
        }
    }

    pub(super) fn deal_damage_around(
        &mut self,
        position: vec2<Coord>,
        damage: Hp,
        range: Coord,
        after: Vec<Id>,
    ) {
        let source_fraction = Fraction::Player;
        for (target, entity) in &self.entities {
            if source_fraction != entity.fraction && distance(entity.position, position) <= range {
                self.animations.insert(
                    Animation::new(
                        self.config.animation_time,
                        AnimationKind::Damage {
                            from: position,
                            target,
                            damage,
                        },
                    )
                    .after(after.clone()),
                );
            }
        }
    }

    pub(super) fn deal_damage_nearest(
        &mut self,
        position: vec2<Coord>,
        damage: Hp,
        after: Vec<Id>,
    ) -> Option<Id> {
        let source_fraction = Fraction::Player;
        let nearest = self
            .entities
            .iter()
            .filter(|(_, entity)| source_fraction != entity.fraction)
            .min_by_key(|(_, entity)| distance(entity.position, position));
        nearest.map(|(target, _)| {
            self.animations.insert(
                Animation::new(
                    self.config.animation_time,
                    AnimationKind::Damage {
                        from: position,
                        target,
                        damage,
                    },
                )
                .after(after),
            )
        })
    }

    pub(super) fn deal_damage_random(
        &mut self,
        position: vec2<Coord>,
        damage: Hp,
        after: Vec<Id>,
    ) -> Option<Id> {
        let source_fraction = Fraction::Player;
        let mut rng = thread_rng();
        let target = self
            .entities
            .iter()
            .filter(|(_, entity)| source_fraction != entity.fraction)
            .choose(&mut rng);
        target.map(|(target, _)| {
            self.animations.insert(
                Animation::new(
                    self.config.animation_time,
                    AnimationKind::Damage {
                        from: position,
                        target,
                        damage,
                    },
                )
                .after(after),
            )
        })
    }

    pub(super) fn count_items_near(&self, position: vec2<Coord>, item_ref: ItemRef) -> Vec<Id> {
        self.items
            .iter()
            .filter(|(_, board_item)| {
                let d = distance(position, board_item.position);
                let item = &self.player.items[board_item.item_id];
                item_ref.check(&item.kind) && d > 0 && d <= 1
            })
            .map(|(i, _)| i)
            .collect()
    }
}
