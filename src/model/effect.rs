use super::*;

#[derive(Debug)]
pub struct QueuedEffect {
    /// The trigger that proc'ed the effect.
    pub trigger: Trigger,
    /// The id of the item that proc'ed the effect.
    pub proc_item: Id,
    pub effect: Effect,
}

#[derive(Debug, Clone, Copy)]
pub enum Trigger {
    Night,
    Day,
    Active,
}

#[derive(Debug)]
pub enum Effect {
    SetUsed {
        item_id: Id,
    },
    Damage {
        target: Id,
        damage: ScriptFunction,
    },
    Bonus {
        from: vec2<Coord>,
        target: Id,
        bonus: ItemStats,
        permanent: bool,
    },
}

impl Trigger {
    /// The name of the method in scripts responsible for handling the trigger.
    pub fn method_name(&self) -> &'static str {
        match self {
            Trigger::Night => "night",
            Trigger::Day => "day",
            Trigger::Active => "active",
        }
    }
}

impl Effect {
    /// The effect's priority for sorting when multiple effects are happening at the same time.
    pub fn priority(&self) -> i64 {
        match self {
            Effect::SetUsed { .. } => -999999,
            Effect::Damage { .. } => 0,
            Effect::Bonus { .. } => 10,
        }
    }
}
