use tiny_skia::Pixmap;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

pub struct LoadedImage {
    pub pixmap: Pixmap,
    pub name: String,
    #[cfg(not(target_arch = "wasm32"))]
    pub path: PathBuf,
}

pub struct SavedImage {
    pub name: String,
    #[cfg(not(target_arch = "wasm32"))]
    pub path: PathBuf,
}

fn pixmap_from_image(image: image::DynamicImage) -> Result<Pixmap, String> {
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut pixmap = Pixmap::new(width, height).ok_or_else(|| "Invalid image size".to_string())?;
    for (dst, src) in pixmap.pixels_mut().iter_mut().zip(rgba.pixels()) {
        let [r, g, b, a] = src.0;
        *dst = tiny_skia::ColorU8::from_rgba(r, g, b, a).premultiply();
    }
    Ok(pixmap)
}

fn encode_png(pixmap: &Pixmap) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(pixmap.width() as usize * pixmap.height() as usize * 4);
    for pixel in pixmap.pixels() {
        let color = pixel.demultiply();
        bytes.extend_from_slice(&[color.red(), color.green(), color.blue(), color.alpha()]);
    }
    let image = image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), bytes)
        .ok_or_else(|| "Failed to build PNG image".to_string())?;
    let mut png = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|err| format!("Failed to encode PNG: {err}"))?;
    Ok(png)
}

fn encode_jpeg(pixmap: &Pixmap) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(pixmap.width() as usize * pixmap.height() as usize * 3);
    for pixel in pixmap.pixels() {
        let alpha = u32::from(pixel.alpha());
        let red = (u32::from(pixel.red()) + 255 - alpha).min(255) as u8;
        let green = (u32::from(pixel.green()) + 255 - alpha).min(255) as u8;
        let blue = (u32::from(pixel.blue()) + 255 - alpha).min(255) as u8;
        bytes.extend_from_slice(&[red, green, blue]);
    }
    let image = image::RgbImage::from_raw(pixmap.width(), pixmap.height(), bytes)
        .ok_or_else(|| "Failed to build JPEG image".to_string())?;
    let mut jpeg = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut jpeg);
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 90);
        encoder
            .encode_image(&image)
            .map_err(|err| format!("Failed to encode JPEG: {err}"))?;
    }
    Ok(jpeg)
}

#[cfg(target_arch = "wasm32")]
fn encode_for_name(pixmap: &Pixmap, name: &str) -> Result<Vec<u8>, String> {
    match extension_of(name).as_deref() {
        Some("png") => encode_png(pixmap),
        Some("jpg" | "jpeg") => encode_jpeg(pixmap),
        _ => Err("Unsupported file extension. Use .png, .jpg or .jpeg.".to_string()),
    }
}

#[cfg(target_arch = "wasm32")]
fn extension_of(name: &str) -> Option<String> {
    let dot = name.rfind('.')?;
    if dot + 1 >= name.len() {
        return None;
    }
    Some(name[dot + 1..].to_ascii_lowercase())
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::path::{Path, PathBuf};

    use tiny_skia::Pixmap;

    use super::{LoadedImage, SavedImage, pixmap_from_image};

    pub fn open_image() -> Result<Option<LoadedImage>, String> {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg"])
            .pick_file()
        else {
            return Ok(None);
        };
        let image = image::open(&path).map_err(|err| format!("Failed to open image: {err}"))?;
        let pixmap = pixmap_from_image(image)?;
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string());
        Ok(Some(LoadedImage { pixmap, path, name }))
    }

    pub fn save_image_as(
        pixmap: &Pixmap,
        default_name: &str,
    ) -> Result<Option<SavedImage>, String> {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .add_filter("JPEG image", &["jpg", "jpeg"])
            .set_file_name(default_name)
            .save_file()
        else {
            return Ok(None);
        };
        let path = ensure_extension(path);
        save_image_to(pixmap, &path)?;
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string());
        Ok(Some(SavedImage { path, name }))
    }

    pub fn save_image_to(pixmap: &Pixmap, path: &Path) -> Result<(), String> {
        let format = format_for_path(path)
            .ok_or_else(|| "Unsupported file extension. Use .png, .jpg or .jpeg.".to_string())?;
        let bytes = match format {
            SaveFormat::Png => super::encode_png(pixmap)?,
            SaveFormat::Jpeg => super::encode_jpeg(pixmap)?,
        };
        std::fs::write(path, bytes).map_err(|err| format!("Failed to save image: {err}"))
    }

    fn ensure_extension(mut path: PathBuf) -> PathBuf {
        if path.extension().is_none() {
            path.set_extension("png");
        }
        path
    }

    #[derive(Clone, Copy)]
    enum SaveFormat {
        Png,
        Jpeg,
    }

    fn format_for_path(path: &Path) -> Option<SaveFormat> {
        let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
        match extension.as_str() {
            "png" => Some(SaveFormat::Png),
            "jpg" | "jpeg" => Some(SaveFormat::Jpeg),
            _ => None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::sync::{Arc, Mutex};

    use eframe::egui;
    use tiny_skia::Pixmap;

    use super::{LoadedImage, SavedImage, encode_for_name, pixmap_from_image};

    type OpenSlot = Arc<Mutex<Option<Result<Option<LoadedImage>, String>>>>;
    type SaveSlot = Arc<Mutex<Option<Result<Option<SavedImage>, String>>>>;

    pub struct WebTask {
        kind: WebTaskKind,
    }

    enum WebTaskKind {
        Open(OpenSlot),
        Save(SaveSlot),
    }

    pub enum WebOutcome {
        Opened(Option<LoadedImage>),
        Saved(Option<SavedImage>),
    }

    impl WebTask {
        pub fn poll(&self) -> Option<Result<WebOutcome, String>> {
            match &self.kind {
                WebTaskKind::Open(slot) => {
                    let mut guard = slot.lock().ok()?;
                    let result = guard.take()?;
                    Some(result.map(WebOutcome::Opened))
                }
                WebTaskKind::Save(slot) => {
                    let mut guard = slot.lock().ok()?;
                    let result = guard.take()?;
                    Some(result.map(WebOutcome::Saved))
                }
            }
        }
    }

    pub fn open_image(ctx: &egui::Context) -> WebTask {
        let slot = Arc::new(Mutex::new(None));
        let task_slot = Arc::clone(&slot);
        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let Some(handle) = rfd::AsyncFileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg"])
                    .pick_file()
                    .await
                else {
                    return Ok(None);
                };
                let bytes = handle.read().await;
                let image = image::load_from_memory(&bytes)
                    .map_err(|err| format!("Failed to open image: {err}"))?;
                let pixmap = pixmap_from_image(image)?;
                Ok(Some(LoadedImage {
                    pixmap,
                    name: handle.file_name(),
                }))
            }
            .await;
            if let Ok(mut guard) = task_slot.lock() {
                *guard = Some(result);
            }
            ctx.request_repaint();
        });
        WebTask {
            kind: WebTaskKind::Open(slot),
        }
    }

    pub fn save_image_as(ctx: &egui::Context, pixmap: Pixmap, default_name: String) -> WebTask {
        let slot = Arc::new(Mutex::new(None));
        let task_slot = Arc::clone(&slot);
        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = async {
                let Some(handle) = rfd::AsyncFileDialog::new()
                    .add_filter("PNG image", &["png"])
                    .add_filter("JPEG image", &["jpg", "jpeg"])
                    .set_file_name(&default_name)
                    .save_file()
                    .await
                else {
                    return Ok(None);
                };
                let name = with_default_extension(handle.file_name());
                let bytes = encode_for_name(&pixmap, &name)?;
                handle
                    .write(&bytes)
                    .await
                    .map_err(|err| format!("Failed to save image: {err}"))?;
                Ok(Some(SavedImage { name }))
            }
            .await;
            if let Ok(mut guard) = task_slot.lock() {
                *guard = Some(result);
            }
            ctx.request_repaint();
        });
        WebTask {
            kind: WebTaskKind::Save(slot),
        }
    }

    fn with_default_extension(name: String) -> String {
        if super::extension_of(&name).is_none() {
            format!("{name}.png")
        } else {
            name
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::{open_image, save_image_as, save_image_to};

#[cfg(target_arch = "wasm32")]
pub use web::{WebOutcome, WebTask, open_image, save_image_as};
