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

You need a Rust toolchain (1.93+) and the
[tailwindcss](https://tailwindcss.com/blog/standalone-cli) standalone CLI binary
in your `PATH`.

```bash
git clone https://github.com/matze/weave.git
cd weave
cargo build --release
```

The binary ends up in `target/release/weave`.

## Quickstart

Point Weave at a zk notebook directory (here we use the demo notebook), set a
password and run the application from source with:

```bash
ZK_NOTEBOOK_DIR="$(pwd)/notebook" WEAVE_PASSWORD="secret" cargo run --release
```

This starts the server on [http://localhost:8000](http://localhost:8000). A demo
instance can be accessed at https://weave.bloerg.net.


## Environment variables

| Variable | Description | Default |
|---|---|---|
| `ZK_NOTEBOOK_DIR` | Path to the zk notebook directory | (required) |
| `WEAVE_PASSWORD` | Password for signing in | (empty, login disabled) |
| `WEAVE_PORT` | Port the server listens on | `8000` |


## License

MIT
