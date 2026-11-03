#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::io::BufWriter;
    use std::path::{Path, PathBuf};

    use tiny_skia::Pixmap;

    pub struct LoadedImage {
        pub pixmap: Pixmap,
        pub path: PathBuf,
        pub name: String,
    }

    pub struct SavedImage {
        pub path: PathBuf,
        pub name: String,
    }

    #[derive(Clone, Copy)]
    pub enum SaveFormat {
        Png,
        Jpeg,
    }

    pub fn open_image() -> Result<Option<LoadedImage>, String> {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg"])
            .pick_file()
        else {
            return Ok(None);
        };
        let image = image::open(&path).map_err(|err| format!("Failed to open image: {err}"))?;
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        let mut pixmap =
            Pixmap::new(width, height).ok_or_else(|| "Invalid image size".to_string())?;
        for (dst, src) in pixmap.pixels_mut().iter_mut().zip(rgba.pixels()) {
            let [r, g, b, a] = src.0;
            *dst = tiny_skia::ColorU8::from_rgba(r, g, b, a).premultiply();
        }
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
        match format {
            SaveFormat::Png => save_png(pixmap, path),
            SaveFormat::Jpeg => save_jpeg(pixmap, path),
        }
    }

    fn ensure_extension(mut path: PathBuf) -> PathBuf {
        if path.extension().is_none() {
            path.set_extension("png");
        }
        path
    }

    fn format_for_path(path: &Path) -> Option<SaveFormat> {
        let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
        match extension.as_str() {
            "png" => Some(SaveFormat::Png),
            "jpg" | "jpeg" => Some(SaveFormat::Jpeg),
            _ => None,
        }
    }

    fn save_png(pixmap: &Pixmap, path: &Path) -> Result<(), String> {
        let mut bytes = Vec::with_capacity(pixmap.width() as usize * pixmap.height() as usize * 4);
        for pixel in pixmap.pixels() {
            let color = pixel.demultiply();
            bytes.extend_from_slice(&[color.red(), color.green(), color.blue(), color.alpha()]);
        }
        let image = image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), bytes)
            .ok_or_else(|| "Failed to build PNG image".to_string())?;
        image
            .save_with_format(path, image::ImageFormat::Png)
            .map_err(|err| format!("Failed to save PNG: {err}"))
    }

    fn save_jpeg(pixmap: &Pixmap, path: &Path) -> Result<(), String> {
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
        let file =
            std::fs::File::create(path).map_err(|err| format!("Failed to create file: {err}"))?;
        let mut writer = BufWriter::new(file);
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 90);
        encoder
            .encode_image(&image)
            .map_err(|err| format!("Failed to save JPEG: {err}"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::{open_image, save_image_as, save_image_to};
