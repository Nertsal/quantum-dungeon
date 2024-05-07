use geng::{Key, MouseButton, Touch};

use crate::{prelude::*, render::GameRender};

pub struct Game {
    // geng: Geng,
    assets: Rc<Assets>,
    render: GameRender,
    model: Model,
    framebuffer_size: vec2<usize>,
    time: f32,

    cursor_pos: vec2<f64>,
    cursor_ui_pos: vec2<f32>,
    cursor_world_pos: vec2<f32>,
    cursor_grid_pos: vec2<f32>,

    active_touch: Option<ActiveTouch>,
    // TODO
    // controls: Controls,
}

#[derive(Debug, Clone)]
struct ActiveTouch {
    start: Touch,
    start_time: f32,
    current: Touch,
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        config: Config,
        all_items: &Rc<ItemAssets>,
    ) -> Self {
        Self {
            // geng: geng.clone(),
            assets: assets.clone(),
            render: GameRender::new(geng, assets, all_items),
            model: Model::new(assets.clone(), config, all_items),
            framebuffer_size: vec2(1, 1),
            time: 0.0,

            cursor_pos: vec2::ZERO,
            cursor_ui_pos: vec2::ZERO,
            cursor_world_pos: vec2::ZERO,
            cursor_grid_pos: vec2::ZERO,

            active_touch: None,
        }
    }

    fn handle_lmb(&mut self) {
        if self.render.inventory_button.contains(self.cursor_ui_pos) {
            self.render.show_inventory = !self.render.show_inventory;
            self.assets.sounds.step.play();
            return;
        }
        match self.model.phase {
            Phase::GameOver => {
                if self.render.retry_button.contains(self.cursor_ui_pos) {
                    self.model.player_action(PlayerInput::Retry);
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
                self.model.player_action(PlayerInput::Tile(target));
            }
        }
    }

    fn handle_touch_end(&mut self) {
        let Some(touch) = self.active_touch.take() else {
            return;
        };

        if self.time - touch.start_time < 0.5 && touch.start.position == touch.current.position {
            // Short tap
            self.handle_lmb();
        }
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
            self.cursor_ui_pos,
            self.cursor_grid_pos.map(|x| x.floor() as Coord),
            framebuffer,
        );
    }

    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::KeyPress {
            key: geng::Key::Space,
        } = event
        {
            println!("{}", self.cursor_ui_pos);
        }

        if let geng::Event::CursorMove { position } = event {
            self.cursor_pos = position;
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
            self.handle_lmb();
        }

        match event {
            geng::Event::TouchStart(touch) => {
                self.active_touch = Some(ActiveTouch {
                    start: touch,
                    start_time: self.time,
                    current: touch,
                });
            }
            geng::Event::TouchMove(touch) => {
                if let Some(active) = &mut self.active_touch {
                    active.current = touch;
                    self.cursor_pos = touch.position;
                }
            }
            geng::Event::TouchEnd(touch) => {
                if let Some(active) = &mut self.active_touch {
                    active.current = touch;
                    self.handle_touch_end();
                }
            }
            _ => (),
        }
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);

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
