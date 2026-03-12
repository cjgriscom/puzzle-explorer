use std::path::Path;
use std::process::Command;

use chrono::Utc;

/// Glyph subsets to extract from source fonts.
/// Each entry: (source TTF, output TTF, list of Unicode codepoints)
const FONT_SUBSETS: &[(&str, &str, &[&str])] = &[
    (
        "fonts/sources/emoji-icon-font.ttf",
        "fonts/generated/icons.ttf",
        &[
            "U+1F5D1", // 🗑
            "U+270F",  // ✏
            "U+1F441", // 👁
            "U+21BB",  // ↻
            "U+21BA",  // ↺
        ],
    ),
    (
        "fonts/sources/Hack-Regular.ttf",
        "fonts/generated/arrows.ttf",
        &[
            "U+25B2", // ▲
            "U+25BC", // ▼
        ],
    ),
];

fn main() {
    // --- Build Date ---
    let build_date = Utc::now().format("%-Y-%m-%d %-H:%M UTC").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // --- Glyph subsets ---
    let out_dir = Path::new("fonts/generated");
    std::fs::create_dir_all(out_dir).expect("failed to create fonts/generated");

    for (source, output, codepoints) in FONT_SUBSETS {
        println!("cargo:rerun-if-changed={}", source);

        let unicodes = codepoints.join(",");
        let status = Command::new("pyftsubset")
            .arg(source)
            .arg(format!("--unicodes={}", unicodes))
            .arg(format!("--output-file={}", output))
            .arg("--no-hinting")
            .arg("--desubroutinize")
            .status()
            .expect("failed to run pyftsubset (is fonttools installed?)");

        assert!(
            status.success(),
            "pyftsubset failed for {} -> {}",
            source,
            output
        );

        let size = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);
        assert!(size > 200, "Failed to generate font subset");
    }
}
