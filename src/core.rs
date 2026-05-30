use crate::services::ClipboardService;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ImviewError {
    #[error("Failed to access clipboard: {0}")]
    Clipboard(String),
    #[error("Failed to load image: {0}")]
    Load(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RawImage {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImageSource {
    Uri(String),
    Raw(RawImage),
}

#[derive(Debug)]
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

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum GridColor {
    Red,
    Cyan,
    Green,
    White,
    Black,
}

impl GridColor {
    pub fn to_rgba8(self) -> [u8; 4] {
        match self {
            GridColor::Red => [255, 0, 0, 180],
            GridColor::Cyan => [0, 255, 255, 180],
            GridColor::Green => [0, 255, 0, 180],
            GridColor::White => [255, 255, 255, 180],
            GridColor::Black => [0, 0, 0, 180],
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

    pub fn handle_paste(&mut self) {
        println!("[DEBUG] handle_paste invoked");
        // Try getting image first
        match self.clipboard.get_image() {
            Ok(image) => {
                println!("[DEBUG] Clipboard returned image: {}x{}", image.width, image.height);
                self.state = ImageState::Loaded {
                    source: ImageSource::Raw(RawImage {
                        width: image.width as u32,
                        height: image.height as u32,
                        bytes: image.bytes.to_vec(),
                    }),
                    dimensions: Some((image.width as u32, image.height as u32)),
                };
                return;
            }
            Err(e) => {
                println!("[DEBUG] Clipboard get_image error: {}", e);
                // Don't fail yet, try text
            }
        }

        // Try getting text (URL or Path)
        match self.clipboard.get_text() {
            Ok(text) => {
                println!("[DEBUG] Clipboard returned text: length={}", text.len());
                let trimmed = text.trim().trim_matches('"');
                if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                    println!("[DEBUG] Pasted text recognized as URL: {}", trimmed);
                    self.state = ImageState::Loaded {
                        source: ImageSource::Uri(trimmed.to_string()),
                        dimensions: None,
                    };
                } else {
                    let path = PathBuf::from(trimmed);
                    if path.exists() && path.is_file() {
                        println!("[DEBUG] Pasted text recognized as existing file path: {:?}", path);
                        self.handle_open_file(path);
                    } else {
                        println!("[DEBUG] Pasted text was not a URL or valid file path");
                        self.state = ImageState::Failed(ImviewError::Clipboard("Clipboard contains text that is not a valid URL or image path.".to_string()));
                    }
                }
            }
            Err(e) => {
                println!("[DEBUG] Clipboard get_text error: {}", e);
                self.state = ImageState::Failed(ImviewError::Clipboard(format!("Failed to access clipboard: {}", e)));
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
        } = &core.state
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
        } = &core.state
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
        core.handle_paste();

        if let ImageState::Loaded {
            source: ImageSource::Uri(uri),
            ..
        } = &core.state
        {
            assert_eq!(uri, "https://example.com/image.png");
        } else {
            panic!("State should be Loaded(Uri)");
        }
    }

    #[test]
    fn test_handle_paste_image() {
        let pixels = vec![255; 100 * 100 * 4];
        let image = arboard::ImageData {
            width: 100,
            height: 100,
            bytes: std::borrow::Cow::Owned(pixels),
        };
        let mock_cb = Box::new(MockClipboard {
            image: Some(image),
            text: None,
        });
        let mut core = AppCore::new(mock_cb);
        core.handle_paste();

        if let ImageState::Loaded {
            source: ImageSource::Raw(raw),
            dimensions: Some((w, h)),
        } = &core.state
        {
            assert_eq!(raw.width, 100);
            assert_eq!(raw.height, 100);
            assert_eq!(*w, 100);
            assert_eq!(*h, 100);
        } else {
            panic!("State should be Loaded(Raw)");
        }
    }
}
