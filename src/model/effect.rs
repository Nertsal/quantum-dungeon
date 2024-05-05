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
    SwapItems {
        board_a: Id,
        board_b: Id,
    },
    TransformItem {
        item_id: Id,
        target_name: String,
    },
    EmitLight {
        position: vec2<Coord>,
        radius: Coord,
        duration: usize,
    },
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
