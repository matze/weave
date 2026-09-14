# Weave

Weave is a self-hosted, single-user, web-based frontend to view and edit
[zk](https://github.com/zk-org/zk) notes. It is lightweight, quick and
opinionated. It features

- a single binary with a built-in zk re-implementation (no `zk` binary needed)
- fuzzy search across all note titles and tags
- cross-linking and note editing
- syntax highlighting of code blocks
- real-time file watching (external edits show up immediately)
- light and dark mode support
- focus mode

<p align="center"><strong><a href="https://weave.bloerg.net/note/weave">DEMO</a></strong></p>


## Building from source

You need a Rust toolchain (1.93+).

```bash
git clone https://github.com/matze/weave.git
cd weave
cargo build --release
```

The binary ends up in `target/release/weave`.

Run the test suite with:

```bash
cargo test
```

## Quickstart

Point Weave at a zk notebook directory (here we use the demo notebook), set a
password and run the application from source with:

```bash
ZK_NOTEBOOK_DIR="$(pwd)/notebook" WEAVE_PASSWORD="secret" cargo run --release
```

This starts the server on [http://localhost:8000](http://localhost:8000). A demo
instance can be accessed at https://weave.bloerg.net.

To work on your notes locally without signing in, leave `WEAVE_PASSWORD` unset.
Login is then disabled, every note is readable and editable, and the sign-in
button is hidden. Only do this on a machine you trust, since anyone who can
reach the port can edit your notes.

```bash
ZK_NOTEBOOK_DIR="$(pwd)/notebook" cargo run --release
```


## Environment variables

| Variable | Description | Default |
|---|---|---|
| `ZK_NOTEBOOK_DIR` | Path to the zk notebook directory | (required) |
| `WEAVE_PASSWORD` | Password for signing in; unset disables login and opens all notes | (empty, login disabled) |
| `WEAVE_PORT` | Port the server listens on | `8000` |
| `WEAVE_HOST` | IP address the server listens on | `127.0.0.1` |
| `WEAVE_ATTACHMENTS` | Subdirectory inside `ZK_NOTEBOOK_DIR` to serve as static files (e.g. `media`) | (disabled) |


## License

MIT
