use crate::services::ClipboardService;
use eframe::egui;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ImviewError {
    #[error("Failed to access clipboard: {0}")]
    Clipboard(String),
    #[error("Failed to load image: {0}")]
    Load(String),
}

#[derive(Clone)]
pub enum ImageSource {
    Uri(String),
    Texture(egui::TextureHandle),
}

#[allow(dead_code)]
pub enum ImageState {
    Idle,
    Loading {
        source: String,
    }, // For URI loading
    Loaded {
        source: ImageSource,
        dimensions: Option<(u32, u32)>,
    },
    Failed(ImviewError),
}

#[derive(PartialEq, Clone, Copy)]
pub enum GridColor {
    Red,
    Cyan,
    Green,
    White,
    Black,
}

impl GridColor {
    pub fn to_color32(self) -> egui::Color32 {
        match self {
            GridColor::Red => egui::Color32::from_rgba_premultiplied(255, 0, 0, 180),
            GridColor::Cyan => egui::Color32::from_rgba_premultiplied(0, 255, 255, 180),
            GridColor::Green => egui::Color32::from_rgba_premultiplied(0, 255, 0, 180),
            GridColor::White => egui::Color32::from_rgba_premultiplied(255, 255, 255, 180),
            GridColor::Black => egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
        }
    }
}

pub struct AppCore {
    pub state: ImageState,
    pub opacity: f32,
    pub rotation: f32,
    pub always_on_top: bool,
    pub show_grid: bool,
    pub grid_cols: u32,
    pub grid_rows: u32,
    pub grid_color: GridColor,
    pub thick_line_width: f32,
    pub thin_line_width: f32,
    pub grid_subdivision: u32,
    pub web_url: String,
    pub fit_to_window: bool,
    pub show_settings_panel: bool,

    pub clipboard: Box<dyn ClipboardService>,
}

impl AppCore {
    pub fn new(clipboard: Box<dyn ClipboardService>) -> Self {
        Self {
            state: ImageState::Idle,
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
            web_url: String::new(),
            fit_to_window: true,
            show_settings_panel: false,
            clipboard,
        }
    }

    pub fn handle_open_file(&mut self, path: PathBuf) {
        let uri = format!("file://{}", path.to_string_lossy());
        let dimensions = image::image_dimensions(&path).ok();
        self.state = ImageState::Loaded {
            source: ImageSource::Uri(uri),
            dimensions,
        };
    }

    pub fn handle_paste(&mut self, ctx: &egui::Context) {
        // Try getting image first
        if let Ok(image) = self.clipboard.get_image() {
            let size = [image.width, image.height];
            let pixels = egui::ColorImage::from_rgba_unmultiplied(size, &image.bytes);
            let texture = ctx.load_texture("clipboard_image", pixels, Default::default());
            self.state = ImageState::Loaded {
                source: ImageSource::Texture(texture),
                dimensions: Some((image.width as u32, image.height as u32)),
            };
            return;
        }

        // Try getting text (URL)
        if let Ok(text) = self.clipboard.get_text() {
            let trimmed = text.trim();
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                self.state = ImageState::Loaded {
                    source: ImageSource::Uri(trimmed.to_string()),
                    dimensions: None,
                };
            }
        }
    }

    pub fn handle_url_load(&mut self) {
        if !self.web_url.is_empty() {
            self.state = ImageState::Loaded {
                source: ImageSource::Uri(self.web_url.clone()),
                dimensions: None,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::MockClipboard;

    #[test]
    fn test_handle_open_file() {
        let mock_cb = Box::new(MockClipboard {
            image: None,
            text: None,
        });
        let mut core = AppCore::new(mock_cb);
        let path = PathBuf::from("test.png");
        core.handle_open_file(path);

        if let ImageState::Loaded {
            source: ImageSource::Uri(uri),
            ..
        } = core.state
        {
            assert!(uri.contains("test.png"));
        } else {
            panic!("State should be Loaded(Uri)");
        }
    }

    #[test]
    fn test_handle_url_load() {
        let mock_cb = Box::new(MockClipboard {
            image: None,
            text: None,
        });
        let mut core = AppCore::new(mock_cb);
        core.web_url = "https://example.com/image.png".to_string();
        core.handle_url_load();

        if let ImageState::Loaded {
            source: ImageSource::Uri(uri),
            ..
        } = core.state
        {
            assert_eq!(uri, "https://example.com/image.png");
        } else {
            panic!("State should be Loaded(Uri)");
        }
    }

    #[test]
    fn test_handle_paste_url() {
        let mock_cb = Box::new(MockClipboard {
            image: None,
            text: Some("https://example.com/image.png".to_string()),
        });
        let mut core = AppCore::new(mock_cb);
        let ctx = egui::Context::default();
        core.handle_paste(&ctx);

        if let ImageState::Loaded {
            source: ImageSource::Uri(uri),
            ..
        } = core.state
        {
            assert_eq!(uri, "https://example.com/image.png");
        } else {
            panic!("State should be Loaded(Uri)");
        }
    }
}
