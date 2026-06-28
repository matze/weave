//! Render Markdown as HTML.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{LazyLock, Mutex};

use maud::{Markup, PreEscaped, html};
use merman::render::HeadlessRenderer;
use merman::MermaidConfig;
use pulldown_cmark::{BlockQuoteKind, CodeBlockKind, Event, Options, Parser, Tag as CmarkTag};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;

#[derive(Debug, Clone, Copy)]
enum Segment<'a> {
    Text(&'a str),
    Tag(&'a str),
    ColonTags(&'a str),
    Url(&'a str),
}

static SPLITTER_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?m)(?P<tag>#[\w]+)|(?P<url>https?://[^\s<>]+)|(?:(?:^|\s)(?P<colontags>:[\w-]+(?::[\w-]+)*:))"#,
    )
    .expect("compiling regex")
});

struct Splitter<'a> {
    text: &'a str,
    pos: usize,
    next: Option<(usize, usize, Segment<'a>)>,
}

impl<'a> Splitter<'a> {
    fn next_match(
        re: &regex::Regex,
        text: &'a str,
        start: usize,
    ) -> Option<(usize, usize, Segment<'a>)> {
        let caps = re.captures_at(text, start)?;
        if let Some(m) = caps.name("colontags") {
            Some((m.start(), m.end(), Segment::ColonTags(m.as_str())))
        } else {
            let m = caps.get(0).unwrap();
            if caps.name("tag").is_some() {
                Some((m.start(), m.end(), Segment::Tag(m.as_str())))
            } else {
                let trimmed = m.as_str().trim_end_matches(|c: char| {
                    matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']')
                });
                let end = m.start() + trimmed.len();
                Some((m.start(), end, Segment::Url(trimmed)))
            }
        }
    }

    fn new(text: &'a str) -> Self {
        let next = Self::next_match(&SPLITTER_RE, text, 0);

        Self { text, pos: 0, next }
    }
}

impl<'a> Iterator for Splitter<'a> {
    type Item = Segment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.text.len() {
            return None;
        }

        match self.next {
            Some((start, end, segment)) => {
                if self.pos < start {
                    let text = &self.text[self.pos..start];
                    self.pos = start;
                    Some(Segment::Text(text))
                } else {
                    self.pos = end;
                    self.next = Self::next_match(&SPLITTER_RE, self.text, self.pos);
                    Some(segment)
                }
            }
            None => {
                let text = &self.text[self.pos..];
                self.pos = self.text.len();
                Some(Segment::Text(text))
            }
        }
    }
}

enum MdTag {
    Root,
    Paragraph,
    Heading(u8),
    BlockQuote,
    Admonition(BlockQuoteKind),
    CodeBlock(Option<String>),
    OrderedList,
    UnorderedList,
    ListItem,
    Emphasis,
    Strong,
    Strikethrough,
    WikiLink(String),
    ExternalLink(String),
    Table,
    TableHead,
    TableRow,
    TableHeadCell,
    TableBodyCell,
    Image { url: String, title: String },
}

enum MdNode {
    Element(MdTag, Vec<MdNode>),
    /// Text that goes through `Splitter` for hashtag/URL detection.
    Text(String),
    /// Text rendered verbatim (inside code blocks, link labels).
    Plain(String),
    InlineCode(String),
    RawHtml(String),
    SoftBreak,
    HardBreak,
    Rule,
}

static WIKI_LINK_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^(?:\.{0,2}/)*(?P<stem>[\w-]+)$").expect("compiling regex"));

fn build_tree(parser: Parser) -> MdNode {
    let mut stack: Vec<(MdTag, Vec<MdNode>)> = vec![(MdTag::Root, Vec::new())];

    for event in parser {
        match event {
            Event::Start(tag) => {
                let md_tag = match tag {
                    CmarkTag::Paragraph => MdTag::Paragraph,
                    CmarkTag::Heading { level, .. } => MdTag::Heading(level as u8),
                    CmarkTag::BlockQuote(Some(kind)) => MdTag::Admonition(kind),
                    CmarkTag::BlockQuote(_) => MdTag::BlockQuote,
                    CmarkTag::CodeBlock(kind) => {
                        let lang = match kind {
                            CodeBlockKind::Fenced(lang) if !lang.is_empty() => {
                                Some(lang.to_string())
                            }
                            _ => None,
                        };
                        MdTag::CodeBlock(lang)
                    }
                    CmarkTag::List(Some(_)) => MdTag::OrderedList,
                    CmarkTag::List(None) => MdTag::UnorderedList,
                    CmarkTag::Item => MdTag::ListItem,
                    CmarkTag::Emphasis => MdTag::Emphasis,
                    CmarkTag::Strong => MdTag::Strong,
                    CmarkTag::Strikethrough => MdTag::Strikethrough,
                    CmarkTag::Link { dest_url, .. } => {
                        if let Some(caps) = WIKI_LINK_RE.captures(&dest_url) {
                            MdTag::WikiLink(caps["stem"].to_string())
                        } else {
                            MdTag::ExternalLink(dest_url.to_string())
                        }
                    }
                    CmarkTag::Table(_) => MdTag::Table,
                    CmarkTag::TableHead => MdTag::TableHead,
                    CmarkTag::TableRow => MdTag::TableRow,
                    CmarkTag::TableCell => {
                        if stack
                            .iter()
                            .rev()
                            .any(|(t, _)| matches!(t, MdTag::TableHead))
                        {
                            MdTag::TableHeadCell
                        } else {
                            MdTag::TableBodyCell
                        }
                    }
                    CmarkTag::Image {
                        dest_url, title, ..
                    } => MdTag::Image {
                        url: dest_url.to_string(),
                        title: title.to_string(),
                    },
                    _ => MdTag::Root,
                };
                stack.push((md_tag, Vec::new()));
            }
            Event::End(_) => {
                let (tag, children) = stack.pop().unwrap();
                stack
                    .last_mut()
                    .unwrap()
                    .1
                    .push(MdNode::Element(tag, children));
            }
            Event::Text(t) => {
                let suppress_splitter = stack.iter().any(|(tag, _)| {
                    matches!(
                        tag,
                        MdTag::CodeBlock(_) | MdTag::WikiLink(_) | MdTag::ExternalLink(_)
                    )
                });
                let node = if suppress_splitter {
                    MdNode::Plain(t.to_string())
                } else {
                    MdNode::Text(t.to_string())
                };
                stack.last_mut().unwrap().1.push(node);
            }
            Event::Code(c) => {
                stack
                    .last_mut()
                    .unwrap()
                    .1
                    .push(MdNode::InlineCode(c.to_string()));
            }
            Event::Html(h) | Event::InlineHtml(h) => {
                stack
                    .last_mut()
                    .unwrap()
                    .1
                    .push(MdNode::RawHtml(h.to_string()));
            }
            Event::SoftBreak => stack.last_mut().unwrap().1.push(MdNode::SoftBreak),
            Event::HardBreak => stack.last_mut().unwrap().1.push(MdNode::HardBreak),
            Event::Rule => stack.last_mut().unwrap().1.push(MdNode::Rule),
            _ => {}
        }
    }

    let (_, children) = stack.pop().unwrap();
    MdNode::Element(MdTag::Root, children)
}

/// Modify tags and internal links and keep the rest untouched.
fn text_to_html(text: &str) -> Markup {
    let splitter = Splitter::new(text);

    html! {
        @for part in splitter {
            @match part {
                Segment::Text(t) => { (t) },
                Segment::Tag(tag) => {
                    a href="#"
                        class="md-tag"
                        hx-post="/f/search"
                        hx-vals={ "{\"query\": \"" (tag) "\"}" }
                        hx-target="#search-list"
                        hx-on-htmx-after-request="document.querySelector('input[name=query]').value = this.getAttribute('data-tag')"
                        onclick="showList()"
                        data-tag=(tag)
                    {
                        (tag)
                    }
                },
                Segment::ColonTags(raw) => {
                    ":"
                    @for tag_name in raw.split(':').filter(|s| !s.is_empty()) {
                        a href="#"
                            class="md-tag"
                            hx-post="/f/search"
                            hx-vals={ "{\"query\": \"#" (tag_name) "\"}" }
                            hx-target="#search-list"
                            hx-on-htmx-after-request="document.querySelector('input[name=query]').value = this.getAttribute('data-tag')"
                            onclick="showList()"
                            data-tag={ "#" (tag_name) }
                        {
                            (tag_name)
                        }
                        ":"
                    }
                },
                Segment::Url(url) => {
                    a href=(url) { (url) span class="md-ext-icon" { "\u{2197}" } }
                },
            }
        }
    }
}

fn render_children(children: &[MdNode]) -> Markup {
    html! {
        @for child in children {
            (render_node(child))
        }
    }
}

fn collect_text(nodes: &[MdNode]) -> String {
    let mut s = String::new();
    for node in nodes {
        match node {
            MdNode::Text(t) | MdNode::Plain(t) | MdNode::InlineCode(t) => s.push_str(t),
            MdNode::Element(_, children) => s.push_str(&collect_text(children)),
            MdNode::SoftBreak | MdNode::HardBreak => s.push(' '),
            _ => {}
        }
    }
    s
}

fn render_node(node: &MdNode) -> Markup {
    match node {
        MdNode::Element(tag, children) => match tag {
            MdTag::Root => render_children(children),
            MdTag::Paragraph => html! { p { (render_children(children)) } },
            MdTag::Heading(level) => {
                let id = heading_anchor(&collect_text(children));
                let inner = render_children(children);
                match level {
                    1 => html! { h1 id=(id) { (inner) } },
                    2 => html! { h2 id=(id) { (inner) } },
                    3 => html! { h3 id=(id) { (inner) } },
                    4 => html! { h4 id=(id) { (inner) } },
                    5 => html! { h5 id=(id) { (inner) } },
                    _ => html! { h6 id=(id) { (inner) } },
                }
            }
            MdTag::BlockQuote => html! {
                blockquote { (render_children(children)) }
            },
            MdTag::Admonition(kind) => {
                let (modifier, label) = match kind {
                    BlockQuoteKind::Note => ("md-admonition--note", "Note"),
                    BlockQuoteKind::Tip => ("md-admonition--tip", "Tip"),
                    BlockQuoteKind::Important => ("md-admonition--important", "Important"),
                    BlockQuoteKind::Warning => ("md-admonition--warning", "Warning"),
                    BlockQuoteKind::Caution => ("md-admonition--caution", "Caution"),
                };
                let container_class = format!("md-admonition {modifier}");
                html! {
                    div class=(container_class) {
                        div class="md-admonition-title" { (label) }
                        div { (render_children(children)) }
                    }
                }
            }
            MdTag::CodeBlock(lang) => {
                let code = collect_text(children);

                if lang.as_deref() == Some("mermaid")
                    && let Some(diagram) = render_mermaid(&code)
                {
                    return diagram;
                }

                match highlight_code(&code, lang.as_deref()) {
                    Some(highlighted) => html! {
                        pre { code { (PreEscaped(highlighted)) } }
                    },
                    None => html! {
                        pre { code { (code) } }
                    },
                }
            }
            MdTag::OrderedList => html! { ol { (render_children(children)) } },
            MdTag::UnorderedList => html! { ul { (render_children(children)) } },
            MdTag::ListItem => html! { li { (render_children(children)) } },
            MdTag::Emphasis => html! { em { (render_children(children)) } },
            MdTag::Strong => html! { strong { (render_children(children)) } },
            MdTag::Strikethrough => html! { del { (render_children(children)) } },
            MdTag::WikiLink(url) => html! {
                a href="#" class="md-wikilink"
                    hx-get={ "/f/" (url) }
                    hx-target="#note-content"
                    hx-push-url={ "/note/" (url) }
                { (render_children(children)) }
            },
            MdTag::ExternalLink(url) => html! {
                a href=(url) {
                    (render_children(children))
                    span class="md-ext-icon" { "\u{2197}\u{FE0E}" }
                }
            },
            MdTag::Table => {
                let mut head = html! {};
                let mut body_rows = Vec::new();
                for child in children {
                    if matches!(child, MdNode::Element(MdTag::TableHead, _)) {
                        head = render_node(child);
                    } else {
                        body_rows.push(child);
                    }
                }
                html! {
                    table {
                        (head)
                        tbody {
                            @for row in body_rows { (render_node(row)) }
                        }
                    }
                }
            }
            MdTag::TableHead => html! {
                thead { tr { (render_children(children)) } }
            },
            MdTag::TableRow => html! { tr { (render_children(children)) } },
            MdTag::TableHeadCell => html! { th { (render_children(children)) } },
            MdTag::TableBodyCell => html! { td { (render_children(children)) } },
            MdTag::Image { url, title } => {
                let alt = collect_text(children);
                let title = if title.is_empty() {
                    None
                } else {
                    Some(title.as_str())
                };
                let url = if !url.starts_with('/')
                    && !url.starts_with("http://")
                    && !url.starts_with("https://")
                {
                    format!("/{url}")
                } else {
                    url.clone()
                };
                html! { img src=(url) alt=(alt) title=[title]; }
            }
        },
        MdNode::Text(t) => text_to_html(t),
        MdNode::Plain(t) => html! { (t) },
        MdNode::InlineCode(c) => html! { code { (c) } },
        MdNode::RawHtml(h) => PreEscaped(h.clone()),
        MdNode::SoftBreak => PreEscaped("\n".to_owned()),
        MdNode::HardBreak => html! { br; },
        MdNode::Rule => html! { hr; },
    }
}

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_newlines);

fn highlight_code(source: &str, lang: Option<&str>) -> Option<String> {
    let lang = lang?;
    let syntax = SYNTAX_SET.find_syntax_by_token(lang)?;

    let mut generator = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        &SYNTAX_SET,
        ClassStyle::SpacedPrefixed { prefix: "hl-" },
    );

    for line in syntect::util::LinesWithEndings::from(source) {
        generator
            .parse_html_for_line_which_includes_newline(line)
            .ok()?;
    }

    Some(generator.finalize())
}

/// Cache of rendered Mermaid diagrams, keyed by a hash of the diagram source.
/// Each entry holds the `(light, dark)` SVG pair. Avoids re-rendering on every
/// note view and on every keystroke in the live editor preview.
static MERMAID_CACHE: LazyLock<Mutex<HashMap<u64, (String, String)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Render a `mermaid` fenced block into a light and a dark SVG variant, toggled
/// by CSS (`.mermaid-light` / `.mermaid-dark`). Returns `None` if either variant
/// fails to render, so the caller can fall back to showing the source.
fn render_mermaid(source: &str) -> Option<Markup> {
    let key = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    };

    let (light, dark) = {
        if let Some(cached) = MERMAID_CACHE.lock().unwrap().get(&key).cloned() {
            cached
        } else {
            let light = render_mermaid_svg(source, MermaidTheme::Light, key)?;
            let dark = render_mermaid_svg(source, MermaidTheme::Dark, key)?;
            MERMAID_CACHE
                .lock()
                .unwrap()
                .insert(key, (light.clone(), dark.clone()));
            (light, dark)
        }
    };

    Some(html! {
        div class="mermaid" {
            div class="mermaid-light" { (PreEscaped(light)) }
            div class="mermaid-dark" { (PreEscaped(dark)) }
        }
    })
}

/// Subset of weave's `app.css` design tokens needed to theme diagrams. `base` is
/// the Mermaid theme the variables layer on top of: `"base"` (light) gives a
/// clean light canvas, `"dark"` already darkens diagram families that don't read
/// every theme variable (sequence/er/gantt/...), so we only nudge them toward the
/// weave palette.
struct MermaidPalette {
    base: &'static str,
    bg: &'static str,
    surface: &'static str,
    surface_alt: &'static str,
    border: &'static str,
    fg: &'static str,
    muted: &'static str,
}

#[derive(Clone, Copy)]
enum MermaidTheme {
    Light,
    Dark,
}

impl MermaidTheme {
    fn id_suffix(self) -> &'static str {
        match self {
            MermaidTheme::Light => "light",
            MermaidTheme::Dark => "dark",
        }
    }

    fn palette(self) -> MermaidPalette {
        match self {
            MermaidTheme::Light => MermaidPalette {
                base: "base",
                bg: "#fefdfc",
                surface: "#f6f5f2",
                surface_alt: "#efedea",
                border: "#cccac6",
                fg: "#1d1a17",
                muted: "#7d7a75",
            },
            MermaidTheme::Dark => MermaidPalette {
                base: "dark",
                bg: "#0f0d0a",
                surface: "#1a1815",
                surface_alt: "#151311",
                border: "#302d2b",
                fg: "#eae7e4",
                muted: "#82807c",
            },
        }
    }

    /// Mermaid config mapping weave's design tokens onto `themeVariables` across
    /// diagram families (flowchart, sequence, class, state, er, ...), so diagrams
    /// match the surrounding UI in both modes. `fontFamily`/`fontSize` match the
    /// `.md` body (IBM Plex Sans, 15px) so diagram text is optically the same size
    /// as the prose; merman re-measures boxes for the family, so labels still fit.
    /// Box padding/sizing is trimmed from merman's roomy defaults.
    fn config(self) -> MermaidConfig {
        let p = self.palette();

        let mut variables = serde_json::Map::new();
        let mut set = |key: &str, value: &str| {
            variables.insert(key.to_owned(), serde_json::Value::String(value.to_owned()));
        };
        // Shared surfaces / text / lines.
        set("background", p.bg);
        set("primaryColor", p.surface);
        set("mainBkg", p.surface);
        set("secondBkg", p.surface_alt);
        set("secondaryColor", p.surface_alt);
        set("tertiaryColor", p.surface_alt);
        set("primaryBorderColor", p.border);
        set("nodeBorder", p.border);
        set("secondaryBorderColor", p.border);
        set("tertiaryBorderColor", p.border);
        set("clusterBkg", p.surface_alt);
        set("clusterBorder", p.border);
        set("primaryTextColor", p.fg);
        set("nodeTextColor", p.fg);
        set("textColor", p.fg);
        set("titleColor", p.fg);
        set("lineColor", p.muted);
        set("edgeLabelBackground", p.surface);
        // Sequence diagram specifics.
        set("actorBkg", p.surface);
        set("actorBorder", p.border);
        set("actorTextColor", p.fg);
        set("actorLineColor", p.muted);
        set("signalColor", p.muted);
        set("signalTextColor", p.fg);
        set("labelBoxBkgColor", p.surface_alt);
        set("labelBoxBorderColor", p.border);
        set("labelTextColor", p.fg);
        set("loopTextColor", p.fg);
        set("noteBkgColor", p.surface_alt);
        set("noteTextColor", p.fg);
        set("noteBorderColor", p.border);
        set("activationBkgColor", p.border);
        set("activationBorderColor", p.muted);
        set("sequenceNumberColor", p.bg);
        // ER diagram attribute rows.
        set("attributeBackgroundColorOdd", p.surface);
        set("attributeBackgroundColorEven", p.surface_alt);
        // Typography matched to the note body.
        set("fontFamily", "\"IBM Plex Sans\", system-ui, sans-serif");
        set("fontSize", "15px");

        MermaidConfig::from_value(serde_json::json!({
            "theme": p.base,
            // merman's defaults are roomy; trim node padding and the fixed actor
            // box size (width is a minimum and still grows to fit long labels).
            "flowchart": { "padding": 10 },
            "sequence": { "width": 90, "height": 40, "boxMargin": 8 },
            "themeVariables": serde_json::Value::Object(variables),
        }))
    }
}

/// Render a single Mermaid diagram to SVG for one theme variant. The diagram id
/// is baked into the SVG element ids and must be unique per diagram on a page to
/// avoid marker/clip-path collisions.
fn render_mermaid_svg(source: &str, theme: MermaidTheme, key: u64) -> Option<String> {
    let diagram_id = format!("mermaid-{key:x}-{}", theme.id_suffix());

    let svg = HeadlessRenderer::new()
        .with_diagram_id(&diagram_id)
        .render_svg_with_site_config_sync(source, theme.config())
        .ok()
        .flatten()?;

    Some(harmonize_colors(fit_svg(&svg), theme))
}

/// Replace the few light defaults merman hardcodes regardless of `themeVariables`
/// (sequence actor boxes `#eaeaea`/`#666`, and entity/node fills that stay light
/// in dark mode) with the variant palette, so no diagram family shows a glaring
/// light box against a dark note.
fn harmonize_colors(svg: String, theme: MermaidTheme) -> String {
    let p = theme.palette();

    let mut svg = svg
        .replace("fill=\"#eaeaea\"", &format!("fill=\"{}\"", p.surface))
        .replace("fill:#eaeaea", &format!("fill:{}", p.surface))
        .replace("stroke=\"#666\"", &format!("stroke=\"{}\"", p.border));

    if matches!(theme, MermaidTheme::Dark) {
        svg = svg
            .replace("fill=\"white\"", &format!("fill=\"{}\"", p.surface))
            .replace("fill=\"#ECECFF\"", &format!("fill=\"{}\"", p.surface));
    }

    svg
}

/// Rewrite the root `<svg>` so it renders at its native pixel size instead of
/// scaling to the container width. Mermaid emits `width="100%"` plus a
/// `max-width` cap, which downscales wide diagrams and shrinks their text below
/// the surrounding prose. Pinning the width keeps text at the configured size;
/// the `.mermaid` container scrolls horizontally when a diagram is too wide.
/// Also drops the hardcoded white background so the diagram inherits the note
/// surface. Only the opening tag is touched, so `foreignObject` `max-width`
/// label caps are left intact.
fn fit_svg(svg: &str) -> String {
    let Some(end) = svg.find('>') else {
        return svg.to_owned();
    };
    let (open_tag, rest) = svg.split_at(end);

    let open_tag = open_tag
        .replace(" width=\"100%\"", "")
        .replace("max-width:", "width:")
        .replace("background-color: white", "background-color: transparent");

    format!("{open_tag}{rest}")
}

pub fn markdown_to_html(source: &str) -> Markup {
    let parser = Parser::new_ext(
        source,
        Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_SMART_PUNCTUATION
            | Options::ENABLE_GFM,
    );

    let tree = build_tree(parser);

    render_node(&tree)
}

#[derive(Debug, Clone)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub anchor: String,
}

/// Convert heading plain text to a URL-safe anchor string.
pub fn heading_anchor(text: &str) -> String {
    let mut anchor = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            anchor.push(ch.to_ascii_lowercase());
        } else if !anchor.ends_with('-') {
            anchor.push('-');
        }
    }
    anchor.trim_end_matches('-').to_owned()
}

/// Parse markdown once; return rendered HTML and extracted headings together.
pub fn markdown_to_html_with_headings(source: &str) -> (Markup, Vec<Heading>) {
    let parser = Parser::new_ext(
        source,
        Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_SMART_PUNCTUATION
            | Options::ENABLE_GFM,
    );
    let tree = build_tree(parser);
    let headings = collect_headings_from_tree(&tree);
    let html = render_node(&tree);
    (html, headings)
}

fn collect_headings_from_tree(node: &MdNode) -> Vec<Heading> {
    let mut out = Vec::new();
    collect_headings_inner(node, &mut out);
    out
}

fn collect_headings_inner(node: &MdNode, out: &mut Vec<Heading>) {
    match node {
        MdNode::Element(MdTag::Heading(level), children) => {
            let text = collect_text(children);
            let anchor = heading_anchor(&text);
            out.push(Heading {
                level: *level,
                text,
                anchor,
            });
        }
        MdNode::Element(_, children) => {
            for child in children {
                collect_headings_inner(child, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wiki_stem(url: &str) -> Option<String> {
        WIKI_LINK_RE
            .captures(url)
            .map(|caps| caps["stem"].to_string())
    }

    #[test]
    fn test_wiki_link_bare_stem() {
        assert_eq!(wiki_stem("65bs"), Some("65bs".into()));
    }

    #[test]
    fn test_wiki_link_relative_parent() {
        assert_eq!(wiki_stem("../65bs"), Some("65bs".into()));
    }

    #[test]
    fn test_wiki_link_relative_current() {
        assert_eq!(wiki_stem("./65bs"), Some("65bs".into()));
    }

    #[test]
    fn test_wiki_link_relative_deep() {
        assert_eq!(wiki_stem("../../abc123"), Some("abc123".into()));
    }

    #[test]
    fn test_wiki_link_rejects_url() {
        assert_eq!(wiki_stem("https://example.com"), None);
        assert_eq!(wiki_stem("http://example.com/foo"), None);
    }

    #[test]
    fn test_wiki_link_rejects_extension() {
        assert_eq!(wiki_stem("file.txt"), None);
    }

     #[test]
    fn test_wiki_link_with_hyphens() {
        assert_eq!(wiki_stem("my-note"), Some("my-note".into()));
        assert_eq!(wiki_stem("a-b-c"), Some("a-b-c".into()));
        assert_eq!(wiki_stem("../my-note"), Some("my-note".into()));
    }

    #[test]
    fn test_render_wiki_link_with_hyphens() {
        let html = markdown_to_html("[note](my-note)").into_string();
        assert!(html.contains(r#"hx-get="/f/my-note""#), "{html}");
        assert!(html.contains(r#"hx-push-url="/note/my-note""#), "{html}");
        assert!(html.contains("note"));
    }

    #[test]
    fn test_render_bare_wiki_link() {
        let html = markdown_to_html("[note](abc1)").into_string();
        assert!(html.contains(r#"hx-get="/f/abc1""#), "{html}");
        assert!(html.contains(r#"hx-push-url="/note/abc1""#), "{html}");
        assert!(html.contains("note"));
    }

    #[test]
    fn test_render_relative_wiki_link() {
        let html = markdown_to_html("[weave](../65bs)").into_string();
        assert!(html.contains(r#"hx-get="/f/65bs""#), "{html}");
        assert!(html.contains(r#"hx-push-url="/note/65bs""#), "{html}");
        assert!(html.contains("weave"));
    }

    #[test]
    fn test_render_external_link() {
        let html = markdown_to_html("[site](https://example.com)").into_string();
        assert!(html.contains(r#"href="https://example.com""#), "{html}");
        assert!(!html.contains("hx-get"), "{html}");
    }

    #[test]
    fn test_external_link_with_url_label_has_single_icon() {
        let html =
            markdown_to_html("[http://localhost:8000](http://localhost:8000)").into_string();
        assert_eq!(html.matches("md-ext-icon").count(), 1, "{html}");
        assert_eq!(html.matches("<a ").count(), 1, "{html}");
    }

    #[test]
    fn test_autolink_strips_trailing_period() {
        let html = markdown_to_html("see https://example.com.").into_string();
        assert!(html.contains(r#"href="https://example.com""#), "{html}");
        assert!(html.contains("</a>."), "{html}");
    }

    #[test]
    fn test_hashtag_calls_show_list() {
        let html = markdown_to_html("hello #topic world").into_string();
        assert!(html.contains(r#"onclick="showList()""#), "{html}");
        assert!(!html.contains("showSidebar"), "{html}");
    }

    #[test]
    fn test_hashtag_targets_search_list() {
        let html = markdown_to_html("see #topic").into_string();
        assert!(html.contains(r##"hx-target="#search-list""##), "{html}");
        assert!(html.contains(r#"hx-post="/f/search""#), "{html}");
    }

    #[test]
    fn test_colon_tags_call_show_list() {
        let html = markdown_to_html("status :draft:review:").into_string();
        assert!(html.contains(r#"onclick="showList()""#), "{html}");
        assert!(!html.contains("showSidebar"), "{html}");
    }

    #[test]
    fn test_wiki_link_has_push_url() {
        let html = markdown_to_html("[my note](abc1)").into_string();
        assert!(html.contains(r#"hx-push-url="/note/abc1""#), "{html}");
        assert!(html.contains(r##"hx-target="#note-content""##), "{html}");
    }

    #[test]
    fn test_wiki_link_no_sidebar_call() {
        let html = markdown_to_html("[link](abc1)").into_string();
        assert!(!html.contains("showSidebar"), "{html}");
        assert!(!html.contains("showList"), "{html}");
        assert!(!html.contains("goBack"), "{html}");
    }

    #[test]
    fn test_heading_anchor_basic() {
        assert_eq!(heading_anchor("Hello World"), "hello-world");
    }

    #[test]
    fn test_heading_anchor_deduplicates_hyphens() {
        assert_eq!(heading_anchor("Foo  Bar"), "foo-bar");
        assert_eq!(heading_anchor("A - B"), "a-b");
    }

    #[test]
    fn test_heading_anchor_trims_trailing_hyphen() {
        assert_eq!(heading_anchor("Hello!"), "hello");
    }

    #[test]
    fn test_heading_anchor_alphanumeric() {
        assert_eq!(heading_anchor("Step 1: Setup"), "step-1-setup");
    }

    #[test]
    fn test_markdown_to_html_with_headings_extracts_headings() {
        let src = "# First\n\nBody.\n\n## Second\n\nMore body.";
        let (html, headings) = markdown_to_html_with_headings(src);
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[0].text, "First");
        assert_eq!(headings[0].anchor, "first");
        assert_eq!(headings[1].level, 2);
        assert_eq!(headings[1].text, "Second");
        assert_eq!(headings[1].anchor, "second");
        let html_str = html.into_string();
        assert!(html_str.contains(r#"id="first""#), "{html_str}");
        assert!(html_str.contains(r#"id="second""#), "{html_str}");
    }

    #[test]
    fn test_markdown_to_html_with_headings_empty() {
        let src = "Just a paragraph.";
        let (_, headings) = markdown_to_html_with_headings(src);
        assert!(headings.is_empty());
    }

    #[test]
    fn test_mermaid_renders_light_and_dark_svg() {
        let src = "```mermaid\nflowchart TD\n  A[Start] --> B[Done]\n```";
        let html = markdown_to_html(src).into_string();
        assert!(html.contains(r#"class="mermaid""#), "{html}");
        assert!(html.contains(r#"class="mermaid-light""#), "{html}");
        assert!(html.contains(r#"class="mermaid-dark""#), "{html}");
        assert_eq!(html.matches("<svg").count(), 2, "{html}");
        assert!(!html.contains("<pre>"), "{html}");
    }

    #[test]
    fn test_mermaid_invalid_falls_back_to_code_block() {
        let src = "```mermaid\nnot a real diagram @#$%\n```";
        let html = markdown_to_html(src).into_string();
        assert!(!html.contains("<svg"), "{html}");
        assert!(html.contains("<pre>"), "{html}");
    }

    /// merman hardcodes a few light defaults in diagram families that don't read
    /// every theme variable (sequence actors, er entities, ...). Guard that the
    /// dark variant has none of them, so nothing glares against a dark note.
    #[test]
    fn test_mermaid_dark_variant_has_no_light_fills() {
        let diagrams = [
            "```mermaid\nsequenceDiagram\n participant A as Editor\n participant W as File watcher\n A->>W: PUT\n Note over A,W: hi\n W-->>A: ok\n```",
            "```mermaid\nerDiagram\n NOTE ||--o{ TAG : has\n NOTE{string title PK}\n```",
            "```mermaid\nstateDiagram-v2\n [*]-->Read\n Read-->Edit\n Edit-->[*]\n```",
        ];
        for src in diagrams {
            let html = markdown_to_html(src).into_string();
            // Markup is light variant then dark variant; the dark SVG follows the
            // `mermaid-dark` class.
            let (_, dark) = html.split_once("mermaid-dark").expect("dark variant present");
            for bad in ["#eaeaea", "fill=\"white\"", "#ECECFF", "#fff5ad"] {
                assert!(!dark.contains(bad), "dark variant still contains {bad}: {dark}");
            }
        }
    }
}
