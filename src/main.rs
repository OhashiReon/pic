#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0])
            .with_transparent(true)
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native(
        "Moshya Viewer",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MoshyaApp::default()))
        }),
    )
}

enum AppImage {
    Uri(String),
    Texture(egui::TextureHandle),
}

struct MoshyaApp {
    current_image: Option<AppImage>,
    opacity: f32,
    rotation: f32,
    always_on_top: bool,
    show_grid: bool,
    grid_cols: u32,
    grid_rows: u32,
    grid_color: GridColor,
    thick_line_width: f32,
    thin_line_width: f32,
    grid_subdivision: u32,
    image_size: Option<(u32, u32)>,
    web_url: String,
    fit_to_window: bool,
}

#[derive(PartialEq)]
enum GridColor {
    Red,
    Cyan,
    Green,
    White,
    Black,
}

impl GridColor {
    fn to_color32(&self) -> egui::Color32 {
        match self {
            GridColor::Red => egui::Color32::from_rgba_premultiplied(255, 0, 0, 180),
            GridColor::Cyan => egui::Color32::from_rgba_premultiplied(0, 255, 255, 180),
            GridColor::Green => egui::Color32::from_rgba_premultiplied(0, 255, 0, 180),
            GridColor::White => egui::Color32::from_rgba_premultiplied(255, 255, 255, 180),
            GridColor::Black => egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
        }
    }
}

impl Default for MoshyaApp {
    fn default() -> Self {
        Self {
            current_image: None,
            opacity: 1.0,
            rotation: 0.0,
            always_on_top: false,
            show_grid: false,
            grid_cols: 4,
            grid_rows: 4,
            grid_color: GridColor::Red,
            thick_line_width: 2.0,
            thin_line_width: 1.0,
            grid_subdivision: 2,
            image_size: None,
            web_url: String::new(),
            fit_to_window: true,
        }
    }
}

impl MoshyaApp {
    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
            .pick_file()
        {
            self.load_path(path);
        }
    }

    fn load_path(&mut self, path: PathBuf) {
        if let Ok((width, height)) = image::image_dimensions(&path) {
            self.image_size = Some((width, height));
        } else {
            self.image_size = None;
        }
        let uri = format!("file://{}", path.to_string_lossy());
        self.current_image = Some(AppImage::Uri(uri));
    }

    fn load_from_clipboard(&mut self, ctx: &egui::Context) {
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(cb) => cb,
            Err(_) => return,
        };

        // Try getting image first
        if let Ok(image) = clipboard.get_image() {
            let size = [image.width as usize, image.height as usize];
            let pixels = egui::ColorImage::from_rgba_unmultiplied(size, &image.bytes);
            let texture = ctx.load_texture("clipboard_image", pixels, Default::default());
            self.image_size = Some((image.width as u32, image.height as u32));
            self.current_image = Some(AppImage::Texture(texture));
            return;
        }

        // Try getting text (URL)
        if let Ok(text) = clipboard.get_text() {
            let trimmed = text.trim();
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                self.image_size = None;
                self.current_image = Some(AppImage::Uri(trimmed.to_string()));
            }
        }
    }
}

impl eframe::App for MoshyaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle Ctrl+V
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::V)) {
            self.load_from_clipboard(ctx);
        }

        let panel_frame = egui::Frame::default().fill(egui::Color32::from_rgba_premultiplied(
            0,
            0,
            0,
            (255.0 * self.opacity * 0.1) as u8,
        ));

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("📂").on_hover_text("Open File").clicked() {
                        self.open_file();
                    }
                    if ui
                        .button("📋")
                        .on_hover_text("Paste from Clipboard (Ctrl+V)")
                        .clicked()
                    {
                        self.load_from_clipboard(ctx);
                    }

                    ui.separator();
                    ui.add(egui::TextEdit::singleline(&mut self.web_url).hint_text("Image URL..."));
                    if ui.button("🌐").on_hover_text("Load URL").clicked() {
                        if !self.web_url.is_empty() {
                            self.image_size = None;
                            self.current_image = Some(AppImage::Uri(self.web_url.clone()));
                        }
                    }

                    ui.separator();
                    let fit_text = if self.fit_to_window {
                        "Actual Size"
                    } else {
                        "Fit Window"
                    };
                    if ui
                        .button(fit_text)
                        .on_hover_text("Toggle Fit to Window / Actual Size")
                        .clicked()
                    {
                        self.fit_to_window = !self.fit_to_window;
                    }

                    ui.separator();
                    let pin_text = if self.always_on_top { "Unpin" } else { "Pin" };
                    if ui.button(pin_text).clicked() {
                        self.always_on_top = !self.always_on_top;
                        let level = if self.always_on_top {
                            egui::WindowLevel::AlwaysOnTop
                        } else {
                            egui::WindowLevel::Normal
                        };
                        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                    }

                    ui.separator();
                    ui.label("Op:");
                    ui.add(egui::Slider::new(&mut self.opacity, 0.1..=1.0).show_value(false));

                    ui.separator();
                    let grid_btn_text = if self.show_grid {
                        "Grid On"
                    } else {
                        "Grid Off"
                    };
                    if ui.button(grid_btn_text).clicked() {
                        self.show_grid = !self.show_grid;
                    }
                    if self.show_grid {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut self.grid_cols).range(1..=100));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut self.grid_rows).range(1..=100));
                        egui::ComboBox::from_id_salt("grid_color")
                            .selected_text(match self.grid_color {
                                GridColor::Red => "Red",
                                GridColor::Cyan => "Cyan",
                                GridColor::Green => "Green",
                                GridColor::White => "White",
                                GridColor::Black => "Black",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.grid_color, GridColor::Red, "Red");
                                ui.selectable_value(&mut self.grid_color, GridColor::Cyan, "Cyan");
                                ui.selectable_value(
                                    &mut self.grid_color,
                                    GridColor::Green,
                                    "Green",
                                );
                                ui.selectable_value(
                                    &mut self.grid_color,
                                    GridColor::White,
                                    "White",
                                );
                                ui.selectable_value(
                                    &mut self.grid_color,
                                    GridColor::Black,
                                    "Black",
                                );
                            });
                        ui.label("Div:");
                        ui.add(egui::DragValue::new(&mut self.grid_subdivision).range(1..=10));
                        ui.label("Thick:");
                        ui.add(
                            egui::DragValue::new(&mut self.thick_line_width)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.label("Thin:");
                        ui.add(
                            egui::DragValue::new(&mut self.thin_line_width)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                    }
                    ui.separator();
                    if let Some((width, height)) = self.image_size {
                        ui.label(format!("{}x{}", width, height));
                    }
                });

                ui.separator();

                if let Some(app_img) = &self.current_image {
                    let available_size = ui.available_size();
                    let alpha = (255.0 * self.opacity) as u8;
                    let tint = egui::Color32::from_rgba_premultiplied(alpha, alpha, alpha, alpha);

                    let mut image = match app_img {
                        AppImage::Uri(uri) => egui::Image::new(uri),
                        AppImage::Texture(texture) => egui::Image::from_texture(texture),
                    };

                    if self.fit_to_window {
                        image = image.fit_to_exact_size(available_size);
                    } else {
                        image = image.max_size(available_size);
                    }

                    image = image
                        .rotate(self.rotation, egui::Vec2::splat(0.5))
                        .tint(tint);

                    ui.vertical_centered(|ui| {
                        let response = ui.add(image);
                        let rect = response.rect;
                        if self.show_grid {
                            let painter = ui.painter();
                            let grid_color = self.grid_color.to_color32();
                            painter.rect_stroke(
                                rect,
                                0.0,
                                egui::Stroke::new(self.thick_line_width, grid_color),
                            );
                            for i in 1..self.grid_cols {
                                let t = i as f32 / self.grid_cols as f32;
                                let x = egui::lerp(rect.x_range(), t);
                                let width = if i % self.grid_subdivision == 0 {
                                    self.thick_line_width
                                } else {
                                    self.thin_line_width
                                };
                                painter.line_segment(
                                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                    egui::Stroke::new(width, grid_color),
                                );
                            }
                            for i in 1..self.grid_rows {
                                let t = i as f32 / self.grid_rows as f32;
                                let y = egui::lerp(rect.y_range(), t);
                                let width = if i % self.grid_subdivision == 0 {
                                    self.thick_line_width
                                } else {
                                    self.thin_line_width
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
                        ui.label("Drop an image, paste, or enter a URL");
                    });
                }
            });

        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            if let Some(file) = dropped_files.first() {
                if let Some(path) = &file.path {
                    self.load_path(path.clone());
                }
            }
        }
    }
}
