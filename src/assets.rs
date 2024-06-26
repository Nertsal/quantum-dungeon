use crate::prelude::*;

#[derive(geng::asset::Load)]
pub struct Assets {
    pub sprites: Sprites,
    pub sounds: Sounds,
    pub music: geng::Sound,
    #[load(path = "font/Bodo Amat.ttf")]
    pub font: Rc<geng::Font>,
}

#[derive(geng::asset::Load)]
pub struct Sounds {
    #[load(ext = "mp3")]
    pub step: geng::Sound,
    #[load(ext = "mp3")]
    pub damage: geng::Sound,
    #[load(ext = "mp3")]
    pub enemy_death: geng::Sound,
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
    pub player_vision: ugli::Texture,
    pub reroll_button: ugli::Texture,
    pub panel: ugli::Texture,
    pub destroy_effect: ugli::Texture,
    pub enemy_death: ugli::Texture,
    pub outer_square: ugli::Texture,

    pub play_button: ugli::Texture,
    pub skip_button: ugli::Texture,
}

impl Assets {
    pub async fn load(manager: &geng::asset::Manager) -> anyhow::Result<Self> {
        geng::asset::Load::load(manager, &run_dir().join("assets"), &())
            .await
            .context("failed to load assets")
    }

    pub fn get_category_color(&self, category: Category) -> Color {
        match category {
            Category::Weapon => Color::try_from("#ffe7cd").unwrap(),
            Category::Tech => Color::try_from("#6467b6").unwrap(),
            Category::Treasure => Color::try_from("#cd8c66").unwrap(),
            Category::Spooky => Color::try_from("#469fe1").unwrap(),
            Category::Magic => Color::try_from("#d083c3").unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct ItemAssets {
    /// Map from item name to its asset.
    pub assets: HashMap<Rc<str>, ItemAsset>,
}

#[derive(Clone)]
pub struct ItemAsset {
    pub config: ItemConfig,
    pub description: Option<String>,
    pub script: Option<String>,
    pub texture: Option<Rc<ugli::Texture>>,
}

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct ItemConfig {
    pub name: Rc<str>,
    pub categories: Rc<[Category]>,
    pub appears_in_shop: ShopAppearance,
    #[serde(default)]
    pub base_stats: ItemStats,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShopAppearance {
    Always,
    MapNotFull,
    Never,
}

impl ShopAppearance {
    pub fn check(self, map_full: bool) -> bool {
        match self {
            ShopAppearance::Always => true,
            ShopAppearance::MapNotFull => !map_full,
            ShopAppearance::Never => false,
        }
    }
}

impl ItemAssets {
    pub fn get(&self, item: &str) -> &ItemAsset {
        self.assets
            .get(item)
            .unwrap_or_else(|| panic!("no assets found for item {}", item))
    }

    pub fn get_texture(&self, item: &str) -> Option<&ugli::Texture> {
        self.get(item).texture.as_deref()
    }
}

impl geng::asset::Load for ItemAssets {
    type Options = ();

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        &(): &Self::Options,
    ) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        async move {
            let list: Vec<String> = file::load_detect(path.join("_list.ron")).await?;
            let item_loaders = list.into_iter().map(|name| {
                let manager = &manager;
                let path = &path;
                async move {
                    let path = path.join(name);
                    let item: ItemAsset = geng::asset::Load::load(manager, &path, &()).await?;
                    anyhow::Ok((Rc::clone(&item.config.name), item))
                }
            });
            let items = future::join_all(item_loaders)
                .await
                .into_iter()
                .flatten()
                .collect();
            Ok(Self { assets: items })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}

impl geng::asset::Load for ItemAsset {
    type Options = ();

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        (): &Self::Options,
    ) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        async move {
            Ok(Self {
                config: geng::asset::Load::load(&manager, &path.join("config.ron"), &())
                    .await
                    .context("'config.ron' expected")?,
                description: geng::asset::Load::load(&manager, &path.join("description.txt"), &())
                    .await
                    .ok(),
                script: geng::asset::Load::load(&manager, &path.join("script.rn"), &())
                    .await
                    .ok(),
                texture: geng::asset::Load::load(
                    &manager,
                    &path.join("texture.png"),
                    &geng::asset::TextureOptions::default(),
                )
                .await
                .ok(),
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}
