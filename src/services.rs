use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

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

pub struct NativeClipboard {
    // We use a static OnceLock to ensure only one Clipboard instance exists across the app,
    // as recommended by the arboard documentation.
    clipboard: &'static OnceLock<Mutex<Result<arboard::Clipboard, String>>>,
}

impl NativeClipboard {
    pub fn new() -> Self {
        static INSTANCE: OnceLock<Mutex<Result<arboard::Clipboard, String>>> = OnceLock::new();
        Self {
            clipboard: &INSTANCE,
        }
    }

    fn get_cb(&self) -> Result<&Mutex<Result<arboard::Clipboard, String>>, String> {
        Ok(self.clipboard.get_or_init(|| {
            Mutex::new(arboard::Clipboard::new().map_err(|e| e.to_string()))
        }))
    }
}

impl ClipboardService for NativeClipboard {
    fn get_image(&self) -> Result<arboard::ImageData<'static>, String> {
        let mutex = self.get_cb()?;
        let mut guard = mutex.lock().map_err(|e| e.to_string())?;
        let cb = guard.as_mut().map_err(|e| e.clone())?;
        cb.get_image().map_err(|e| e.to_string())
    }

    fn get_text(&self) -> Result<String, String> {
        let mutex = self.get_cb()?;
        let mut guard = mutex.lock().map_err(|e| e.to_string())?;
        let cb = guard.as_mut().map_err(|e| e.clone())?;
        cb.get_text().map_err(|e| e.to_string())
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
