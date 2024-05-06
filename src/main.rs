mod assets;
mod config;
mod game;
mod main_menu;
mod model;
mod prelude;
mod render;
mod util;

use geng::prelude::*;

#[derive(clap::Parser)]
struct Opts {
    #[clap(long)]
    config: Option<std::path::PathBuf>,
    #[clap(flatten)]
    geng: geng::CliArgs,
}

fn main() {
    logger::init();
    geng::setup_panic_handler();

    let opts: Opts = clap::Parser::parse();

    let mut geng_options = geng::ContextOptions::default();
    geng_options.window.title = "Geng Game".to_string();
    geng_options.with_cli(&opts.geng);

    Geng::run_with(&geng_options, |geng| async move {
        let manager = geng.asset_manager();

        let mut assets = assets::Assets::load(manager).await.unwrap();
        assets.music.set_looped(true);
        let mut music = assets.music.play();
        music.set_volume(0.2);

        let config_path = opts.config.unwrap_or_else(|| "assets/config.ron".into());
        let config = config::Config::load(config_path).await.unwrap();

        let items: assets::ItemAssets =
            geng::asset::Load::load(manager, &run_dir().join("assets").join("items"), &())
                .await
                .unwrap();

        let state = main_menu::MainMenu::new(&geng, &Rc::new(assets), config, &Rc::new(items));
        geng.run_state(state).await;
    });
}
