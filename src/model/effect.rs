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
        damage: Rc<ScriptFunction>,
    },
    Bonus {
        from: vec2<Coord>,
        target: Id,
        bonus: ItemStats,
        permanent: bool,
    },
    /// Uncover tiles on the map.
    OpenTiles {
        tiles: usize,
    },
    /// Completely remove the item from the map.
    Destroy {
        item_id: Id,
    },
    /// Duplicate an item, making a copy in the inventory, and, if there is space, on the board.
    Duplicate {
        item_id: Id,
    },
    /// Gain extra moves for this turn.
    GainMoves {
        moves: usize,
    },
    Portal,
}

impl Trigger {
    /// The name of the method in scripts responsible for handling the trigger.
    pub fn method_name(&self) -> &'static str {
        match self {
            Self::Night => "night",
            Self::Day => "day",
            Self::Active => "active",
        }
    }
}

impl Effect {
    /// The effect's priority for sorting when multiple effects are happening at the same time.
    pub fn priority(&self) -> i64 {
        match self {
            Self::SetUsed { .. } => -999999,
            Self::Damage { .. } => 0,
            Self::Bonus { .. } => 10,
            Self::OpenTiles { .. } => 999,
            Self::Portal { .. } => 999,
            Self::Destroy { .. } => -999999,
            Self::Duplicate { .. } => 20,
            Self::GainMoves { .. } => 0,
        }
    }
}
