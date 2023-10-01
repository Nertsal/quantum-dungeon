use super::*;

impl Model {
    pub(super) fn resolve_animations(&mut self, delta_time: Time) {
        match &mut self.phase {
            Phase::Night {
                fade_time,
                light_time,
            } => {
                if fade_time.is_above_min() {
                    fade_time.change(-delta_time);
                    if fade_time.is_min() {
                        // Night effects
                        self.resolve_night_effects();
                        self.shift_items();
                        self.spawn_items();
                    }
                } else if self.animations.is_empty() {
                    light_time.change(-delta_time);
                    if light_time.is_min() {
                        self.resolution_phase();
                    }
                }
            }
            Phase::Passive {
                item_queue,
                start_delay,
                ..
            } => {
                // Start animation
                if start_delay.is_above_min() {
                    start_delay.change(-delta_time);
                    if start_delay.is_min() {
                        // Apply effect
                        if let Some(&item_id) = item_queue.last() {
                            self.passive_effect(item_id);
                        }
                    }
                } else if self.animations.is_empty() {
                    // End animation
                    if let Phase::Passive {
                        item_queue,
                        end_delay,
                        ..
                    } = &mut self.phase
                    {
                        end_delay.change(-delta_time);
                        if end_delay.is_min() {
                            item_queue.pop();
                            self.resolve_current();
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
            item_queue: self.items.iter().map(|(i, _)| i).collect(),
            start_delay: Lifetime::new_max(r32(0.2)),
            end_delay: Lifetime::new_max(r32(0.2)),
        };
        for (_, item) in &mut self.player.items {
            item.temp_stats = item.perm_stats.clone();
        }
        self.resolve_current();
    }

    fn resolve_night_effects(&mut self) {
        // TODO: sequential
        let ids: Vec<_> = self.items.iter().map(|(i, _)| i).collect();
        for id in ids {
            self.resolve_item_night(id);
        }
    }

    fn resolve_current(&mut self) {
        if let Phase::Passive { item_queue, .. } = &self.phase {
            let Some(&current_item) = item_queue.last() else {
                self.day_phase();
                return;
            };
            if !self.resolve_item_passive(current_item) {
                // No animation - skip
                while let Phase::Passive { item_queue, .. } = &mut self.phase {
                    item_queue.pop();
                    let Some(&item) = item_queue.last() else {
                        self.day_phase();
                        return;
                    };
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

    /// Start item night resolution animation.
    // TODO: return bool like resolve_item_passive
    fn resolve_item_night(&mut self, item_id: Id) {
        let Some(board_item) = self.items.get(item_id) else {
            self.day_phase();
            return; // false
        };

        let item = &self.player.items[board_item.item_id];
        if let ItemKind::Ghost = item.kind {
            if self.visible_tiles.contains(&board_item.position) {
                // Death
                self.animations.push(Animation {
                    time: Lifetime::new_max(r32(0.5)),
                    kind: AnimationKind::Death { item: item_id },
                });
            }
            // true
        }
    }

    /// Start item passive resolution animation.
    /// If there is no animation required for the item, false is returned.
    fn resolve_item_passive(&mut self, item_id: Id) -> bool {
        let Some(board_item) = self.items.get(item_id) else {
            self.day_phase();
            return false;
        };

        let item = &self.player.items[board_item.item_id];
        #[allow(clippy::match_like_matches_macro)]
        match item.kind {
            ItemKind::Forge => true,
            ItemKind::Ghost => true,
            _ => false,
        }
    }

    fn passive_effect(&mut self, item_id: Id) {
        let Some(board_item) = self.items.get(item_id) else {
            log::error!("tried passivating an invalid item {:?}", item_id);
            return;
        };

        let item = &self.player.items[board_item.item_id];
        match item.kind {
            ItemKind::Forge => {
                self.bonus_near_temporary(
                    board_item.position,
                    1,
                    ItemRef::Category(ItemCategory::Weapon),
                    ItemStats { damage: Some(2) },
                );
            }
            ItemKind::Ghost => {
                let weapons = self
                    .count_items_near(board_item.position, ItemRef::Category(ItemCategory::Weapon));
                if let Some(&weapon) = weapons.choose(&mut thread_rng()) {
                    self.resolve_item_active(Fraction::Player, weapon);
                }
            }
            _ => {}
        }
    }

    /// Start item active resolution animation.
    pub(super) fn resolve_item_active(&mut self, fraction: Fraction, item_id: Id) {
        let Some(board_item) = self.items.get(item_id) else {
            return;
        };

        let item = &self.player.items[board_item.item_id];
        let resolution = match item.kind {
            ItemKind::Sword => {
                let bonus = self
                    .count_items_near(board_item.position, ItemRef::Specific(ItemKind::Sword))
                    .len() as i64;
                let bonus = ItemStats {
                    damage: Some(bonus * 2),
                };
                let item = &mut self.player.items[board_item.item_id];
                item.temp_stats = item.temp_stats.combine(&bonus);
                Some(true)
            }
            ItemKind::Forge => None,
            ItemKind::Boots => Some(false),
            ItemKind::Map => Some(false),
            ItemKind::Camera => {
                // TODO: animation
                let spooky = self
                    .count_items_near(board_item.position, ItemRef::Category(ItemCategory::Spooky));
                let mut rng = thread_rng();
                match spooky.choose(&mut rng) {
                    None => None, // Do nothing
                    Some(&item) => {
                        self.animations.push(Animation {
                            time: Lifetime::new_max(r32(0.5)),
                            kind: AnimationKind::CameraDupe { item },
                        });
                        Some(true)
                    }
                }
            }
            ItemKind::Ghost => None,
        };

        match resolution {
            Some(true) => {
                // Animation
                self.animations.push(Animation {
                    time: Lifetime::new_max(r32(0.2)),
                    kind: AnimationKind::UseActive { fraction, item_id },
                });
            }
            Some(false) => {
                // Activate immediately
                self.active_effect(fraction, item_id);
            }
            None => {
                // Do nothing
            }
        }
    }

    pub(super) fn active_effect(&mut self, fraction: Fraction, item_id: Id) {
        let Some(item) = self.items.remove(item_id) else {
            log::error!("tried activating an invalid item {:?}", item_id);
            return;
        };
        self.use_item(fraction, item);
    }

    fn use_item(&mut self, fraction: Fraction, board_item: BoardItem) {
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
            ItemKind::Ghost => {}
        }
    }
}
