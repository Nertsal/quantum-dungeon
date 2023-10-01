use super::*;

impl Model {
    pub fn player_action(&mut self, player_input: PlayerInput) {
        log::debug!(
            "Player action: {:?}, current phase: {:?}",
            player_input,
            self.phase
        );
        match &self.phase {
            Phase::Player => self.player_move(player_input),
            Phase::Vision => self.player_vision(player_input),
            Phase::Map => self.map_action(player_input),
            Phase::Select { options } => {
                if let PlayerInput::SelectItem(i) = player_input {
                    self.select_item(options[i]);
                } else {
                    log::error!("invalid input during phase Select, expected an item selection");
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
        if self.grid.check_pos(pos) {
            log::error!("position {} is already valid, select an empty one", pos);
            return;
        }
        self.grid.expand(pos);

        if self.player.moves_left == 0 {
            self.vision_phase();
        } else {
            self.phase = Phase::Player;
        }
    }

    fn player_move(&mut self, player_input: PlayerInput) {
        if self.player.moves_left == 0 {
            // Should be unreachable
            log::error!("tried to move, but no moves are left");
            self.vision_phase();
            return;
        }

        let mut moves = Vec::new();
        let mut move_dir = vec2::ZERO;
        for (i, entity) in self.entities.iter_mut().enumerate() {
            if let EntityKind::Player = entity.kind {
                // TODO: if there are multiple players, resolve conflicting movement
                move_dir = match player_input {
                    PlayerInput::Dir(dir) => dir,
                    PlayerInput::Tile(pos) => pos - entity.position,
                    _ => {
                        log::error!("invalid input during phase Player, expected tile or dir");
                        return;
                    }
                };
                moves.push(i);
            }
        }
        move_dir = move_dir.map(|x| x.clamp_abs(1));

        let mut moved = false;
        for i in moves {
            let entity = self.entities.get_mut(i).unwrap();
            let target = entity.position + move_dir;
            if self.grid.check_pos(target) {
                let fraction = entity.fraction;
                self.move_entity_swap(i, target);
                self.collect_item_at(fraction, target);
                self.update_vision();
                moved = true;
            }
        }

        if moved {
            self.check_deaths();
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
        self.player.items.push(item);
        self.turn += 1;
        self.night_phase();
    }

    fn player_vision(&mut self, player_input: PlayerInput) {
        for entity in &mut self.entities {
            if let EntityKind::Player = entity.kind {
                let dir = match player_input {
                    PlayerInput::Dir(dir) => dir,
                    PlayerInput::Tile(pos) => pos - entity.position,
                    _ => {
                        log::error!("invalid input during phase Vision, expected tile or dir");
                        return;
                    }
                };
                entity.look_dir = dir.map(|x| x.clamp_abs(1));
            }
        }
        self.select_phase();
    }
}