use super::*;

impl Model {
    pub(super) fn resolve_animations(&mut self, delta_time: Time) {
        match &mut self.phase {
            Phase::LevelFinished { timer, .. } => {
                timer.change(-delta_time);
                if timer.is_min() {
                    self.next_level();
                }
            }
            Phase::PostVision { timer } => {
                timer.change(-delta_time);
                if timer.is_min() {
                    self.select_phase(self.player.extra_items);
                }
            }
            Phase::Night {
                fade_time,
                light_time,
            } => {
                if fade_time.is_above_min() {
                    fade_time.change(-delta_time);
                    if fade_time.is_min() {
                        // Night effects
                        self.resolve_night_effects();
                        self.shift_everything();
                        // TODO: shift entities
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
        let ids: Vec<_> = self.items.iter().map(|(i, _)| i).collect();
        let item_queue = ids
            .into_iter()
            .flat_map(|i| self.resolve_item_passive(i).map(|p| (i, p)))
            .sorted_by_key(|(_, p)| *p) // Sort by priority - last one is processed first
            .map(|(i, _)| i)
            .collect();
        self.phase = Phase::Passive {
            item_queue,
            start_delay: Lifetime::new_max(r32(0.2)),
            end_delay: Lifetime::new_max(r32(0.2)),
        };

        // Clear temp stats
        for (_, item) in &mut self.player.items {
            item.temp_stats = ItemStats::default();
        }
        // Update turn counter
        for (_, item) in &mut self.items {
            item.turns_alive += 1;
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
        if let Phase::Passive {
            item_queue,
            start_delay,
            end_delay,
            ..
        } = &mut self.phase
        {
            if item_queue.last().is_none() {
                self.day_phase();
                return;
            }
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

        let item = &mut self.player.items[board_item.item_id];
        match item.kind {
            ItemKind::Ghost => {
                if self.visible_tiles.contains(&board_item.position) {
                    // Death
                    self.animations.insert(Animation::new(
                        self.config.animation_time,
                        AnimationKind::ItemDeath {
                            item: item_id,
                            pos: board_item.position,
                        },
                    ));
                }
                // true
            }
            ItemKind::CharmingStaff => {
                // Change damage
                let delta = if self.visible_tiles.contains(&board_item.position) {
                    2
                } else {
                    -2
                };
                item.perm_stats.damage =
                    Some((item.perm_stats.damage.unwrap_or_default() + delta).max(0));
                // true
            }
            _ => (), // false
        }
    }

    /// Start item passive resolution animation.
    /// If there is an animation required for the item, its priority is returned.
    fn resolve_item_passive(&mut self, item_id: Id) -> Option<isize> {
        let Some(board_item) = self.items.get(item_id) else {
            return None;
        };

        let mut rng = thread_rng();
        let item = &self.player.items[board_item.item_id];
        match item.kind {
            ItemKind::Forge => Some(10),
            ItemKind::Ghost => Some(-10),
            ItemKind::SoulCrystal => Some(0),
            ItemKind::RadiationCore => Some(0),
            ItemKind::GreedyPot => Some(0),
            ItemKind::SpiritCoin => Some(0),
            ItemKind::Chest => Some(0),
            ItemKind::MagicTreasureBag if board_item.turns_alive >= 5 => Some(0),
            ItemKind::ElectricRod => Some(0),
            ItemKind::MagicWire if rng.gen_bool(0.1) => Some(0),
            ItemKind::Melter if rng.gen_bool(0.2) => Some(0),
            ItemKind::CursedSkull if board_item.position.y == self.grid.bounds().max.y => Some(0),
            ItemKind::Solitude
                if self
                    .items
                    .iter()
                    .filter(|(_, item)| {
                        ItemRef::Category(ItemCategory::Weapon)
                            .check(self.player.items[item.item_id].kind)
                    })
                    .count()
                    == 1 =>
            {
                Some(0)
            }
            _ => None,
        }
    }

    fn passive_effect(&mut self, item_id: Id) {
        let Some(board_item) = self.items.get(item_id) else {
            log::error!("tried passivating an invalid item {:?}", item_id);
            return;
        };

        let mut rng = thread_rng();
        let item = &mut self.player.items[board_item.item_id];
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
                let mut weapons = self
                    .count_items_near(board_item.position, ItemRef::Category(ItemCategory::Weapon));
                while !weapons.is_empty() {
                    // Find a weapon with an active effect
                    let i = rng.gen_range(0..weapons.len());
                    let weapon = weapons.swap_remove(i);
                    if self.resolve_item_active(Fraction::Player, weapon) {
                        break;
                    }
                }
            }
            ItemKind::SoulCrystal => {
                item.perm_stats.damage = Some(item.perm_stats.damage.unwrap_or_default() + 1);
            }
            ItemKind::RadiationCore => {
                let damage = item.current_stats().damage.unwrap_or_default();
                self.deal_damage_around(board_item.position, Fraction::Player, damage, 1, vec![]);
            }
            ItemKind::GreedyPot => {
                let mut bonus_animation = None;
                let mut stats = item.current_stats();
                if rng.gen_bool(0.1) {
                    // Destroy nearby treasure and gain +2 dmg
                    let treasures = self.count_items_near(
                        board_item.position,
                        ItemRef::Category(ItemCategory::Treasure),
                    );
                    if let Some(&treasure_id) = treasures.choose(&mut rng) {
                        let treasure = &self.items[treasure_id];
                        self.animations.insert(Animation::new(
                            self.config.animation_time,
                            AnimationKind::ItemDeath {
                                item: treasure_id,
                                pos: treasure.position,
                            },
                        ));

                        let bonus = ItemStats { damage: Some(2) };
                        stats = stats.combine(&bonus);
                        bonus_animation = Some(self.animations.insert(Animation::new(
                            self.config.animation_time,
                            AnimationKind::Bonus {
                                from: treasure.position,
                                target: item_id,
                                bonus,
                                permanent: true,
                            },
                        )));
                    }
                }

                let board_item = self.items.get(item_id).unwrap();
                let enemies: Vec<Id> = self
                    .entities
                    .iter()
                    .filter(|(_, e)| matches!(e.fraction, Fraction::Enemy))
                    .map(|(i, _)| i)
                    .collect();
                if let Some(&enemy) = enemies.choose(&mut rng) {
                    let animation = Animation::new(
                        self.config.animation_time,
                        AnimationKind::Damage {
                            from: board_item.position,
                            target: enemy,
                            damage: stats.damage.unwrap_or_default(),
                        },
                    )
                    .after(bonus_animation);
                    self.animations.insert(animation);
                }
            }
            ItemKind::SpiritCoin => {
                // Duplicate if near a chest
                if !self
                    .count_items_near(board_item.position, ItemRef::Specific(ItemKind::Chest))
                    .is_empty()
                {
                    self.animations.insert(Animation::new(
                        self.config.animation_time,
                        AnimationKind::Dupe {
                            kind: ItemKind::SpiritCoin,
                        },
                    ));
                }

                if rng.gen_bool(0.2) {
                    let mut damage_anim = None;

                    // Deal 5 damage
                    let enemies: Vec<Id> = self
                        .entities
                        .iter()
                        .filter(|(_, e)| matches!(e.fraction, Fraction::Enemy))
                        .map(|(i, _)| i)
                        .collect();
                    if let Some(&enemy) = enemies.choose(&mut rng) {
                        damage_anim = Some(self.animations.insert(Animation::new(
                            self.config.animation_time,
                            AnimationKind::Damage {
                                from: board_item.position,
                                target: enemy,
                                damage: 5,
                            },
                        )));
                    }

                    // Destroy self
                    self.animations.insert(
                        Animation::new(
                            self.config.animation_time,
                            AnimationKind::ItemDeath {
                                item: item_id,
                                pos: board_item.position,
                            },
                        )
                        .after(damage_anim),
                    );
                }
            }
            ItemKind::Chest => {
                let chests: Vec<Id> = self
                    .items
                    .iter()
                    .filter(|(_, item)| self.player.items[item.item_id].kind == ItemKind::Chest)
                    .map(|(i, _)| i)
                    .collect();
                if chests.len() >= 3 {
                    // Destroy 3 chests, gain 1 item
                    for i in chests.into_iter().take(3) {
                        let board_item = self.items.remove(i).unwrap();
                        self.player.items.remove(board_item.item_id);
                    }
                    self.player.extra_items += 1;
                }
            }
            ItemKind::MagicTreasureBag => {
                // Turn into a random treasure
                let board_item = self.items.remove(item_id).unwrap();
                self.player.items.remove(board_item.item_id);

                let options = ItemKind::all()
                    .into_iter()
                    .filter(|kind| ItemRef::Category(ItemCategory::Treasure).check(*kind));
                if let Some(new_item) = options.choose(&mut rng) {
                    let item_id = self.player.items.insert(new_item.instantiate());
                    let item = &mut self.player.items[item_id];
                    let on_board = self.items.insert(BoardItem {
                        position: board_item.position,
                        item_id,
                        turns_alive: 0,
                    });
                    item.on_board = Some(on_board);
                }
            }
            ItemKind::ElectricRod => {
                let damage = item.current_stats().damage.unwrap_or_default();
                let position = board_item.position;

                let item_ref = ItemRef::Category(ItemCategory::Tech);
                let mut bonus = 0;
                let mut animations = Vec::new();
                for (_, board_item) in &self.items {
                    let d = position - board_item.position;
                    let d = d.x.abs() + d.y.abs();
                    let item = &self.player.items[board_item.item_id];
                    if item_ref.check(item.kind) && d > 0 && d <= 1 {
                        animations.push(self.animations.insert(Animation::new(
                            self.config.animation_time,
                            AnimationKind::Bonus {
                                from: board_item.position,
                                target: item_id,
                                bonus: ItemStats { damage: Some(2) },
                                permanent: false,
                            },
                        )));
                        bonus += 1;
                    }
                }
                let damage = damage + bonus as i64 * 2;
                self.deal_damage_around(
                    board_item.position,
                    Fraction::Player,
                    damage,
                    1,
                    animations,
                );
            }
            ItemKind::MagicWire => {
                // Duplicate
                self.animations.insert(Animation::new(
                    self.config.animation_time,
                    AnimationKind::Dupe {
                        kind: ItemKind::MagicWire,
                    },
                ));
            }
            ItemKind::Melter => {
                // Destroy nearby tech item
                let from_position = board_item.position;
                let tech = self
                    .count_items_near(board_item.position, ItemRef::Category(ItemCategory::Tech));
                if let Some(&item) = tech.choose(&mut rng) {
                    let item = self.items.remove(item).unwrap();
                    self.player.items.remove(item.item_id);

                    // +1 dmg permanent to all weapons
                    for (target, item) in &self.items {
                        let item = &mut self.player.items[item.item_id];
                        if ItemRef::Category(ItemCategory::Weapon).check(item.kind) {
                            self.animations.insert(Animation::new(
                                self.config.animation_time,
                                AnimationKind::Bonus {
                                    from: from_position,
                                    target,
                                    bonus: ItemStats { damage: Some(1) },
                                    permanent: true,
                                },
                            ));
                        }
                    }
                }
            }
            ItemKind::CursedSkull => {
                // Turn into a king's skull
                let board_item = self.items.remove(item_id).unwrap();
                self.player.items.remove(board_item.item_id);

                let item_id = self.player.items.insert(ItemKind::KingSkull.instantiate());
                let item = &mut self.player.items[item_id];
                let on_board = self.items.insert(BoardItem {
                    position: board_item.position,
                    item_id,
                    turns_alive: 0,
                });
                item.on_board = Some(on_board);
            }
            ItemKind::Solitude => {
                self.animations.insert(Animation::new(
                    self.config.animation_time,
                    AnimationKind::Bonus {
                        from: board_item.position,
                        target: item_id,
                        bonus: ItemStats { damage: Some(2) },
                        permanent: true,
                    },
                ));
            }
            _ => {}
        }
    }

    /// Start item active resolution animation.
    /// Returns false, if the item does not have an active effect.
    pub(super) fn resolve_item_active(&mut self, fraction: Fraction, item_id: Id) -> bool {
        let Some(board_item) = self.items.get(item_id) else {
            return false;
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
            ItemKind::Boots => Some(false),
            ItemKind::Map => Some(false),
            ItemKind::Camera => {
                let spooky = self
                    .count_items_near(board_item.position, ItemRef::Category(ItemCategory::Spooky));
                let mut rng = thread_rng();
                match spooky.choose(&mut rng) {
                    None => None, // Do nothing
                    Some(&item) => {
                        self.animations.insert(Animation::new(
                            self.config.animation_time,
                            AnimationKind::Dupe {
                                kind: self.player.items[self.items[item].item_id].kind,
                            },
                        ));
                        Some(true)
                    }
                }
            }
            ItemKind::FireScroll => Some(true),
            ItemKind::SoulCrystal => Some(true),
            ItemKind::Phantom => Some(true),
            ItemKind::KingSkull => Some(true),
            ItemKind::GoldenLantern => Some(true),
            ItemKind::WarpPortal => Some(true),
            ItemKind::Solitude => Some(true),
            _ => None,
        };

        match resolution {
            Some(true) => {
                // Animation
                self.animations.insert(Animation::new(
                    self.config.effect_padding_time,
                    AnimationKind::UseActive { fraction, item_id },
                ));
                true
            }
            Some(false) => {
                // Activate immediately
                self.active_effect(fraction, item_id);
                true
            }
            None => {
                // Do nothing
                false
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
        let item = &mut self.player.items[board_item.item_id];
        match item.kind {
            ItemKind::Sword => {
                let damage = item.current_stats().damage.unwrap_or_default();
                let range = 1;
                self.deal_damage_around(board_item.position, fraction, damage, range, vec![]);
            }
            ItemKind::Map => self.phase = Phase::Map { tiles_left: 2 },
            ItemKind::Boots => {
                self.player.items.remove(board_item.item_id);
                self.player.moves_left += 3;
            }
            ItemKind::Camera => {
                self.player.items.remove(board_item.item_id);
            }
            ItemKind::FireScroll => {
                let enemies: Vec<Id> = self
                    .entities
                    .iter()
                    .filter(|(_, e)| matches!(e.fraction, Fraction::Enemy))
                    .map(|(i, _)| i)
                    .collect();
                if let Some(&enemy) = enemies.choose(&mut thread_rng()) {
                    self.animations.insert(Animation::new(
                        self.config.animation_time,
                        AnimationKind::Damage {
                            from: board_item.position,
                            target: enemy,
                            damage: item.current_stats().damage.unwrap_or_default(),
                        },
                    ));
                    // TODO: after the animation
                    self.player.items.remove(board_item.item_id);
                }
            }
            ItemKind::SoulCrystal => {
                let enemies: Vec<Id> = self
                    .entities
                    .iter()
                    .filter(|(_, e)| matches!(e.fraction, Fraction::Enemy))
                    .map(|(i, _)| i)
                    .collect();
                if let Some(&enemy) = enemies.choose(&mut thread_rng()) {
                    self.animations.insert(Animation::new(
                        self.config.animation_time,
                        AnimationKind::Damage {
                            from: board_item.position,
                            target: enemy,
                            damage: item.current_stats().damage.unwrap_or_default(),
                        },
                    ));
                    // TODO: after the animation
                    self.player.items.remove(board_item.item_id);
                }
            }
            ItemKind::Phantom => {
                let damage = item.current_stats().damage.unwrap_or_default();
                item.perm_stats.damage = Some(item.perm_stats.damage.unwrap_or_default() + 1);
                self.deal_damage_around(board_item.position, Fraction::Player, damage, 1, vec![]);
            }
            ItemKind::KingSkull => {
                // deal damage to all enemies
                let damage = item.current_stats().damage.unwrap_or_default();
                for (target, entity) in &self.entities {
                    if let Fraction::Enemy = entity.fraction {
                        self.animations.insert(Animation::new(
                            self.config.animation_time,
                            AnimationKind::Damage {
                                from: board_item.position,
                                target,
                                damage,
                            },
                        ));
                    }
                }
            }
            ItemKind::GoldenLantern => {
                // Destroy and light up for 3 turns
                self.player.items.remove(board_item.item_id);
                self.grid.light_up(board_item.position, 1, 3);
            }
            ItemKind::CharmingStaff => {
                let damage = item.current_stats().damage.unwrap_or_default();
                self.deal_damage_around(board_item.position, Fraction::Player, damage, 1, vec![]);
            }
            ItemKind::WarpPortal => {
                if self.items.iter().any(|(_, i)| {
                    ItemRef::Category(ItemCategory::Magic).check(self.player.items[i.item_id].kind)
                }) {
                    self.phase = Phase::Portal
                }
            }
            ItemKind::Solitude => {
                let damage = item.current_stats().damage.unwrap_or_default();
                self.deal_damage_around(board_item.position, Fraction::Player, damage, 1, vec![]);
            }
            _ => {}
        }
    }
}
