use crate::prelude::*;

pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
    camera: Camera2d,
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
            cell_size: vec2(1.0, 1.0),
        }
    }

    pub fn draw(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        self.camera.center = (model.grid.size.as_f32() / 2.0 - vec2::splat(0.5)) * self.cell_size;

        for x in 0..model.grid.size.x {
            for y in 0..model.grid.size.y {
                self.draw_cell(vec2(x, y), framebuffer);
            }
        }

        for item in &model.items {
            self.draw_item(item, framebuffer);
        }
        for entity in &model.entities {
            self.draw_entity(entity, framebuffer);
        }

        // Vision
        for x in 0..model.grid.size.x {
            for y in 0..model.grid.size.y {
                if !model.visible_tiles.contains(&vec2(x, y)) {
                    self.geng.draw2d().draw2d(
                        framebuffer,
                        &self.camera,
                        &draw2d::Quad::new(
                            Aabb2::point(vec2(x, y).as_f32() * self.cell_size)
                                .extend_symmetric(self.cell_size / 2.0),
                            Color::new(0.0, 0.0, 0.0, 0.5),
                        ),
                    );
                }
            }
        }
    }

    fn draw_item(&self, item: &Item, framebuffer: &mut ugli::Framebuffer) {
        let position = item.position.as_f32() * self.cell_size;
        let color = Color::GREEN;

        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::Ellipse::circle(position, 0.3, color),
        );
    }

    fn draw_entity(&self, entity: &Entity, framebuffer: &mut ugli::Framebuffer) {
        let position = entity.position.as_f32() * self.cell_size;
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
        let position = position.as_f32() * self.cell_size;
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::TexturedQuad::new(
                Aabb2::point(position).extend_symmetric(self.cell_size / 2.0),
                &self.assets.sprites.cell,
            ),
        );
    }
}
