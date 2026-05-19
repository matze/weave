use syntect::html::{ClassStyle, css_for_theme_with_class_style};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    std::fs::write(format!("{out_dir}/highlight.css"), build_highlight_css())
        .expect("writing highlight CSS");
}

fn build_highlight_css() -> String {
    let class_style = ClassStyle::SpacedPrefixed { prefix: "hl-" };

    let extra = two_face::theme::extra();
    let light_theme = extra.get(two_face::theme::EmbeddedThemeName::InspiredGithub);

    let light_css = css_for_theme_with_class_style(light_theme, class_style).unwrap();
    let light_css = strip_background_color(&light_css);

    let dark_theme = extra.get(two_face::theme::EmbeddedThemeName::Nord);
    let dark_css = css_for_theme_with_class_style(dark_theme, class_style).unwrap();
    let dark_css = strip_background_color(&dark_css);

    // The dark theme applies in two cases:
    //   * the user picked dark explicitly       (<html data-theme="dark">)
    //   * the OS prefers dark and the user has not picked light
    let dark_explicit = prefix_selectors(&dark_css, r#"[data-theme="dark"]"#);
    let dark_os = prefix_selectors(&dark_css, r#":root:not([data-theme="light"])"#);

    format!(
        "{light_css}\n{dark_explicit}\n@media (prefers-color-scheme: dark) {{\n{dark_os}\n}}\n"
    )
}

fn strip_background_color(css: &str) -> String {
    css.lines()
        .filter(|line| !line.contains("background-color"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Prefix every selector in `css` with `prefix` + space. Handles comma-separated
/// selector lists and ignores lines that don't open a rule.
fn prefix_selectors(css: &str, prefix: &str) -> String {
    css.lines()
        .map(|line| {
            let Some(brace_pos) = line.find('{') else {
                return line.to_string();
            };
            let (selectors_part, rest) = line.split_at(brace_pos);
            let selectors = selectors_part.trim();
            if selectors.is_empty() {
                return line.to_string();
            }
            let prefixed = selectors
                .split(',')
                .map(|s| format!("{prefix} {}", s.trim()))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{prefixed} {rest}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
