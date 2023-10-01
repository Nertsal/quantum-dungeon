mod gen;

use super::*;

impl Model {
    pub fn update(&mut self, delta_time: Time) {
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

    pub fn day_phase(&mut self) {
        log::debug!("Day phase");
        self.phase = Phase::Player;
        self.player.moves_left = 5;
    }

    fn active_phase(&mut self, fraction: Fraction, item_id: usize) {
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
                // TODO
                false
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

    fn select_item(&mut self, item: ItemKind) {
        log::debug!("Select item {:?}", item);
        self.player.items.push(item);
        self.turn += 1;
        self.night_phase();
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

    fn vision_phase(&mut self) {
        log::debug!("Vision phase");
        self.phase = Phase::Vision;
    }

    fn select_phase(&mut self) {
        log::debug!("Select phase");
        // TODO
        self.update_vision();

        let options = [
            ItemKind::Sword,
            ItemKind::Forge,
            ItemKind::Boots,
            ItemKind::Map,
        ];
        let mut rng = thread_rng();
        let options = (0..3).map(|_| *options.choose(&mut rng).unwrap()).collect();
        self.phase = Phase::Select { options };
    }

    fn calculate_empty_space(&self) -> HashSet<vec2<Coord>> {
        let mut available: HashSet<_> = self.grid.tiles.clone();

        for entity in &self.entities {
            available.remove(&entity.position);
        }
        for item in &self.items {
            available.remove(&item.position);
        }

        available
    }

    pub fn update_vision(&mut self) {
        log::debug!("Updating vision");
        let mut visible = HashSet::new();
        for entity in &self.entities {
            if let EntityKind::Player = entity.kind {
                let mut pos = entity.position;
                visible.insert(pos);
                loop {
                    let target = pos + entity.look_dir;
                    if !self.grid.check_pos(target) {
                        break;
                    }
                    visible.insert(target);
                    pos = target;
                }
            }
        }
        self.visible_tiles = visible;
    }

    fn check_deaths(&mut self) {
        self.entities.retain(|e| e.health.is_above_min());
    }

    /// Move the entity to the target position and swap with the entity occupying the target (if any).
    fn move_entity_swap(&mut self, entity_id: usize, target_pos: vec2<Coord>) {
        let Some(entity) = self.entities.get_mut(entity_id) else {
            log::error!("entity does not exist: {}", entity_id);
            return;
        };

        let from_pos = entity.position;
        let target_pos = if self.grid.check_pos(target_pos) {
            target_pos
        } else {
            log::error!("tried to move to an invalid position: {}", target_pos);
            return;
        };
        if let Some(target) = self.entities.iter_mut().find(|e| e.position == target_pos) {
            target.position = from_pos;
        }

        let entity = self.entities.get_mut(entity_id).unwrap();
        entity.position = target_pos;
    }

    /// Collect an item at the given position.
    fn collect_item_at(&mut self, fraction: Fraction, position: vec2<Coord>) {
        for i in (0..self.items.len()).rev() {
            let item = &mut self.items[i];
            if item.position == position {
                self.active_phase(fraction, i);
                return;
            }
        }
    }

    fn use_item(&mut self, fraction: Fraction, item: Item) {
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

    fn count_items_near(&self, position: vec2<Coord>, kind: ItemKind) -> usize {
        self.items
            .iter()
            .filter(|item| {
                let d = distance(position, item.position);
                item.kind == kind && d > 0 && d <= 1
            })
            .count()
    }
}

fn distance(a: vec2<Coord>, b: vec2<Coord>) -> Coord {
    let delta = b - a;
    delta.x.abs().max(delta.y.abs())
}
