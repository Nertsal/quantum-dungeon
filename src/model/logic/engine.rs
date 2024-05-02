use super::*;

impl ScriptEffects<'_> {
    pub fn set_used(&mut self, item_id: Id) {
        self.0.push(Effect::SetUsed { item_id });
    }

    pub fn damage(&mut self, target: Id, damage: ScriptFunction) {
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
}