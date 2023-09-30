use geng::prelude::*;

#[derive(geng::asset::Load)]
pub struct Assets {
    pub sprites: Sprites,
}

#[derive(geng::asset::Load)]
pub struct Sprites {
    pub cell: ugli::Texture,
    pub cell_dark: ugli::Texture,
    pub boot: ugli::Texture,
    pub enemy: ugli::Texture,
    pub heart: ugli::Texture,
    pub item_shadow: ugli::Texture,
    pub overlay: ugli::Texture,
    pub player: ugli::Texture,
    pub sword: ugli::Texture,
    pub turn_time: ugli::Texture,
}

impl Assets {
    pub async fn load(manager: &geng::asset::Manager) -> anyhow::Result<Self> {
        geng::asset::Load::load(manager, &run_dir().join("assets"), &())
            .await
            .context("failed to load assets")
    }
}
