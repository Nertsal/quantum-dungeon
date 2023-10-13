use super::*;

use rhai::EvalAltResult;

type Result<T, E = Box<EvalAltResult>> = std::result::Result<T, E>;

impl ModelState {
    pub fn effect_damage(&mut self, target: Id, damage: Hp) -> Result<()> {
        println!("TODO: {damage} damage to target {target:?}");
        Ok(())
    }
}
