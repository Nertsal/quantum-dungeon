use super::*;

pub type Lifetime = geng_utils::bounded::Bounded<Time>;

#[derive(Debug, Clone)]
pub struct Animation {
    pub time: Lifetime,
    pub kind: AnimationKind,
}

#[derive(Debug, Clone)]
pub enum AnimationKind {
    BonusFly { from: vec2<Coord>, to: vec2<Coord> },
}
