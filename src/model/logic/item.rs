use super::*;

// NOTE: expose functions in src/model/engine.rs
impl ScriptItem<'_> {
    pub fn damage_nearest(&mut self, damage: ScriptFunction) {
        let source_fraction = Fraction::Player;
        let nearest = self
            .model
            .entities
            .iter()
            .filter(|(_, entity)| source_fraction != entity.fraction)
            .min_by_key(|(_, entity)| distance(entity.position, self.board_item.position));
        if let Some((target, _)) = nearest {
            self.effects.push(Effect::Damage { target, damage });
        }
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
                self.effects.push(Effect::Bonus {
                    from: board_item.position,
                    target: self.item.on_board.unwrap(),
                    bonus: bonus.clone(),
                    permanent,
                });
            }
        }
    }

    /// Lets the player uncover new tiles on the map.
    pub fn open_tiles(&mut self, tiles: usize) {
        self.effects.push(Effect::OpenTiles { tiles });
    }

    /// Destroys the self item.
    pub fn destroy(&mut self) {
        self.effects.push(Effect::Destroy {
            item_id: self.board_item.item_id,
        });
    }
}
