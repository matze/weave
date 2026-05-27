# Usage

## Starting the server

See [installation](installation) for build instructions and environment
variables. Once built, start the server with:

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

> [!TIP]
> Tagging a note with `#public` makes it viewable (but not editable) without
> signing in. This is useful for sharing individual notes with others.

### Searching

Press <kbd>s</kbd> or click the "Filter notes..." box to search. Weave uses
fuzzy matching against note titles, so you do not need to type the exact title.
Prefix a word with `#` to filter by tag instead, e.g. `#public`. To leave the
search bar, press <kbd>esc</kbd>.

### Editing

Press <kbd>e</kbd> or click the pencil icon to edit the current note. The editor
is a plain text area with a live preview panel. Save your changes or discard
them with the corresponding buttons.

Weave watches the notebook directory for changes, so edits made outside of Weave
(in your text editor, via Git, etc.) are picked up automatically.

### Sidebar navigation

Use <kbd>j</kbd> to move to the next note and <kbd>k</kbd> to move to the
previous note.

### Focus mode

For a distraction-free reading experience enable focus mode with the
<kbd>f</kbd> key.


## Attachments

If your notebook contains images or other files in a subdirectory, set
`WEAVE_ATTACHMENTS` to that subdirectory so Weave serves them. For example, with
a layout like:

```
notebook/
  note.md
  media/
    photo.jpg
```

Start Weave with `WEAVE_ATTACHMENTS=media` and reference the image from a note
as `![photo](media/photo.jpg)`. Weave makes relative image paths root-absolute
so they resolve correctly regardless of the page URL. Nested paths like
`WEAVE_ATTACHMENTS=assets/img` work as well.


#public
