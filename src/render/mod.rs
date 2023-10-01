use crate::prelude::*;

pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
    pub camera: Camera2d,
    pub cell_size: vec2<f32>,
    pub buttons: Vec<(ItemKind, Aabb2<f32>)>,
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
            buttons: Vec::new(),
        }
    }

    pub fn draw(
        &mut self,
        model: &Model,
        cursor_world_pos: vec2<f32>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        for &pos in &model.grid.tiles {
            let light = if let Phase::Vision | Phase::Night = model.phase {
                model.visible_tiles.contains(&pos)
            } else {
                true
            };
            self.draw_cell(pos, light, framebuffer);
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

        self.buttons.clear();
        let text = match &model.phase {
            Phase::Vision => "Select a direction to look at",
            Phase::Map => "Select a position to place a new tile",
            Phase::Select { options } => {
                let mut color = Color::BLACK;
                color.a = 0.5;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    &self.camera,
                    &draw2d::Quad::new(overlay, color),
                );

                let size = 2.0;
                let offset = size * (options.len() as f32 - 1.0) / 2.0;
                self.buttons = options
                    .iter()
                    .enumerate()
                    .map(|(i, &item)| {
                        let pos = vec2(i as f32 * size - offset, 0.0);
                        let target =
                            Aabb2::point(pos).extend_symmetric(vec2::splat(size) / 2.0 * 0.9);
                        (item, target)
                    })
                    .collect();

                for &(item, target) in &self.buttons {
                    let texture = self.assets.sprites.item_texture(item);
                    let background = if target.contains(cursor_world_pos) {
                        &self.assets.sprites.cell
                    } else {
                        &self.assets.sprites.cell_dark
                    };
                    self.draw_at(target, background, framebuffer);
                    self.draw_at(target, texture, framebuffer);
                }

                "Select an item"
            }
            _ => "",
        };

        self.geng.default_font().draw(
            framebuffer,
            &self.camera,
            text,
            vec2(geng::TextAlign::CENTER, geng::TextAlign::TOP),
            mat3::translate(self.camera.center + vec2(0.0, 0.8 * self.camera.fov / 2.0))
                * mat3::scale_uniform(0.7),
            Color::BLACK,
        );
    }

    fn draw_item(&self, item: &Item, framebuffer: &mut ugli::Framebuffer) {
        let texture = self.assets.sprites.item_texture(item.kind);
        // TODO: place the shadow
        // self.draw_at(item.position, &self.assets.sprites.item_shadow, framebuffer);
        self.draw_at_grid(item.position, texture, framebuffer);

        // Damage value
        if let Some(damage) = item.temp_stats.damage {
            let pos = (item.position.as_f32() + vec2(0.0, 0.3)) * self.cell_size;
            let mut color = Color::BLACK;
            color.a = 0.5;
            let radius = 0.1;
            let target = Aabb2::point(pos).extend_uniform(0.1);
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::Ellipse::circle(pos, radius * 1.5, color),
            );
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::Text::unit(
                    self.geng.default_font().clone(),
                    format!("{}", damage),
                    Color::WHITE,
                )
                .fit_into(target),
            );
        }
    }

    fn draw_at_grid(
        &self,
        position: vec2<Coord>,
        texture: &ugli::Texture,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let position = position.as_f32() * self.cell_size;
        self.draw_at(
            Aabb2::point(position).extend_symmetric(self.cell_size / 2.0),
            texture,
            framebuffer,
        )
    }

    fn draw_at(
        &self,
        target: Aabb2<f32>,
        texture: &ugli::Texture,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let size =
            texture.size().as_f32() * target.size() / self.assets.sprites.cell.size().as_f32();
        let target = Aabb2::point(target.center()).extend_symmetric(size / 2.0);
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

        self.draw_at_grid(entity.position, texture, framebuffer)
    }

    fn draw_cell(&self, position: vec2<Coord>, light: bool, framebuffer: &mut ugli::Framebuffer) {
        let texture = if light {
            &self.assets.sprites.cell
        } else {
            &self.assets.sprites.cell_dark
        };
        self.draw_at_grid(position, texture, framebuffer)
    }
}
