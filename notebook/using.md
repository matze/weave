# Usage

## Starting the server

See [Installing](install) for build instructions and environment variables. Once
built, start the server with:

```bash
ZK_NOTEBOOK_DIR="/path/to/notebook" WEAVE_PASSWORD="secret" cargo run --release
```


## User interface

The main interface consists of a sidebar listing all notes and a content area
showing the selected note. On mobile the sidebar is hidden and can be toggled
with the back button.

### Authentication

Weave is a single-user application. Set `WEAVE_PASSWORD` to require sign-in.
When signed in you can view and edit all notes. Without a password, login is
disabled entirely.

Tagging a note with `#public` makes it viewable (but not editable) without
signing in. This is useful for sharing individual notes with others.

### Searching

Press <kbd>f</kbd> or click the "Filter notes..." box to search. Weave uses
fuzzy matching against note titles, so you do not need to type the exact title.
Prefix a word with `#` to filter by tag instead, e.g. `#public`.

### Editing

Press <kbd>e</kbd> or click the pencil icon to edit the current note. The editor
is a plain text area with a live preview panel. Save your changes or discard
them with the corresponding buttons.

Weave watches the notebook directory for changes, so edits made outside of Weave
(in your text editor, via Git, etc.) are picked up automatically.


## Special tags

Zk uses [tags][] to group and find notes of a related topic. Weave adds special
behaviour to three of them:

- `#public` makes a note accessible without signing in, useful for sharing
  notes with others.
- `#pin` moves a note to the top of the sidebar for quick access.
- `#archived` pushes a note to the bottom of the sidebar and greys it out to
  reduce clutter.

Tags can be placed in the YAML frontmatter (`tags: [public, pin]`) or inline
in the note body.


[tags]: https://zk-org.github.io/zk/notes/tags.html

#public
