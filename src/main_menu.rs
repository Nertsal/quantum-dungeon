use crate::prelude::*;

pub struct MainMenu {
    geng: Geng,
    assets: Rc<Assets>,
    config: Config,
    camera: Camera2d,
    framebuffer_size: vec2<usize>,
    cursor_pos: vec2<f64>,
    cursor_ui_pos: vec2<f32>,
    play_button: Aabb2<f32>,
    transition: Option<geng::state::Transition>,
}

impl MainMenu {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: Config) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            transition: None,
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
        }
    }

    fn play(&mut self) {
        self.transition = Some(geng::state::Transition::Push(Box::new(
            crate::game::Game::new(&self.geng, &self.assets, self.config.clone()),
        )));
    }

    fn draw_at(
        &self,
        target: Aabb2<f32>,
        texture: &ugli::Texture,
        camera: &Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let color = Color::WHITE;
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
        self.transition.take()
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
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
                &self.camera,
                framebuffer,
            );

            let target = Aabb2::point(vec2(-4.0, -0.8)).extend_uniform(1.8);
            self.draw_at(
                target,
                &self.assets.sprites.player_vision,
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
                self.draw_at(target, &self.assets.sprites.cell, &self.camera, framebuffer);
            }

            // Play
            self.play_button = Aabb2::point(vec2(4.0, -0.5)).extend_uniform(size / 2.0);
            self.draw_at(
                self.play_button,
                &self.assets.sprites.play_button,
                &self.camera,
                framebuffer,
            );
        }
    }
}
