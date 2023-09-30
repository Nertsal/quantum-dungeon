use geng::{Key, MouseButton};

use crate::{prelude::*, render::GameRender};

#[allow(dead_code)]
pub struct Game {
    geng: Geng,
    render: GameRender,
    model: Model,
    framebuffer_size: vec2<usize>,
    cursor_pos: vec2<f64>,
    // TODO
    // controls: Controls,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: Config) -> Self {
        Self {
            geng: geng.clone(),
            render: GameRender::new(geng, assets),
            model: Model::new(config),
            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
        }
    }

    fn cursor_cell_pos(&self) -> vec2<Coord> {
        let world_pos = self
            .render
            .camera
            .screen_to_world(self.framebuffer_size.as_f32(), self.cursor_pos.as_f32());
        let cell_pos = world_pos / self.render.cell_size + vec2::splat(0.5);
        cell_pos.map(|x| x.floor() as Coord)
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
        self.render.draw(&self.model, framebuffer);
    }

    fn handle_event(&mut self, event: geng::Event) {
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
            if let Phase::Select { .. } = self.model.phase {
                // TODO
            } else {
                let target = self.cursor_cell_pos();
                self.model.player_action(PlayerInput::Tile(target));
            }
            // return;
        }
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.model.update(delta_time);
    }
}
