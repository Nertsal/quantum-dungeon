mod action;
mod animation;
mod gen;
mod item;
mod resolve;

use super::*;

impl Model {
    pub fn update(&mut self, delta_time: Time) {
        self.update_animations(delta_time);
        self.resolve_animations(delta_time);
    }

    pub fn day_phase(&mut self) {
        log::debug!("Day phase");
        self.phase = Phase::Player;
        self.player.moves_left = 5;
    }

    fn player_phase(&mut self) {
        if self.player.moves_left == 0 {
            self.vision_phase();
        } else {
            self.phase = Phase::Player;
        }
    }

    fn vision_phase(&mut self) {
        log::debug!("Vision phase");
        self.phase = Phase::Vision;
        self.update_vision();
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
            ItemKind::Camera,
            ItemKind::Ghost,
        ];
        let mut rng = thread_rng();
        let options = (0..3).map(|_| *options.choose(&mut rng).unwrap()).collect();
        self.phase = Phase::Select { options };
    }

    fn next_turn(&mut self) {
        self.player.turns_left = self.player.turns_left.saturating_sub(1);
        if self.player.turns_left == 0 {
            // Damage for every enemy left on the board
            let damage = self
                .entities
                .iter()
                .filter(|e| e.fraction == Fraction::Enemy)
                .count();
            self.player.hearts = self.player.hearts.saturating_sub(damage);
            if self.player.hearts == 0 {
                self.game_over();
            } else {
                self.next_level();
            }
        } else {
            self.night_phase();
        }
    }

    fn game_over(&mut self) {
        log::info!("Game over");
        // TODO
    }

    fn calculate_empty_space(&self) -> HashSet<vec2<Coord>> {
        let mut available: HashSet<_> = self.grid.tiles.clone();

        for entity in &self.entities {
            available.remove(&entity.position);
        }
        for (_, item) in &self.items {
            available.remove(&item.position);
        }

        available
    }

    pub fn update_vision(&mut self) {
        log::debug!("Updating vision");
        let mut visible = HashSet::new();
        for entity in &self.entities {
            if let EntityKind::Player = entity.kind {
                if entity.look_dir == vec2::ZERO {
                    log::error!("entity has zero look dir");
                    continue;
                }
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
        if !self.entities.iter().any(|e| e.fraction == Fraction::Enemy) {
            // All enemies died -> next level
            self.next_level();
        }
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
}

fn distance(a: vec2<Coord>, b: vec2<Coord>) -> Coord {
    let delta = b - a;
    delta.x.abs().max(delta.y.abs())
}
