mod logic;

use crate::prelude::*;

pub type Time = R32;

pub struct Model {
    config: Config,
}

impl Model {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}
