use super::*;

impl Model {
    pub fn update(&mut self, _delta_time: Time) {}

    pub fn next_turn(&mut self, player_input: PlayerInput) {
        self.player.position += player_input.move_dir;

        self.turn += 1;
    }
}
