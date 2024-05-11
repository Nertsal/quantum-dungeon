use super::*;

pub type Lifetime = geng_utils::bounded::Bounded<Time>;

#[derive(Debug)]
pub struct Animation {
    pub time: Lifetime,
    pub ending_time: Time,
    pub kind: AnimationKind,
    /// The id's of the animations this one has to start after.
    pub dependent_on: Vec<Id>,
}

#[derive(Debug)]
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
    ItemEffect {
        item: Id,
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
        let time = time.as_r32();
        let zero = Time::ZERO;
        let (time, ending_time) = match &kind {
            AnimationKind::MoveItem { .. } => (time, zero),
            AnimationKind::MoveEntity { .. } => (time, zero),
            AnimationKind::ItemEffect { .. } => (time, time),
            AnimationKind::EntityDeath { .. } => (zero, time),
            AnimationKind::ItemDeath { .. } => (time, time),
            AnimationKind::Dupe { .. } => (time, time),
            AnimationKind::Damage { .. } => (time, time),
            AnimationKind::Bonus { .. } => (time, time),
        };
        Self {
            time: Lifetime::new_max(time),
            ending_time,
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
