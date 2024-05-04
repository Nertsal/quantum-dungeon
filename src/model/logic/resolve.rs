use super::*;

impl Model {
    pub(super) fn resolve_animations(&mut self, delta_time: Time) {
        for item in &mut self.resolved_items {
            item.time.change(-delta_time);
        }
        self.resolved_items.retain(|item| item.time.is_above_min());

        for item in &mut self.resolving_items {
            item.animations.retain(|id| self.animations.contains(*id));
            if item.animations.is_empty() {
                self.resolved_items.insert(ItemResolved {
                    board_item: item.board_item,
                    time: Lifetime::new_max(self.config.animation_time),
                });
            }
        }
        self.resolving_items
            .retain(|item| !item.animations.is_empty());

        let wait_for_effects = self.wait_for_effects() && self.resolved_items.is_empty();
        match &mut self.phase {
            Phase::LevelStarting { timer } => {
                timer.change(-delta_time);
                if timer.is_min() {
                    self.night_phase(true);
                }
            }
            Phase::LevelFinished { timer, .. } => {
                timer.change(-delta_time);
                if timer.is_min() {
                    self.next_level();
                }
            }
            Phase::PostVision { timer } => {
                timer.change(-delta_time);
                if timer.is_min() {
                    let mut state = self.state.borrow_mut();
                    state.player.refreshes = 2;
                    let extra = state.player.extra_items;
                    drop(state);
                    self.select_phase(extra);
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
                        self.shift_everything();
                        self.spawn_items();
                        self.resolve_trigger(Trigger::Night, None);
                    }
                } else if self.animations.is_empty() {
                    light_time.change(-delta_time);
                    if light_time.is_min() {
                        self.resolution_phase();
                    }
                }
            }
            Phase::Passive { start_delay, .. } => {
                if start_delay.is_above_min() {
                    // Start animation
                    start_delay.change(-delta_time);
                    if start_delay.is_min() {
                        self.resolve_trigger(Trigger::Day, None);
                    }
                } else if self.wait_for_effects() {
                    // End animation
                    if let Phase::Passive { end_delay, .. } = &mut self.phase {
                        end_delay.change(-delta_time);
                        if end_delay.is_min() {
                            self.day_phase();
                        }
                    }
                }
            }
            Phase::Active {
                entity_id,
                position,
            } if wait_for_effects => {
                let entity_id = *entity_id;
                let target_pos = *position;

                self.animations.insert(Animation::new(
                    self.config.animation_time,
                    AnimationKind::MoveEntity {
                        entity_id,
                        target_pos,
                    },
                ));

                let state = self.state.borrow();
                let from_pos = target_pos;
                let target_pos = state.entities[entity_id].position;
                if let Some((entity_id, _)) =
                    state.entities.iter().find(|(_, e)| e.position == from_pos)
                {
                    self.animations.insert(Animation::new(
                        self.config.animation_time,
                        AnimationKind::MoveEntity {
                            entity_id,
                            target_pos,
                        },
                    ));
                }
                if let Some((item_id, _)) = state.items.iter().find(|(_, i)| i.position == from_pos)
                {
                    self.animations.insert(Animation::new(
                        self.config.animation_time,
                        AnimationKind::MoveItem {
                            item_id,
                            target_pos,
                        },
                    ));
                }
                drop(state);

                self.player_phase();
            }
            _ => (),
        }
    }

    pub fn resolution_phase(&mut self) {
        log::debug!("Day resolution phase");
        self.phase = Phase::Passive {
            start_delay: Lifetime::new_max(r32(0.2)),
            end_delay: Lifetime::new_max(r32(0.2)),
        };

        // What is this trick KEKW
        let mut state = self.state.borrow_mut();
        let state = &mut *state;

        // Clear temp stats
        for (_, item) in &mut state.player.items {
            item.temp_stats = ItemStats::default();
        }
        // Update turn counter
        for (_, item) in &mut state.items {
            item.used = false;
            state.player.items[item.item_id].turns_on_board += 1;
        }
    }

    pub(super) fn resolve_trigger(&mut self, trigger: Trigger, specific: Option<Id>) {
        let mut ids: Vec<_> = if let Some(id) = specific {
            vec![id]
        } else {
            self.state.borrow().items.iter().map(|(i, _)| i).collect()
        };

        // Sort by item position
        ids.sort_by_key(|&id| {
            let pos = self.state.borrow().items[id].position;
            // Left -> Right
            // Top -> Bottom
            (pos.x, -pos.y)
        });

        let effects = ids
            .into_iter()
            .flat_map(|id| {
                self.resolve_item(id, trigger)
                    .into_iter()
                    .map(move |effect| QueuedEffect {
                        trigger,
                        proc_item: id,
                        effect,
                    })
            })
            .sorted_by_key(|eff| -eff.effect.priority()) // Sort by priority
            .collect();

        self.effect_queue_stack.push(effects);
    }

    /// Resolve the item's response to the trigger.
    fn resolve_item(&mut self, item_id: Id, trigger: Trigger) -> Vec<Effect> {
        log::debug!("Resolving item {:?} trigger {:?}", item_id, trigger);

        let state = self.state.borrow();
        let Some(board_item) = state.items.get(item_id) else {
            log::debug!("Item {:?} not found on the board", item_id);
            return vec![];
        };

        if let Trigger::Active = trigger {
            if board_item.used {
                log::debug!("Item {:?} has already been activated", item_id);
                return vec![];
            }
        }

        let item = &state.player.items[board_item.item_id];

        // Execute
        // NOTE: requires immutable access to [ModelState]
        let item_state = match self
            .engine
            .item_trigger(item, board_item, trigger.method_name())
        {
            Ok(state) => state,
            Err(err) => {
                log::error!("Trigger handler failed: {:?}", err);
                return vec![];
            }
        };

        // Update item state
        drop(state);
        // What is this trick KEKW
        let mut state = self.state.borrow_mut();
        let state = &mut *state;

        let mut effects = std::mem::take(&mut *self.side_effects.borrow_mut());
        let board_item = state.items.get(item_id).unwrap();
        if !effects.is_empty()
            && matches!(trigger, Trigger::Active)
            && !matches!(effects.last(), Some(Effect::Destroy { .. }))
        {
            effects.push(Effect::SetUsed { item_id });
        }
        state.player.items[board_item.item_id].state = item_state;

        log::debug!("Item {:?} resolved: {:?}", item_id, effects);
        effects
    }

    // Night effect
    // match item.kind {
    //     ItemKind::Ghost => {
    //         if self.visible_tiles.contains(&board_item.position) {
    //             // Death
    //             self.animations.insert(Animation::new(
    //                 self.config.animation_time,
    //                 AnimationKind::ItemDeath {
    //                     item: item_id,
    //                     pos: board_item.position,
    //                 },
    //             ));
    //         }
    //         // true
    //     }
    //     ItemKind::CharmingStaff => {
    //         // Change damage
    //         let delta = if self.visible_tiles.contains(&board_item.position) {
    //             2
    //         } else {
    //             -2
    //         };
    //         item.perm_stats.damage =
    //             Some((item.perm_stats.damage.unwrap_or_default() + delta).max(0));
    //         // true
    //     }
    //     _ => (), // false
    // }

    // Day resolution
    // match item.kind {
    //     ItemKind::Forge => Some(10),
    //     ItemKind::Ghost => Some(-10),
    //     ItemKind::SoulCrystal => Some(0),
    //     ItemKind::RadiationCore => Some(0),
    //     ItemKind::GreedyPot => Some(0),
    //     ItemKind::SpiritCoin => Some(0),
    //     ItemKind::Chest => Some(0),
    //     ItemKind::MagicTreasureBag if board_item.turns_on_board >= 3 => Some(0),
    //     ItemKind::MagicWire if rng.gen_bool(0.1) => Some(0),
    //     ItemKind::Melter if rng.gen_bool(0.2) => Some(0),
    //     ItemKind::CursedSkull if board_item.position.y == self.grid.bounds().max.y => Some(0),
    //     ItemKind::Solitude
    //         if self
    //             .items
    //             .iter()
    //             .filter(|(_, item)| {
    //                 ItemRef::Category(ItemCategory::Weapon)
    //                     .check(self.player.items[item.item_id].kind)
    //             })
    //             .count()
    //             == 1 =>
    //     {
    //         Some(0)
    //     }
    //     _ => None,
    // }

    // Day effect
    // match item.kind {
    //     ItemKind::Forge => {
    //         self.bonus_near_temporary(
    //             board_item.position,
    //             1,
    //             ItemRef::Category(ItemCategory::Weapon),
    //             ItemStats { damage: Some(2) },
    //         );
    //     }
    //     ItemKind::Ghost => {
    //         let mut weapons = self
    //             .count_items_near(board_item.position, ItemRef::Category(ItemCategory::Weapon));
    //         while !weapons.is_empty() {
    //             // Find a weapon with an active effect
    //             let i = rng.gen_range(0..weapons.len());
    //             let weapon = weapons.swap_remove(i);
    //             if self.resolve_item_active(Fraction::Player, weapon) {
    //                 break;
    //             }
    //         }
    //     }
    //     ItemKind::SoulCrystal => {
    //         item.perm_stats.damage = Some(item.perm_stats.damage.unwrap_or_default() + 1);
    //     }
    //     ItemKind::RadiationCore => {
    //         let damage = item.current_stats().damage.unwrap_or_default();
    //         self.deal_damage_around(board_item.position, damage, 1, vec![]);
    //     }
    //     ItemKind::GreedyPot => {
    //         let mut bonus_animation = None;
    //         let mut stats = item.current_stats();
    //         if rng.gen_bool(0.1) {
    //             // Destroy nearby treasure and gain +2 dmg
    //             let treasures = self.count_items_near(
    //                 board_item.position,
    //                 ItemRef::Category(ItemCategory::Treasure),
    //             );
    //             if let Some(&treasure_id) = treasures.choose(&mut rng) {
    //                 let treasure = &self.items[treasure_id];
    //                 self.animations.insert(Animation::new(
    //                     self.config.animation_time,
    //                     AnimationKind::ItemDeath {
    //                         item: treasure_id,
    //                         pos: treasure.position,
    //                     },
    //                 ));

    //                 let bonus = ItemStats { damage: Some(2) };
    //                 stats = stats.combine(&bonus);
    //                 bonus_animation = Some(self.animations.insert(Animation::new(
    //                     self.config.animation_time,
    //                     AnimationKind::Bonus {
    //                         from: treasure.position,
    //                         target: item_id,
    //                         bonus,
    //                         permanent: true,
    //                     },
    //                 )));
    //             }
    //         }

    //         let board_item = self.items.get(item_id).unwrap();
    //         self.deal_damage_random(
    //             board_item.position,
    //             stats.damage.unwrap_or_default(),
    //             bonus_animation.into_iter().collect(),
    //         );
    //     }
    //     ItemKind::SpiritCoin => {
    //         // Duplicate if near a chest
    //         if !self
    //             .count_items_near(board_item.position, ItemRef::Specific(ItemKind::Chest))
    //             .is_empty()
    //         {
    //             self.animations.insert(Animation::new(
    //                 self.config.animation_time,
    //                 AnimationKind::Dupe {
    //                     kind: ItemKind::SpiritCoin,
    //                 },
    //             ));
    //         }

    //         if rng.gen_bool(0.2) {
    //             let mut damage_anim = None;

    //             // Deal 5 damage
    //             let position = board_item.position;
    //             self.deal_damage_random(position, 5, vec![]);
    //             let enemies: Vec<Id> = self
    //                 .entities
    //                 .iter()
    //                 .filter(|(_, e)| matches!(e.fraction, Fraction::Enemy))
    //                 .map(|(i, _)| i)
    //                 .collect();
    //             if let Some(&enemy) = enemies.choose(&mut rng) {
    //                 damage_anim = Some(self.animations.insert(Animation::new(
    //                     self.config.animation_time,
    //                     AnimationKind::Damage {
    //                         from: position,
    //                         target: enemy,
    //                         damage: 5,
    //                     },
    //                 )));
    //             }

    //             // Destroy self
    //             self.animations.insert(
    //                 Animation::new(
    //                     self.config.animation_time,
    //                     AnimationKind::ItemDeath {
    //                         item: item_id,
    //                         pos: position,
    //                     },
    //                 )
    //                 .after(damage_anim),
    //             );
    //         }
    //     }
    //     ItemKind::Chest => {
    //         let chests: Vec<Id> = self
    //             .items
    //             .iter()
    //             .filter(|(_, item)| self.player.items[item.item_id].kind == ItemKind::Chest)
    //             .map(|(i, _)| i)
    //             .collect();
    //         if chests.len() >= 3 {
    //             // Destroy 3 chests, gain 1 item
    //             for i in chests.into_iter().take(3) {
    //                 let board_item = self.items.remove(i).unwrap();
    //                 self.player.items.remove(board_item.item_id);
    //             }
    //             self.player.extra_items += 1;
    //         }
    //     }
    //     ItemKind::MagicTreasureBag => {
    //         // Turn into a random treasure
    //         let board_item = self.items.remove(item_id).unwrap();
    //         self.player.items.remove(board_item.item_id);

    //         let options = ItemKind::all()
    //             .into_iter()
    //             .filter(|kind| ItemRef::Category(ItemCategory::Treasure).check(*kind));
    //         if let Some(new_item) = options.choose(&mut rng) {
    //             let item_id = self.player.items.insert(new_item.instantiate());
    //             let item = &mut self.player.items[item_id];
    //             let on_board = self.items.insert(BoardItem {
    //                 position: board_item.position,
    //                 item_id,
    //                 turns_on_board: 0,
    //                 used: false,
    //             });
    //             item.on_board = Some(on_board);
    //         }
    //     }
    //     ItemKind::MagicWire => {
    //         // Duplicate
    //         self.animations.insert(Animation::new(
    //             self.config.animation_time,
    //             AnimationKind::Dupe {
    //                 kind: ItemKind::MagicWire,
    //             },
    //         ));
    //     }
    //     ItemKind::Melter => {
    //         // Destroy nearby tech item
    //         let from_position = board_item.position;
    //         let tech = self
    //             .count_items_near(board_item.position, ItemRef::Category(ItemCategory::Tech));
    //         if let Some(&item) = tech.choose(&mut rng) {
    //             let item = self.items.remove(item).unwrap();
    //             self.player.items.remove(item.item_id);

    //             // +1 dmg permanent to all weapons
    //             for (target, item) in &self.items {
    //                 let item = &mut self.player.items[item.item_id];
    //                 if ItemRef::Category(ItemCategory::Weapon).check(item.kind) {
    //                     self.animations.insert(Animation::new(
    //                         self.config.animation_time,
    //                         AnimationKind::Bonus {
    //                             from: from_position,
    //                             target,
    //                             bonus: ItemStats { damage: Some(1) },
    //                             permanent: true,
    //                         },
    //                     ));
    //                 }
    //             }
    //         }
    //     }
    //     ItemKind::CursedSkull => {
    //         // Turn into a king's skull
    //         let board_item = self.items.remove(item_id).unwrap();
    //         self.player.items.remove(board_item.item_id);

    //         let item_id = self.player.items.insert(ItemKind::KingSkull.instantiate());
    //         let item = &mut self.player.items[item_id];
    //         let on_board = self.items.insert(BoardItem {
    //             position: board_item.position,
    //             item_id,
    //             turns_on_board: 0,
    //             used: false,
    //         });
    //         item.on_board = Some(on_board);
    //     }
    //     ItemKind::Solitude => {
    //         self.animations.insert(Animation::new(
    //             self.config.animation_time,
    //             AnimationKind::Bonus {
    //                 from: board_item.position,
    //                 target: item_id,
    //                 bonus: ItemStats { damage: Some(2) },
    //                 permanent: true,
    //             },
    //         ));
    //     }
    //     _ => {}
    // }

    // Active resolution
    // let resolution = match item.kind {
    //     ItemKind::Sword => {
    //         // TODO: animation
    //         let bonus = self
    //             .count_items_near(board_item.position, ItemRef::Specific(ItemKind::Sword))
    //             .len() as i64;
    //         let bonus = ItemStats {
    //             damage: Some(bonus * 2),
    //         };
    //         let item = &mut self.player.items[board_item.item_id];
    //         item.temp_stats = item.temp_stats.combine(&bonus);
    //         Some(true)
    //     }
    //     ItemKind::Boots => Some(false),
    //     ItemKind::Map => Some(false),
    //     ItemKind::Camera => {
    //         let spooky = self
    //             .count_items_near(board_item.position, ItemRef::Category(ItemCategory::Spooky));
    //         let mut rng = thread_rng();
    //         match spooky.choose(&mut rng) {
    //             None => None, // Do nothing
    //             Some(&item) => {
    //                 self.animations.insert(Animation::new(
    //                     self.config.animation_time,
    //                     AnimationKind::Dupe {
    //                         kind: self.player.items[self.items[item].item_id].kind,
    //                     },
    //                 ));
    //                 Some(true)
    //             }
    //         }
    //     }
    //     ItemKind::FireScroll => Some(true),
    //     ItemKind::SoulCrystal => Some(true),
    //     ItemKind::Phantom => Some(true),
    //     ItemKind::KingSkull => Some(true),
    //     ItemKind::GoldenLantern => Some(true),
    //     ItemKind::CharmingStaff => Some(true),
    //     ItemKind::WarpPortal => Some(true),
    //     ItemKind::Solitude => Some(true),
    //     ItemKind::ElectricRod => {
    //         let position = board_item.position;
    //         let item_ref = ItemRef::Category(ItemCategory::Tech);
    //         let mut animations = Vec::new();
    //         for (_, board_item) in &self.items {
    //             let d = position - board_item.position;
    //             let d = d.x.abs() + d.y.abs();
    //             let item = &self.player.items[board_item.item_id];
    //             if item_ref.check(item.kind) && d > 0 && d <= 1 {
    //                 animations.push(self.animations.insert(Animation::new(
    //                     self.config.animation_time,
    //                     AnimationKind::Bonus {
    //                         from: board_item.position,
    //                         target: item_id,
    //                         bonus: ItemStats { damage: Some(2) },
    //                         permanent: false,
    //                     },
    //                 )));
    //             }
    //         }
    //         Some(true)
    //     }
    //     _ => None,
    // };

    // match resolution {
    //     Some(true) => {
    //         // Animation
    //         self.animations.insert(Animation::new(
    //             self.config.effect_padding_time,
    //             AnimationKind::UseActive { fraction, item_id },
    //         ));
    //         true
    //     }
    //     Some(false) => {
    //         // Activate immediately
    //         self.active_effect(fraction, item_id);
    //         true
    //     }
    //     None => {
    //         // Do nothing
    //         false
    //     }
    // }

    // Active effect
    // match item.kind {
    //     ItemKind::Sword => {
    //         let damage = item.current_stats().damage.unwrap_or_default();
    //         let position = board_item.position;
    //         self.deal_damage_nearest(position, damage, vec![]);
    //     }
    //     ItemKind::Map => {
    //         if !self.grid.is_max() {
    //             self.phase = Phase::Map { tiles_left: 1 };
    //         }
    //     }
    //     ItemKind::Boots => {
    //         self.player.moves_left += 3;
    //         self.animations.insert(Animation::new(
    //             self.config.animation_time,
    //             AnimationKind::ItemDeath {
    //                 item: item_id,
    //                 pos: board_item.position,
    //             },
    //         ));
    //     }
    //     ItemKind::Camera => {
    //         self.animations.insert(Animation::new(
    //             self.config.animation_time,
    //             AnimationKind::ItemDeath {
    //                 item: item_id,
    //                 pos: board_item.position,
    //             },
    //         ));
    //     }
    //     ItemKind::FireScroll => {
    //         let enemies: Vec<Id> = self
    //             .entities
    //             .iter()
    //             .filter(|(_, e)| matches!(e.fraction, Fraction::Enemy))
    //             .map(|(i, _)| i)
    //             .collect();
    //         if let Some(&enemy) = enemies.choose(&mut thread_rng()) {
    //             let damage_animation = self.animations.insert(Animation::new(
    //                 self.config.animation_time,
    //                 AnimationKind::Damage {
    //                     from: board_item.position,
    //                     target: enemy,
    //                     damage: item.current_stats().damage.unwrap_or_default(),
    //                 },
    //             ));
    //             self.animations.insert(
    //                 Animation::new(
    //                     self.config.animation_time,
    //                     AnimationKind::ItemDeath {
    //                         item: item_id,
    //                         pos: board_item.position,
    //                     },
    //                 )
    //                 .after([damage_animation]),
    //             );
    //         }
    //     }
    //     ItemKind::SoulCrystal => {
    //         let damage = item.current_stats().damage.unwrap_or_default();
    //         let position = board_item.position;
    //         let damage_animation = self.deal_damage_nearest(position, damage, vec![]);
    //         self.animations.insert(
    //             Animation::new(
    //                 self.config.animation_time,
    //                 AnimationKind::ItemDeath {
    //                     item: item_id,
    //                     pos: position,
    //                 },
    //             )
    //             .after(damage_animation),
    //         );
    //     }
    //     ItemKind::Phantom => {
    //         let damage = item.current_stats().damage.unwrap_or_default();
    //         item.perm_stats.damage = Some(item.perm_stats.damage.unwrap_or_default() + 1);
    //         let position = board_item.position;
    //         self.deal_damage_nearest(position, damage, vec![]);
    //     }
    //     ItemKind::KingSkull => {
    //         // deal damage to all enemies
    //         let damage = item.current_stats().damage.unwrap_or_default();
    //         for (target, entity) in &self.entities {
    //             if let Fraction::Enemy = entity.fraction {
    //                 self.animations.insert(Animation::new(
    //                     self.config.animation_time,
    //                     AnimationKind::Damage {
    //                         from: board_item.position,
    //                         target,
    //                         damage,
    //                     },
    //                 ));
    //             }
    //         }
    //     }
    //     ItemKind::GoldenLantern => {
    //         // Destroy and light up for 3 turns
    //         self.grid.light_up(board_item.position, 1, 3);
    //         self.animations.insert(Animation::new(
    //             self.config.animation_time,
    //             AnimationKind::ItemDeath {
    //                 item: item_id,
    //                 pos: board_item.position,
    //             },
    //         ));
    //     }
    //     ItemKind::CharmingStaff => {
    //         let damage = item.current_stats().damage.unwrap_or_default();
    //         let position = board_item.position;
    //         self.deal_damage_random(position, damage, vec![]);
    //     }
    //     ItemKind::WarpPortal => {
    //         if self.items.iter().any(|(_, i)| {
    //             ItemRef::Category(ItemCategory::Magic).check(self.player.items[i.item_id].kind)
    //         }) {
    //             self.phase = Phase::Portal
    //         }
    //     }
    //     ItemKind::Solitude => {
    //         let damage = item.current_stats().damage.unwrap_or_default();
    //         let position = board_item.position;
    //         self.deal_damage_nearest(position, damage, vec![]);
    //     }
    //     ItemKind::ElectricRod => {
    //         let damage = item.current_stats().damage.unwrap_or_default();
    //         let position = board_item.position;
    //         self.deal_damage_nearest(position, damage, vec![]);
    //     }
    //     _ => {}
    // }
}
