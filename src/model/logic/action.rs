use super::*;

impl Model {
    pub fn player_action(&mut self, player_input: PlayerInput) {
        if let PlayerInput::Vision { .. } = player_input {
        } else {
            log::debug!(
                "Player action: {:?}, current phase: {:?}",
                player_input,
                self.phase
            );
        }
        match &self.phase {
            Phase::Player if self.wait_for_effects() => self.player_move(player_input),
            Phase::Vision => self.player_vision(player_input),
            Phase::Map { .. } => self.map_action(player_input),
            Phase::Portal { .. } => self.portal_action(player_input),
            Phase::Select {
                options,
                extra_items,
            } => match player_input {
                PlayerInput::SelectItem(i) => self.select_item(options[i].clone()),
                PlayerInput::Skip => {
                    self.select_phase(0);
                    self.assets.sounds.step.play();
                }
                PlayerInput::Reroll => {
                    let mut state = self.state.borrow_mut();
                    if state.player.refreshes > 0 {
                        state.player.refreshes -= 1;
                        drop(state);
                        self.select_phase(extra_items + 1);
                        self.assets.sounds.step.play();
                    }
                }
                _ => {
                    log::error!("invalid input during phase Select, expected an item selection")
                }
            },
            Phase::GameOver => {
                if let PlayerInput::Retry = player_input {
                    self.retry();
                    self.assets.sounds.step.play();
                }
            }
            _ => {}
        }
    }

    /// Uncover a tile.
    fn map_action(&mut self, player_input: PlayerInput) {
        let PlayerInput::Tile(pos) = player_input else {
            log::error!("invalid input during phase Map, expected a tile");
            return;
        };
        let mut state = self.state.borrow_mut();
        if !state.grid.check_pos_near(pos) {
            log::error!(
                "position {} is not valid, select an empty one on the edge",
                pos
            );
            return;
        }

        if let Phase::Map {
            tiles_left,
            next_phase,
        } = &mut self.phase
        {
            state.grid.expand(pos);
            *tiles_left = tiles_left.saturating_sub(1);
            if *tiles_left == 0 {
                log::debug!("Moving from Map phase to {:?}", next_phase);
                let mut phase = Phase::Vision;
                std::mem::swap(&mut self.phase, &mut phase);
                if let Phase::Map { next_phase, .. } = phase {
                    self.phase = *next_phase;
                }
            }
            self.assets.sounds.step.play();
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
        let state = self.state.borrow();
        if !state.grid.check_pos(target_pos) {
            log::error!("position {} is not valid, select a valid tile", target_pos);
            return;
        }
        if state.grid.fractured.contains(&target_pos) {
            log::error!("cannot move to a fractured position");
            return;
        }
        drop(state);

        if let Phase::Portal { .. } = self.phase {
            // What is this trick KEKW
            let mut state = self.state.borrow_mut();
            let state_ref = &mut *state;

            if let Some((_, target)) = state_ref
                .items
                .iter_mut()
                .find(|(_, item)| item.position == target_pos)
            {
                let item = &state_ref.player.items[target.item_id];
                if ItemFilter::Category(Category::Magic).check(&item.kind) {
                    let Some((_, player)) = state_ref
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
                    state.grid.fractured.insert(target_pos);
                    drop(state);

                    let mut phase = Phase::Vision;
                    std::mem::swap(&mut self.phase, &mut phase);
                    if let Phase::Portal { next_phase } = phase {
                        self.phase = *next_phase;
                    }

                    self.assets.sounds.step.play();
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
        let state = self.state.borrow();
        if state.player.moves_left == 0 {
            // Should be unreachable
            log::error!("tried to move, but no moves are left");
            drop(state);
            self.vision_phase();
            return;
        }

        if let PlayerInput::Skip = player_input {
            log::debug!("Skipping turn");
            drop(state);
            self.vision_phase();
            self.assets.sounds.step.play();
            return;
        }

        let mut moves = Vec::new();
        let mut move_dir = vec2::ZERO;
        drop(state);
        let mut state_ref = self.state.borrow_mut();
        let state = &mut *state_ref;
        for (i, entity) in &mut state.entities {
            if let EntityKind::Player = entity.kind {
                // TODO: if there are multiple players, resolve conflicting movement
                move_dir = match player_input {
                    PlayerInput::Dir(dir) => dir,
                    PlayerInput::Tile(pos) => {
                        if !state.grid.check_pos(pos) {
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
        drop(state_ref);
        for i in moves {
            let mut state = self.state.borrow_mut();
            let entity = state.entities.get_mut(i).unwrap();
            let target = entity.position + move_dir;
            // Fracture tiles as we walk
            if state.grid.check_pos(target) && state.grid.fractured.insert(target) {
                drop(state);
                self.move_entity_swap(i, target);
                moved = true;
            }
        }

        if moved {
            let mut state = self.state.borrow_mut();
            state.player.moves_left = state.player.moves_left.saturating_sub(1);
            self.assets.sounds.step.play();
        }
    }

    fn select_item(&mut self, item: ItemKind) {
        log::debug!("Select item {:?}", item);
        let item = self
            .engine
            .init_item(item)
            .expect("Item initialization failed"); // TODO: handle error
        self.state.borrow_mut().player.items.insert(item);
        let items = if let Phase::Select { extra_items, .. } = self.phase {
            extra_items
        } else {
            0
        };
        self.select_phase(items);
        self.assets.sounds.step.play();
    }

    fn player_vision(&mut self, player_input: PlayerInput) {
        for (_, entity) in &mut self.state.borrow_mut().entities {
            if let EntityKind::Player = entity.kind {
                let dir = match player_input {
                    PlayerInput::Dir(dir) => dir,
                    PlayerInput::Tile(pos) | PlayerInput::Vision { pos, .. } => {
                        pos - entity.position
                    }
                    _ => {
                        log::error!("invalid input during phase Vision, expected tile or dir");
                        return;
                    }
                };
                if dir.x != 0 && dir.y != 0 {
                    // log::error!("invalid input direction during phase Vision: {}", dir);
                    return;
                }
                entity.look_dir = dir.map(|x| x.clamp_abs(1));
            }
        }

        self.update_vision();
        if let PlayerInput::Vision { commit: true, .. } = player_input {
            self.assets.sounds.step.play();
            self.phase = Phase::PostVision {
                timer: Lifetime::new_max(r32(1.0)),
            };
        }
    }
}
