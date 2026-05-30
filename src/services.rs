use std::path::PathBuf;

pub trait FileDialogService: Send + Sync {
    fn pick_file(&self) -> Option<PathBuf>;
}

pub trait ClipboardService: Send + Sync {
    fn get_image(&self) -> Result<arboard::ImageData<'static>, String>;
    fn get_text(&self) -> Result<String, String>;
}

pub struct NativeFileDialog;

impl FileDialogService for NativeFileDialog {
    fn pick_file(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
            .pick_file()
    }
}

pub struct NativeClipboard;

impl ClipboardService for NativeClipboard {
    fn get_image(&self) -> Result<arboard::ImageData<'static>, String> {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        clipboard.get_image().map_err(|e| e.to_string())
    }

    fn get_text(&self) -> Result<String, String> {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        clipboard.get_text().map_err(|e| e.to_string())
    }
}

#[cfg(test)]
pub struct MockClipboard {
    pub image: Option<arboard::ImageData<'static>>,
    pub text: Option<String>,
}

#[cfg(test)]
impl ClipboardService for MockClipboard {
    fn get_image(&self) -> Result<arboard::ImageData<'static>, String> {
        self.image.clone().ok_or_else(|| "No image".to_string())
    }

    fn get_text(&self) -> Result<String, String> {
        self.text.clone().ok_or_else(|| "No text".to_string())
    }
}
