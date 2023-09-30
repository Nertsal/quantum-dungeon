use crate::prelude::*;

pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
    pub camera: Camera2d,
    pub cell_size: vec2<f32>,
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
                let light = if let Phase::Vision | Phase::Night = model.phase {
                    model.visible_tiles.contains(&vec2(x, y))
                } else {
                    true
                };
                self.draw_cell(vec2(x, y), light, framebuffer);
            }
        }

        for item in &model.items {
            self.draw_item(item, framebuffer);
        }
        for entity in &model.entities {
            self.draw_entity(entity, framebuffer);
        }

        let overlay_texture = &self.assets.sprites.overlay;
        let size = overlay_texture.size().as_f32();
        let size = size * self.camera.fov / size.y;
        let overlay = Aabb2::point(self.camera.center).extend_symmetric(size / 2.0);

        let mut color = Color::WHITE;
        color.a = 0.5;
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::TexturedQuad::colored(overlay, overlay_texture, color),
        );

        if let Phase::Vision = model.phase {
            self.geng.default_font().draw(
                framebuffer,
                &self.camera,
                "Select a direction to look at",
                vec2(geng::TextAlign::CENTER, geng::TextAlign::TOP),
                mat3::translate(self.camera.center + vec2(0.0, 0.8 * self.camera.fov / 2.0))
                    * mat3::scale_uniform(0.7),
                Color::BLACK,
            );
        }
    }

    fn draw_item(&self, item: &Item, framebuffer: &mut ugli::Framebuffer) {
        let texture = match item.kind {
            ItemKind::Sword => &self.assets.sprites.sword,
        };
        // TODO: place the shadow
        // self.draw_at(item.position, &self.assets.sprites.item_shadow, framebuffer);
        self.draw_at(item.position, texture, framebuffer);
    }

    fn draw_at(
        &self,
        position: vec2<Coord>,
        texture: &ugli::Texture,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let position = position.as_f32() * self.cell_size;
        let size =
            texture.size().as_f32() * self.cell_size / self.assets.sprites.cell.size().as_f32();
        let target = Aabb2::point(position).extend_symmetric(size / 2.0);
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::TexturedQuad::new(target, texture),
        );
    }

    fn draw_entity(&self, entity: &Entity, framebuffer: &mut ugli::Framebuffer) {
        let texture = match entity.fraction {
            Fraction::Player => &self.assets.sprites.player,
            Fraction::Enemy => &self.assets.sprites.enemy,
        };

        self.draw_at(entity.position, texture, framebuffer)
    }

    fn draw_cell(&self, position: vec2<Coord>, light: bool, framebuffer: &mut ugli::Framebuffer) {
        let texture = if light {
            &self.assets.sprites.cell
        } else {
            &self.assets.sprites.cell_dark
        };
        self.draw_at(position, texture, framebuffer)
    }
}
