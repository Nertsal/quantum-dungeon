use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub animation_time: Time,
    /// The time before and after the effect.
    pub effect_padding_time: Time,
}

impl Config {
    pub async fn load(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let content = file::load_string(path)
            .await
            .context(format!("when loading config file at {:?}", path))?;
        ron::from_str(&content).context(format!("when parsing config file at {:?}", path))
    }
}
