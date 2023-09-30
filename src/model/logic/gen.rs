use super::*;

impl Model {
    pub fn generate(&mut self) {
        // TODO

        self.entities.push(Entity {
            position: vec2(4, 5),
            fraction: Fraction::Enemy,
            health: Health::new_max(5),
            kind: EntityKind::Dummy,
        });
    }
}
