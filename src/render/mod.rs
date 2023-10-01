use crate::prelude::*;

pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
    pub ui_camera: Camera2d,
    pub world_camera: Camera2d,
    pub cell_size: vec2<f32>,
    pub buttons: Vec<(ItemKind, Aabb2<f32>)>,
    pub inventory_button: Aabb2<f32>,
    pub show_inventory: bool,
}

#[derive(Debug)]
enum TileLight {
    Normal,
    Dark,
    Light,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            ui_camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            world_camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 7.0,
            },
            cell_size: vec2(1.0, 1.0),
            buttons: Vec::new(),
            inventory_button: Aabb2::point(vec2(0.0, -4.0))
                .extend_symmetric(vec2::splat(1.5) / 2.0),
            show_inventory: false,
        }
    }

    pub fn draw(
        &mut self,
        model: &Model,
        cursor_ui_pos: vec2<f32>,
        cursor_cell_pos: vec2<Coord>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        // Tiles
        for &pos in &model.grid.tiles {
            let light = if let Phase::Vision | Phase::Night = model.phase {
                if model.visible_tiles.contains(&pos) {
                    TileLight::Light
                } else {
                    TileLight::Dark
                }
            } else if model.grid.fractured.contains(&pos) {
                TileLight::Dark
            } else {
                TileLight::Normal
            };
            self.draw_cell(pos, light, framebuffer);
        }

        // Items
        for (i, item) in &model.items {
            let resolving = if let Phase::Passive {
                item_queue,
                start_delay,
                end_delay,
            } = &model.phase
            {
                item_queue.last().map(|i| (i, start_delay, end_delay))
            } else if let Phase::Active {
                item_id,
                start_delay,
                end_delay,
                ..
            } = &model.phase
            {
                Some((item_id, start_delay, end_delay))
            } else {
                None
            };

            let resolution_t = if let Some((item_id, start_delay, end_delay)) = resolving {
                if *item_id == i {
                    if start_delay.is_above_min() {
                        1.0 - start_delay.get_ratio().as_f32()
                    } else {
                        end_delay.get_ratio().as_f32()
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            };

            self.draw_item(item, resolution_t, model, framebuffer);
        }

        // Entities
        for entity in &model.entities {
            self.draw_entity(entity, framebuffer);
        }

        // Hearts
        for i in 0..model.player.hearts {
            let pos = self.ui_camera.center + vec2(-3.0, 3.3) + vec2(i, 0).as_f32() * 0.6;
            let size = vec2::splat(1.5);
            let target = Aabb2::point(pos).extend_symmetric(size / 2.0);
            self.draw_at_ui(target, &self.assets.sprites.heart, framebuffer);
        }

        {
            // Timer
            let pos = vec2(2.5, 3.3);
            let size = vec2::splat(1.5);
            let icon_target = Aabb2::point(pos).extend_symmetric(size / 2.0);
            self.draw_at_ui(icon_target, &self.assets.sprites.turn_time, framebuffer);
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.ui_camera,
                &draw2d::Text::unit(
                    self.geng.default_font().clone(),
                    format!("{}", model.player.turns_left),
                    Color::try_from("#c9464b").unwrap(),
                )
                .scale_uniform(0.13)
                .align_bounding_box(vec2(0.0, 0.5))
                .translate(pos + vec2(0.3, 0.0)),
            );
        }

        // Overlay
        let overlay_texture = &self.assets.sprites.overlay;
        let size = overlay_texture.size().as_f32();
        let size = size * self.ui_camera.fov / size.y;
        let overlay = Aabb2::point(self.ui_camera.center).extend_symmetric(size / 2.0);
        let mut color = Color::WHITE;
        color.a = 0.5;
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.ui_camera,
            &draw2d::TexturedQuad::colored(overlay, overlay_texture, color),
        );

        self.buttons.clear();
        let text = match &model.phase {
            Phase::Vision => "Select a direction to look at",
            Phase::Map { .. } => {
                // Tile plus
                if model.grid.check_pos_near(cursor_cell_pos) {
                    self.draw_at_grid(
                        cursor_cell_pos.as_f32(),
                        &self.assets.sprites.cell_plus,
                        framebuffer,
                    );
                }

                "Select a position to place a new tile"
            }
            Phase::Select { options } => {
                // Darken the game
                let mut color = Color::BLACK;
                color.a = 0.5;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    &self.ui_camera,
                    &draw2d::Quad::new(overlay, color),
                );

                // Buttons
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

                let mut hint = None;
                for &(item, target) in &self.buttons {
                    let texture = self.assets.sprites.item_texture(item);
                    let background = if target.contains(cursor_ui_pos) {
                        hint = Some(item);
                        &self.assets.sprites.cell
                    } else {
                        &self.assets.sprites.cell_dark
                    };
                    self.draw_at_ui(target, background, framebuffer);
                    self.draw_at_ui(target, texture, framebuffer);
                }

                if let Some(item) = hint {
                    self.draw_item_hint(item, cursor_ui_pos, framebuffer);
                }

                "Select an item"
            }
            _ => "",
        };

        if self.show_inventory {
            self.draw_inventory(model, cursor_ui_pos, framebuffer);
        } else {
            // Text
            self.geng.default_font().draw(
                framebuffer,
                &self.ui_camera,
                text,
                vec2(geng::TextAlign::CENTER, geng::TextAlign::TOP),
                mat3::translate(self.ui_camera.center + vec2(0.0, 0.8 * self.ui_camera.fov / 2.0))
                    * mat3::scale_uniform(0.7),
                Color::BLACK,
            );

            // Item hint
            if let Some((_, item)) = model
                .items
                .iter()
                .find(|(_, item)| item.position == cursor_cell_pos)
            {
                let item = &model.player.items[item.item_id];
                self.draw_item_hint(item.kind, cursor_ui_pos, framebuffer);
            }
        }

        // Inventory button
        self.draw_at_ui(
            self.inventory_button,
            &self.assets.sprites.inventory,
            framebuffer,
        );
    }

    fn draw_inventory(
        &self,
        model: &Model,
        cursor_ui_pos: vec2<f32>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        // Darken the game
        let size = vec2(16.0 / 9.0, 1.0) * self.ui_camera.fov;
        let overlay = Aabb2::point(self.ui_camera.center).extend_symmetric(size / 2.0);
        let mut color = Color::BLACK;
        color.a = 0.5;
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.ui_camera,
            &draw2d::Quad::new(overlay, color),
        );

        // let mut items = Vec::new();
        // for &item in &model.player.items {
        //     if let Some((_, count)) = items.iter_mut().find(|(kind, _)| *kind == item) {
        //         *count += 1;
        //     } else {
        //         items.push((item, 1));
        //     }
        // }

        let size = vec2(1.5, 1.5);
        let mut hint = None;
        for (i, item) in model.player.items.iter().enumerate() {
            let pos = vec2(-1.5, 2.0) + vec2(i, 0).as_f32() * size;
            let target = Aabb2::point(pos).extend_symmetric(size / 2.0);

            if target.contains(cursor_ui_pos) {
                hint = Some(item);
            }

            self.draw_at_ui(target, &self.assets.sprites.cell, framebuffer);
            let texture = self.assets.sprites.item_texture(item.kind);
            self.draw_at_ui(target, texture, framebuffer);

            // if count > 1 {
            //     let pos = pos + vec2(0.3, 0.3) * size;
            //     let radius = 0.15;
            //     let target = Aabb2::point(pos).extend_uniform(radius);
            //     // let mut color = Color::BLACK;
            //     // color.a = 0.5;
            //     // self.geng.draw2d().draw2d(
            //     //     framebuffer,
            //     //     &self.camera,
            //     //     &draw2d::Ellipse::circle(pos, radius * 1.5, color),
            //     // );
            //     self.geng.draw2d().draw2d(
            //         framebuffer,
            //         &self.ui_camera,
            //         &draw2d::Text::unit(
            //             self.geng.default_font().clone(),
            //             format!("x{}", count),
            //             Color::WHITE,
            //         )
            //         .fit_into(target),
            //     );
            // }
        }

        if let Some(item) = hint {
            self.draw_item_hint(item.kind, cursor_ui_pos, framebuffer);
        }
    }

    fn draw_item_hint(
        &self,
        item: ItemKind,
        cursor_ui_pos: vec2<f32>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let mut target =
            Aabb2::point(cursor_ui_pos).extend_positive(self.cell_size * vec2(0.0, 3.5));
        if target.max.y > self.ui_camera.fov / 2.0 {
            target = target.translate(vec2(0.0, self.ui_camera.fov / 2.0 - target.max.y));
        }

        let background = &self.assets.sprites.item_card;
        let size = background.size().as_f32();
        let target = geng_utils::layout::fit_aabb_height(size, target, 0.0);

        self.geng.draw2d().draw2d(
            framebuffer,
            &self.ui_camera,
            &draw2d::TexturedQuad::new(target, background),
        );

        // Name
        let mut text_target = target.extend_uniform(-target.height() * 0.07);
        text_target.min.y = text_target.max.y - text_target.height() / 10.0;
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.ui_camera,
            &draw2d::Text::unit(
                self.geng.default_font().clone(),
                format!("{}", item),
                Color::try_from("#333").unwrap(),
            )
            .align_bounding_box(vec2(0.5, 1.0))
            .fit_into(text_target),
        );

        // Icon
        let icon = self.assets.sprites.item_texture(item);
        let mut icon_target = target.extend_uniform(-target.height() * 0.1);
        icon_target = icon_target.extend_up(-target.height() * 0.09);
        icon_target = icon_target.extend_down(-target.height() * 0.4);
        icon_target =
            geng_utils::layout::fit_aabb(icon.size().as_f32(), icon_target, vec2::splat(0.5));
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.ui_camera,
            &draw2d::TexturedQuad::new(icon_target, icon),
        );

        // Description
        let mut desc_target = target.extend_uniform(-target.height() * 0.1);
        desc_target = desc_target.extend_up(-target.height() * 0.47);
        desc_target = desc_target.extend_down(-target.height() * 0.015);
        desc_target = desc_target.extend_uniform(-desc_target.height() * 0.05);
        // let mut color = Color::GREEN;
        // color.a = 0.5;
        // self.geng.draw2d().draw2d(
        //     framebuffer,
        //     &self.camera,
        //     &draw2d::Quad::new(desc_target, color),
        // );
        let color = Color::try_from("#ffe7cd").unwrap();
        let description = self.assets.items.get_description(item);
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.ui_camera,
            &draw2d::Text::unit(self.geng.default_font().clone(), description, color)
                .align_bounding_box(vec2(0.0, 1.0))
                .fit_into(desc_target),
        );
    }

    fn draw_item(
        &self,
        board_item: &BoardItem,
        resolution_t: f32,
        model: &Model,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let item = &model.player.items[board_item.item_id];

        let texture = self.assets.sprites.item_texture(item.kind);
        // TODO: place the shadow
        // self.draw_at(item.position, &self.assets.sprites.item_shadow, framebuffer);
        let offset = vec2(0.0, crate::util::smoothstep(resolution_t) * 0.2);
        self.draw_at_grid(board_item.position.as_f32() + offset, texture, framebuffer);

        // Damage value
        if let Some(damage) = item.temp_stats.damage {
            let pos = (board_item.position.as_f32() + vec2(0.3, 0.3)) * self.cell_size;
            let target = Aabb2::point(pos).extend_uniform(0.06);
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.world_camera,
                &draw2d::TexturedQuad::new(
                    Aabb2::point(pos).extend_uniform(0.14),
                    &self.assets.sprites.weapon_damage,
                ),
            );
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.world_camera,
                &draw2d::Text::unit(
                    self.geng.default_font().clone(),
                    format!("{}", damage),
                    Color::try_from("#424242").unwrap(),
                )
                .fit_into(target),
            );
        }
    }

    fn draw_at_grid(
        &self,
        position: vec2<f32>,
        texture: &ugli::Texture,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let position = position * self.cell_size;
        self.draw_at(
            Aabb2::point(position).extend_symmetric(self.cell_size / 2.0),
            texture,
            &self.world_camera,
            framebuffer,
        )
    }

    fn draw_at_ui(
        &self,
        target: Aabb2<f32>,
        texture: &ugli::Texture,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_at(target, texture, &self.ui_camera, framebuffer)
    }

    fn draw_at(
        &self,
        target: Aabb2<f32>,
        texture: &ugli::Texture,
        camera: &Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let size =
            texture.size().as_f32() * target.size() / self.assets.sprites.cell.size().as_f32();
        let target = Aabb2::point(target.center()).extend_symmetric(size / 2.0);
        self.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::TexturedQuad::new(target, texture),
        );
    }

    fn draw_entity(&self, entity: &Entity, framebuffer: &mut ugli::Framebuffer) {
        let texture = match entity.fraction {
            Fraction::Player => &self.assets.sprites.player,
            Fraction::Enemy => {
                let pos = (entity.position.as_f32() + vec2(0.3, 0.3)) * self.cell_size;
                let target = Aabb2::point(pos).extend_uniform(0.06);
                self.geng.draw2d().draw2d(
                    framebuffer,
                    &self.world_camera,
                    &draw2d::TexturedQuad::new(
                        Aabb2::point(pos).extend_uniform(0.14),
                        &self.assets.sprites.enemy_health,
                    ),
                );
                self.geng.draw2d().draw2d(
                    framebuffer,
                    &self.world_camera,
                    &draw2d::Text::unit(
                        self.geng.default_font().clone(),
                        format!("{}", entity.health.value()),
                        Color::try_from("#424242").unwrap(),
                    )
                    .fit_into(target),
                );

                &self.assets.sprites.enemy
            }
        };

        self.draw_at_grid(entity.position.as_f32(), texture, framebuffer)
    }

    fn draw_cell(
        &self,
        position: vec2<Coord>,
        tile_light: TileLight,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let texture = match tile_light {
            TileLight::Normal => &self.assets.sprites.cell,
            TileLight::Dark => &self.assets.sprites.cell_dark,
            TileLight::Light => &self.assets.sprites.cell_light,
        };
        self.draw_at_grid(position.as_f32(), texture, framebuffer)
    }
}
