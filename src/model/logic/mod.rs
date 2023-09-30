mod gen;

use super::*;

impl Model {
    pub fn update(&mut self, _delta_time: Time) {}

    pub fn player_move(&mut self, player_input: PlayerInput) {
        let mut moves = Vec::new();
        for (i, entity) in self.entities.iter_mut().enumerate() {
            if let EntityKind::Player = entity.kind {
                // TODO: if there are multiple players, resolve conflicting movement
                moves.push(i);
            }
        }

        let mut moved = false;
        for i in moves {
            let entity = self.entities.get_mut(i).unwrap();
            let target = self.grid.clamp_pos(entity.position + player_input.move_dir);
            if target != entity.position {
                self.move_entity_swap(i, target);
                moved = true;
            }
        }

        if moved {
            self.turn += 1;
        }
    }

    /// Move the entity to the target position and swap with the entity occupying the target (if any).
    fn move_entity_swap(&mut self, entity_id: usize, target_pos: vec2<Coord>) {
        let Some(entity) = self.entities.get_mut(entity_id) else {
            log::error!("entity does not exist: {}", entity_id);
            return;
        };

        let from_pos = entity.position;
        let target_pos = self.grid.clamp_pos(target_pos);
        if let Some(target) = self.entities.iter_mut().find(|e| e.position == target_pos) {
            target.position = from_pos;
        }

        let entity = self.entities.get_mut(entity_id).unwrap();
        entity.position = target_pos;
    }
}
