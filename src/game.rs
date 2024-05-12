use geng::{Key, MouseButton};

use crate::{controls::*, prelude::*, render::GameRender};

pub struct Game {
    // geng: Geng,
    assets: Rc<Assets>,
    render: GameRender,
    model: Model,
    framebuffer_size: vec2<usize>,

    cursor_pos: vec2<f64>,
    cursor_ui_pos: vec2<f32>,
    cursor_world_pos: vec2<f32>,
    cursor_grid_pos: vec2<f32>,
    touch_controller: TouchController,
    // TODO
    // controls: Controls,
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
            model: Model::new(assets.clone(), config, all_items.clone()),
            framebuffer_size: vec2(1, 1),

            cursor_pos: vec2::ZERO,
            cursor_ui_pos: vec2::ZERO,
            cursor_world_pos: vec2::ZERO,
            cursor_grid_pos: vec2::ZERO,
            touch_controller: TouchController::new(),
        }
    }

    fn handle_lmb(&mut self) {
        self.render.hide_item_hint = true;
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
            Phase::Player | Phase::Map { .. } | Phase::Portal { .. }
                if self.render.skip_turn_button.contains(self.cursor_ui_pos) =>
            {
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
        if let Some(action) = self.touch_controller.handle_event(&event) {
            match action {
                TouchAction::ShortTap { position } => {
                    self.cursor_pos = position;
                    self.handle_lmb();
                }
                TouchAction::Move { position } => {
                    self.render.hide_item_hint = false;
                    self.cursor_pos = position;
                }
            }
        }

        if let geng::Event::KeyPress {
            key: geng::Key::Space,
        } = event
        {
            println!("{}", self.cursor_ui_pos);
        }

        if let geng::Event::CursorMove { position } = event {
            self.render.hide_item_hint = false;
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
    }

    fn update(&mut self, delta_time: f64) {
        self.touch_controller.update(delta_time);
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
