use super::*;

// NOTE: expose functions in src/model/engine.rs
impl ScriptItem<'_> {
    fn effects(&self) -> ScriptEffects {
        ScriptEffects(self.effects.borrow_mut())
    }

    pub fn damage(&mut self, target: Target, damage: ScriptFunction) {
        let damage = Rc::new(damage);
        let mut rng = thread_rng(); // TODO: seeded rng

        let source_fraction = Fraction::Player; // TODO: non-player items?

        let target = match target {
            Target::Nearest => self
                .model
                .entities
                .iter()
                .filter(|(_, entity)| source_fraction != entity.fraction)
                .min_by_key(|(_, entity)| distance(entity.position, self.board_item.position))
                .map(|(i, _)| i),
            Target::Random => self
                .model
                .entities
                .iter()
                .filter(|(_, entity)| source_fraction != entity.fraction)
                .choose(&mut rng)
                .map(|(i, _)| i),
        };

        if let Some(target) = target {
            self.effects().damage(target, damage);
        } else {
            log::debug!("Item tried attacking but no target was found");
        }
    }

    pub fn damage_all_nearby(&mut self, range: Option<Coord>, damage: ScriptFunction) {
        let damage = Rc::new(damage);
        let source_fraction = Fraction::Player; // TODO: non-player items?
        for (target, _) in self.model.entities.iter().filter(|(_, entity)| {
            source_fraction != entity.fraction
                && range.map_or(true, |range| {
                    distance(entity.position, self.board_item.position) <= range
                })
        }) {
            self.effects().damage(target, damage.clone());
        }
    }

    pub fn bonus(&mut self, bonus: ItemStats, permanent: bool) {
        self.effects().bonus(
            self.board_item.position,
            self.item.on_board.unwrap(),
            bonus.clone(),
            permanent,
        );
    }

    pub fn bonus_from(&mut self, from: vec2<Coord>, bonus: ItemStats, permanent: bool) {
        self.effects()
            .bonus(from, self.item.on_board.unwrap(), bonus.clone(), permanent);
    }

    pub fn bonus_from_nearby(
        &mut self,
        range: Coord,
        filter: ItemFilter,
        bonus: ItemStats,
        permanent: bool,
    ) {
        for (_, board_item) in &self.model.items {
            let item = &self.model.player.items[board_item.item_id];
            let dist = distance(board_item.position, self.board_item.position);
            if (1..=range).contains(&dist) && filter.check(&item.kind) {
                self.effects().bonus(
                    board_item.position,
                    self.item.on_board.unwrap(),
                    bonus.clone(),
                    permanent,
                );
            }
        }
    }

    pub fn bonus_from_connected(&mut self, filter: ItemFilter, bonus: ItemStats, permanent: bool) {
        for (_, board_item) in &self.model.items {
            let item = &self.model.player.items[board_item.item_id];
            let dist = distance_manhattan(board_item.position, self.board_item.position);
            if dist == 1 && filter.check(&item.kind) {
                self.effects().bonus(
                    board_item.position,
                    self.item.on_board.unwrap(),
                    bonus.clone(),
                    permanent,
                );
            }
        }
    }

    pub fn bonus_to_nearby(
        &mut self,
        range: Coord,
        filter: ItemFilter,
        bonus: ItemStats,
        permanent: bool,
    ) {
        for (target, board_item) in &self.model.items {
            let item = &self.model.player.items[board_item.item_id];
            let dist = distance(board_item.position, self.board_item.position);
            if (1..=range).contains(&dist) && filter.check(&item.kind) {
                self.effects()
                    .bonus(self.board_item.position, target, bonus.clone(), permanent);
            }
        }
    }

    pub fn bonus_to_all(&mut self, filter: ItemFilter, bonus: ItemStats, permanent: bool) {
        for (target, board_item) in &self.model.items {
            let item = &self.model.player.items[board_item.item_id];
            if filter.check(&item.kind) {
                self.effects()
                    .bonus(self.board_item.position, target, bonus.clone(), permanent);
            }
        }
    }

    /// Lets the player uncover new tiles on the map.
    pub fn open_tiles(&mut self, tiles: usize) {
        self.effects().open_tiles(tiles);
    }

    /// Destroys the self item.
    pub fn destroy(&mut self) {
        self.effects().destroy(self.board_item.item_id);
    }

    pub fn get_new_item(&mut self, filter: Option<ItemFilter>) {
        if let Some(item) = self
            .model
            .all_items
            .iter()
            .filter(|item| filter.as_ref().map_or(true, |filter| filter.check(item)))
            .choose(&mut thread_rng())
        {
            self.effects().new_item(item.clone());
        }
    }

    pub fn find_multiple(&self, filter: ItemFilter, count: usize) -> Vec<Id> {
        self.model
            .items
            .iter()
            .filter(|(i, board_item)| {
                let item = &self.model.player.items[board_item.item_id];
                *i != self.item.on_board.unwrap() && filter.check(&item.kind)
            })
            .map(|(id, _)| id)
            .choose_multiple(&mut thread_rng(), count)
    }

    pub fn find_nearby(&self, range: Coord, filter: ItemFilter) -> Option<Id> {
        let items = self.model.items.iter().filter(|(_, board_item)| {
            let item = &self.model.player.items[board_item.item_id];
            let dist = distance(board_item.position, self.board_item.position);
            (1..=range).contains(&dist) && filter.check(&item.kind)
        });
        items.choose(&mut thread_rng()).map(|(id, _)| id)
    }

    pub fn find_random(&self, filter: ItemFilter) -> Option<Id> {
        let items = self.model.items.iter().filter(|(i, board_item)| {
            let item = &self.model.player.items[board_item.item_id];
            *i != self.item.on_board.unwrap() && filter.check(&item.kind)
        });
        items.choose(&mut thread_rng()).map(|(id, _)| id)
    }

    pub fn duplicate(&mut self) {
        self.effects().duplicate(self.board_item.item_id);
    }

    pub fn gain_moves(&mut self, moves: usize) {
        self.effects().gain_moves(moves);
    }

    pub fn portal(&mut self) {
        self.effects().portal();
    }

    pub fn swap_with(&mut self, target: Id) {
        let a = self.item.on_board.unwrap();
        self.effects().swap_items(a, target);
    }

    pub fn grid_bounds(&self) -> Aabb2<Coord> {
        self.model.grid.bounds()
    }

    pub fn turn_into(&mut self, target: &str) {
        self.effects()
            .transform_item(self.board_item.item_id, target);
    }

    pub fn emit_light_around(&mut self, position: vec2<Coord>, radius: Coord, duration: usize) {
        self.effects().emit_light(position, radius, duration);
    }

    pub fn is_observed(&self) -> bool {
        self.model.visible_tiles.contains(&self.board_item.position)
    }

    pub fn use_item(&mut self, target: Id) {
        self.effects().use_item(target);
    }
}
