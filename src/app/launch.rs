//! Launch arguments (owner request 2026-11-17): the native command line —
//! `--primary` / `--secondary` color wells, `--tool`, `--canvas-size`,
//! `--help`.
//!
//! OWNS: parsing argv into [`LaunchConfig`] (the pure [`LaunchConfig::parse`]
//! with its [`LaunchConfig::from_env`] adapter) and applying the parsed
//! choices to `App` ([`LaunchConfig::apply_to`]; the CANVAS SIZE is read by
//! `App::build` directly — it constructs the buffers). The color-hex TEXT
//! SYNTAX is owned here too as [`parse_hex_components`] — one primitive for
//! BOTH the `--primary`/`--secondary` flags and the Palette window's hex
//! field (`super::windows`, owner request 2026-11-19), so the two can never
//! drift apart. NEIGHBOURS: `main.rs`
//! calls `from_env` / `report_warnings`, OWNS the `--help` TEXT (printing is
//! entry-point logic — the web build has no argv and never prints it; the
//! platform cfg lives in `main.rs` per AGENTS.md "Repo facts") and exits on
//! `--help` BEFORE opening a window; `App::build` ([`super`]) takes the
//! parsed config; the New dialog's 800×600 constants (`super::document`)
//! stay untouched — the arguments set the STARTUP canvas, not the New-dialog
//! defaults.
//!
//! Platform note: ONE code path compiles for native and wasm alike —
//! `std::env::args_os` yields nothing on wasm32, so the web build silently
//! runs with the defaults (no `#[cfg]` needed; platform cfg branches live in
//! `main.rs` per AGENTS.md "Repo facts").
//!
//! Failure policy: NOTHING panics and NOTHING aborts the launch — an
//! invalid value prints a warning to stderr and keeps the built-in default;
//! unknown arguments are ignored with a warning (`--mcp` is skipped
//! silently: it belongs to the `mcp` feature, `super::mcp`).

use eframe::egui::Color32;

use super::App;
use super::document::{MAX_CANVAS, MIN_CANVAS};
use crate::tools::Tool;

/// The valid `--tool` names, in the [`Tool`] enum's order. The `--tool`
/// warnings print this list; the `--help` text (`main.rs`) stays in sync —
/// pinned by a test.
pub(crate) const TOOL_NAMES: [&str; 7] = [
    "pencil",
    "brush",
    "eraser",
    "eyedropper",
    "line",
    "shape",
    "text",
];

/// The parsed launch arguments of this process. Everything is `Option` —
/// `None` means "not given, the built-in default stays" — and every parse
/// problem collects into [`LaunchConfig::warnings`] instead of aborting.
#[derive(Default)]
pub(crate) struct LaunchConfig {
    /// `-h` / `--help` was given: the NATIVE entry prints its help text
    /// (main.rs) to stdout and exits BEFORE opening a window; the web build
    /// has no argv, so the flag stays false there and is only logged.
    pub(crate) help: bool,
    /// `--primary`: the stroke color well. Hex `RRGGBB`, `#RGB`, `RRGGBBAA`
    /// too; alpha defaults to 255.
    pub(crate) primary: Option<Color32>,
    /// `--secondary`: the secondary color well, the same formats.
    pub(crate) secondary: Option<Color32>,
    /// `--tool`: the tool active on the first frame.
    pub(crate) tool: Option<Tool>,
    /// `--canvas-size` (width, height), ALREADY clamped into
    /// `MIN_CANVAS..=MAX_CANVAS` at parse time (a clamp warns).
    pub(crate) canvas: Option<(u32, u32)>,
    /// Human-readable parse problems; `main.rs` prints each to stderr
    /// (`report_warnings`). Nothing aborts, the defaults stay.
    pub(crate) warnings: Vec<String>,
}

impl LaunchConfig {
    /// The real entry point's adapter: everything after the program name.
    /// `args_os` (not `args`) — a non-UTF-8 argument must not panic: it is
    /// lossily kept and, being unrecognized, warned and ignored.
    pub(crate) fn from_env() -> Self {
        Self::parse(
            std::env::args_os()
                .skip(1)
                .map(|arg| arg.to_string_lossy().into_owned()),
        )
    }

    /// The pure parser: `--flag value` and `--flag=value` forms, `-h` /
    /// `--help`, an unrecognized word warns and is ignored. A flag whose
    /// value is missing (nothing follows, or the next word starts with
    /// `-`) warns and lets the next word parse as a flag.
    pub(crate) fn parse<I: IntoIterator<Item = String>>(args: I) -> Self {
        let mut config = Self::default();
        // The flag whose VALUE the previous iteration expected (`--flag
        // value`), when it did not carry an inline one (`--flag=value`).
        // `take` clears it the moment the value is consumed (or declared
        // missing), so the trailing check sees only a really un-fed flag.
        let mut awaiting: Option<&'static str> = None;
        for arg in args {
            // A value is due, and this does not look like one: report the
            // missing value and treat this word as the NEXT flag instead.
            if let Some(flag) = awaiting.take() {
                if !arg.starts_with('-') {
                    config.set_value(flag, &arg);
                    continue;
                }
                config.warn(format!("--{flag} is missing its value — the default stays"));
            }
            let (name, inline) = match arg.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (arg.as_str(), None),
            };
            let flag: &'static str = match name {
                "-h" | "--help" => {
                    config.help = true;
                    if let Some(value) = inline {
                        config.warn(format!("--help takes no value ('{value}' ignored)"));
                    }
                    continue;
                }
                // Owned by the `mcp` feature (`super::mcp::maybe_spawn`
                // scans argv itself) — not an error here, just not ours.
                "--mcp" => continue,
                "--primary" => "primary",
                "--secondary" => "secondary",
                "--tool" => "tool",
                "--canvas-size" => "canvas-size",
                _ => {
                    config.warn(format!(
                        "unrecognized argument '{arg}' ignored (try --help)"
                    ));
                    continue;
                }
            };
            match inline {
                Some(value) => config.set_value(flag, value),
                None => awaiting = Some(flag),
            }
        }
        if let Some(flag) = awaiting {
            config.warn(format!("--{flag} is missing its value — the default stays"));
        }
        config
    }

    /// Prints every parse problem to stderr — each entry point calls this
    /// once (a release build on Windows has no console by
    /// `windows_subsystem`; the daily `cargo run` lane does).
    pub(crate) fn report_warnings(&self) {
        for warning in &self.warnings {
            eprintln!("russet: {warning}");
        }
    }

    /// Applies the parsed choices to the freshly built `App` (the tail of
    /// `App::build`): the color wells and the startup tool — plus
    /// `last_tool`, so the "tool changed since the previous frame" guard
    /// does not fire on the first frame. The CANVAS SIZE is not here: it
    /// constructs the buffers in `App::build` itself.
    pub(crate) fn apply_to(&self, app: &mut App) {
        if let Some(color) = self.primary {
            app.tools.primary = color;
        }
        if let Some(color) = self.secondary {
            app.tools.secondary = color;
        }
        if let Some(tool) = self.tool {
            app.tools.tool = tool;
            app.last_tool = tool;
        }
    }

    fn warn(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    fn set_value(&mut self, flag: &'static str, value: &str) {
        match flag {
            "primary" | "secondary" => self.set_color(flag, value),
            "tool" => self.set_tool(value),
            _ => self.set_canvas(value),
        }
    }

    fn set_color(&mut self, flag: &'static str, value: &str) {
        let Some(color) = parse_color(value) else {
            self.warn(format!(
                "--{flag}: invalid color '{value}' (expected hex RRGGBB; #RGB, RRGGBB and \
                 RRGGBBAA also work) — the default stays"
            ));
            return;
        };
        let slot = if flag == "primary" {
            &mut self.primary
        } else {
            &mut self.secondary
        };
        // `replace` already stores the new value; `Some(old)` means the
        // flag was given twice — the last value wins, loudly.
        if slot.replace(color).is_some() {
            self.warn(format!("--{flag} given twice — the last value wins"));
        }
    }

    fn set_tool(&mut self, value: &str) {
        let Some(tool) = parse_tool(value) else {
            self.warn(format!(
                "--tool: unknown tool '{}' (one of: {}) — the default stays",
                value.trim(),
                TOOL_NAMES.join(", ")
            ));
            return;
        };
        if self.tool.replace(tool).is_some() {
            self.warn("--tool given twice — the last value wins");
        }
    }

    fn set_canvas(&mut self, value: &str) {
        let Some((raw_width, raw_height)) = parse_canvas(value) else {
            self.warn(format!(
                "--canvas-size: invalid size '{value}' (expected WIDTHxHEIGHT, e.g. \
                 1024x768) — the default stays"
            ));
            return;
        };
        // The same 1..=4096 per-side clamp the New / Resize dialogs enforce
        // (`super::document`); a clamp is reported, not silent.
        let clamp = |side: u32| side.clamp(MIN_CANVAS, MAX_CANVAS);
        let (width, height) = (clamp(raw_width), clamp(raw_height));
        if (width, height) != (raw_width, raw_height) {
            self.warn(format!(
                "--canvas-size {raw_width}x{raw_height} is out of the 1..={MAX_CANVAS} \
                 limit — clamped to {width}x{height}"
            ));
        }
        if self.canvas.replace((width, height)).is_some() {
            self.warn("--canvas-size given twice — the last value wins");
        }
    }
}

/// The color-hex TEXT SYNTAX shared by the `--primary`/`--secondary` launch
/// flags and the Palette window's hex field (owner request 2026-11-19).
/// JUNK-TOLERANT (owner request 2026-11-19: keep only what means color and
/// delete every other character): an optional `0x`/`0X` prefix is dropped
/// first, then EVERY non-hex-digit character (`#` signs, spaces, anything
/// else) is stripped, and the remaining digit COUNT picks the form:
/// 3 = `RGB` (each digit doubled: `#0f0` = `#00ff00`), 4 = `RGBA` (each
/// doubled), 5 = `RGBAA` (the shorthand RGB + a FULL alpha byte), 6 =
/// `RRGGBB`, 7 = `RRGGBBA` (the full RGB + a DOUBLED alpha digit), 8 =
/// `RRGGBBAA`. The 5- and 7-digit forms carry DIFFERENT RGB digit counts
/// (3 vs 6) — the owner calls the asymmetry deliberate. Fewer than 3 or
/// more than 8 digits left is `None`. Returns the RGB triple plus the
/// alpha ONLY when the text carried one — the CALLER decides the fallback,
/// which is the one behavioral difference between the two consumers: the
/// launch flags default a missing alpha to opaque, the hex field keeps the
/// alpha the well had when its edit session started (the way its R/G/B
/// DragValues don't touch it either).
pub(crate) fn parse_hex_components(value: &str) -> Option<([u8; 3], Option<u8>)> {
    let trimmed = value.trim();
    let no_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    let hex: String = no_prefix
        .chars()
        .filter(|character| character.is_ascii_hexdigit())
        .collect();
    // The filter above leaves only ASCII hex digits — one byte per char —
    // so the byte-indexed slicing below stays panic-free.
    let pair = |index: usize| u8::from_str_radix(&hex[index..index + 2], 16).ok();
    let digit = |index: usize| {
        u8::from_str_radix(&hex[index..index + 1], 16)
            .ok()
            .map(|digit| digit * 17) // one hex digit doubled: F -> FF
    };
    match hex.len() {
        3 => Some(([digit(0)?, digit(1)?, digit(2)?], None)),
        4 => Some(([digit(0)?, digit(1)?, digit(2)?], Some(digit(3)?))),
        5 => Some(([digit(0)?, digit(1)?, digit(2)?], Some(pair(3)?))),
        6 => Some(([pair(0)?, pair(2)?, pair(4)?], None)),
        7 => Some(([pair(0)?, pair(2)?, pair(4)?], Some(digit(6)?))),
        8 => Some(([pair(0)?, pair(2)?, pair(4)?], Some(pair(6)?))),
        _ => None,
    }
}

/// The launch-argument color form of [`parse_hex_components`]: a missing
/// alpha means OPAQUE (the flags set a startup color, not a partial edit).
pub(crate) fn parse_color(value: &str) -> Option<Color32> {
    parse_hex_components(value)
        .map(|([r, g, b], alpha)| Color32::from_rgba_unmultiplied(r, g, b, alpha.unwrap_or(255)))
}

/// The `--tool` name → [`Tool`] map: case-insensitive, the names of
/// [`TOOL_NAMES`].
pub(crate) fn parse_tool(value: &str) -> Option<Tool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "pencil" => Some(Tool::Pencil),
        "brush" => Some(Tool::Brush),
        "eraser" => Some(Tool::Eraser),
        "select" => Some(Tool::Select),
        "eyedropper" => Some(Tool::Eyedropper),
        "line" => Some(Tool::Line),
        "shape" => Some(Tool::Shape),
        "text" => Some(Tool::Text),
        _ => None,
    }
}

/// `WIDTHxHEIGHT` (also `X` and `×` as the separator, surrounding spaces
/// fine). No clamping here — [`LaunchConfig`] clamps and warns.
pub(crate) fn parse_canvas(value: &str) -> Option<(u32, u32)> {
    let (width, height) = value.trim().split_once(['x', 'X', '×'])?;
    Some((width.trim().parse().ok()?, height.trim().parse().ok()?))
}

#[cfg(test)]
mod tests;
