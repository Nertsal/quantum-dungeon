use crate::{prelude::*, Secrets, PLAYER_NAME_STORAGE};

pub struct MainMenu {
    geng: Geng,
    assets: Rc<Assets>,
    secrets: Option<Secrets>,
    config: Config,
    camera: Camera2d,
    framebuffer_size: vec2<usize>,
    cursor_pos: vec2<f64>,
    cursor_ui_pos: vec2<f32>,
    play_button: Aabb2<f32>,
    transition: Option<geng::state::Transition>,
    player_name: String,
}

impl MainMenu {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, secrets: Option<Secrets>, config: Config) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            transition: None,
            secrets,
            config,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            cursor_ui_pos: vec2::ZERO,
            play_button: Aabb2::ZERO,
            player_name: batbox::preferences::load(PLAYER_NAME_STORAGE).unwrap_or("".to_string()),
        }
    }

    fn play(&mut self) {
        self.transition = Some(geng::state::Transition::Push(Box::new(
            crate::game::Game::new(
                &self.geng,
                &self.assets,
                self.secrets.clone(),
                self.config.clone(),
                self.player_name.clone(),
            ),
        )));
    }

    fn draw_at(
        &self,
        target: Aabb2<f32>,
        texture: &ugli::Texture,
        color: Color,
        camera: &Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let size =
            texture.size().as_f32() * target.size() / self.assets.sprites.cell.size().as_f32();
        let target = Aabb2::point(target.center()).extend_symmetric(size / 2.0);
        self.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::TexturedQuad::colored(target, texture, color),
        );
    }
}

impl geng::State for MainMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        let trans = self.transition.take();
        if trans.is_some() {
            self.geng.window().stop_text_edit();
            batbox::preferences::save(PLAYER_NAME_STORAGE, &self.player_name);
        }
        trans
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::EditText(text) => {
                self.player_name = crate::fix_name(&text);
                self.geng.window().start_text_edit(&self.player_name);
            }
            geng::Event::CursorMove { position } => {
                self.cursor_pos = position;
            }
            geng::Event::MousePress {
                button: geng::MouseButton::Left,
            } => {
                if self.play_button.contains(self.cursor_ui_pos) {
                    self.play();
                }
            }
            _ => {}
        }
    }

    fn update(&mut self, _delta_time: f64) {
        self.cursor_ui_pos = self
            .camera
            .screen_to_world(self.framebuffer_size.as_f32(), self.cursor_pos.as_f32());

        if !self.geng.window().is_editing_text() {
            self.geng.window().start_text_edit(&self.player_name);
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(
            framebuffer,
            Some(Rgba::try_from("#333333").unwrap()),
            None,
            None,
        );

        {
            // Title
            let pos = self.camera.center + vec2(0.0, 2.5);
            let size = vec2(
                3.0 * self.assets.sprites.panel.size().as_f32().aspect(),
                3.0,
            );
            let target = Aabb2::point(pos).extend_symmetric(size / 2.0);
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::TexturedQuad::new(target, &self.assets.sprites.panel),
            );

            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::Text::unit(
                    self.assets.font.clone(),
                    "QUANTUM",
                    Color::try_from("#ffe7cd").unwrap(),
                )
                .fit_into(Aabb2::point(target.center()).extend_symmetric(vec2(10.0, 0.6) / 2.0)),
            );

            let target = target.translate(vec2(0.0, -0.95));
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::Text::unit(
                    self.assets.font.clone(),
                    "DUNGEON",
                    Color::try_from("#c9464b").unwrap(),
                )
                .fit_into(Aabb2::point(target.center()).extend_symmetric(vec2(10.0, 0.4) / 2.0)),
            );
        }

        {
            // Player
            let target = Aabb2::point(vec2(-5.0, -1.0)).extend_uniform(2.0);
            self.draw_at(
                target,
                &self.assets.sprites.player,
                Color::WHITE,
                &self.camera,
                framebuffer,
            );

            let target = Aabb2::point(vec2(-4.0, -0.8)).extend_uniform(1.8);
            self.draw_at(
                target,
                &self.assets.sprites.player_vision,
                Color::WHITE,
                &self.camera,
                framebuffer,
            );
        }

        {
            // Tiles
            let size = 1.7;
            for i in 0..3 {
                let pos = vec2(i as f32 - 1.0, 0.0) * size + vec2(0.0, -0.5);
                let target = Aabb2::point(pos).extend_uniform(size / 2.0);
                let texture = if target.contains(self.cursor_ui_pos) {
                    &self.assets.sprites.cell_light
                } else {
                    &self.assets.sprites.cell
                };
                self.draw_at(target, texture, Color::WHITE, &self.camera, framebuffer);
            }

            // Play
            self.play_button = Aabb2::point(vec2(4.0, -0.5)).extend_uniform(size / 2.0);
            let color = if self.play_button.contains(self.cursor_ui_pos) {
                Color::WHITE.map_rgb(|x| x * 1.2)
            } else {
                Color::WHITE
            };
            self.draw_at(
                self.play_button,
                &self.assets.sprites.play_button,
                color,
                &self.camera,
                framebuffer,
            );
        }

        {
            // Player name
            let text_color = Color::try_from("#ffe7cd").unwrap();
            let (text, color) = if self.player_name.is_empty() {
                let mut color = text_color;
                color.a = 0.7;
                ("<just type your name>", color)
            } else {
                (self.player_name.as_str(), text_color)
            };
            self.assets.font.draw(
                framebuffer,
                &self.camera,
                text,
                vec2(0.5, 0.5).map(geng::TextAlign),
                mat3::translate(vec2(0.0, -3.0))
                    * mat3::scale_uniform(0.4)
                    * mat3::translate(vec2(0.0, -0.25)),
                color,
            );
        }

        {
            // Overlay
            let overlay_texture = &self.assets.sprites.overlay;
            let size = overlay_texture.size().as_f32();
            let size = size * self.camera.fov / size.y;
            let overlay = Aabb2::point(self.camera.center).extend_symmetric(size / 2.0);

            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::TexturedQuad::new(overlay, &self.assets.sprites.outer_square),
            );

            let mut color = Color::WHITE;
            color.a = 0.5;
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::TexturedQuad::colored(overlay, overlay_texture, color),
            );
        }
    }
}
