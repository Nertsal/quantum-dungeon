use super::*;

impl ScriptEffects<'_> {
    // pub fn set_used(&mut self, item_id: Id) {
    //     self.0.push(Effect::SetUsed { item_id });
    // }

    pub fn damage(&mut self, target: Id, damage: Rc<ScriptFunction>) {
        self.0.push(Effect::Damage { target, damage });
    }

    pub fn bonus(&mut self, from: vec2<Coord>, target: Id, bonus: ItemStats, permanent: bool) {
        self.0.push(Effect::Bonus {
            from,
            target,
            bonus,
            permanent,
        });
    }

    pub fn open_tiles(&mut self, tiles: usize) {
        self.0.push(Effect::OpenTiles { tiles });
    }

    pub fn destroy(&mut self, item_id: Id) {
        self.0.push(Effect::Destroy { item_id });
    }

    pub fn duplicate(&mut self, item_id: Id) {
        self.0.push(Effect::Duplicate { item_id });
    }

    pub fn gain_moves(&mut self, moves: usize) {
        self.0.push(Effect::GainMoves { moves });
    }

    pub fn portal(&mut self) {
        self.0.push(Effect::Portal);
    }

    pub fn swap_items(&mut self, board_a: Id, board_b: Id) {
        self.0.push(Effect::SwapItems { board_a, board_b });
    }

    pub fn transform_item(&mut self, item: Id, target_name: &str) {
        self.0.push(Effect::TransformItem {
            item_id: item,
            target_name: target_name.to_owned(),
        });
    }

    pub fn emit_light(&mut self, position: vec2<Coord>, radius: Coord, duration: usize) {
        self.0.push(Effect::EmitLight {
            position,
            radius,
            duration,
        });
    }

    pub fn use_item(&mut self, item: Id) {
        self.0.push(Effect::UseItem { item });
    }

    pub fn new_item(&mut self, kind: ItemKind) {
        self.0.push(Effect::NewItem { kind });
    }
}
