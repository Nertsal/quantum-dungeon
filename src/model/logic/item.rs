use super::*;

impl Model {
    /// Collect an item at the given position.
    pub(super) fn collect_item_at(&mut self, fraction: Fraction, position: vec2<Coord>) {
        let ids: Vec<_> = self.items.iter().map(|(i, _)| i).collect();
        for i in ids {
            let item = &mut self.items[i];
            if item.position == position {
                self.active_phase(fraction, i);
                return;
            }
        }
    }

    pub(super) fn use_item(&mut self, fraction: Fraction, board_item: BoardItem) {
        log::debug!("Use item by fraction {:?}: {:?}", fraction, board_item);
        let item = &self.player.items[board_item.item_id];
        match item.kind {
            ItemKind::Sword => {
                let damage = item.temp_stats.damage.unwrap_or_default();
                let range = 1;
                self.deal_damage_around(board_item.position, fraction, damage, range);
            }
            ItemKind::Forge => self.bonus_near_temporary(
                board_item.position,
                1,
                ItemRef::Category(ItemCategory::Weapon),
                ItemStats { damage: Some(2) },
            ),
            ItemKind::Map => self.phase = Phase::Map { tiles_left: 2 },
            ItemKind::Boots => {
                self.player.items.remove(board_item.item_id);
                self.player.moves_left += 3;
            }
            ItemKind::Camera => {
                self.player.items.remove(board_item.item_id);
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

    fn deal_damage_around(
        &mut self,
        position: vec2<Coord>,
        source_fraction: Fraction,
        damage: Hp,
        range: Coord,
    ) {
        for entity in &mut self.entities {
            if source_fraction != entity.fraction && distance(entity.position, position) <= range {
                entity.health.change(-damage);
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
