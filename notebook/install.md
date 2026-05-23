# Installation

## Building from source

You need a Rust toolchain (1.93+). Clone the repository and build a release
binary:

```bash
git clone https://github.com/matze/weave.git
cd weave
cargo build --release
```

The binary ends up in `target/release/weave`.

## Running

Point Weave at a zk notebook directory and (optionally) set a password:

```bash
ZK_NOTEBOOK_DIR="/path/to/notebook" WEAVE_PASSWORD="secret" ./target/release/weave
```

If `WEAVE_PASSWORD` is left empty, login is disabled and all notes are
accessible without authentication.

By default the server listens on port 8000. Set `WEAVE_PORT` to change it:

```bash
WEAVE_PORT=3000 ZK_NOTEBOOK_DIR="/path/to/notebook" ./target/release/weave
```

## Environment variables

| Variable | Description | Default |
|---|---|---|
| `ZK_NOTEBOOK_DIR` | Path to the zk notebook directory | (required) |
| `WEAVE_PASSWORD` | Password for signing in | (empty) |
| `WEAVE_PORT` | Port the server listens on | `8000` |

#public
