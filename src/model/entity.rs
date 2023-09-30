use super::*;

pub type Hp = i64;
pub type Health = geng_utils::bounded::Bounded<Hp>;

#[derive(Debug, Clone)]
pub struct Entity {
    pub position: vec2<Coord>,
    pub fraction: Fraction,
    pub health: Health,
    pub look_dir: vec2<Coord>,
    pub kind: EntityKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fraction {
    Player,
    Enemy,
}

#[derive(Debug, Clone)]
pub enum EntityKind {
    Player,
    Dummy,
}
