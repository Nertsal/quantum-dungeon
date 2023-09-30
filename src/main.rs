mod assets;
mod config;
mod game;
mod model;
mod prelude;
mod render;

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
        let assets = assets::Assets::load(manager).await.unwrap();
        let config_path = opts.config.unwrap_or_else(|| "assets/config.ron".into());
        let config = config::Config::load(config_path).await.unwrap();
        let state = game::Game::new(&geng, &Rc::new(assets), config);
        geng.run_state(state).await;
    });
}
