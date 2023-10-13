use crate::prelude::*;

pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
    items: Rc<ItemAssets>,
    pub ui_camera: Camera2d,
    pub world_camera: Camera2d,
    pub cell_size: vec2<f32>,
    pub buttons: Vec<(ItemKind, Aabb2<f32>)>,
    pub skip_turn_button: Aabb2<f32>,
    pub skip_item_button: Aabb2<f32>,
    pub reroll_button: Aabb2<f32>,
    pub inventory_button: Aabb2<f32>,
    pub show_inventory: bool,
    pub retry_button: Aabb2<f32>,
}

#[derive(Debug)]
enum TileLight {
    Normal,
    Dark,
    Light,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, items: &Rc<ItemAssets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            items: items.clone(),
            ui_camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            world_camera: Camera2d {
                center: vec2(0.0, 0.5),
                rotation: Angle::ZERO,
                fov: 7.0,
            },
            cell_size: vec2(1.0, 1.0),
            buttons: Vec::new(),
            skip_turn_button: Aabb2::point(vec2(7.0, -1.0))
                .extend_symmetric(vec2::splat(1.5) / 2.0),
            skip_item_button: Aabb2::point(vec2(0.75, -3.0))
                .extend_symmetric(vec2::splat(1.5) / 2.0),
            reroll_button: Aabb2::point(vec2(-0.75, -3.0)).extend_symmetric(vec2::splat(1.5) / 2.0),
            inventory_button: Aabb2::point(vec2(7.0, 1.0)).extend_symmetric(vec2::splat(1.5) / 2.0),
            retry_button: Aabb2::point(vec2(0.0, -3.0)).extend_symmetric(vec2::splat(1.5) / 2.0),
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
        if let Phase::GameOver = model.phase {
            self.draw_game_over(model, cursor_ui_pos, framebuffer);
            return;
        }

        // Tiles
        for &pos in &model.grid.tiles {
            let light = match model.phase {
                Phase::Vision | Phase::PostVision { .. } | Phase::Select { .. } => {
                    if model.visible_tiles.contains(&pos) {
                        TileLight::Light
                    } else {
                        TileLight::Normal
                    }
                }
                Phase::LevelStarting { .. } | Phase::Night { .. } => {
                    // Crossfade
                    let t = model.get_light_level(pos);
                    let t = crate::util::smoothstep(t);
                    let mut color = Color::WHITE;
                    color.a = 1.0 - t;
                    self.draw_at_grid(
                        pos.as_f32(),
                        Angle::ZERO,
                        &self.assets.sprites.cell_dark,
                        color,
                        framebuffer,
                    );
                    color.a = t;
                    self.draw_at_grid(
                        pos.as_f32(),
                        Angle::ZERO,
                        &self.assets.sprites.cell,
                        color,
                        framebuffer,
                    );
                    continue;
                }
                _ => {
                    if model.grid.fractured.contains(&pos) {
                        TileLight::Dark
                    } else if let Phase::Portal = model.phase {
                        // Highlight magic items
                        if model.items.iter().any(|(_, item)| {
                            item.position == pos
                                && ItemRef::Category(ItemCategory::Magic)
                                    .check(&model.player.items[item.item_id].kind)
                        }) {
                            TileLight::Light
                        } else {
                            TileLight::Normal
                        }
                    } else if model.grid.lights.contains_key(&pos) {
                        TileLight::Light
                    } else {
                        TileLight::Normal
                    }
                }
            };
            self.draw_cell(pos, light, framebuffer);
        }

        // Entities
        for (id, entity) in &model.entities {
            self.draw_entity(id, entity, model, framebuffer);
        }

        // Items
        for (i, item) in &model.items {
            // let resolving = if let Phase::Passive {
            //     start_delay,
            //     end_delay,
            // } = &model.phase
            // {
            //     item_queue
            //         .last()
            //         .and_then(|&item_id| (item_id == i).then_some((*start_delay, *end_delay)))
            // } else {
            //     None
            // };

            // let resolving = resolving
            //     .or_else(|| {
            //         model.animations.iter().find_map(|(_, anim)| {
            //             if let AnimationKind::UseActive { item_id, .. } = anim.kind {
            //                 if item_id == i {
            //                     return Some((anim.time, Lifetime::new_max(R32::ONE)));
            //                 }
            //             }
            //             None
            //         })
            //     })
            //     .or_else(|| {
            //         model.ending_animations.iter().find_map(|anim| {
            //             if let AnimationKind::UseActive { item_id, .. } = anim.kind {
            //                 if item_id == i {
            //                     return Some((Lifetime::new_max(R32::ONE), anim.time));
            //                 }
            //             }
            //             None
            //         })
            //     });

            // let resolution_t = if let Some((start_delay, end_delay)) = resolving {
            //     if start_delay.is_above_min() {
            //         1.0 - start_delay.get_ratio().as_f32()
            //     } else {
            //         end_delay.get_ratio().as_f32()
            //     }
            // } else {
            //     0.0
            // };

            // TODO
            let resolution_t = 0.0;

            self.draw_item(i, item, resolution_t, model, framebuffer);
        }

        self.draw_animations(model, framebuffer);

        // Hearts
        for i in 0..model.player.hearts {
            let pos = self.ui_camera.center + vec2(-6.7, 1.7) + vec2(i, 0).as_f32() * 0.6;
            let size = vec2::splat(1.5);
            let target = Aabb2::point(pos).extend_symmetric(size / 2.0);
            self.draw_at_ui(target, &self.assets.sprites.heart, framebuffer);
        }

        {
            // Timer
            let pos = vec2(-6.3, 0.1);
            let size = vec2::splat(1.5);
            let icon_target = Aabb2::point(pos).extend_symmetric(size / 2.0);
            self.draw_at_ui(icon_target, &self.assets.sprites.turn_time, framebuffer);
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.ui_camera,
                &draw2d::Text::unit(
                    self.assets.font.clone(),
                    format!("{}", model.player.turns_left),
                    Color::try_from("#c9464b").unwrap(),
                )
                .scale_uniform(0.15)
                .align_bounding_box(vec2(0.0, 0.5))
                .translate(pos + vec2(0.3, 0.0)),
            );
        }

        {
            let height = 0.4;

            // Level
            self.assets.font.draw(
                framebuffer,
                &self.ui_camera,
                &format!("LEVEL {}", model.level),
                vec2::splat(geng::TextAlign::LEFT),
                mat3::translate(vec2(-6.8, 0.7))
                    * mat3::scale_uniform(height)
                    * mat3::translate(vec2(0.0, -0.25)),
                Color::try_from("#c03d43").unwrap(),
            );

            // Score
            self.assets.font.draw(
                framebuffer,
                &self.ui_camera,
                &format!("SCORE {}", model.score),
                vec2::splat(geng::TextAlign::RIGHT),
                mat3::translate(vec2(7.5, -3.0))
                    * mat3::scale_uniform(height)
                    * mat3::translate(vec2(0.0, -0.25)),
                Color::try_from("#7a7a7a").unwrap(),
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
            Phase::GameOver => "Game over",
            Phase::Night { .. } | Phase::LevelStarting { .. } => "Night",
            Phase::Player => {
                // Skip button
                self.draw_button(
                    self.skip_turn_button,
                    &self.assets.sprites.skip_button,
                    cursor_ui_pos,
                    framebuffer,
                );

                "Day"
            }
            Phase::Passive { .. } => "Day",
            Phase::Portal => "Select a magic item",
            Phase::Vision => "Select a direction to look at",
            Phase::PostVision { .. } => "Night",
            Phase::LevelFinished { win, .. } => {
                if *win {
                    "Level completed"
                } else {
                    "You ran out of turns"
                }
            }
            Phase::Map { .. } => {
                // Tile plus
                for pos in model.grid.outside_tiles() {
                    self.draw_at_grid(
                        pos.as_f32(),
                        Angle::ZERO,
                        &self.assets.sprites.cell_plus,
                        Color::WHITE,
                        framebuffer,
                    );
                }

                "Select a position to place a new tile"
            }
            Phase::Select { .. } => {
                if !self.show_inventory {
                    // Darken the game
                    let mut color = Color::BLACK;
                    color.a = 0.5;
                    self.geng.draw2d().draw2d(
                        framebuffer,
                        &self.ui_camera,
                        &draw2d::Quad::new(overlay, color),
                    );
                }

                "Select an item"
            }
        };

        {
            // Text
            let pos = self.ui_camera.center + vec2(0.0, 0.8 * self.ui_camera.fov / 2.0);
            let size = vec2(
                3.0 * self.assets.sprites.panel.size().as_f32().aspect(),
                3.0,
            );
            let target = Aabb2::point(pos).extend_symmetric(size / 2.0);
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.ui_camera,
                &draw2d::TexturedQuad::new(target, &self.assets.sprites.panel),
            );
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.ui_camera,
                &draw2d::Text::unit(
                    self.assets.font.clone(),
                    text,
                    Color::try_from("#ffe7cd").unwrap(),
                )
                .fit_into(Aabb2::point(target.center()).extend_symmetric(vec2(3.0, 0.5) / 2.0)),
            );
        }

        if self.show_inventory {
            self.draw_inventory(model, cursor_ui_pos, framebuffer);
        } else if let Phase::Select { options, .. } = &model.phase {
            // Buttons
            let size = 2.0;
            let offset = size * (options.len() as f32 - 1.0) / 2.0;
            self.buttons = options
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let pos = vec2(i as f32 * size - offset, 0.0);
                    let target = Aabb2::point(pos).extend_symmetric(vec2::splat(size) / 2.0 * 0.9);
                    (item.clone(), target)
                })
                .collect();

            let mut hint = None;
            for (item, target) in &self.buttons {
                // TODO: default texture
                let texture = self.items.get_texture(&item.config.name);
                let background = if target.contains(cursor_ui_pos) {
                    hint = Some(item);
                    &self.assets.sprites.cell
                } else {
                    &self.assets.sprites.cell_dark
                };
                self.draw_at_ui(*target, background, framebuffer);
                self.draw_at_ui(*target, texture, framebuffer);
            }

            if model.player.refreshes > 0 {
                self.draw_button(
                    self.reroll_button,
                    &self.assets.sprites.reroll_button,
                    cursor_ui_pos,
                    framebuffer,
                )
            };
            self.draw_button(
                self.skip_item_button,
                &self.assets.sprites.skip_button,
                cursor_ui_pos,
                framebuffer,
            );

            if let Some(item) = hint {
                self.draw_item_hint(item, cursor_ui_pos, framebuffer);
            }
        } else if let Some((_, item)) = model
            .items
            .iter()
            .find(|(_, item)| item.position == cursor_cell_pos)
        {
            // Item hint
            let item = &model.player.items[item.item_id];
            self.draw_item_hint(&item.kind, cursor_ui_pos, framebuffer);
        }

        // Inventory button
        self.draw_button(
            self.inventory_button,
            &self.assets.sprites.inventory,
            cursor_ui_pos,
            framebuffer,
        );

        let transition_t = match model.phase {
            Phase::LevelStarting { timer } => timer.get_ratio().as_f32(),
            Phase::LevelFinished { timer, .. } => {
                let t = 0.8;
                1.0 - (timer.get_ratio().as_f32().max(t) - t) / (1.0 - t)
            }
            _ => 0.0,
        };
        let mut color = Color::BLACK;
        color.a = crate::util::smoothstep(transition_t);
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.ui_camera,
            &draw2d::Quad::new(overlay, color),
        );
    }

    fn draw_game_over(
        &self,
        model: &Model,
        cursor_ui_pos: vec2<f32>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        {
            // Text
            let pos = self.ui_camera.center + vec2(0.0, 2.5);
            let size = vec2(
                3.0 * self.assets.sprites.panel.size().as_f32().aspect(),
                3.0,
            );
            let target = Aabb2::point(pos).extend_symmetric(size / 2.0);
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.ui_camera,
                &draw2d::TexturedQuad::new(target, &self.assets.sprites.panel),
            );
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.ui_camera,
                &draw2d::Text::unit(
                    self.assets.font.clone(),
                    "Game over",
                    Color::try_from("#ffe7cd").unwrap(),
                )
                .fit_into(Aabb2::point(target.center()).extend_symmetric(vec2(3.0, 0.5) / 2.0)),
            );
        }

        {
            let x = -1.5;
            let height = 0.8;

            // Level
            let y = 0.5;
            let target = Aabb2::point(vec2(x, y)).extend_uniform(1.0);
            self.draw_at_ui(target, &self.assets.sprites.enemy, framebuffer);

            let target = vec2(x + 0.8, y);
            self.assets.font.draw(
                framebuffer,
                &self.ui_camera,
                &format!("Level  {}", model.level),
                vec2::splat(geng::TextAlign::LEFT),
                mat3::translate(target)
                    * mat3::scale_uniform(height)
                    * mat3::translate(vec2(0.0, -0.25)),
                Color::try_from("#c9464b").unwrap(),
            );

            // Score
            let y = y - 1.5;
            let target = Aabb2::point(vec2(x, y)).extend_uniform(1.0);
            self.draw_at_ui(target, &self.assets.sprites.player, framebuffer);

            let target = vec2(x + 0.8, y);
            self.assets.font.draw(
                framebuffer,
                &self.ui_camera,
                &format!("Score {}", model.score),
                vec2::splat(geng::TextAlign::LEFT),
                mat3::translate(target)
                    * mat3::scale_uniform(height)
                    * mat3::translate(vec2(0.0, -0.25)),
                Color::try_from("#ffcd6c").unwrap(),
            );
        }

        {
            // Retry
            self.draw_button(
                self.retry_button,
                &self.assets.sprites.reroll_button,
                cursor_ui_pos,
                framebuffer,
            );

            // Exit
            // self.draw_at_ui(
            //     self.exit_button,
            //     &self.assets.sprites.exit_button,
            //     framebuffer,
            // );
        }

        {
            // Overlay
            let overlay_texture = &self.assets.sprites.overlay;
            let size = overlay_texture.size().as_f32();
            let size = size * self.ui_camera.fov / size.y;
            let overlay = Aabb2::point(self.ui_camera.center).extend_symmetric(size / 2.0);

            self.geng.draw2d().draw2d(
                framebuffer,
                &self.ui_camera,
                &draw2d::TexturedQuad::new(overlay, &self.assets.sprites.outer_square),
            );

            let mut color = Color::WHITE;
            color.a = 0.5;
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.ui_camera,
                &draw2d::TexturedQuad::colored(overlay, overlay_texture, color),
            );
        }
    }

    fn draw_animations(&self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        for (_, animation) in &model.animations {
            if animation.time.is_max() {
                // Not started yet
                continue;
            }
            self.draw_animation(
                animation,
                1.0 - animation.time.get_ratio().as_f32(),
                1.0,
                model,
                framebuffer,
            );
        }
        for animation in &model.ending_animations {
            self.draw_animation(
                animation,
                1.0,
                animation.time.get_ratio().as_f32(),
                model,
                framebuffer,
            );
        }
    }

    fn draw_animation(
        &self,
        animation: &Animation,
        start_t: f32, // 0.0 -> 1.0
        end_t: f32,   // 1.0 -> 0.0
        model: &Model,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match &animation.kind {
            AnimationKind::ItemDeath { pos, .. } if start_t == 1.0 => {
                let mut color = Color::WHITE;
                color.a = crate::util::smoothstep(end_t);
                self.draw_at_grid(
                    pos.as_f32(),
                    Angle::ZERO,
                    &self.assets.sprites.destroy_effect,
                    color,
                    framebuffer,
                );
            }
            AnimationKind::EntityDeath { pos, .. } if start_t == 1.0 => {
                let mut color = Color::WHITE;
                color.a = crate::util::smoothstep(end_t);
                self.draw_at_grid(
                    pos.as_f32(),
                    Angle::ZERO,
                    &self.assets.sprites.enemy_death,
                    color,
                    framebuffer,
                );
            }
            AnimationKind::Damage {
                from,
                target,
                damage,
            } if end_t == 1.0 => {
                let from = (from.as_f32() + vec2(0.3, 0.3)) * self.cell_size;
                let target =
                    (model.entities[*target].position.as_f32() + vec2(0.3, 0.3)) * self.cell_size;
                let t = crate::util::smoothstep(start_t);
                let pos = from + (target - from) * t;

                // let pos = (board_item.position.as_f32() + vec2(0.3, 0.3)) * self.cell_size;
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
                        self.assets.font.clone(),
                        format!("{}", damage),
                        Color::try_from("#424242").unwrap(),
                    )
                    .fit_into(target),
                );
            }
            AnimationKind::Bonus {
                from,
                target,
                bonus,
                ..
            } if end_t == 1.0 => {
                let from = from.as_f32() * self.cell_size;
                let target =
                    (model.items[*target].position.as_f32() + vec2(0.3, 0.3)) * self.cell_size;
                let t = crate::util::smoothstep(start_t);
                let pos = from + (target - from) * t;

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
                        self.assets.font.clone(),
                        format!("{}", bonus.damage.unwrap_or_default()),
                        Color::try_from("#424242").unwrap(),
                    )
                    .fit_into(target),
                );
            }
            _ => (),
        }
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

        let mut items = Vec::new();
        for (i, item) in &model.player.items {
            // if let Some((_, count)) = items.iter_mut().find(|(kind, _)| *kind == item) {
            //     *count += 1;
            // } else {
            //     items.push((item, 1));
            // }
            items.push((i, item));
        }
        items.sort_by_key(|&(i, _)| i); // TODO: sort by age

        let size = vec2(1.5, 1.5);
        let mut hint = None;
        let row_max = 5;
        let offset = items.len().min(row_max).saturating_sub(1) as f32 / 2.0;
        for (i, (_, item)) in items.iter().enumerate() {
            let x = i % row_max;
            let y = i / row_max;
            let pos = vec2(0.0, 2.0) + vec2(x as f32 - offset, -(y as f32)) * size;
            let target = Aabb2::point(pos).extend_symmetric(size / 2.0);

            if target.contains(cursor_ui_pos) {
                hint = Some(item);
            }

            self.draw_at_ui(target, &self.assets.sprites.cell, framebuffer);
            let texture = self.items.get_texture(&item.kind.config.name);
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
            self.draw_item_hint(&item.kind, cursor_ui_pos, framebuffer);
        }
    }

    fn draw_item_hint(
        &self,
        item: &ItemKind,
        cursor_ui_pos: vec2<f32>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let mut target =
            Aabb2::point(cursor_ui_pos).extend_positive(self.cell_size * vec2(0.0, 4.5));
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
                self.assets.font.clone(),
                format!("{}", item.config.name),
                Color::try_from("#333").unwrap(),
            )
            .align_bounding_box(vec2(0.5, 1.0))
            .fit_into(text_target),
        );

        // Icon
        let icon = self.items.get_texture(&item.config.name);
        let mut icon_target = target.extend_uniform(-target.height() * 0.1);
        icon_target = icon_target.extend_up(-target.height() * 0.09);
        icon_target = icon_target.extend_down(-target.height() * 0.4);
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.ui_camera,
            &draw2d::TexturedQuad::new(
                geng_utils::layout::fit_aabb(icon.size().as_f32(), icon_target, vec2::splat(0.5)),
                icon,
            ),
        );

        {
            // Categories
            let positions = [vec2(0.0, 0.0), vec2(1.0, 0.0)];
            let size = icon_target.height() / 5.0; // / 12.0;
            let icon_target = icon_target.extend_uniform(-size / 2.0);
            for (i, &category) in item
                .config
                .categories
                .iter()
                .enumerate()
                .take(positions.len())
            {
                let alignment = positions[i];
                let target = geng_utils::layout::aabb_pos(icon_target, alignment);
                self.assets.font.draw(
                    framebuffer,
                    &self.ui_camera,
                    &format!("{:?}", category),
                    alignment.map(geng::TextAlign),
                    mat3::translate(target)
                        * mat3::scale_uniform(size)
                        * mat3::translate(vec2(0.0, -0.25)),
                    self.assets.get_category_color(category),
                );
                // self.geng.draw2d().draw2d(
                //     framebuffer,
                //     &self.ui_camera,
                //     &draw2d::Text::unit(
                //         self.geng.default_font().clone(),
                //         format!("{:?}", category),
                //         self.assets.get_category_color(category),
                //     )
                //     .align_bounding_box(alignment)
                //     .scale_uniform(size)
                //     .translate(target),
                // );
            }
        }

        // Description
        let mut desc_target = target.extend_uniform(-target.height() * 0.1);
        desc_target = desc_target.extend_up(-target.height() * 0.49);
        desc_target = desc_target.extend_down(-target.height() * 0.015);
        desc_target = desc_target.extend_uniform(-desc_target.height() * 0.05);

        let color = Color::try_from("#ffe7cd").unwrap();
        let description = self
            .items
            .get(&item.config.name)
            .description
            .as_deref()
            .unwrap_or("<description missing>");

        let mut lines = Vec::new();
        for source_line in description.lines() {
            let mut line = String::new();
            for word in source_line.split_whitespace() {
                if line.is_empty() {
                    line += word;
                    continue;
                }
                if line.len() + word.len() > 30 {
                    lines.push(line);
                    line = word.to_string();
                } else {
                    line += " ";
                    line += word;
                }
            }
            if !line.is_empty() {
                lines.push(line);
            }
        }

        let font_size = 0.15;
        for (i, line) in lines.into_iter().enumerate() {
            let position = desc_target.top_left() - vec2(0.0, i as f32) * font_size * 1.2;
            self.assets.font.draw(
                framebuffer,
                &self.ui_camera,
                &line,
                vec2::splat(geng::TextAlign::LEFT),
                mat3::translate(position)
                    * mat3::scale_uniform(font_size)
                    * mat3::translate(vec2(0.0, -0.25)),
                color,
            );
        }

        // self.geng.draw2d().draw2d(
        //     framebuffer,
        //     &self.ui_camera,
        //     &draw2d::Text::unit(self.assets.font.clone(), description, color)
        //         .align_bounding_box(vec2(0.0, 1.0))
        //         .fit_into(desc_target),
        // );
    }

    fn draw_item(
        &self,
        id: Id,
        board_item: &BoardItem,
        resolution_t: f32,
        model: &Model,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let item = &model.player.items[board_item.item_id];

        let alpha = model.get_light_level(board_item.position);
        let alpha = crate::util::smoothstep(alpha);
        let mut color = Color::WHITE;
        color.a = alpha;

        let mut position = board_item.position.as_f32();
        let target = model
            .animations
            .iter()
            .find_map(|(_, anim)| match anim.kind {
                AnimationKind::MoveItem {
                    item_id,
                    target_pos,
                } if item_id == id => Some((target_pos, 1.0 - anim.time.get_ratio().as_f32())),
                _ => None,
            });
        if let Some((target, t)) = target {
            let t = crate::util::smoothstep(t);
            position = position + (target.as_f32() - position) * t;
        }

        let texture = self.items.get_texture(&item.kind.config.name);
        // TODO: place the shadow
        // self.draw_at(item.position, &self.assets.sprites.item_shadow, framebuffer);
        let offset = vec2(0.0, crate::util::smoothstep(resolution_t) * 0.2);
        self.draw_at_grid(position + offset, Angle::ZERO, texture, color, framebuffer);

        if !board_item.used {
            // Damage value
            if let Some(damage) = item.current_stats().damage {
                let pos = (position + vec2(0.3, 0.3)) * self.cell_size;
                let target = Aabb2::point(pos).extend_uniform(0.06);
                self.geng.draw2d().draw2d(
                    framebuffer,
                    &self.world_camera,
                    &draw2d::TexturedQuad::colored(
                        Aabb2::point(pos).extend_uniform(0.14),
                        &self.assets.sprites.weapon_damage,
                        color,
                    ),
                );
                let mut color = Color::try_from("#424242").unwrap();
                color.a = alpha;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    &self.world_camera,
                    &draw2d::Text::unit(self.assets.font.clone(), format!("{}", damage), color)
                        .fit_into(target),
                );
            }
        }
    }

    fn draw_at_grid(
        &self,
        position: vec2<f32>,
        rotation: Angle<f32>,
        texture: &ugli::Texture,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let position = position * self.cell_size;
        self.draw_at(
            Aabb2::point(position).extend_symmetric(self.cell_size / 2.0),
            rotation,
            texture,
            color,
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
        self.draw_at(
            target,
            Angle::ZERO,
            texture,
            Color::WHITE,
            &self.ui_camera,
            framebuffer,
        )
    }

    fn draw_at(
        &self,
        target: Aabb2<f32>,
        rotation: Angle<f32>,
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
            &draw2d::TexturedQuad::colored(
                Aabb2::ZERO.extend_symmetric(target.size() / 2.0),
                texture,
                color,
            )
            .rotate(rotation)
            .translate(target.center()),
        );
    }

    fn draw_entity(
        &self,
        id: Id,
        entity: &Entity,
        model: &Model,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let alpha = model.get_light_level(entity.position);
        let alpha = crate::util::smoothstep(alpha);
        let mut color = Color::WHITE;
        color.a = alpha;

        let mut position = entity.position.as_f32();
        let target = model
            .animations
            .iter()
            .find_map(|(_, anim)| match anim.kind {
                AnimationKind::MoveEntity {
                    entity_id,
                    target_pos,
                } if entity_id == id => Some((target_pos, 1.0 - anim.time.get_ratio().as_f32())),
                _ => None,
            });
        if let Some((target, t)) = target {
            let t = crate::util::smoothstep(t);
            position = position + (target.as_f32() - position) * t;
        }

        let texture = match entity.fraction {
            Fraction::Player => {
                if let Phase::Vision
                | Phase::PostVision { .. }
                | Phase::Select { .. }
                | Phase::Night { .. } = model.phase
                {
                    if entity.look_dir != vec2::ZERO {
                        let rotation = entity.look_dir.as_f32().arg();
                        self.draw_at_grid(
                            position + entity.look_dir.as_f32() * 0.35,
                            rotation,
                            &self.assets.sprites.player_vision,
                            color,
                            framebuffer,
                        )
                    };
                }

                &self.assets.sprites.player
            }
            Fraction::Enemy => {
                let pos = (position + vec2(0.3, 0.3)) * self.cell_size;
                let target = Aabb2::point(pos).extend_uniform(0.06);
                self.geng.draw2d().draw2d(
                    framebuffer,
                    &self.world_camera,
                    &draw2d::TexturedQuad::colored(
                        Aabb2::point(pos).extend_uniform(0.14),
                        &self.assets.sprites.enemy_health,
                        color,
                    ),
                );
                let mut color = Color::try_from("#424242").unwrap();
                color.a = alpha;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    &self.world_camera,
                    &draw2d::Text::unit(
                        self.assets.font.clone(),
                        format!("{}", entity.health.value()),
                        color,
                    )
                    .fit_into(target),
                );

                &self.assets.sprites.enemy
            }
        };

        self.draw_at_grid(position, Angle::ZERO, texture, color, framebuffer);

        if let EntityKind::Player = entity.kind {
            // Draw the remaining moves as circles
            let moves = model.player.moves_left.min(6);
            let offset = (moves as f32 - 1.0) / 2.0;
            for i in 0..moves {
                let pos = (position + vec2(0.0, -0.27)) * self.cell_size
                    + vec2(i as f32 - offset, 0.0) * 0.1;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    &self.world_camera,
                    &draw2d::Ellipse::circle(pos, 0.03, Color::try_from("#ffcd6c").unwrap()),
                );
            }
        }
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
        self.draw_at_grid(
            position.as_f32(),
            Angle::ZERO,
            texture,
            Color::WHITE,
            framebuffer,
        )
    }

    fn draw_button(
        &self,
        button: Aabb2<f32>,
        texture: &ugli::Texture,
        cursor_ui_pos: vec2<f32>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let color = if button.contains(cursor_ui_pos) {
            Color::WHITE.map_rgb(|x| x * 1.2)
        } else {
            Color::WHITE
        };
        self.draw_at(
            button,
            Angle::ZERO,
            texture,
            color,
            &self.ui_camera,
            framebuffer,
        )
    }
}
