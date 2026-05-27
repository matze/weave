# Markdown

Weave renders [CommonMark](https://commonmark.org/) with a handful of GitHub
Flavored Markdown extensions and a few zk-specific niceties. This page
demonstrates every supported feature so you can copy the patterns into your own
notes.


## Headings

Use `#` through `######` for headings. Each heading gets a URL-safe anchor so
you can deep-link to a section, like [the lists section](#lists).

### Third level

#### Fourth level

##### Fifth level

###### Sixth level


## Inline formatting

You can write **bold**, *italic*, ***bold italic*** and ~~strikethrough~~
inline. Inline `code spans` use backticks. Smart punctuation turns "straight"
quotes into curly ones and `--` into en dashes automatically.

A line ending with two trailing spaces  
forces a hard line break.


## Lists

Unordered lists use `-` or `*`:

- First item
- Second item
  - Nested item
  - Another nested item
    - Even deeper
- Third item

Ordered lists use numbers:

1. Build the binary
2. Point it at a notebook
3. Open the browser


## Blockquotes

> A regular blockquote is rendered with a thin accent bar and slightly muted
> italic text. Useful for citing other notes or external sources.


## Admonitions

Admonitions reuse the GitHub Flavored Markdown alert syntax. Five kinds are
recognised:

> [!NOTE]
> Use a note to highlight information that a reader should know, even when
> skimming.

> [!TIP]
> Tips suggest a better or faster way to accomplish a task.

> [!IMPORTANT]
> Important admonitions call out crucial information needed to succeed.

> [!WARNING]
> Warnings flag content that requires immediate attention to avoid problems.

> [!CAUTION]
> Cautions describe negative outcomes of an action.


## Code

Fenced code blocks support syntax highlighting when a language is provided.

```rust
fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

fn main() {
    println!("{}", greet("Weave"));
}
```

```bash
ZK_NOTEBOOK_DIR="$(pwd)/notebook" cargo run --release
```

```python
def fib(n: int) -> int:
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a
```

Code blocks without a language are rendered verbatim, without highlighting:

```
plain monospace text
```


## Tables

| Variable           | Description                            | Default    |
|--------------------|----------------------------------------|------------|
| `ZK_NOTEBOOK_DIR`  | Path to the notebook directory         | (required) |
| `WEAVE_PASSWORD`   | Password for signing in                | (empty)    |
| `WEAVE_PORT`       | Port the server listens on             | `8000`     |


## Links

External links open the target URL and are marked with a small arrow icon, like
[the zk project page](https://github.com/zk-org/zk). Bare URLs are auto-linked
too: https://example.com.

Reference-style links work as well, e.g. [the zk tags docs][tags].

Internal links to other notes use the destination note's filename stem:

- [Weave overview](weave)
- [Installation guide](installation)
- [Usage tips](usage)

Relative paths such as `./usage` or `../usage` are accepted and resolve to the
same note.


## Tags

Zk uses [tags][] to group and find notes of a related topic. Two tag styles
are recognised inline and turned into clickable filters:

- Hashtags like #example or #public open the sidebar filtered to that tag.
- Colon tags borrow the zk convention: :draft:review: behaves the same way for
  each segment.

Tags can also be placed in the YAML frontmatter (`tags: [public, pin]`) instead
of inline in the note body.

Weave adds special behaviour to three tags:

- `#public` makes a note accessible without signing in, useful for sharing
  notes with others.
- `#pin` moves a note to the top of the sidebar for quick access.
- `#archived` pushes a note to the bottom of the sidebar and greys it out to
  reduce clutter.


## Images

Images use the standard `![alt](path)` syntax. Relative paths are made
root-absolute so they resolve regardless of the current note URL, which is
particularly useful together with the `WEAVE_ATTACHMENTS` directory.

```markdown
![diagram](media/diagram.png)
```


## Horizontal rules

Three or more dashes on their own line produce a horizontal rule:

---

Useful for separating loosely related sections inside a single note.


[tags]: https://zk-org.github.io/zk/notes/tags.html

#public
