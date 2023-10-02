use super::*;

impl Model {
    /// Collect an item at the given position.
    pub(super) fn collect_item_at(&mut self, fraction: Fraction, position: vec2<Coord>) {
        let ids: Vec<_> = self.items.iter().map(|(i, _)| i).collect();
        for i in ids {
            let item = &mut self.items[i];
            if item.position == position {
                self.resolve_item_active(fraction, i);
                return;
            }
        }
    }

    /// Give a temporary bonus to nearby items.
    pub(super) fn bonus_near_temporary(
        &mut self,
        position: vec2<Coord>,
        range: Coord,
        item_ref: ItemRef,
        bonus: ItemStats,
    ) {
        for (_, board_item) in &mut self.items {
            let item = &mut self.player.items[board_item.item_id];
            if distance(board_item.position, position) <= range && item_ref.check(item.kind) {
                item.temp_stats = item.temp_stats.combine(&bonus);
            }
        }
    }

    pub(super) fn deal_damage_around(
        &mut self,
        position: vec2<Coord>,
        source_fraction: Fraction,
        damage: Hp,
        range: Coord,
    ) {
        for (target, entity) in self.entities.iter().enumerate() {
            if source_fraction != entity.fraction && distance(entity.position, position) <= range {
                self.animations.push(Animation::new(
                    self.config.animation_time,
                    AnimationKind::Damage {
                        from: position,
                        target,
                        damage,
                    },
                ));
            }
        }
    }

    pub(super) fn count_items_near(&self, position: vec2<Coord>, item_ref: ItemRef) -> Vec<Id> {
        self.items
            .iter()
            .filter(|(_, board_item)| {
                let d = distance(position, board_item.position);
                let item = &self.player.items[board_item.item_id];
                item_ref.check(item.kind) && d > 0 && d <= 1
            })
            .map(|(i, _)| i)
            .collect()
    }
}
