use super::*;

impl Model {
    pub fn update(&mut self, _delta_time: Time) {}

    pub fn player_move(&mut self, player_input: PlayerInput) {
        for entity in &mut self.entities {
            if let EntityKind::Player = entity.kind {
                entity.position = self.grid.clamp_pos(entity.position + player_input.move_dir);
            }
        }

        self.turn += 1;
    }
}
