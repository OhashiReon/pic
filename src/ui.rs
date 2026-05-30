use crate::core::{AppCore, ImageSource, ImageState};
use crate::services::FileDialogService;
use eframe::egui;

pub struct MoshyaApp {
    pub core: AppCore,
    pub file_dialog: Box<dyn FileDialogService>,
    texture: Option<egui::TextureHandle>,
}

impl MoshyaApp {
    pub fn new(core: AppCore, file_dialog: Box<dyn FileDialogService>) -> Self {
        Self {
            core,
            file_dialog,
            texture: None,
        }
    }

    fn sync_texture(&mut self, ctx: &egui::Context) {
        if let ImageState::Loaded {
            source: ImageSource::Raw(raw),
            ..
        } = &self.core.state
        {
            let needs_update = match &self.texture {
                Some(t) => t.name() != "clipboard_image", // Simple check or check raw data
                None => true,
            };

            if needs_update {
                let size = [raw.width as usize, raw.height as usize];
                let pixels = egui::ColorImage::from_rgba_unmultiplied(size, &raw.bytes);
                self.texture = Some(ctx.load_texture("clipboard_image", pixels, Default::default()));
            }
        } else {
            self.texture = None;
        }
    }
}

impl eframe::App for MoshyaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.sync_texture(ctx);

        // --- FACT-BASED PASTE TRIGGER ---
        let mut trigger_paste = false;
        ctx.input(|i| {
            for event in &i.events {
                if matches!(event, egui::Event::Paste(_)) {
                    trigger_paste = true;
                }
                if let egui::Event::Key {
                    key: egui::Key::V,
                    pressed: false,
                    modifiers,
                    ..
                } = event
                {
                    if modifiers.ctrl || modifiers.command {
                        trigger_paste = true;
                    }
                }
            }
        });

        if trigger_paste {
            self.core.handle_paste();
        }

        // Force repaints while loading
        if let ImageState::Loaded {
            dimensions: None, ..
        } = &self.core.state
        {
            ctx.request_repaint();
        }

        // Top Panel: Toolbar
        egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                // File Operations
                if ui.button("📂").on_hover_text("Open File").clicked() {
                    if let Some(path) = self.file_dialog.pick_file() {
                        self.core.handle_open_file(path);
                    }
                }
                if ui.button("📋").on_hover_text("Paste (Ctrl+V)").clicked() {
                    self.core.handle_paste();
                }

                ui.separator();

                // URL Input
                ui.add(
                    egui::TextEdit::singleline(&mut self.core.web_url)
                        .hint_text("Image URL...")
                        .desired_width(150.0),
                );
                if ui.button("🌐").on_hover_text("Load URL").clicked() {
                    self.core.handle_url_load();
                }

                ui.separator();

                // View Controls
                let fit_text = if self.core.fit_to_window {
                    "Actual Size"
                } else {
                    "Fit Window"
                };
                if ui.button(fit_text).clicked() {
                    self.core.fit_to_window = !self.core.fit_to_window;
                }

                let pin_text = if self.core.always_on_top {
                    "Unpin"
                } else {
                    "Pin"
                };
                if ui.button(pin_text).clicked() {
                    self.core.always_on_top = !self.core.always_on_top;
                    let level = if self.core.always_on_top {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    };
                    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                }

                // Right-aligned settings toggle
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.toggle_value(&mut self.core.show_settings_panel, "⚙ Settings");
                });
            });
        });

        // Right Panel: Settings Sidebar
        if self.core.show_settings_panel {
            egui::SidePanel::right("settings_sidebar")
                .resizable(false)
                .default_width(200.0)
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.heading("Settings");
                    ui.separator();

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Display").strong());
                    ui.horizontal(|ui| {
                        ui.label("Opacity:");
                        ui.add(
                            egui::Slider::new(&mut self.core.opacity, 0.1..=1.0).show_value(false),
                        );
                    });

                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("Grid").strong());
                    ui.checkbox(&mut self.core.show_grid, "Enable Grid");

                    if self.core.show_grid {
                        ui.indent("grid_settings", |ui| {
                            egui::Grid::new("grid_config")
                                .num_columns(2)
                                .spacing([10.0, 8.0])
                                .show(ui, |ui| {
                                    ui.label("X Columns:");
                                    ui.add(
                                        egui::DragValue::new(&mut self.core.grid_cols)
                                            .range(1..=100),
                                    );
                                    ui.end_row();

                                    ui.label("Y Rows:");
                                    ui.add(
                                        egui::DragValue::new(&mut self.core.grid_rows)
                                            .range(1..=100),
                                    );
                                    ui.end_row();

                                    ui.label("Color:");
                                    egui::ComboBox::from_id_salt("grid_color")
                                        .selected_text(match self.core.grid_color {
                                            crate::core::GridColor::Red => "Red",
                                            crate::core::GridColor::Cyan => "Cyan",
                                            crate::core::GridColor::Green => "Green",
                                            crate::core::GridColor::White => "White",
                                            crate::core::GridColor::Black => "Black",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut self.core.grid_color,
                                                crate::core::GridColor::Red,
                                                "Red",
                                            );
                                            ui.selectable_value(
                                                &mut self.core.grid_color,
                                                crate::core::GridColor::Cyan,
                                                "Cyan",
                                            );
                                            ui.selectable_value(
                                                &mut self.core.grid_color,
                                                crate::core::GridColor::Green,
                                                "Green",
                                            );
                                            ui.selectable_value(
                                                &mut self.core.grid_color,
                                                crate::core::GridColor::White,
                                                "White",
                                            );
                                            ui.selectable_value(
                                                &mut self.core.grid_color,
                                                crate::core::GridColor::Black,
                                                "Black",
                                            );
                                        });
                                    ui.end_row();

                                    ui.label("Subdivision:");
                                    ui.add(
                                        egui::DragValue::new(&mut self.core.grid_subdivision)
                                            .range(1..=10),
                                    );
                                    ui.end_row();

                                    ui.label("Thick Line:");
                                    ui.add(
                                        egui::DragValue::new(&mut self.core.thick_line_width)
                                            .range(0.1..=10.0)
                                            .speed(0.1),
                                    );
                                    ui.end_row();

                                    ui.label("Thin Line:");
                                    ui.add(
                                        egui::DragValue::new(&mut self.core.thin_line_width)
                                            .range(0.1..=10.0)
                                            .speed(0.1),
                                    );
                                    ui.end_row();
                                });
                        });
                    }
                });
        }

        // Bottom Panel: Status Bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| match &self.core.state {
                ImageState::Loaded {
                    dimensions: Some((w, h)),
                    ..
                } => {
                    ui.label(format!("{}x{}", w, h));
                }
                ImageState::Loaded {
                    dimensions: None, ..
                } => {
                    ui.label("Loading...");
                }
                ImageState::Idle => {
                    ui.label("No image loaded");
                }
                ImageState::Failed(e) => {
                    ui.label(
                        egui::RichText::new(format!("Error: {}", e)).color(egui::Color32::RED),
                    );
                }
                _ => {
                    ui.label("Loading...");
                }
            });
        });

        // Central Panel: Image Area
        let panel_frame = egui::Frame::none().fill(egui::Color32::from_rgba_premultiplied(
            0,
            0,
            0,
            (255.0 * self.core.opacity * 0.1) as u8,
        ));

        let mut new_image_size = None;
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                if let ImageState::Loaded { source, dimensions } = &self.core.state {
                    let available_size = ui.available_size();
                    let alpha = (255.0 * self.core.opacity) as u8;
                    let tint = egui::Color32::from_rgba_premultiplied(alpha, alpha, alpha, alpha);

                    let mut image = match source {
                        ImageSource::Uri(uri) => egui::Image::new(uri),
                        ImageSource::Raw(_) => {
                            if let Some(texture) = &self.texture {
                                egui::Image::from_texture(texture)
                            } else {
                                return;
                            }
                        }
                    };

                    if self.core.fit_to_window {
                        image = image.fit_to_exact_size(available_size);
                    } else {
                        image = image.max_size(available_size);
                    }

                    // Capture image size when it becomes available
                    if dimensions.is_none() {
                        if let Ok(poll) = image.load_for_size(ui.ctx(), available_size) {
                            if let Some(size) = poll.size() {
                                new_image_size = Some((size.x as u32, size.y as u32));
                            }
                        }
                    }

                    image = image
                        .rotate(self.core.rotation, egui::Vec2::splat(0.5))
                        .tint(tint);

                    ui.vertical_centered(|ui| {
                        let response = ui.add(image);
                        let rect = response.rect;
                        if self.core.show_grid {
                            let painter = ui.painter();
                            let rgba = self.core.grid_color.to_rgba8();
                            let grid_color = egui::Color32::from_rgba_premultiplied(
                                rgba[0], rgba[1], rgba[2], rgba[3],
                            );
                            painter.rect_stroke(
                                rect,
                                0.0,
                                egui::Stroke::new(self.core.thick_line_width, grid_color),
                            );
                            for i in 1..self.core.grid_cols {
                                let t = i as f32 / self.core.grid_cols as f32;
                                let x = egui::lerp(rect.x_range(), t);
                                let width = if i % self.core.grid_subdivision == 0 {
                                    self.core.thick_line_width
                                } else {
                                    self.core.thin_line_width
                                };
                                painter.line_segment(
                                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                    egui::Stroke::new(width, grid_color),
                                );
                            }
                            for i in 1..self.core.grid_rows {
                                let t = i as f32 / self.core.grid_rows as f32;
                                let y = egui::lerp(rect.y_range(), t);
                                let width = if i % self.core.grid_subdivision == 0 {
                                    self.core.thick_line_width
                                } else {
                                    self.core.thin_line_width
                                };
                                painter.line_segment(
                                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                                    egui::Stroke::new(width, grid_color),
                                );
                            }
                        }
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new("Drop an image, paste, or enter a URL").weak(),
                            )
                            .selectable(false),
                        );
                    });
                }
            });

        if let Some((w, h)) = new_image_size {
            if let ImageState::Loaded { source, .. } = &self.core.state {
                self.core.state = ImageState::Loaded {
                    source: source.clone(),
                    dimensions: Some((w, h)),
                };
            }
        }

        // Handle file drops
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            if let Some(file) = dropped_files.first() {
                if let Some(path) = &file.path {
                    self.core.handle_open_file(path.clone());
                }
            }
        }
    }
}
