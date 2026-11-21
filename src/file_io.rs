use tiny_skia::Pixmap;

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

/// The verdict of one in-flight file task ([`FileTask`]): what the Open
/// pick, the Save download or the native folder pick came back with.
pub enum FileOutcome {
    Opened(Option<LoadedImage>),
    /// The web download's verdict — the NATIVE save is a synchronous file
    /// write with no dialog in it, so no native task ever carries this
    /// variant (same shape as [`SaveFormat::mime_type`]).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Saved(Option<SavedImage>),
    /// The native folder pick of the "Save Image" dialog (the row's
    /// Browse… button and the Save-click fallback). `None` = cancelled.
    Folder(Option<PathBuf>),
}

/// ONE in-flight file dialog (F21, owner pick of variant A 2026-11-21):
/// the platform pickers no longer run on the UI thread — a task owns the
/// blocking call and the UI loop keeps painting while the chooser is
/// open. `App`'s `file_task: Option<FileTask>` slot doubles as the
/// SINGLE-DIALOG GUARD: a task is spawned only while the slot is empty,
/// so two portal choosers can never stack (a second request while one is
/// open is rejected — the first keeps the screen; the Open menu item and
/// the Save dialog's Browse… button gray while it lasts). When no task is
/// in flight the slot is `None` and polling it is one option check — no
/// threads, channels or repaint timers exist until a picker is requested.
#[cfg(not(target_arch = "wasm32"))]
pub struct FileTask {
    rx: std::sync::mpsc::Receiver<Result<FileOutcome, String>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileTask {
    /// `None` while the dialog is open (the UI loop runs normally — F21);
    /// the result EXACTLY once once the worker thread has delivered it.
    /// A worker that died without a verdict (a panic inside the picker)
    /// resolves the task with an error instead of wedging the
    /// single-dialog slot shut (the slot consumes the task on its first
    /// verdict — `poll_file_task`'s take-then-drop — so the App never
    /// polls one task twice).
    pub fn poll(&mut self) -> Option<Result<FileOutcome, String>> {
        match self.rx.try_recv() {
            Ok(result) => Some(result),
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                Some(Err("The file dialog closed without a result".to_string()))
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use web::FileTask;

/// The save formats of the "Save Image" dialog (SPEC §3.7; owner request
/// 2026-11-19): the format choice lives IN the dialog — a dropdown since
/// 2026-11-20, a radio before — not in the OS filter list — so the enum is
/// shared by the dialog state, the codecs and the platform save glue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveFormat {
    Png,
    Jpeg,
}

impl SaveFormat {
    /// The extension the dialog appends (`.jpg` — the canonical spelling;
    /// a typed `.jpeg` is canonicalized to it, `compose_file_name` in
    /// `app/save_dialog.rs`).
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }

    /// The dialog dropdown label (the filter names of the OS save dialog
    /// this dialog replaced).
    pub fn label(self) -> &'static str {
        match self {
            Self::Png => "PNG image",
            Self::Jpeg => "JPEG image",
        }
    }

    /// The MIME type the web download's Blob carries (the browser names the
    /// file by the `download` attribute, the type is what it knows the bytes
    /// to be). Its production caller is the wasm download path only — on
    /// native the mapping is exercised by the test below.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
        }
    }

    /// The format behind a file-name extension (case-insensitive); `None`
    /// for anything the app does not read/write.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            _ => None,
        }
    }
}

/// The "Save Image" dialog's encoding choices for one save (SPEC §3.7):
/// the format dropdown plus the per-format knob — JPEG's quality, PNG's
/// compression effort. A struct, not two loose `u8` parameters after the
/// format: a transposed knob would compile silently.
#[derive(Clone, Copy, Debug)]
pub struct SaveOptions {
    pub format: SaveFormat,
    /// JPEG 1..=100 (the dialog clamps its field before composing this).
    pub jpeg_quality: u8,
    /// PNG 0..=9 — encoder EFFORT only: file size vs encode time. 0 stores
    /// the rows uncompressed; 1..=9 are the deflate levels. Every level
    /// decodes to the same pixels (PNG is lossless at all of them; tested
    /// byte-for-byte in `tests.rs`).
    pub png_compression: u8,
}

impl SaveOptions {
    /// The path-based exports (the MCP `export_png` tool — no dialog is
    /// involved): the documented defaults for the format.
    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub(crate) fn defaults_for(format: SaveFormat) -> Self {
        Self {
            format,
            jpeg_quality: DEFAULT_JPEG_QUALITY,
            png_compression: DEFAULT_PNG_COMPRESSION,
        }
    }
}

/// The JPEG quality the path-based export (the MCP `export_png` tool) uses
/// and the "Save Image" dialog starts from (the value the old hardcoded
/// encoder used — SPEC §3.7). The dialog keeps the user's last choice
/// across opens.
pub(crate) const DEFAULT_JPEG_QUALITY: u8 = 90;

/// The PNG compression effort the path-based exports use and the "Save
/// Image" dialog starts from (6 = the pinned encoder's balanced deflate
/// level — flate2's own default, SPEC §3.7). The dialog keeps the user's
/// last choice across opens.
pub(crate) const DEFAULT_PNG_COMPRESSION: u8 = 6;

/// Whether the "Save Image" dialog shows its folder row (native only — on
/// web the browser owns where a download lands). A `cfg!()` MACRO, not a
/// `#[cfg]` attribute: the platform cfg branches live only in the places
/// AGENTS.md lists, and the dialog code is shared — the row is hidden at
/// RUNTIME on web (the web `pick_save_folder` stub below returns `None`).
pub(crate) const FOLDER_PICKING_SUPPORTED: bool = !cfg!(target_arch = "wasm32");

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

/// The slider's 0..=9 value mapped onto the pinned encoder's compression
/// efforts (SPEC §3.7): `CompressionType::Level` — png maps Level(0) to
/// NoCompression (the rows are stored raw) and Level(1..) to the EXACT
/// deflate level 1..=9, so every slider stop is a real, distinct setting:
/// higher = a smaller file, a slower encode, THE SAME pixels (PNG is
/// lossless at every level — the promise the dialog's Lossless row
/// displays). Out-of-range values clamp to 9 — no panic can reach the
/// user-data path. Filters stay Adaptive at every level.
pub(crate) fn png_compression_type(compression: u8) -> image::codecs::png::CompressionType {
    image::codecs::png::CompressionType::Level(compression.min(9))
}

/// The pixmap's premultiplied bytes demultiplied into an `RgbaImage` — the
/// source both PNG encoders (default and slider-driven) build from.
fn demultiplied_rgba(pixmap: &Pixmap) -> Result<image::RgbaImage, String> {
    let mut bytes = Vec::with_capacity(pixmap.width() as usize * pixmap.height() as usize * 4);
    for pixel in pixmap.pixels() {
        let color = pixel.demultiply();
        bytes.extend_from_slice(&[color.red(), color.green(), color.blue(), color.alpha()]);
    }
    image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), bytes)
        .ok_or_else(|| "Failed to build PNG image".to_string())
}

/// Shared with the MCP server (feature `mcp`): the snapshot tool returns
/// the canvas as PNG bytes. The settings are the encoder's OWN defaults
/// (`PngEncoder::new` = the Fast fdeflate effort + Adaptive filter) — the
/// settings every PNG save used before the dialog's Compression slider
/// existed; the MCP export has no dialog, so it deliberately stays on
/// them, off the slider's 0..=9 scale.
#[cfg_attr(not(feature = "mcp"), allow(dead_code))]
pub(crate) fn encode_png(pixmap: &Pixmap) -> Result<Vec<u8>, String> {
    let image = demultiplied_rgba(pixmap)?;
    let mut png = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|err| format!("Failed to encode PNG: {err}"))?;
    Ok(png)
}

/// PNG encode with the "Save Image" dialog's compression choice (owner
/// request 2026-11-20 — the slider). The value rides
/// [`png_compression_type`]: any `u8` maps onto a legal effort, so no
/// panic ever reaches the user-data path.
pub(crate) fn encode_png_with_compression(
    pixmap: &Pixmap,
    compression: u8,
) -> Result<Vec<u8>, String> {
    let image = demultiplied_rgba(pixmap)?;
    let mut png = Vec::new();
    use image::ImageEncoder;
    let mut cursor = std::io::Cursor::new(&mut png);
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut cursor,
        png_compression_type(compression),
        image::codecs::png::FilterType::Adaptive,
    );
    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|err| format!("Failed to encode PNG: {err}"))?;
    Ok(png)
}

/// JPEG encode with the "Save Image" dialog's quality choice (owner
/// request 2026-11-19 — the quality used to be hardcoded 90). Transparent
/// pixels flatten onto white (§5.1). The quality is CLAMPED 1..=100: the
/// slider is not the only writer and the encoder rejects out-of-range
/// values — no panic ever reaches the user-data path.
pub(crate) fn encode_jpeg_with_quality(pixmap: &Pixmap, quality: u8) -> Result<Vec<u8>, String> {
    let quality = quality.clamp(1, 100);
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
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
        encoder
            .encode_image(&image)
            .map_err(|err| format!("Failed to encode JPEG: {err}"))?;
    }
    Ok(jpeg)
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{self, Sender};

    use eframe::egui;
    use tiny_skia::Pixmap;

    use super::{
        FileOutcome, FileTask, LoadedImage, SaveFormat, SaveOptions, SavedImage,
        encode_jpeg_with_quality, encode_png_with_compression, pixmap_from_image,
    };

    /// The blocking picker + decode (the pre-F21 `open_image` body): runs
    /// on the TASK'S WORKER THREAD now — the UI thread only spawns it and
    /// polls the result (F21).
    fn open_image_blocking() -> Result<Option<LoadedImage>, String> {
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

    /// The blocking folder picker (the pre-F21 body), worker thread too.
    fn pick_save_folder_blocking(start: Option<&Path>) -> Option<PathBuf> {
        let mut dialog = rfd::FileDialog::new();
        if let Some(start) = start {
            dialog = dialog.set_directory(start);
        }
        dialog.pick_folder()
    }

    /// The F21 spawn glue shared by both constructors: `job` (the blocking
    /// picker plus the Open-path decode) runs on a named worker thread, its
    /// result goes through `tx` and the UI is woken so the next frame
    /// consumes it instead of waiting for an interaction with our window.
    /// A failed spawn (practically impossible) resolves the task with an
    /// error instead of panicking or leaving the single-dialog slot wedged.
    fn spawn_dialog_task(
        tx: Sender<Result<FileOutcome, String>>,
        ctx: egui::Context,
        job: impl FnOnce() -> Result<FileOutcome, String> + Send + 'static,
    ) {
        let spawned = std::thread::Builder::new()
            .name("russet-file-dialog".to_string())
            .spawn({
                let tx = tx.clone();
                move || {
                    let _ = tx.send(job());
                    // The chooser just closed: wake the loop so the result
                    // is consumed on the next frame.
                    ctx.request_repaint();
                }
            });
        if let Err(error) = spawned {
            // The ORIGINAL `tx` handle stays here: a failed spawn resolves
            // the task with an error instead of wedging the slot.
            let _ = tx.send(Err(format!("Failed to open the file dialog: {error}")));
        }
    }

    /// The File ▸ Open picker (F21: async — the portal chooser runs on a
    /// worker thread and the UI loop keeps running while it is open; the
    /// image decode moved onto the same thread).
    pub fn open_image(ctx: &egui::Context) -> FileTask {
        let (tx, rx) = mpsc::channel();
        spawn_dialog_task(tx, ctx.clone(), || {
            open_image_blocking().map(FileOutcome::Opened)
        });
        FileTask { rx }
    }

    /// The "Save Image" dialog's folder row / Save-click fallback (owner
    /// request 2026-11-19), opened at `start` when there is one — F21:
    /// async, the same slot mechanics as [`open_image`]. On web the dialog
    /// hides the row and the stub below answers "no folder".
    pub fn pick_save_folder(ctx: &egui::Context, start: Option<&Path>) -> FileTask {
        let (tx, rx) = mpsc::channel();
        let start = start.map(Path::to_path_buf);
        spawn_dialog_task(tx, ctx.clone(), move || {
            Ok(FileOutcome::Folder(pick_save_folder_blocking(
                start.as_deref(),
            )))
        });
        FileTask { rx }
    }

    /// The dialog's save: an already-resolved folder + the COMPOSED file
    /// name (the extension follows the chosen format —
    /// `app/save_dialog.rs::compose_file_name`) + the dialog's encoding
    /// choices ([`SaveOptions`]). The path-based [`save_image_to`] keeps
    /// the extension-decides rule for the MCP export tool.
    pub fn save_image_as(
        pixmap: &Pixmap,
        dir: &Path,
        name: &str,
        options: SaveOptions,
    ) -> Result<SavedImage, String> {
        let path = dir.join(name);
        save_image_to_path(pixmap, &path, options)?;
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string());
        Ok(SavedImage { path, name })
    }

    /// Path-based export (the MCP `export_png` tool, feature `mcp` — its
    /// only consumer): the path's EXTENSION decides the format; quality
    /// [`DEFAULT_JPEG_QUALITY`] (no dialog is involved). Unknown extension —
    /// an error, never a guess.
    #[cfg(feature = "mcp")]
    pub fn save_image_to(pixmap: &Pixmap, path: &Path) -> Result<(), String> {
        let format = path
            .extension()
            .and_then(|ext| SaveFormat::from_extension(&ext.to_string_lossy()))
            .ok_or_else(|| "Unsupported file extension. Use .png, .jpg or .jpeg.".to_string())?;
        save_image_to_path(pixmap, path, SaveOptions::defaults_for(format))
    }

    fn save_image_to_path(
        pixmap: &Pixmap,
        path: &Path,
        options: SaveOptions,
    ) -> Result<(), String> {
        let bytes = match options.format {
            SaveFormat::Png => encode_png_with_compression(pixmap, options.png_compression)?,
            SaveFormat::Jpeg => encode_jpeg_with_quality(pixmap, options.jpeg_quality)?,
        };
        std::fs::write(path, bytes).map_err(|err| format!("Failed to save image: {err}"))
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    use eframe::egui;
    use tiny_skia::Pixmap;
    use wasm_bindgen_futures::js_sys;
    use wasm_bindgen_futures::wasm_bindgen::JsCast;

    use super::{
        FileOutcome, LoadedImage, SaveFormat, SaveOptions, SavedImage, encode_jpeg_with_quality,
        encode_png_with_compression, pixmap_from_image,
    };

    /// The one-slot mailbox of a web task: `None` while the task runs,
    /// `Some(result)` once it has finished (set by the `spawn_local`
    /// closure, consumed by the frame poll — the §3.10 mechanism).
    type Slot = Arc<Mutex<Option<Result<FileOutcome, String>>>>;

    pub struct FileTask {
        kind: SlotKind,
    }

    enum SlotKind {
        Open(Slot),
        Save(Slot),
        /// Never produced by a real web flow — the shared `pick_save_folder`
        /// stub (the folder row hides on web) returns an already-resolved
        /// "no folder" task so the shared dialog code compiles unchanged.
        Folder(Slot),
    }

    impl FileTask {
        /// The result exactly once; `None` while the task is still running.
        pub fn poll(&mut self) -> Option<Result<FileOutcome, String>> {
            let slot = match &mut self.kind {
                SlotKind::Open(slot) | SlotKind::Save(slot) | SlotKind::Folder(slot) => slot,
            };
            let mut guard = slot.lock().ok()?;
            guard.take()
        }
    }

    pub fn open_image(ctx: &egui::Context) -> FileTask {
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
                *guard = Some(result.map(FileOutcome::Opened));
            }
            ctx.request_repaint();
        });
        FileTask {
            kind: SlotKind::Open(slot),
        }
    }

    /// Web stub of the dialog's folder row [`super::FOLDER_PICKING_SUPPORTED`]
    /// hides: the browser owns where a download lands, so there is nothing
    /// to pick. Present so the SHARED dialog code compiles unchanged — the
    /// row's Browse… click never fires on web, and the stub is an
    /// already-resolved "no folder" task.
    pub fn pick_save_folder(_ctx: &egui::Context, _start: Option<&Path>) -> FileTask {
        let slot = Arc::new(Mutex::new(Some(Ok(FileOutcome::Folder(None)))));
        FileTask {
            kind: SlotKind::Folder(slot),
        }
    }

    /// The dialog's save (owner request 2026-11-19): name/format/quality
    /// are already collected by the in-app "Save Image" dialog. The encoded
    /// bytes go STRAIGHT to the browser (owner request 2026-11-20 — rfd's
    /// web save fallback window with its "click here to download your file"
    /// link is gone): a Blob of the format's MIME type over an object URL,
    /// one synthetic click on a detached `<a download>` anchor carrying the
    /// composed name, the URL revoked on a grace timer. The browser owns
    /// where the file lands (§3.10), so no filter list here, and there is
    /// nothing to cancel — the task always resolves to a saved image (or an
    /// error popup).
    pub fn save_image_as(
        ctx: &egui::Context,
        pixmap: Pixmap,
        name: String,
        options: SaveOptions,
    ) -> FileTask {
        let slot = Arc::new(Mutex::new(None));
        let task_slot = Arc::clone(&slot);
        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result =
                download_image(&name, &pixmap, options).map(|()| Some(SavedImage { name }));
            if let Ok(mut guard) = task_slot.lock() {
                *guard = Some(result.map(FileOutcome::Saved));
            }
            ctx.request_repaint();
        });
        FileTask {
            kind: SlotKind::Save(slot),
        }
    }

    /// The synchronous DOM download behind [`save_image_as`]: encode → Blob
    /// → object URL → one click on a detached `<a download>` (rfd's own
    /// `set_download_link` mechanics, minus its fallback window). Never
    /// panics — every browser failure becomes the dialog's "Error" popup
    /// text (§3.7's error rule).
    fn download_image(name: &str, pixmap: &Pixmap, options: SaveOptions) -> Result<(), String> {
        let bytes = match options.format {
            SaveFormat::Png => encode_png_with_compression(pixmap, options.png_compression),
            SaveFormat::Jpeg => encode_jpeg_with_quality(pixmap, options.jpeg_quality),
        }?;

        let window = web_sys::window()
            .ok_or_else(|| "Failed to save image: no browser window".to_string())?;
        let document = window
            .document()
            .ok_or_else(|| "Failed to save image: no browser document".to_string())?;
        let anchor: web_sys::HtmlAnchorElement = document
            .create_element("a")
            .map_err(|err| format!("Failed to save image: {err:?}"))?
            .dyn_into()
            .map_err(|err| format!("Failed to save image: {err:?}"))?;

        let array = js_sys::Array::new();
        array.push(&js_sys::Uint8Array::from(bytes.as_slice()));
        let bag = web_sys::BlobPropertyBag::new();
        bag.set_type(options.format.mime_type());
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &bag)
            .map_err(|err| format!("Failed to save image: {err:?}"))?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|err| format!("Failed to save image: {err:?}"))?;

        anchor.set_href(&url);
        anchor.set_download(name);
        anchor.click();

        // A grace timer revokes the object URL once the download has
        // certainly started (an immediate revoke can abort the file in some
        // browsers). The one-shot closure is forgotten on purpose — a save
        // costs one tiny timer until the callback fires.
        let revoke =
            wasm_bindgen_futures::wasm_bindgen::prelude::Closure::wrap(Box::new(move || {
                let _ = web_sys::Url::revoke_object_url(&url);
            })
                as Box<dyn FnMut()>);
        let callback: &js_sys::Function = revoke.as_ref().unchecked_ref();
        const REVOKE_GRACE_MS: i32 = 10_000;
        let _ =
            window.set_timeout_with_callback_and_timeout_and_arguments_0(callback, REVOKE_GRACE_MS);
        revoke.forget();

        Ok(())
    }
}

/// The MCP `export_png` tool's path export (its only consumer — feature
/// `mcp`, so the default build carries no dead saver).
#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
pub use native::save_image_to;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{open_image, pick_save_folder, save_image_as};

#[cfg(target_arch = "wasm32")]
pub use web::{open_image, pick_save_folder, save_image_as};

#[cfg(test)]
mod tests;
