use super::*;

impl Model {
    /// Collect an item at the given position.
    pub(super) fn collect_item_at(&mut self, fraction: Fraction, position: vec2<Coord>) {
        for i in (0..self.items.len()).rev() {
            let item = &mut self.items[i];
            if item.position == position {
                self.active_phase(fraction, i);
                return;
            }
        }
    }

    pub(super) fn use_item(&mut self, fraction: Fraction, item: Item) {
        log::debug!("Use item by fraction {:?}: {:?}", fraction, item);
        match item.kind {
            ItemKind::Sword => {
                let damage = item.temp_stats.damage.unwrap_or_default();
                let range = 1;
                self.deal_damage_around(item.position, fraction, damage, range);
            }
            ItemKind::Forge => self.bonus_near_temporary(
                item.position,
                1,
                ItemRef::Category(ItemCategory::Weapon),
                ItemStats { damage: Some(2) },
            ),
            ItemKind::Map => self.phase = Phase::Map,
            ItemKind::Boots => self.player.moves_left += 3,
        }
    }

    /// Give a temporary bonus to nearby items.
    fn bonus_near_temporary(
        &mut self,
        position: vec2<Coord>,
        range: Coord,
        item_ref: ItemRef,
        bonus: ItemStats,
    ) {
        for item in &mut self.items {
            if distance(item.position, position) <= range && item_ref.check(item.kind) {
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

    pub(super) fn count_items_near(&self, position: vec2<Coord>, kind: ItemKind) -> usize {
        self.items
            .iter()
            .filter(|item| {
                let d = distance(position, item.position);
                item.kind == kind && d > 0 && d <= 1
            })
            .count()
    }
}
