use crate::prelude::*;

#[derive(geng::asset::Load)]
pub struct Assets {
    pub sprites: Sprites,
    pub items: ItemAssets,
}

#[derive(geng::asset::Load)]
pub struct Sprites {
    pub cell: ugli::Texture,
    pub cell_dark: ugli::Texture,
    pub cell_light: ugli::Texture,
    pub cell_plus: ugli::Texture,
    pub overlay: ugli::Texture,
    pub item_card: ugli::Texture,
    pub inventory: ugli::Texture,
    pub enemy_health: ugli::Texture,
    pub weapon_damage: ugli::Texture,
    pub heart: ugli::Texture,
    pub turn_time: ugli::Texture,
    pub item_shadow: ugli::Texture,
    pub enemy: ugli::Texture,
    pub player: ugli::Texture,

    pub boot: ugli::Texture,
    pub camera: ugli::Texture,
    pub chest: ugli::Texture,
    pub fire_scroll: ugli::Texture,
    pub ghost: ugli::Texture,
    pub grand_map: ugli::Texture,
    pub greedy_pot: ugli::Texture,
    pub magic_treasure_bag: ugli::Texture,
    pub radiation_core: ugli::Texture,
    pub spirit_coin: ugli::Texture,
    pub sword: ugli::Texture,
}

pub struct ItemAssets {
    pub descriptions: HashMap<String, String>,
}

impl Assets {
    pub async fn load(manager: &geng::asset::Manager) -> anyhow::Result<Self> {
        geng::asset::Load::load(manager, &run_dir().join("assets"), &())
            .await
            .context("failed to load assets")
    }
}

impl Sprites {
    pub fn item_texture(&self, item: ItemKind) -> &ugli::Texture {
        match item {
            ItemKind::Sword => &self.sword,
            ItemKind::Forge => &self.item_shadow,
            ItemKind::Boots => &self.boot,
            ItemKind::Map => &self.grand_map,
            ItemKind::Camera => &self.camera,
            ItemKind::Ghost => &self.ghost,
            ItemKind::FireScroll => &self.fire_scroll,
            ItemKind::SoulCrystal => &self.item_shadow,
            ItemKind::RadiationCore => &self.radiation_core,
            ItemKind::GreedyPot => &self.greedy_pot,
            ItemKind::SpiritCoin => &self.spirit_coin,
            ItemKind::Chest => &self.chest,
            ItemKind::MagicTreasureBag => &self.magic_treasure_bag,
        }
    }
}

impl ItemAssets {
    pub fn get_description(&self, item: ItemKind) -> &str {
        self.descriptions
            .get(&format!("{:?}", item).to_lowercase())
            .map(|x| x.as_str())
            .unwrap_or("<Description missing>")
    }
}

impl geng::asset::Load for ItemAssets {
    type Options = ();

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        &(): &Self::Options,
    ) -> geng::asset::Future<Self> {
        let _manager = manager.clone();
        let path = path.to_owned();
        async move {
            let list: Vec<String> = file::load_detect(path.join("_list.ron")).await?;
            let mut descriptions = HashMap::new();
            for name in list {
                let name = name.to_lowercase().replace(' ', "_");
                let desc = file::load_string(path.join(format!("{}.txt", name))).await?;
                descriptions.insert(name, desc);
            }
            Ok(Self { descriptions })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}
