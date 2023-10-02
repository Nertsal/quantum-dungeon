use super::*;

pub type Lifetime = geng_utils::bounded::Bounded<Time>;

#[derive(Debug, Clone)]
pub struct Animation {
    pub time: Lifetime,
    pub kind: AnimationKind,
}

#[derive(Debug, Clone)]
pub enum AnimationKind {
    UseActive {
        fraction: Fraction,
        item_id: Id,
    },
    Death {
        item: Id,
    },
    Dupe {
        kind: ItemKind,
    },
    Damage {
        from: vec2<Coord>,
        target: usize,
        damage: Hp,
    },
}

impl Animation {
    pub fn new(time: Time, kind: AnimationKind) -> Self {
        Self {
            time: Lifetime::new_max(time),
            kind,
        }
    }
}
