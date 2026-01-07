//! Render Markdown as HTML.

use maud::{Markup, PreEscaped, html};
use regex_split::RegexSplit;

/// Modify tags and internal links and keep the rest untouched.
fn text_to_html(node: &markdown::mdast::Text) -> Markup {
    enum Part<'a> {
        Text(&'a str),
        Tag(&'a str),
    }

    // TODO: constify
    let re = regex::Regex::new("#[\\w]+").expect("compiling regex");
    let parts = re.split_inclusive_left(&node.value).map(|part| {
        if part.starts_with('#') {
            Part::Tag(unsafe { part.get_unchecked(1..) })
        } else {
            Part::Text(part)
        }
    });

    html! {
        @for part in parts {
            @match part {
                Part::Text(text) => { (text) },
                Part::Tag(tag) => {
                    a href="#"
                        class="text-sky-600 hover:underline"
                        hx-post="/f/search"
                        hx-vals={ "{\"query\": \"#" (tag) "\"}" }
                        hx-target="#search-list"
                        hx-on-htmx-after-request="document.querySelector('input[name=query]').value = this.getAttribute('data-tag')"
                        data-tag={ "#" (tag) }
                    {
                        { "#" (tag) }
                    }
                },
            }
        }
    }
}

/// Turn [`markdown::mdast::Node`] into servable [`Markup`]
fn node_to_html(node: &markdown::mdast::Node) -> Markup {
    match node {
        markdown::mdast::Node::Root(root) => html! {
            @for node in &root.children {
                (node_to_html(node))
            }
        },
        markdown::mdast::Node::Blockquote(blockquote) => html! {
            blockquote class="border-s-4 border-blue-600 bg-blue-50 p-4 rounded my-4 italic text-gray-700" {
                @for node in &blockquote.children {
                    (node_to_html(node))
                }
            }
        },
        markdown::mdast::Node::FootnoteDefinition(_) => todo!(),
        markdown::mdast::Node::List(list) => html! {
            @if list.ordered {
                ol {
                    @for node in &list.children {
                        li class="list-decimal ml-6 my-2" {
                            (node_to_html(&node))
                        }
                    }
                }
            }
            @else {
                ul {
                    @for node in &list.children {
                        li class="list-disc ml-6 my-2" {
                            (node_to_html(&node))
                        }
                    }
                }
            }
        },
        markdown::mdast::Node::ListItem(item) => html! {
            @for node in &item.children {
                (node_to_html(&node))
            }
        },
        markdown::mdast::Node::Toml(_) => todo!(),
        markdown::mdast::Node::Yaml(_) => todo!(),
        markdown::mdast::Node::Break(_) => todo!(),
        markdown::mdast::Node::InlineCode(code) => html! {
            code class="bg-gray-100 dark:bg-gray-900 p-1 rounded font-mono" {
                (code.value)
            }
        },
        markdown::mdast::Node::InlineMath(_) => todo!(),
        markdown::mdast::Node::Delete(delete) => html! {
            del class="line-through" {
                @for node in &delete.children {
                    (node_to_html(node))
                }
            }
        },
        markdown::mdast::Node::Emphasis(emphasis) => html! {
            em class="italic" {
                @for node in &emphasis.children {
                    (node_to_html(&node))
                }
            }
        },
        markdown::mdast::Node::MdxFlowExpression(_) => todo!(),
        markdown::mdast::Node::MdxjsEsm(_) => todo!(),
        markdown::mdast::Node::MdxJsxFlowElement(_) => todo!(),
        markdown::mdast::Node::MdxJsxTextElement(_) => todo!(),
        markdown::mdast::Node::MdxTextExpression(_) => todo!(),
        markdown::mdast::Node::FootnoteReference(_) => todo!(),
        markdown::mdast::Node::Html(node) => html! {
            (PreEscaped(node.value.clone()))
        },
        markdown::mdast::Node::Image(_) => todo!(),
        markdown::mdast::Node::ImageReference(_) => todo!(),
        markdown::mdast::Node::Link(link) => {
            // TODO: constify
            let re = regex::Regex::new("[\\w\\d]+").expect("compiling regex");

            let text = html! {
                @for node in &link.children {
                    (node_to_html(&node))
                }
            };

            let css = "text-sky-600 hover:underline font-semibold";

            if re.is_match(&link.url) {
                html! {
                    a href="#" class=(css)
                    hx-get={ "/f/" (link.url) }
                    hx-target="#note-content"
                    hx-push-url="true" { (text) }
                }
            } else {
                html! {
                    a href=(link.url) class=(css) { (text) }
                }
            }
        }
        markdown::mdast::Node::LinkReference(_) => todo!(),
        markdown::mdast::Node::Strong(strong) => html! {
            strong class="font-bold" {
                @for node in &strong.children {
                    (node_to_html(&node))
                }
            }
        },
        markdown::mdast::Node::Text(text) => html! {
            (text_to_html(text))
        },
        markdown::mdast::Node::Code(code) => html! {
            pre class="bg-gray-100 dark:bg-gray-900 p-4 rounded my-4 overflow-x-auto font-mono" {
                (code.value)
            }
        },
        markdown::mdast::Node::Math(_) => todo!(),
        markdown::mdast::Node::Heading(heading) => {
            let children = html! {
                @for node in &heading.children {
                    (node_to_html(&node))
                }
            };

            match heading.depth {
                1 => html! { h1 class="font-semibold mb-4" { (children) } },
                2 => html! { h2 class="font-semibold mb-4" { (children) } },
                3 => html! { h3 class="font-semibold mb-4" { (children) } },
                4 => html! { h4 class="font-semibold mb-4" { (children) } },
                5 => html! { h5 class="font-semibold mb-4" { (children) } },
                _ => html! { h6 class="font-semibold mb-4" { (children) } },
            }
        }
        markdown::mdast::Node::ThematicBreak(_) => html! {
            hr class="m-4" {}
        },
        markdown::mdast::Node::Table(table) => html! {
            table class="border-collapse my-4 w-full" {
                @for node in &table.children {
                    (node_to_html(&node))
                }
            }
        },
        markdown::mdast::Node::TableRow(row) => html! {
            tr {
                @for node in &row.children {
                    (node_to_html(&node))
                }
            }
        },
        markdown::mdast::Node::TableCell(cell) => html! {
            td {
                @for node in &cell.children {
                    (node_to_html(&node))
                }
            }
        },
        markdown::mdast::Node::Definition(_) => todo!(),
        markdown::mdast::Node::Paragraph(paragraph) => html! {
            p class="my-4 leading-relaxed" {
                @for node in &paragraph.children {
                    (node_to_html(&node))
                }
            }
        },
    }
}

pub fn markdown_to_html(source: &str) -> Markup {
    let mut options = markdown::ParseOptions::default();
    options.constructs.gfm_table = true;
    let root = markdown::to_mdast(source, &options).expect("parsing markdown");

    html! {
        div {
            (node_to_html(&root))
        }
    }
}
