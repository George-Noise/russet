# ТЗ: растровый редактор «Russet» (Rust, MVP в духе Paint.NET)

> **Инструкция агенту — прочитать ДО начала работы, выполнить буквально.**
> 1. Читай этот файл целиком, потом делай вехи из §4 строго по порядку, по одной.
> 2. Веха считается готовой только когда `cargo check` и `cargo clippy -- -D warnings` проходят без ошибок и предупреждений; затем `cargo fmt`.
> 3. Не добавляй НИКАКИЕ зависимости, кроме перечисленных в §1. Не подключай `winit`, `wgpu` напрямую — они уже внутри `eframe`.
> 4. Не реализуй ничего из §6 (вне скоупа). Не улучшай то, что не просят.
> 5. **Все строки пользовательского интерфейса — только на английском**, захардкоженные (§0). Никаких файлов локализации, i18n-крейтов и переводов.
> 6. Если пример из интернета противоречит ошибкам компилятора — верь компилятору и официальному шаблону <https://github.com/emilk/eframe_template> для той версии `eframe`, что указана в Cargo.toml (у новых версий eframe сигнатура метода трейта `App` отличается от старых статей).
> 7. Один коммит на веху, формат сообщения: `M<N>: <краткое описание>`.
> 8. Никаких `unwrap()`/`expect()` на путях выполнения с данными пользователя (файлы, индексы пикселей) — ошибки показывай в popup (§3.7), паника приложения недопустима. (`expect` в точке входа `main.rs` из шаблона §2 допустим.)
>
> **Agent instructions (EN):** Read this file fully. Implement milestones (§4) strictly in order, one at a time. A milestone is done only when `cargo check` and `cargo clippy -- -D warnings` pass, then run `cargo fmt`. Do not add any dependency beyond §1; do not depend on `winit`/`wgpu` directly (they come with `eframe`). All user-facing strings must be hardcoded English; no i18n crates, no localization. Trust the compiler over old internet examples; follow the official eframe_template for the pinned eframe version. One commit per milestone (`M<N>: <summary>`). No `unwrap()` on user-data paths; report errors in a popup instead. Panics are forbidden.

---

## 0. Цель (Goal)

Однооконное растровое приложение **«Russet»**: холст с рисованием тремя инструментами
(карандаш, кисть, ластик), верхнее меню, плавающие окна «Tools» и «Palette»,
undo/redo, зум/панорама, открытие и сохранение PNG/JPEG.

- **Платформы.** Разработка и запуск — **Windows**. Обязательна **web-сборка (WASM, через trunk)** — веха M8. Linux-сборку сейчас не проверять (код и так переносимый, `std::path` разрешён только в `file_io.rs`). **macOS — вне скоупа навсегда** (у владельца проекта нет и не будет Mac).
- **Язык интерфейса — только английский.** Все подписи меню, кнопок, диалогов и заголовков — захардкоженные английские строки (списки — в §3). Переводов не будет никогда: не добавлять i18n.
- Стиль разработки: простой, прямолинейный код. Без макро-магии, без преждевременных абстракций, без многопоточности в коде приложения.
- **Сборка:** ежедневная работа — только дев-сборка (`cargo run`), она настроена на минимальное время пересборки и щадящий режим для машины (§1). Релизный профиль собирается владельцем вручную, когда всё готово, — агенту `--release` собирать НЕ нужно (кроме финального прогона M9).

**EN:** Goal: a single-window raster editor "Russet" (Pencil/Brush/Eraser, menu bar, floating Tools & Palette windows, undo/redo, zoom/pan, PNG/JPEG I/O). Dev machine is Windows; a web (WASM/trunk) build is required (M8). Linux is untested-but-portable; macOS is permanently out of scope. UI is hardcoded English only. Use fast dev builds; release builds are run manually by the owner.

## 1. Технологии и профиль сборки (Stack & build profile)

| Крейт | Версия | Назначение |
|---|---|---|
| `eframe` (включает `egui`) | `0.37` | окно (внутри winit), цикл событий, весь UI, рендер (внутри wgpu) |
| `tiny-skia` | `0.11` | растеризация штрихов в пиксельный буфер `Pixmap` |
| `image` | `0.25`, `default-features = false`, `features = ["png", "jpeg"]` | чтение/запись PNG и JPEG |
| `rfd` | `0.15` | диалоги открытия/сохранения (натив: синхронные; web: только `AsyncFileDialog`) |
| `log` + `env_logger` (натив) | `0.4` / `0.11` | логирование (в web — `eframe::WebLogger`) |
| `wasm-bindgen-futures`, `web-sys` (web) | `0.4` / `0.3` | обвязка web-точки входа |

Инструмент web-сборки: **trunk** (ставится один раз, §8). Это не зависимость Cargo.

Полный `Cargo.toml` — использовать как есть:

```toml
[package]
name = "russet"
version = "0.1.0"
edition = "2024"
rust-version = "1.95"

[dependencies]
eframe = "0.37"
tiny-skia = "0.11"
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
rfd = "0.15"
log = "0.4"

# натив (Windows):
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
env_logger = "0.11"

# web:
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen-futures = "0.4"
web-sys = "0.3"

# БЫСТРАЯ РАЗРАБОТКА:
# - свой код пересобирается без оптимизаций (быстро);
# - зависимости (eframe, wgpu, image...) оптимизируются ОДИН раз и берутся из кэша —
#   UI остаётся отзывчивым в dev-сборке;
# - укороченная отладочная информация ускоряет линковку и почти не грузит диск.
[profile.dev]
opt-level = 1
debug = "line-tables-only"

[profile.dev.package."*"]
opt-level = 2

# РЕЛИЗ: собирается владельцем ВРУЧНУЮ, когда всё готово (LTO медленный).
# Во время разработки `cargo build --release` НЕ запускать.
[profile.release]
lto = "fat"
codegen-units = 1
strip = true
```

Про скорость сборки:
- cargo и trunk **уже собирают параллельно на все ядра** — не указывать `-j`, не отключать параллелизм, `codegen-units` в dev-профиле не трогать.
- Первый `cargo run` долгий (компилируются зависимости с оптимизацией) — это норма; все последующие пересборки занимают секунды.
- Требование к окружению: Rust новее `1.95` (`rustup update`). Для Linux-сборки когда-нибудь позже понадобятся `libxcb-*-dev`, `libxkbcommon-dev`, `libgtk-3-dev` (крейту `rfd`) — сейчас НЕ требуется.

**EN:** Stack pinned: eframe 0.37, tiny-skia 0.11, image 0.25 (png+jpeg only), rfd 0.15, log/env_logger + wasm-bindgen-futures/web-sys (target-gated). Web builds via trunk. Dev profile is tuned for fast rebuilds and low machine load (own crate unoptimized, deps optimized once and cached, minimal debuginfo); cargo parallelizes automatically — do not touch. Release (fat LTO) is built manually by the owner only.

## 2. Архитектура (Architecture)

Структура проекта (разбивка обязательна, монолит запрещён):

```
russet/
  Cargo.toml
  index.html      — страница для trunk: <canvas id="russet_canvas">, заголовок "Russet" (§2)
  src/
    main.rs       — ДВЕ точки входа: нативная (run_native) и web (WebRunner); шаблон ниже
    app.rs        — struct App (всё состояние), impl eframe::App, весь UI: панели, меню, окна, диалоги, хоткеи
    canvas.rs     — struct Canvas: пиксельный буфер, изменение размера, доступ к пикселям
    tools.rs      — enum Tool, struct ToolState, растеризация сегмента штриха
    history.rs    — undo/redo на снимках
    file_io.rs    — открыть/сохранить PNG/JPEG; ЕДИНСТВЕННОЕ место с cfg-ветками native/web
    palette.rs    — фиксированный массив из 20 предустановленных цветов
```

### 2.1 `main.rs` — скелет (обе платформы сразу, по шаблону eframe_template)

```rust
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // без консоли в release на Windows

mod app;
mod canvas;
mod file_io;
mod history;
mod palette;
mod tools;

use app::App;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native("Russet", native_options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().expect("No window").document().expect("No document");
        let canvas = document
            .get_element_by_id("russet_canvas")
            .expect("Failed to find russet_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("russet_canvas was not a HtmlCanvasElement");
        let start_result = eframe::WebRunner::new()
            .start(canvas, web_options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
            .await;
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(()) => { loading_text.remove(); }
                Err(err) => {
                    loading_text.set_inner_html("The app has crashed. See the developer console for details.");
                    panic!("Failed to start eframe: {err:?}");
                }
            }
        }
    });
}
```

### 2.2 `index.html` (корень проекта, для trunk)

```html
<!DOCTYPE html>
<html>
<head>
  <meta http-equiv="Content-Type" content="text/html; charset=utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no" />
  <title>Russet</title>
  <style>
    html, body { margin: 0; padding: 0; width: 100%; height: 100%;
                 overflow: hidden; background: #1e1e1e; }
    #russet_canvas { width: 100%; height: 100%; display: block; touch-action: none; }
    #loading_text { position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
                    color: #cccccc; font-family: sans-serif; }
  </style>
</head>
<body>
  <canvas id="russet_canvas"></canvas>
  <div id="loading_text">Loading Russet…</div>
  <!-- тег <script> для wasm добавляет trunk сам -->
</body>
</html>
```

Требование к странице: ничего не скроллится (`overflow: hidden`), canvas на весь экран, иначе Space и колесо мыши будут прокручивать страницу вместо приложения.

### 2.3 Ключевые структуры (поля можно дополнять, семантику менять нельзя)

```rust
// canvas.rs
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pixmap: tiny_skia::Pixmap, // RGBA8, premultiplied alpha
}
// методы: new(w, h, Color32) — заливка цветом;
// resize_canvas(w, h) — контент сохраняется, якорь ЛЕВЫЙ ВЕРХНИЙ, новая область прозрачная;
// region_bytes(x, y, w, h) -> Vec<u8> — байты региона для обновления текстуры.

// tools.rs
pub enum Tool { Pencil, Brush, Eraser }
pub struct ToolState {
    pub tool: Tool,
    pub size: f32,                  // 1..=100; карандаш игнорирует (всегда 1)
    pub primary: egui::Color32,     // рисование ЛКМ
    pub secondary: egui::Color32,   // рисование ПКМ (карандаш/кисть)
}

// history.rs
pub struct History { undo: Vec<tiny_skia::Pixmap>, redo: Vec<tiny_skia::Pixmap> }
// push_snapshot(pixmap) — перед началом каждого штриха и перед resize холста;
// лимиты: максимум 20 снимков И суммарно <= 256 МиБ (при переполнении выкидывать самый старый);
// любой новый снимок очищает redo; undo()/redo() возвращают Option<Pixmap>.

// app.rs — состояние приложения
pub struct App {
    canvas: Canvas,
    canvas_tex: egui::TextureHandle,   // egui-текстура холста, обновляется частично
    tools: ToolState,
    history: History,
    zoom: f32,                          // 0.1..=8.0, старт 1.0
    pan: egui::Vec2,                    // смещение холста относительно центра панели, старт (0,0)
    is_stroking: bool,
    last_pos: Option<egui::Pos2>,       // последняя точка штриха В КООРДИНАТАХ ХОЛСТА (пиксели, f32)
    current_path: Option<std::path::PathBuf>, // только натив; на web не используется
    display_name: String,               // "Untitled" по умолчанию; имя файла после Open/Save As
    dirty: bool,                        // есть несохранённые изменения
    show_new_dialog: bool,
    show_resize_dialog: bool,
    tools_window_open: bool,            // true
    palette_window_open: bool,          // true
    error_popup: Option<String>,
    // + буферы диалогов New/Resize: width: f32, height: f32
}
```

**EN:** Fixed layout: `index.html` + `src/{main,app,canvas,tools,history,file_io,palette}.rs`. `main.rs` carries BOTH entries per eframe_template (native `run_native` window 1200×800 min 800×600 titled "Russet"; web `WebRunner` bound to `#russet_canvas`). All platform `cfg` branches live only in `main.rs` and `file_io.rs`. The canvas is a tiny-skia `Pixmap` (premultiplied RGBA8) mirrored into an egui texture; history is snapshot-based.

## 3. Функциональные требования (Requirements)

### 3.1 Верхнее меню (все строки — английские)
`egui::TopBottomPanel` + `egui::menu::bar`. Пункты:

- **File**: New… (Ctrl+N), Open… (Ctrl+O), Save (Ctrl+S), Save As… (Ctrl+Shift+S), разделитель, Exit (только в нативной сборке; на web не показывать).
- **Edit**: Undo (Ctrl+Z), Redo (Ctrl+Y). Пункты задизейблены, когда история пуста.
- **Canvas**: Resize Canvas…
- **View**: Zoom 100% (Ctrl+0), Fit to Window, разделитель, галочки Tools и Palette (открывают/закрывают соответствующие окна).

### 3.2 Окно «Tools»
Плавающее `egui::Window` с заголовком "Tools", открыто по умолчанию, переносимое.

- Три кнопки: «Pencil (P)», «Brush (B)», «Eraser (E)»; активный инструмент визуально выделен (`selected`).
- Слайдер «Size» 1..=100 — влияет на кисть и ластик; для карандаша задизейблен.
- Внизу два квадратика-образца с подписями «Primary» и «Secondary».

### 3.3 Окно «Palette»
Плавающее `egui::Window` с заголовком "Palette", открыто по умолчанию.

- Сетка 10×2 из 20 предустановленных цветов (классическая палитра MS Paint), в `palette.rs` зафиксировать массив:

  | | | | | | | | | | |
  |---|---|---|---|---|---|---|---|---|---|
  | `#000000` | `#7F7F7F` | `#880015` | `#ED1C24` | `#FF7F27` | `#FFF200` | `#22B14C` | `#00A2E8` | `#3F48CC` | `#A349A4` |
  | `#FFFFFF` | `#C3C3C3` | `#B97A57` | `#FFAEC9` | `#FFC90E` | `#EFE4B0` | `#B5E61D` | `#99D9EA` | `#7092BE` | `#C8BFE7` |

- Клик ЛКМ по свотчу — назначить Primary; клик ПКМ — Secondary.
- Две крупные ячейки Primary/Secondary и кнопка «Swap (X)».
- Кнопка «Custom…» — встроенный `ui.color_edit_button_srgba` для Primary. Свой HSV-пикер не писать.

### 3.4 Холст: отображение, зум, панорама, координаты

- Холст рисуется в `egui::CentralPanel`. Фон панели — тёмно-серый `#404040`.
- Под изображением — «шахматка» (прозрачность): сгенерировать один раз текстуру 2×2 пикселя (цвета `#FFFFFF` и `#CCCCCC`) с `TextureOptions` и wrap-режимом Repeat, нарисовать `painter().image(...)` с uv, кратными размеру холста на экране (плитка ~16 экранных пикселей). Если с Repeat возникнут проблемы — допустимый fallback: сплошная заливка `#999999`.
- Поверх шахматки — текстура холста: `painter().image(canvas_tex.id(), screen_rect, uv(0..1), Color32::WHITE)`, где `screen_rect = Rect::from_min_size(panel_center + pan - size*zoom/2, canvas_size * zoom)`.
- **Зум колесом мыши — всегда к позиции курсора.** Шаг: множитель 1.25, clamp 0.1..=8.0. Формула (обязательна):
  `p = (cursor - rect.min) / zoom;` после смены `zoom`: `rect.min' = cursor - p * zoom_new`.
- **Панорама**: drag средней кнопкой ИЛИ drag ЛКМ при зажатом Space. Space-пан имеет приоритет: при зажатом Space штрих не начинается. Состояние Space — через `ctx.input(...).key_down(Key::Space)`.
- Зум сбрасывается в 100%: Ctrl+0 и пункт меню «Zoom 100%»; «Fit to Window» — `zoom = min(panel_w/canvas_w, panel_h/canvas_h)`, clamp тем же диапазоном.
- **Пересчёт координат (критично):** экранная точка → точка холста: `p_canvas = (screen_pos - rect.min) / zoom`; пиксель: `(floor(p_canvas.x), floor(p_canvas.y))`; числа приводить в `i32`/`u32` явно, перед индексацией проверять `0 <= x < width` и `0 <= y < height`. Рисование за границами холста обрезается, не паникует.

### 3.5 Инструменты: карандаш, кисть, ластик

Общий цикл штриха (для всех инструментов):

1. `pointer press` (первичная или вторичная кнопка) на холсте → `history.push_snapshot()`; для карандаша/кисти выбрать цвет: ЛКМ → `primary`, ПКМ → `secondary`; ластик стирает обеими кнопками. Нарисовать точку-касание и запомнить `last_pos`.
2. `pointer drag` → на каждое перемещение рисовать **отрезок** от `last_pos` до текущей позиции (в координатах холста), обновить `last_pos`.
3. `pointer release` → конец штриха: `is_stroking = false`.
4. Каждый сегмент возвращает dirty-прямоугольник (bbox отрезка, расширенный на `ceil(size/2) + 2` px, обрезанный по холсту) — по нему обновляется текстура (§3.4/§5).
5. Во время штриха текстура обновляется по dirty-региону через `set_partial`; после — `ctx.request_repaint()`.

Растеризация сегмента — в `tools.rs`, через `tiny_skia`: `PathBuilder` линия from→to, `Stroke` с шириной и капами; отрезки с round-капами не оставляют разрывов, отдельная интерполяция точек не нужна.

| Инструмент | Width | LineCap | Antialias | Цвет | Примечание |
|---|---|---|---|---|---|
| Pencil | 1 | Round | **выкл** (`paint.anti_alias = false`) | primary/secondary | жёсткий 1-пиксельный след, как в Paint |
| Brush | `size` | Round | вкл | primary/secondary | жёсткая круглая кисть, как «Brush» в Paint.NET |
| Eraser | `size` | Round | вкл | цвет игнорируется | **`paint.blend_mode = BlendMode::DestinationOut`** — стирает до прозрачности |

Ограничение координат: точки отрезка — `f32` в системе холста; клип по `0..width`, `0..height` на стороне растеризации (обрезать сегмент или полагаться на bounds-проверку `tiny_skia` — на усмотрение агента, но без паник и без выхода за `Pixmap`).

### 3.6 Undo / Redo

- Снимок (`Pixmap` целиком) кладётся в историю **перед началом каждого штриха** и **перед изменением размера холста**.
- Лимиты истории: 20 снимков и суммарно ≤ 256 МиБ (`width*height*4` на снимок); при переполнении удаляется самый старый.
- Новый снимок очищает redo. Open и New **сбрасывают** историю полностью.
- Отмена возвращает снимок; при этом текстура холста обновляется целиком (`set`), не `set_partial`.
- Undo/redo на пустой истории — no-op, без паник.

### 3.7 Файлы: New / Open / Save / Save As

- **New…** — диалог с полями Width/Height (по умолчанию 800×600; min 1, max 4096; нечисловой ввод невозможен — `DragValue` или `TextEdit` с фильтрацией), кнопки OK/Cancel. Если `dirty` — сначала подтверждение «You have unsaved changes. Discard them and continue?» (OK/Cancel). Новый холст заливается белым, история сбрасывается, `display_name = "Untitled"`, `current_path = None`.
- **Open…** — если `dirty`, подтверждение как выше. Натив: `rfd::FileDialog::new().add_filter("Images", ["png", "jpg", "jpeg"])` + `image::open`. Web: `rfd::AsyncFileDialog` (§3.10, веха M8) + `image::load_from_memory`. Далее: `to_rgba8` → `Pixmap::from_vec`; размер холста = размер файла; зум 100%, pan (0,0), история сбрасывается, `display_name` = имя файла, на нативе `current_path = Some(path)`, `dirty = false`.
- **Save (Ctrl+S)** — если `current_path == None` (натив) — вести себя как Save As. Записать по расширению пути: `.png` — PNG RGBA; `.jpg`/`.jpeg` — JPEG (альфа компонуется на белый фон, качество 90). После успеха `dirty = false`. На web Save всегда выгружает файл в браузер с текущим `display_name` (§3.10).
- **Save As…** — диалог сохранения с фильтрами «PNG image» и «JPEG image»; если пользователь не ввёл расширение — добавить по выбранному фильтру. Обновить `display_name` и (натив) `current_path`.
- Любая ошибка — НЕ паника и НЕ `unwrap`: popup с заголовком «Error» и текстом (`error_popup`), работа продолжается.
- **Заголовок окна**: формат `{"*" если dirty}{display_name} - Russet` (пример: `*Untitled - Russet`). Обновлять при изменении `dirty`/`display_name` через `ctx.send_viewport_cmd(egui::ViewportCommand::Title(...))`; на web это меняет заголовок вкладки браузера.
- Нативная работа с файлами — синхронно, в главном потоке, без потоков и `tokio`.

**EN:** New/Open/Save/Save As per above; PNG keeps alpha, JPEG is flattened onto white at quality 90. Dirty-flag confirmation before destructive actions; errors in an "Error" popup, never panic. Window title `{"*"}{display_name} - Russet` via `ViewportCommand::Title`. Native file I/O is synchronous.

### 3.8 Диалог «Resize Canvas»

`egui::Window` "Resize Canvas": поля Width/Height (предзаполнены текущими, min 1, max 4096), кнопки OK/Cancel.
По OK: `history.push_snapshot()`, затем `canvas.resize_canvas(w, h)` — контент сохраняется, якорь левый-верхний, добавленная область прозрачная; текстура пересоздаётся целиком; зум и pan не меняются.

### 3.9 Горячие клавиши — ТОЛЬКО этот список

| Клавиши | Действие |
|---|---|
| `P` / `B` / `E` | Pencil / Brush / Eraser |
| `Ctrl+Z`, `Ctrl+Y` (и `Ctrl+Shift+Z`) | Undo / Redo |
| `Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+Shift+S` | New / Open / Save / Save As |
| `Ctrl+0` | Zoom 100% |
| `[` / `]` | размер Brush/Eraser −1 / +1 (в границах 1..=100) |
| `Space` (удерживать) + ЛКМ drag | панорама |
| Колесо мыши | зум к курсору |

Реализация: через `ctx.input_mut(|i| i.consume_key(...))` и модификаторы; **обязательно** гейт `!ctx.wants_keyboard_input()` — хоткеи не должны срабатывать, пока фокус в полях диалогов. Других хоткеев не добавлять. На web браузер может оставить часть Ctrl-сочетаний себе — бороться с этим не нужно, пункты меню обязаны работать всегда.

**EN:** Hotkeys are limited to exactly this table; guard them with `!ctx.wants_keyboard_input()`. On web the browser may keep some Ctrl-shortcuts — do not fight it; menu items must always work.

### 3.10 Web-сборка (WASM) — особенности

- Собирается через **trunk** (§8), страница — `index.html` из §2, canvas `#russet_canvas`.
- **Файловые диалоги — только `rfd::AsyncFileDialog`**: синхронный `rfd::FileDialog` под wasm паникует, использовать его на web запрещено.
  - Open: `AsyncFileDialog::new().add_filter("Images", ["png","jpg","jpeg"]).pick_file().await` → `file.read().await` (байты) → `image::load_from_memory`.
  - Save: `AsyncFileDialog::new().add_filter(...).set_file_name(&display_name).save_file().await` → `handle.write(bytes).await` — браузер скачивает файл. `current_path` на web не используется.
- Асинхронный результат кладётся в `Arc<Mutex<Option<Result<LoadedImage, String>>>>`, после завершения — `ctx.request_repaint()`; App забирает результат в очередном кадре. `spawn_local` из `wasm_bindgen_futures`.
- cfg-ветки платформы допустимы только в `file_io.rs` и в точках вызова из `app.rs`; остальной код о платформе не знает.
- Exit в меню на web не показывается; beforeunload-защиту от закрытия вкладки НЕ делать (§6).

**EN:** Web build via trunk; file dialogs on web are AsyncFileDialog-only (sync panics on wasm); async results delivered to the app via `Arc<Mutex<Option<...>>>` + `ctx.request_repaint()`. Platform `cfg` stays in `file_io.rs` and its call-sites. No beforeunload guard.

## 4. Вехи (Milestones — делать по порядку)

### M0. Каркас
Пакет `russet` (edition 2024), Cargo.toml из §1, `main.rs` с обеими точками входа по §2.1, `App::new` + заглушка «Hello, Russet!» в CentralPanel.
**Готово:** `cargo run` на Windows открывает окно 1200×800 с заголовком «Russet»; check/clippy/fmt чисты.
**EN:** Scaffold package `russet`, Cargo.toml from §1, dual-entry `main.rs` from §2.1, placeholder central panel; `cargo run` opens a 1200×800 window titled "Russet".

### M1. Холст на экране + зум/пан
`Canvas` 800×600, залит белым; текстура egui (`ColorImage::from_rgba_premultiplied`); шахматка; отрисовка в CentralPanel по формулам §3.4; зум колесом к курсору; пан средней кнопкой и Space+ЛКМ; Ctrl+0 и «Fit to Window».
**Готово:** колесо зумит плавно к курсору на всём диапазоне 10–800%; пан работает; при ресайзе окна холст остаётся на месте относительно `pan`.
**EN:** Show the canvas with checkerboard, cursor-anchored wheel zoom, middle-button and Space+LMB pan, Ctrl+0 and Fit to Window.

### M2. Обновление текстуры по dirty-региону
Метод `App::sync_texture(rect)`: из `Pixmap` читается регион и обновляет текстуру через `set_partial` (позиция региона в пикселях текстуры!). Заготовка `App::repaint_full()` — полное `set`.
Unit-тест: `region_bytes` возвращает `w*h*4` байт и правильные угловые пиксели.
**Готово:** временная закраска произвольного прямоугольника холста тестовой кнопкой обновляет на экране только его.
**EN:** Implement partial texture updates from a dirty rect (`set_partial`) plus a full-update helper; add a unit test for `region_bytes`.

### M3. Инструменты + история
`Tool`/`ToolState`, растеризация сегментов по таблице §3.5 (карандаш без AA, кисть AA, ластик `DestinationOut`), цикл штриха press/drag/release, снимок в историю на начало штриха, `History` с лимитами §3.6. Рисование ЛКМ/ПКМ (вторичный цвет). Проверка границ координат.
Unit-тесты: `History` — лимит 20 снимков и эвикция; push очищает redo.
**Готово:** все три инструмента рисуют в соответствии с таблицей; ластик делает прозрачным (видна шахматка); разрывов при быстром движении мыши нет; паник при рисовании за краями холста нет.
**EN:** Implement Pencil/Brush/Eraser per the table (eraser = DestinationOut), stroke press/drag/release lifecycle, snapshot history with 20/256 MiB limits; unit tests for History.

### M4. UI: меню и окна
Меню §3.1 (пункты файлов и undo — пока заглушки-`no-op`), окно «Tools» §3.2, окно «Palette» §3.3, галочки окон в View. Убедиться, что клики по окнам не рисуют на холсте под ними (egui делает это сам — проверить).
**Готово:** инструмент выбирается и из окна, размер слайдером меняется; палитра ЛКМ/ПКМ меняет цвета, «Swap (X)» работает, «Custom…» открывает пикер egui; окна переоткрываются из View.
**EN:** Add the top menu bar, the floating Tools window (tool buttons + size slider), and the Palette window (20 MS-Paint colors, LMB/RMB slots, Swap, egui color picker).

### M5. Файлы (натив, Windows)
`file_io.rs`: New (диалог + подтверждение), Open, Save, Save As по §3.7 — синхронные rfd-диалоги; `dirty`, `display_name`, `current_path`, заголовок окна через `ViewportCommand::Title`; popup «Error». Web-ветки НЕ делать — они появятся в M8.
**Готово:** PNG с прозрачностью корректно сохраняется и открывается; JPEG открывается и сохраняется на белом фоне; Ctrl+S на новом файле открывает save-диалог; подтверждение при потере изменений работает; заголовок показывает `*`.
**EN:** Implement New/Open/Save/Save As natively (PNG alpha, JPEG flattened on white, quality 90), dirty flag, dynamic window title, error popups. No web branches yet.

### M6. Undo/Redo + хоткеи
Undo/redo привязать к меню и клавишам §3.9; полное обновление текстуры при undo/redo; дизейбл пунктов меню на пустой истории; все хоткеи таблицы §3.9 включая `[`/`]`; гейт `wants_keyboard_input`.
**Готово:** 20 шагов отмены на большом числе штрихов; redo работает до нового штриха; хоткеи не срабатывают при фокусе в полях диалогов.
**EN:** Wire undo/redo to menu and hotkeys (full texture update on undo), disable empty-history items, add exactly the §3.9 hotkeys with the keyboard-focus guard.

### M7. Resize Canvas + New
Диалог §3.8: снимок в историю, resize с сохранением контента (якорь левый-верх), пересоздание текстуры целиком. Диалог «New» из M5 (если делался заглушкой — доделать: min 1, max 4096, белый фон).
**Готово:** увеличение и уменьшение холста не теряет рисунок, добавленная зона прозрачна; undo после resize возвращает прежний размер и контент.
**EN:** Canvas-resize dialog (snapshot first, top-left anchor, content preserved), finish the New-canvas dialog with 1..4096 validation.

### M8. Web-сборка (WASM)
Настроить web по §8 (`rustup target add wasm32-unknown-unknown`, `cargo install trunk --locked`), добавить `index.html` из §2.1/§2.2, запустить `trunk serve` и добиться работы приложения в браузере. Затем добавить в `file_io.rs` web-ветки по §3.10 (`AsyncFileDialog`: открытие через `read()`, сохранение через `save_file().write()`, результат через `Arc<Mutex<Option<...>>>` + `ctx.request_repaint()`); в точках вызова `app.rs` — cfg-развилки. Проверить в браузере рисование, зум/пан, undo, Open/Save.
**Готово:** в браузере на `http://localhost:8080` рисуют все три инструмента, ластик прозрачнит, PNG открывается и скачивается, заголовок вкладки обновляется; `trunk build --release` успешен.
**EN:** Set up trunk + `index.html`, get the app running in the browser, then add AsyncFileDialog branches in `file_io.rs` per §3.10; verify drawing, zoom/pan, undo, PNG open/download in the browser.

### M9. Приёмка и полировка
Прогнать чеклист §7 целиком (Windows + web), исправить найденное. Опционально (только если всё остальное готово): круглый курсор-превью кисти/ластика (контур окружности радиуса `size/2 * zoom` на текущей позиции курсора).
**Готово:** весь чеклист §7 выполняется вручную без ошибок; `cargo build --release` успешен.
**EN:** Run the §7 acceptance checklist end-to-end (Windows + web) and fix findings; optional brush-size cursor preview last.

## 5. Известные ловушки (Pitfalls — прочитать до кодинга)

1. **Premultiplied alpha.** `Pixmap` в tiny-skia — premultiplied RGBA8. В egui использовать `ColorImage::from_rgba_premultiplied`, иначе полупрозрачные цвета будут выглядеть неправильно. При записи файлов учесть премультипликацию при компоновке на белый (или распремультиплицировать перед `image::RgbaImage`).
2. **`set_partial`**: первый аргумент — позиция/прямоугольник региона В КООРДИНАТАХ ТЕКСТУРЫ, данные — ровно `w*h*4` байт региона. Несовпадение размеров — паника egui.
3. **Зум к курсору** обязан использовать формулу из §3.4, иначе холст «уплывает».
4. **Координаты**: все переходы экран→холст→пиксель делаются явно (`f32` → `floor` → `i32`/`u32` + clamp/bounds-check). `as` без проверки — источник паник на границах.
5. **Сигнатура `eframe::App`** в 0.37 отличается от старых уроков (метод может называться не `update`). Не копировать старый код — смотреть шаблон той же версии и сообщения компилятора.
6. **Ластик** — это `BlendMode::DestinationOut` на `Pixmap`, а не рисование фоновым цветом.
7. **Правый клик**: у CentralPanel нет контекстного меню по умолчанию, но не вызывать `context_menu`: ПКМ у карандаша/кисти = secondary-цвет, у ластика = стирание.
8. **Пробел-пан**: пока Space зажат — не начинать штрих; если штрих уже идёт, Space его не прерывает (пан активируется только со следующего press).
9. **rfd: платформы разные.** Натив (Windows) — синхронный `rfd::FileDialog`. Web — ТОЛЬКО `rfd::AsyncFileDialog` (синхронный под wasm паникует) + `spawn_local` + `ctx.request_repaint()`.
10. **Браузерные шорткаты**: на web Ctrl+S/Ctrl+O/Ctrl+N может перехватывать браузер — это допустимо и не «баг»; не пытаться блокировать поведение браузера.
11. **UI-строки** — только захардкоженный английский (§0). Никаких `rust-i18n`/`fluent`/файлов переводов.
12. **Сборка**: дев-профиль из §1 уже настроен на скорость и параллелится cargo автоматически — ничего не менять и не отключать. `cargo build --release` (медленный fat-LTO) агенту во время разработки не собирать, кроме финальной проверки M9.

## 6. Вне скоупа (Out of scope — НЕ делать в этой версии)

Слои; выделение и перемещение фрагментов; текст; фигуры (линия/прямоугольник/эллипс); заливка (flood fill); пипетка; трансформации (поворот/отражение/масштаб изображения); история действий списком; сохранение настроек приложения; недавние файлы; темы/кастомизация внешнего вида; кастомная шапка окна (`set_decorations(false)`); многопоточность в коде приложения (rayon/std::thread/tokio); любые новые зависимости; статус-бар; собственный формат проекта; интернационализация и локализация (в любом виде); сборки и CI под macOS и Linux; beforeunload-защита вкладки на web; PWA/офлайн-режим/service worker; оптимизация размера wasm (wasm-opt и т. п.).

## 7. Чеклист приёмки (Acceptance checklist — прогнать вручную в конце M9)

**Windows (натив):**
1. Открывается окно 1200×800 «Russet», видны меню File/Edit/Canvas/View, окна Tools и Palette, белый холст 800×600 на шахматке.
2. Pencil: жёсткая линия 1px (без сглаживания), ЛКМ — Primary, ПКМ — Secondary.
3. Brush: сглаженная линия, размер 1 и 100 дают корректную толщину; `]`/`[` меняют размер.
4. Eraser: стирает до прозрачности (видна шахматка), обеими кнопками.
5. Быстрые зигзаги мышью не оставляют разрывов штриха; рисование за краем холста не паникует и обрезается.
6. Зум колесом к курсору 10–800%; пан средней кнопкой и Space+ЛКМ; Ctrl+0 возвращает 100%; Fit to Window вписывает холст.
7. Undo/redo: последовательность из >20 штрихов отменяется максимум на 20 шагов и не паникует на пустой истории; redo сбрасывается новым штрихом.
8. New: 300×200 даёт белый холст; значения 0 и 5000 отклоняются; при dirty показано подтверждение.
9. Open: PNG с альфой отображается на шахматке правильно (цвета не искажены — проверка premultiplied).
10. Save PNG → файл открывается внешним просмотрщиком, прозрачность на месте; Save JPEG → фон белый.
11. Resize Canvas: содержимое не потеряно, новая зона прозрачна, undo возвращает старый размер.
12. Хоткеи не срабатывают при фокусе в поле ввода диалога.
13. Заголовок вида `*Untitled - Russet` при несохранённых изменениях; Save снимает `*`.

**Web (браузер):**
14. `trunk serve`: страница открывается, «Loading Russet…» исчезает, приложение работает на весь экран без скролла страницы.
15. Рисование, ластик, зум/пан, undo/redo в браузере работают.
16. Open открывает PNG; Save скачивает файл с корректным именем; JPEG скачивается с белым фоном.
17. Заголовок вкладки браузера показывает `*Untitled - Russet` при dirty.
18. Синхронного вызова `rfd::FileDialog` на web нет (только AsyncFileDialog) — приложение не паникует при работе с файлами.

**Сборка:**
19. `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo test` — всё проходит; `cargo build --release` успешен (M9); `trunk build --release` успешен.

## 8. Сборка и запуск (Build & run)

```bash
rustup update                                   # нужен Rust >= 1.95

# --- Ежедневная разработка (Windows) ---
cargo run                                       # дев-сборка; первый запуск долгий (зависимости один раз), дальше — секунды
cargo test                                      # unit-тесты (history, region_bytes)
cargo clippy -- -D warnings

# --- Релиз: ВРУЧНУЮ, когда всё готово (fat LTO, медленный) ---
cargo build --release

# --- Web (веха M8), ставится один раз ---
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve                                     # http://localhost:8080
trunk build --release                           # статика в dist/
```

Cargo и trunk собирают параллельно на все логические ядра по умолчанию — флаги `-j` не нужны и не указывать.
(Для справки на будущее: Linux-сборке потребуются пакеты `libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-dev` — сейчас НЕ ставить и не проверять.)
