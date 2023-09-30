use super::*;

impl Model {
    /// Night phase.
    pub fn generate(&mut self) {
        // TODO

        self.entities.push(Entity {
            position: vec2(4, 5),
            fraction: Fraction::Enemy,
            health: Health::new_max(5),
            kind: EntityKind::Dummy,
        });

        self.items.push(Item {
            position: vec2(3, 5),
            kind: ItemKind::Sword,
        });
        self.items.push(Item {
            position: vec2(3, 6),
            kind: ItemKind::Sword,
        });
        self.items.push(Item {
            position: vec2(2, 5),
            kind: ItemKind::Sword,
        });
    }
}
