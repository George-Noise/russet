//! Tests of the launch arguments: the PURE parser (`parse`) and the parsed
//! config's application to a REAL headless `App` (`new_headless_with` — the
//! same `build` path the entry points run). The startup-canvas promise: the
//! arguments set the STARTUP document (buffers + the white initial layer +
//! the Resize-dialog prefill), while the New dialog keeps its 800×600
//! constants.

use super::App;
use super::{
    LaunchConfig, TOOL_NAMES, parse_canvas, parse_color, parse_hex_components, parse_tool,
};
use crate::USAGE;
use crate::tools::Tool;
use eframe::egui::Color32;

/// The test's argv: `parse` is generic over `Item = String`.
fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| (*word).to_string()).collect()
}

fn canvas(config: &LaunchConfig) -> (u32, u32) {
    config.canvas.expect("the test set --canvas-size")
}

#[test]
fn parsing_nothing_keeps_every_default() {
    let config = LaunchConfig::parse(args(&[]));

    assert!(!config.help);
    assert_eq!(config.primary, None);
    assert_eq!(config.secondary, None);
    assert_eq!(config.tool, None);
    assert_eq!(config.canvas, None);
    assert!(
        config.warnings.is_empty(),
        "unexpected warnings: {:?}",
        config.warnings
    );
}

#[test]
fn every_argument_reaches_the_real_app() {
    let config = LaunchConfig::parse(args(&[
        "--primary",
        "FF0000",
        "--secondary",
        "#00FF0088",
        "--tool",
        "brush",
        "--canvas-size",
        "1024x768",
    ]));
    assert!(
        config.warnings.is_empty(),
        "unexpected warnings: {:?}",
        config.warnings
    );

    let app = App::new_headless_with(&config);

    assert_eq!((app.canvas.width, app.canvas.height), (1024, 768));
    let layer = app.layers.layer(0).expect("the initial layer");
    assert_eq!(
        (layer.pixmap().width(), layer.pixmap().height()),
        (1024, 768)
    );
    assert_eq!(app.tools.tool, Tool::Brush);
    assert_eq!(
        app.last_tool,
        Tool::Brush,
        "the first frame must not see a tool switch"
    );
    assert_eq!(app.tools.primary, Color32::from_rgb(255, 0, 0));
    assert_eq!(
        app.tools.secondary,
        Color32::from_rgba_unmultiplied(0, 255, 0, 136)
    );
    // The white initial layer fills the custom startup canvas (the same
    // §3.8 promise the default document has).
    let corner = app
        .canvas
        .pixmap()
        .pixel(1023, 767)
        .expect("in bounds")
        .demultiply();
    assert_eq!(
        (corner.red(), corner.green(), corner.blue(), corner.alpha()),
        (255, 255, 255, 255)
    );
}

#[test]
fn equals_form_three_digit_color_and_limit_canvas() {
    let config = LaunchConfig::parse(args(&[
        "--primary=#0f0",
        "--tool=shape",
        "--canvas-size=4096x4096",
    ]));

    assert!(
        config.warnings.is_empty(),
        "unexpected warnings: {:?}",
        config.warnings
    );
    assert_eq!(config.primary, Some(Color32::from_rgb(0, 255, 0)));
    assert_eq!(config.tool, Some(Tool::Shape));
    assert_eq!(config.canvas, Some((4096, 4096)));
}

#[test]
fn values_parse_case_insensitively() {
    let config = LaunchConfig::parse(args(&[
        "--tool",
        "BRUSH",
        "--canvas-size",
        "640X480",
        "--primary",
        "#ff0000",
    ]));

    assert_eq!(config.tool, Some(Tool::Brush));
    assert_eq!(config.canvas, Some((640, 480)));
    assert_eq!(config.primary, Some(Color32::from_rgb(255, 0, 0)));
}

#[test]
fn invalid_values_warn_and_keep_the_defaults() {
    let config = LaunchConfig::parse(args(&[
        "--primary",
        "nope",
        "--secondary",
        "#",
        "--tool",
        "marker",
        "--canvas-size",
        "800",
        "--canvas-size",
        "abcx600",
        "--colour",
        "red",
    ]));

    assert_eq!(config.primary, None);
    assert_eq!(config.secondary, None);
    assert_eq!(config.tool, None);
    assert_eq!(config.canvas, None);
    assert_eq!(config.warnings.len(), 7, "warnings: {:?}", config.warnings);
    assert!(config.warnings[0].contains("--primary"));
    assert!(config.warnings[1].contains("--secondary"));
    assert!(config.warnings[2].contains("--tool"));
    assert!(config.warnings[3].contains("--canvas-size"));
    assert!(config.warnings[4].contains("--canvas-size"));
    // `--colour red` warns twice: the unknown flag AND its stray word.
    assert!(config.warnings[5].contains("unrecognized"));
    assert!(config.warnings[6].contains("unrecognized"));
}

#[test]
fn out_of_range_canvas_clamps_to_the_dialog_limits() {
    let config = LaunchConfig::parse(args(&["--canvas-size", "9999x0"]));

    assert_eq!(canvas(&config), (4096, 1));
    assert!(
        config
            .warnings
            .iter()
            .any(|warning| warning.contains("clamped")),
        "warnings: {:?}",
        config.warnings
    );
}

#[test]
fn repeated_flag_warns_and_keeps_the_last_value() {
    let config = LaunchConfig::parse(args(&["--tool", "line", "--tool", "text"]));

    assert_eq!(config.tool, Some(Tool::Text));
    assert!(
        config
            .warnings
            .iter()
            .any(|warning| warning.contains("twice")),
        "warnings: {:?}",
        config.warnings
    );
}

#[test]
fn missing_value_reports_and_lets_the_next_flag_pass() {
    let config = LaunchConfig::parse(args(&["--primary", "--tool", "brush"]));

    assert_eq!(config.primary, None);
    assert_eq!(config.tool, Some(Tool::Brush));
    assert!(
        config
            .warnings
            .iter()
            .any(|warning| warning.contains("--primary") && warning.contains("missing")),
        "warnings: {:?}",
        config.warnings
    );
}

#[test]
fn help_flag_is_detected_alone() {
    assert!(LaunchConfig::parse(args(&["--help"])).help);
    assert!(LaunchConfig::parse(args(&["-h"])).help);

    let config = LaunchConfig::parse(args(&["--help", "--tool", "eraser"]));
    assert!(config.help);
    assert_eq!(
        config.tool,
        Some(Tool::Eraser),
        "--help does not stop parsing"
    );
}

#[test]
fn tool_names_and_help_stay_in_sync() {
    assert_eq!(TOOL_NAMES.len(), 7);
    for name in TOOL_NAMES {
        assert!(parse_tool(name).is_some(), "'{name}' must parse");
        assert!(USAGE.contains(name), "--help must list '{name}'");
    }
    assert!(parse_tool("Marker").is_none(), "unknown names stay None");
}

#[test]
fn startup_canvas_drives_the_resize_dialog_prefill_only() {
    let config = LaunchConfig::parse(args(&["--canvas-size", "640x480"]));
    let mut app = App::new_headless_with(&config);

    app.open_resize_dialog();
    assert_eq!((app.resize_width, app.resize_height), (640.0, 480.0));

    app.open_new_dialog();
    assert_eq!(
        (app.new_width, app.new_height),
        (800.0, 600.0),
        "the New dialog keeps its constants — the arguments set the STARTUP canvas only"
    );
}

#[test]
fn color_and_canvas_helpers_reject_garbage() {
    assert_eq!(parse_color(""), None);
    assert_eq!(parse_color("0x"), None);
    assert_eq!(parse_color("#F"), None);
    assert_eq!(parse_color("#12"), None);
    assert_eq!(parse_color("123456789"), None);
    assert_eq!(parse_color("zzzzzz"), None);
    assert_eq!(parse_color("#GGGGGG"), None);
    assert_eq!(parse_canvas("800"), None);
    assert_eq!(parse_canvas("x"), None);
    assert_eq!(parse_canvas("800x"), None);
    assert_eq!(parse_canvas("800xsix"), None);
    assert_eq!(parse_canvas("-800x600"), None);
}

#[test]
fn color_hex_is_junk_tolerant_and_reads_the_digit_count() {
    // The owner's paste list (2026-11-19): every non-hex-digit character is
    // stripped brazenly — doubled `#`, spaces, an `0x` prefix — and the
    // remaining DIGIT COUNT picks the form. The 5- and 7-digit forms carry
    // different RGB digit counts (3 vs 6): the owner calls the asymmetry
    // deliberate, do not "fix" it into one shape.
    // 4 digits: `RGBA`, every digit doubled.
    let ([r, g, b], alpha) = parse_hex_components("##f70a").expect("parses");
    assert_eq!((r, g, b, alpha), (0xFF, 0x77, 0x00, Some(0xAA)));
    // 5 digits: the shorthand `RGB` + a FULL alpha byte.
    let ([r, g, b], alpha) = parse_hex_components("f70ab").expect("parses");
    assert_eq!((r, g, b, alpha), (0xFF, 0x77, 0x00, Some(0xAB)));
    // 7 digits: full `RRGGBB` + a DOUBLED alpha digit.
    let ([r, g, b], alpha) = parse_hex_components("ff7f00c").expect("parses");
    assert_eq!((r, g, b, alpha), (0xFF, 0x7F, 0x00, Some(0xCC)));
    // Junk-stripping: the 0x prefix, inner spaces, the doubled hash.
    assert_eq!(parse_color("0xFF7F00"), parse_color("#ff7f00"));
    assert_eq!(parse_color("ff 7f 00"), parse_color("#ff7f00"));
    assert_eq!(parse_color("##ff7f00"), parse_color("#ff7f00"));
    // The alpha-less forms stay opaque through the launch-argument lens.
    assert_eq!(
        parse_color("f70"),
        Some(Color32::from_rgb(0xFF, 0x77, 0x00))
    );
}
