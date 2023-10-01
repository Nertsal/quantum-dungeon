use super::*;

impl Model {
    pub(super) fn resolve_animations(&mut self, delta_time: Time) {
        match &mut self.phase {
            Phase::Passive { start_delay, .. } => {
                // Start animation
                if !start_delay.is_min() {
                    start_delay.change(-delta_time);
                } else if self.animations.is_empty() {
                    // End animation
                    if let Phase::Passive {
                        current_item,
                        end_delay,
                        ..
                    } = &mut self.phase
                    {
                        end_delay.change(-delta_time);
                        if end_delay.is_min() {
                            *current_item += 1;
                            self.resolve_current();
                        }
                    }
                }
            }
            Phase::Active { start_delay, .. } => {
                // Start animation
                if !start_delay.is_min() {
                    start_delay.change(-delta_time);
                } else if self.animations.is_empty() {
                    // End animation
                    if let Phase::Active {
                        fraction,
                        item_id,
                        ref mut end_delay,
                        ..
                    } = self.phase
                    {
                        end_delay.change(-delta_time);
                        if end_delay.is_min() {
                            self.phase = Phase::Player;
                            self.active_effect(fraction, item_id);
                        }
                    }
                }
            }
            _ => (),
        }
    }

    pub fn resolution_phase(&mut self) {
        log::debug!("Resolution phase");
        self.phase = Phase::Passive {
            current_item: 0,
            start_delay: Lifetime::new_max(r32(0.2)),
            end_delay: Lifetime::new_max(r32(0.2)),
        };
        for item in &mut self.items {
            item.temp_stats = item.perm_stats.clone();
        }
        self.resolve_current();
    }

    fn resolve_current(&mut self) {
        if let Phase::Passive { current_item, .. } = self.phase {
            if !self.resolve_item_passive(current_item) {
                // No animation - skip
                while let Phase::Passive { current_item, .. } = &mut self.phase {
                    *current_item += 1;
                    let item = *current_item;
                    if self.resolve_item_passive(item) {
                        // Yes animation
                        break;
                    }
                }
            }
        }

        if let Phase::Passive {
            start_delay,
            end_delay,
            ..
        } = &mut self.phase
        {
            start_delay.set_ratio(R32::ONE);
            end_delay.set_ratio(R32::ONE);
        }
    }

    pub(super) fn active_phase(&mut self, fraction: Fraction, item_id: usize) {
        if self.resolve_item_active(item_id) {
            self.phase = Phase::Active {
                fraction,
                item_id,
                start_delay: Lifetime::new_max(r32(0.2)),
                end_delay: Lifetime::new_max(r32(0.2)),
            };
        } else {
            // Activate immediately
            self.active_effect(fraction, item_id);
        }
    }

    fn active_effect(&mut self, fraction: Fraction, item_id: usize) {
        let Some(item) = self.items.get_mut(item_id) else {
            log::error!("tried activating an invalid item {}", item_id);
            return;
        };

        item.use_time = item.use_time.saturating_sub(1);
        let item = if item.use_time == 0 {
            // TODO: check indices safety
            self.items.swap_remove(item_id)
        } else {
            item.clone()
        };
        self.use_item(fraction, item);
    }

    /// Start item passive resolution animation.
    /// If there is no animation required for the item, false is returned.
    fn resolve_item_passive(&mut self, item_id: usize) -> bool {
        let Some(item) = self.items.get(item_id) else {
            self.day_phase();
            return false;
        };

        match item.kind {
            ItemKind::Sword => false,
            ItemKind::Forge => {
                self.bonus_near_temporary(
                    item.position,
                    1,
                    ItemRef::Category(ItemCategory::Weapon),
                    ItemStats { damage: Some(2) },
                );
                true
            }
            ItemKind::Boots => false,
            ItemKind::Map => false,
        }
    }

    /// Start item active resolution animation.
    /// If there is no animation required for the item, false is returned.
    fn resolve_item_active(&mut self, item_id: usize) -> bool {
        let Some(item) = self.items.get(item_id) else {
            self.day_phase();
            return false;
        };

        match item.kind {
            ItemKind::Sword => {
                let bonus = self.count_items_near(item.position, ItemKind::Sword) as i64;
                let bonus = ItemStats {
                    damage: Some(bonus * 2),
                };
                self.items[item_id].temp_stats = item.temp_stats.combine(&bonus);
                true
            }
            ItemKind::Forge => false,
            ItemKind::Boots => false,
            ItemKind::Map => false,
        }
    }
}
