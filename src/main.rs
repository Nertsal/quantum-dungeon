mod assets;
mod config;
mod game;
mod leaderboard;
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

#[derive(geng::asset::Load, Deserialize, Clone)]
#[load(serde = "toml")]
pub struct Secrets {
    leaderboard: LeaderboardSecrets,
}

#[derive(Deserialize, Clone)]
pub struct LeaderboardSecrets {
    id: String,
    key: String,
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

        let secrets: Option<Secrets> = file::load_detect(run_dir().join("secrets.toml")).await.ok();
        let secrets = secrets.or_else(|| {
            Some(crate::Secrets {
                leaderboard: crate::LeaderboardSecrets {
                    id: option_env!("LEADERBOARD_ID")?.to_string(),
                    key: option_env!("LEADERBOARD_KEY")?.to_string(),
                },
            })
        });

        let config_path = opts.config.unwrap_or_else(|| "assets/config.ron".into());
        let config = config::Config::load(config_path).await.unwrap();
        let state = main_menu::MainMenu::new(&geng, &Rc::new(assets), secrets, config);
        geng.run_state(state).await;
    });
}
