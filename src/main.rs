#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod services;
mod ui;

use crate::core::AppCore;
use crate::services::{NativeClipboard, NativeFileDialog};
use crate::ui::MoshyaApp;
use eframe::egui;

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
            cc.egui_ctx.set_style(configure_pro_style());
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let clipboard = Box::new(NativeClipboard::new());
            let file_dialog = Box::new(NativeFileDialog);
            let core = AppCore::new(clipboard);

            Ok(Box::new(MoshyaApp::new(core, file_dialog)))
        }),
    )
}

fn configure_pro_style() -> egui::Style {
    let mut style = egui::Style::default();

    // Structured & Dense Spacing
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(6.0, 2.0);
    style.spacing.window_margin = egui::Margin::same(8.0);

    // Sharp Edges (Pro Tool style)
    let rounding = egui::Rounding::same(2.0);
    style.visuals.window_rounding = rounding;
    style.visuals.menu_rounding = rounding;
    style.visuals.widgets.noninteractive.rounding = rounding;
    style.visuals.widgets.inactive.rounding = rounding;
    style.visuals.widgets.hovered.rounding = rounding;
    style.visuals.widgets.active.rounding = rounding;
    style.visuals.widgets.open.rounding = rounding;

    // Solid & Dark Color Palette (Nordic Frost / Nord Theme)
    let bg_color = egui::Color32::from_rgb(46, 52, 64); // Nord 0
    let panel_color = egui::Color32::from_rgb(59, 66, 82); // Nord 1
    let widget_color = egui::Color32::from_rgb(67, 76, 94); // Nord 2
    let hover_color = egui::Color32::from_rgb(76, 86, 106); // Nord 3
    let accent_color = egui::Color32::from_rgb(136, 192, 208); // Nord 8 (Frost)
    let text_color = egui::Color32::from_rgb(216, 222, 233); // Nord 4

    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(text_color);
    style.visuals.window_fill = bg_color;
    style.visuals.panel_fill = panel_color;

    // Borderless widgets for a cleaner look
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;

    style.visuals.widgets.inactive.bg_fill = widget_color;
    style.visuals.widgets.hovered.bg_fill = hover_color;
    style.visuals.widgets.active.bg_fill = accent_color;

    style.visuals.selection.bg_fill = accent_color;

    style
}
