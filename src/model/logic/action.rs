use super::*;

impl Model {
    pub fn player_action(&mut self, player_input: PlayerInput) {
        log::debug!(
            "Player action: {:?}, current phase: {:?}",
            player_input,
            self.phase
        );
        match &self.phase {
            Phase::Player if self.animations.is_empty() => self.player_move(player_input),
            Phase::Vision => self.player_vision(player_input),
            Phase::Map { .. } => self.map_action(player_input),
            Phase::Portal => self.portal_action(player_input),
            Phase::Select {
                options,
                extra_items,
            } => match player_input {
                PlayerInput::SelectItem(i) => self.select_item(options[i]),
                PlayerInput::Reroll => self.select_phase(extra_items + 1),
                _ => {
                    log::error!("invalid input during phase Select, expected an item selection")
                }
            },
            _ => {}
        }
    }

    /// Uncover a tile.
    fn map_action(&mut self, player_input: PlayerInput) {
        let PlayerInput::Tile(pos) = player_input else {
            log::error!("invalid input during phase Map, expected a tile");
            return;
        };
        if !self.grid.check_pos_near(pos) {
            log::error!(
                "position {} is not valid, select an empty one on the edge",
                pos
            );
            return;
        }

        if let Phase::Map { tiles_left } = &mut self.phase {
            self.grid.expand(pos);
            *tiles_left = tiles_left.saturating_sub(1);
            if *tiles_left == 0 {
                self.player_phase();
            }
        } else {
            log::error!("tried map action but not in a map phase");
        }
    }

    /// Swap position with a magic item.
    fn portal_action(&mut self, player_input: PlayerInput) {
        let PlayerInput::Tile(target_pos) = player_input else {
            log::error!("invalid input during phase Portal, expected a tile");
            return;
        };
        if !self.grid.check_pos(target_pos) {
            log::error!("position {} is not valid, select a valid tile", target_pos);
            return;
        }
        if self.grid.fractured.contains(&target_pos) {
            log::error!("cannot move to a fractured position");
            return;
        }

        if let Phase::Portal = self.phase {
            if let Some((_, target)) = self
                .items
                .iter_mut()
                .find(|(_, item)| item.position == target_pos)
            {
                let item = &self.player.items[target.item_id];
                if ItemRef::Category(ItemCategory::Magic).check(item.kind) {
                    let Some((_, player)) = self
                        .entities
                        .iter_mut()
                        .find(|(_, e)| matches!(e.kind, EntityKind::Player))
                    else {
                        log::error!("Player not found");
                        return;
                    };
                    // Swap
                    target.position = player.position;
                    player.position = target_pos;
                    self.grid.fractured.insert(target_pos);
                    self.player_phase();
                } else {
                    log::error!(
                        "invalid input during phase Portal, expected a magic item position, found a non-magic item"
                    );
                }
            } else {
                log::error!("invalid input during phase Portal, expected a magic item position, found nothing");
            }
        } else {
            log::error!("tried portal action but not in a portal phase");
        }
    }

    fn player_move(&mut self, player_input: PlayerInput) {
        if self.player.moves_left == 0 {
            // Should be unreachable
            log::error!("tried to move, but no moves are left");
            self.vision_phase();
            return;
        }

        if let PlayerInput::Skip = player_input {
            log::debug!("Skipping turn");
            self.vision_phase();
            return;
        }

        let mut moves = Vec::new();
        let mut move_dir = vec2::ZERO;
        for (i, entity) in &mut self.entities {
            if let EntityKind::Player = entity.kind {
                // TODO: if there are multiple players, resolve conflicting movement
                move_dir = match player_input {
                    PlayerInput::Dir(dir) => dir,
                    PlayerInput::Tile(pos) => {
                        if !self.grid.check_pos(pos) {
                            log::error!("invalid input during phase Player, expected a valid tile");
                            return;
                        } else {
                            pos - entity.position
                        }
                    }
                    _ => {
                        log::error!("invalid input during phase Player, expected tile or dir");
                        return;
                    }
                };
                moves.push(i);
            }
        }
        if move_dir.x.abs() + move_dir.y.abs() != 1 {
            // Invalid move
            log::error!("invalid move {}", move_dir);
            return;
        }

        let mut moved = false;
        for i in moves {
            let entity = self.entities.get_mut(i).unwrap();
            let target = entity.position + move_dir;
            // Fracture tiles as we walk
            if self.grid.check_pos(target) && self.grid.fractured.insert(target) {
                self.move_entity_swap(i, target);
                self.update_vision();
                moved = true;
            }
        }

        if moved {
            self.player.moves_left = self.player.moves_left.saturating_sub(1);
            // Phase could have changed when collecting an item
            if let Phase::Player = self.phase {
                if self.player.moves_left == 0 {
                    self.vision_phase();
                }
            }
        }
    }

    fn select_item(&mut self, item: ItemKind) {
        log::debug!("Select item {:?}", item);
        self.player.items.insert(item.instantiate());
        let items = if let Phase::Select { extra_items, .. } = self.phase {
            extra_items
        } else {
            0
        };
        self.select_phase(items);
    }

    fn player_vision(&mut self, player_input: PlayerInput) {
        for (_, entity) in &mut self.entities {
            if let EntityKind::Player = entity.kind {
                let dir = match player_input {
                    PlayerInput::Dir(dir) => dir,
                    PlayerInput::Tile(pos) => pos - entity.position,
                    _ => {
                        log::error!("invalid input during phase Vision, expected tile or dir");
                        return;
                    }
                };
                if dir.x.abs() + dir.y.abs() != 1 {
                    log::error!("invalid input direction during phase Vision: {}", dir);
                    return;
                }
                entity.look_dir = dir;
            }
        }

        self.phase = Phase::PostVision {
            timer: Lifetime::new_max(r32(1.0)),
        };
        self.update_vision();
    }
}
