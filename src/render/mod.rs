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
        self.camera.center = (model.grid.size.as_f32() / 2.0 - vec2::splat(0.5)) * self.grid_size;

        for x in 0..model.grid.size.x {
            for y in 0..model.grid.size.y {
                self.draw_cell(vec2(x, y), framebuffer);
            }
        }

        for entity in &model.entities {
            self.draw_entity(entity, framebuffer);
        }
        for item in &model.items {
            self.draw_item(item, framebuffer);
        }
    }

    fn draw_item(&self, item: &Item, framebuffer: &mut ugli::Framebuffer) {
        let position = item.position.as_f32() * self.grid_size;
        let color = Color::GREEN;

        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::Ellipse::circle(position, 0.3, color),
        );
    }

    fn draw_entity(&self, entity: &Entity, framebuffer: &mut ugli::Framebuffer) {
        let position = entity.position.as_f32() * self.grid_size;
        let color = match entity.fraction {
            Fraction::Player => Color::BLUE,
            Fraction::Enemy => Color::RED,
        };

        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::Ellipse::circle(position, 0.3, color),
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
