use crate::prelude::*;

pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
    camera: Camera2d,
    grid_size: vec2<f32>,
    cell_size: vec2<f32>,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            grid_size: vec2(1.0, 1.0),
            cell_size: vec2(0.9, 0.9),
        }
    }

    pub fn draw(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        for x in 0..model.grid_size.x {
            for y in 0..model.grid_size.y {
                self.draw_cell(vec2(x, y), framebuffer);
            }
        }

        self.draw_player(model.player.position, framebuffer);
    }

    fn draw_player(&self, position: vec2<Coord>, framebuffer: &mut ugli::Framebuffer) {
        let position = position.as_f32() * self.grid_size;

        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::Ellipse::circle(position, 0.3, Color::BLUE),
        );
    }

    fn draw_cell(&self, position: vec2<Coord>, framebuffer: &mut ugli::Framebuffer) {
        let position = position.as_f32() * self.grid_size;
        let outline_width = 0.1;
        let color = Color::WHITE;

        let [a, b, c, d] = Aabb2::ZERO
            .extend_symmetric(self.cell_size / 2.0)
            .extend_uniform(-outline_width / 2.0)
            .corners();
        let m = (a + b) / 2.0;
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::Chain::new(Chain::new(vec![m, b, c, d, a, m]), outline_width, color, 1)
                .translate(position),
        );
    }
}
