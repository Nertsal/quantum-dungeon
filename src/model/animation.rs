use super::*;

pub type Lifetime = geng_utils::bounded::Bounded<Time>;

#[derive(Debug, Clone)]
pub struct Animation {
    pub time: Lifetime,
    pub kind: AnimationKind,
    /// The id's of the animations this one has to start after.
    pub dependent_on: Vec<Id>,
}

#[derive(Debug, Clone)]
pub enum AnimationKind {
    MoveItem {
        /// Id of the item on the board.
        item_id: Id,
        target_pos: vec2<Coord>,
    },
    MoveEntity {
        entity_id: Id,
        target_pos: vec2<Coord>,
    },
    UseActive {
        fraction: Fraction,
        item_id: Id,
    },
    EntityDeath {
        entity: Id,
        pos: vec2<Coord>,
    },
    ItemDeath {
        item: Id,
        pos: vec2<Coord>,
    },
    Dupe {
        kind: ItemKind,
    },
    Damage {
        from: vec2<Coord>,
        target: Id,
        damage: Hp,
    },
    Bonus {
        from: vec2<Coord>,
        /// Id of the item on the board.
        target: Id,
        bonus: ItemStats,
        permanent: bool,
    },
}

impl Animation {
    pub fn new(time: impl Float, kind: AnimationKind) -> Self {
        Self {
            time: Lifetime::new_max(time.as_r32()),
            kind,
            dependent_on: Vec::new(),
        }
    }

    /// Set a dependency.
    pub fn after(self, animations: impl IntoIterator<Item = Id>) -> Self {
        Self {
            dependent_on: animations.into_iter().collect(),
            ..self
        }
    }
}
