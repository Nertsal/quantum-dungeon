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
    UseActive {
        fraction: Fraction,
        item_id: Id,
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
        target: usize,
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
    pub fn new(time: Time, kind: AnimationKind) -> Self {
        Self {
            time: Lifetime::new_max(time),
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
