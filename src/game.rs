use geng::{Key, MouseButton};

use crate::{
    leaderboard::Leaderboard, prelude::*, render::GameRender, Secrets, PLAYER_NAME_STORAGE,
};

#[allow(dead_code)]
pub struct Game {
    geng: Geng,
    assets: Rc<Assets>,
    render: GameRender,
    model: Model,
    framebuffer_size: vec2<usize>,
    cursor_pos: vec2<f64>,
    cursor_ui_pos: vec2<f32>,
    cursor_world_pos: vec2<f32>,
    cursor_grid_pos: vec2<f32>,
    // TODO
    // controls: Controls,
    secrets: Option<Secrets>,
    player_name: String,
    leaderboard: LeaderboardState,
    leaderboard_future: Option<Pin<Box<dyn Future<Output = Leaderboard>>>>,
}

pub enum LeaderboardState {
    None,
    Pending,
    Ready(Leaderboard),
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        secrets: Option<Secrets>,
        config: Config,
        player_name: String,
    ) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            render: GameRender::new(geng, assets),
            model: Model::new(assets.clone(), config),
            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            cursor_ui_pos: vec2::ZERO,
            cursor_world_pos: vec2::ZERO,
            cursor_grid_pos: vec2::ZERO,
            secrets,
            player_name: crate::fix_name(&player_name),
            leaderboard: LeaderboardState::None,
            leaderboard_future: None,
        }
    }

    fn load_leaderboard(&mut self, submit_score: bool) {
        if let Some(secrets) = &self.secrets {
            self.leaderboard = LeaderboardState::Pending;
            let player_name = self.player_name.clone();
            let submit_score = submit_score && !player_name.trim().is_empty();
            let score = submit_score.then_some(self.model.score as f32);
            let secrets = secrets.clone();
            self.leaderboard_future = Some(
                crate::leaderboard::Leaderboard::submit(player_name, score, secrets.leaderboard)
                    .boxed_local(),
            );
        }
    }

    fn verify_name(&self) -> bool {
        !self.player_name.trim().is_empty()
    }

    fn submit_score(&mut self) {
        if !self.verify_name() {
            return;
        }
        log::debug!("Submitting");
        self.geng.window().stop_text_edit();
        batbox::preferences::save(PLAYER_NAME_STORAGE, &self.player_name);
        self.load_leaderboard(true);
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(
            framebuffer,
            Some(Rgba::try_from("#333333").unwrap()),
            None,
            None,
        );
        self.render.draw(
            &self.model,
            &self.leaderboard,
            &self.player_name,
            self.cursor_ui_pos,
            self.cursor_grid_pos.map(|x| x.floor() as Coord),
            framebuffer,
        );
    }

    fn handle_event(&mut self, event: geng::Event) {
        match &event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::Space => println!("{}", self.cursor_ui_pos),
                geng::Key::Enter => {
                    if let Phase::GameOver = self.model.phase {
                        if self.geng.window().is_editing_text() {
                            self.submit_score();
                        }
                    }
                }
                _ => {}
            },
            geng::Event::CursorMove { position } => {
                self.cursor_pos = *position;
            }
            geng::Event::EditText(text) => {
                self.player_name = crate::fix_name(text);
                self.geng.window().start_text_edit(&self.player_name);
            }
            _ => {}
        }

        let move_dir = if geng_utils::key::is_event_press(&event, [Key::ArrowLeft, Key::A]) {
            vec2(-1, 0)
        } else if geng_utils::key::is_event_press(&event, [Key::ArrowRight, Key::D]) {
            vec2(1, 0)
        } else if geng_utils::key::is_event_press(&event, [Key::ArrowDown, Key::S]) {
            vec2(0, -1)
        } else if geng_utils::key::is_event_press(&event, [Key::ArrowUp, Key::W]) {
            vec2(0, 1)
        } else {
            vec2(0, 0)
        };
        if move_dir != vec2::ZERO {
            self.model.player_action(PlayerInput::Dir(move_dir));
            return;
        }

        if geng_utils::key::is_event_press(&event, [Key::Digit1]) {
            self.model.player_action(PlayerInput::SelectItem(0));
        } else if geng_utils::key::is_event_press(&event, [Key::Digit2]) {
            self.model.player_action(PlayerInput::SelectItem(1));
        } else if geng_utils::key::is_event_press(&event, [Key::Digit3]) {
            self.model.player_action(PlayerInput::SelectItem(2));
        }

        if geng_utils::key::is_event_press(&event, [MouseButton::Left]) {
            if let Phase::GameOver = self.model.phase {
            } else if self.render.inventory_button.contains(self.cursor_ui_pos) {
                self.render.show_inventory = !self.render.show_inventory;
                self.assets.sounds.step.play();
                return;
            }
            match self.model.phase {
                Phase::GameOver => {
                    if self.render.retry_button.contains(self.cursor_ui_pos) {
                        self.model.player_action(PlayerInput::Retry);
                    } else if self.geng.window().is_editing_text()
                        && self.render.submit_button.contains(self.cursor_ui_pos)
                    {
                        self.submit_score();
                    }
                }
                Phase::Player if self.render.skip_turn_button.contains(self.cursor_ui_pos) => {
                    self.model.player_action(PlayerInput::Skip);
                }
                Phase::Select { .. } => {
                    if let Some(i) = self
                        .render
                        .buttons
                        .iter()
                        .position(|(_, button)| button.contains(self.cursor_ui_pos))
                    {
                        self.model.player_action(PlayerInput::SelectItem(i));
                    } else if self.render.reroll_button.contains(self.cursor_ui_pos) {
                        self.model.player_action(PlayerInput::Reroll);
                    } else if self.render.skip_item_button.contains(self.cursor_ui_pos) {
                        self.model.player_action(PlayerInput::Skip);
                    }
                }
                Phase::Vision => {
                    let target = self.cursor_grid_pos.map(|x| x.floor() as Coord);
                    // if self.model.grid.check_pos(target) {
                    self.model.player_action(PlayerInput::Vision {
                        pos: target,
                        commit: true,
                    });
                    // }
                }
                _ => {
                    let target = self.cursor_grid_pos.map(|x| x.floor() as Coord);
                    // if self.model.grid.check_pos(target) {
                    self.model.player_action(PlayerInput::Tile(target));
                    // }
                }
            }
        }
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);

        if let Some(future) = &mut self.leaderboard_future {
            // Poll leaderboard
            if let std::task::Poll::Ready(leaderboard) = future.as_mut().poll(
                &mut std::task::Context::from_waker(futures::task::noop_waker_ref()),
            ) {
                self.leaderboard_future = None;
                log::info!("Loaded leaderboard");
                self.leaderboard = LeaderboardState::Ready(leaderboard);
            }
        } else if let Phase::GameOver = self.model.phase {
            if let LeaderboardState::None = self.leaderboard {
                let submit = self.verify_name();
                self.load_leaderboard(submit);
            }
        }

        if let LeaderboardState::Ready(leaderboard) = &self.leaderboard {
            let editing = self.geng.window().is_editing_text();
            if leaderboard.my_position.is_none() {
                if !editing {
                    log::debug!("Starting text edit");
                    self.geng.window().start_text_edit(&self.player_name);
                }
            } else if editing {
                log::debug!("Stopping text edit");
                self.geng.window().stop_text_edit();
            }
        }

        self.cursor_world_pos = self
            .render
            .world_camera
            .screen_to_world(self.framebuffer_size.as_f32(), self.cursor_pos.as_f32());
        self.cursor_ui_pos = self
            .render
            .ui_camera
            .screen_to_world(self.framebuffer_size.as_f32(), self.cursor_pos.as_f32());
        self.cursor_grid_pos = self.cursor_world_pos / self.render.cell_size + vec2::splat(0.5);

        if let Phase::Vision = self.model.phase {
            let target = self.cursor_grid_pos.map(|x| x.floor() as Coord);
            self.model.player_action(PlayerInput::Vision {
                pos: target,
                commit: false,
            });
        }

        self.model.update(delta_time);
    }
}
