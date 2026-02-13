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
struct MoshyaApp {
    image_path: Option<PathBuf>,
    opacity: f32,
    rotation: f32,
    always_on_top: bool,
    show_grid: bool,
    grid_cols: u32,
    grid_rows: u32,
    grid_color: GridColor,
    image_size: Option<(u32, u32)>,
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
            image_path: None,
            opacity: 1.0,
            rotation: 0.0,
            always_on_top: false,
            show_grid: false,
            grid_cols: 4,
            grid_rows: 4,
            grid_color: GridColor::Red,
            image_size: None,
        }
    }
}
impl MoshyaApp {
    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
            .pick_file()
        {
            if let Ok((width, height)) = image::image_dimensions(&path) {
                self.image_size = Some((width, height));
            }
            self.image_path = Some(path);
        }
    }
}
impl eframe::App for MoshyaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    if ui.button("📂").clicked() {
                        self.open_file();
                    }
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
                        ui.add(egui::DragValue::new(&mut self.grid_cols).range(1..=50));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut self.grid_rows).range(1..=50));
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
                    }
                    ui.separator();
                    if let Some((width, height)) = self.image_size {
                        ui.label(format!("{}x{}", width, height));
                    }
                });
                ui.separator();
                if let Some(path) = &self.image_path {
                    let uri = format!("file://{}", path.to_string_lossy());
                    let available_size = ui.available_size();
                    let mut image = egui::Image::new(&uri)
                        .max_size(available_size)
                        .rotate(self.rotation, egui::Vec2::splat(0.5));
                    let alpha = (255.0 * self.opacity) as u8;
                    image = image.tint(egui::Color32::from_rgba_premultiplied(
                        alpha, alpha, alpha, alpha,
                    ));
                    ui.vertical_centered(|ui| {
                        let response = ui.add(image);
                        let rect = response.rect;
                        if self.show_grid {
                            let painter = ui.painter();
                            let stroke = egui::Stroke::new(1.0, self.grid_color.to_color32());
                            painter.rect_stroke(rect, 0.0, stroke);
                            for i in 1..self.grid_cols {
                                let t = i as f32 / self.grid_cols as f32;
                                let x = egui::lerp(rect.x_range(), t);
                                painter.line_segment(
                                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                    stroke,
                                );
                            }
                            for i in 1..self.grid_rows {
                                let t = i as f32 / self.grid_rows as f32;
                                let y = egui::lerp(rect.y_range(), t);
                                painter.line_segment(
                                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                                    stroke,
                                );
                            }
                        }
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Drop an image here");
                    });
                }
            });
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            if let Some(file) = dropped_files.first() {
                if let Some(path) = &file.path {
                    self.image_path = Some(path.clone());
                }
            }
        }
    }
}
