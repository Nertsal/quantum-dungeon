use geng::Key;

use crate::{prelude::*, render::GameRender};

#[allow(dead_code)]
pub struct Game {
    geng: Geng,
    render: GameRender,
    model: Model,
    // TODO
    // controls: Controls,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: Config) -> Self {
        Self {
            geng: geng.clone(),
            render: GameRender::new(geng, assets),
            model: Model::new(config),
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(
            framebuffer,
            Some(Rgba::try_from("#333333").unwrap()),
            None,
            None,
        );
        self.render.draw(&self.model, framebuffer);
    }

    fn handle_event(&mut self, event: geng::Event) {
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
            self.model.player_move(PlayerInput { move_dir })
        }
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.model.update(delta_time);
    }
}
