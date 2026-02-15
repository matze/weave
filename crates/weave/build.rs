use syntect::html::{ClassStyle, css_for_theme_with_class_style};

fn main() {
    println!("cargo:rerun-if-changed=input.css");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/render.rs");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let result = std::process::Command::new("tailwindcss")
        .args([
            "--input",
            &format!("{manifest_dir}/input.css"),
            "--output",
            &format!("{out_dir}/app.css"),
            "--minify",
        ])
        .output();

    let output = match result {
        Ok(output) => output,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => panic!("tailwindcss binary not found"),
            _ => panic!("failed to run tailwindcss: {err}"),
        },
    };

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        println!("cargo:warning=Failed building CSS");
        println!("cargo:warning={error}");
    }

    std::fs::write(&format!("{out_dir}/highlight.css"), build_highlight_css())
        .expect("writing highlight CSS");
}

fn build_highlight_css() -> String {
    let class_style = ClassStyle::SpacedPrefixed { prefix: "hl-" };

    let extra = two_face::theme::extra();
    let light_theme = extra.get(two_face::theme::EmbeddedThemeName::InspiredGithub);

    let light_css = css_for_theme_with_class_style(&light_theme, class_style).unwrap();
    // Strip background-color rules, Tailwind handles backgrounds.
    let light_css = strip_background_color(&light_css);

    let dark_theme = extra.get(two_face::theme::EmbeddedThemeName::Nord);
    let dark_css = css_for_theme_with_class_style(&dark_theme, class_style).unwrap();
    let dark_css = strip_background_color(&dark_css);

    format!("{light_css}\n@media (prefers-color-scheme: dark) {{\n{dark_css}\n}}\n")
}

fn strip_background_color(css: &str) -> String {
    css.lines()
        .filter(|line| !line.contains("background-color"))
        .collect::<Vec<_>>()
        .join("\n")
}
