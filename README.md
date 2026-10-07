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
ZK_NOTEBOOK_DIR="$(pwd)/notebook" WEAVE_ATTACHMENTS="media" WEAVE_PASSWORD="secret" cargo run --release
```

This starts the server on [http://localhost:8000](http://localhost:8000). A demo
instance can be accessed at <https://weave.bloerg.net>. The demo notebook keeps
its images and other static files in `notebook/media`, so `WEAVE_ATTACHMENTS`
points at that subdirectory.

To work on your notes locally without signing in, leave `WEAVE_PASSWORD` unset.
Login is then disabled, every note is readable and editable, and the sign-in
button is hidden. Only do this on a machine you trust, since anyone who can
reach the port can edit your notes.

```bash
ZK_NOTEBOOK_DIR="$(pwd)/notebook" WEAVE_ATTACHMENTS="media" cargo run --release
```

## Run as a container

Pre-built images for `x86_64` and `aarch64` are published to
`quxfoo/weave:<VERSION>` and `quxfoo/weave:latest`. The final image is based on
[scratch](https://hub.docker.com/_/scratch) and holds the statically linked
`weave` binary. The notebook and the certificate store must be mounted
from the host.

Mount the notebook at `/notebook`, which is the image's default
`ZK_NOTEBOOK_DIR`, and make sure user `10001` can write to it. Mount the host's
certificate store as well, otherwise clipping a URL fails with `502 Could not
reach the page`.

```bash
docker run \
    -p 8000:8000 \
    -e WEAVE_PASSWORD=secret \
    -e WEAVE_ATTACHMENTS=media \
    -v /path/to/notebook:/notebook \
    -v /etc/ssl/certs:/etc/ssl/certs:ro \
    quxfoo/weave:latest
```

The certificate store only matters for the URL clipper. Every other route works
without it. Any path the Rust TLS stack probes by default is fine, so
`/etc/pki/tls/certs` or `/etc/ssl/cert.pem` on non-Debian hosts work the same
way.

### Build a container image

The `Dockerfile` is designed to be run on an `x86_64` host but capable of
building images for both `x86_64` and `aarch64` via the `--target` flag:

```bash
docker build -t weave -f Dockerfile --target amd64 .
docker build -t weave -f Dockerfile --target arm64 .
```

`build-docker-image.sh` builds, tags and pushes both architectures together
with a multi-arch manifest. A clean checkout of a tagged commit is published as
that tag and moves `latest`; any other revision, including commits after a tag,
is published as `<nearest tag>-<commit>[-dirty]` and leaves `latest` alone.

## Environment variables

| Variable | Description | Default |
|---|---|---|
| `ZK_NOTEBOOK_DIR` | Path to the zk notebook directory | (required) |
| `WEAVE_PASSWORD` | Password for signing in; unset disables login and opens all notes | (empty, login disabled) |
| `WEAVE_PORT` | Port the server listens on | `8000` |
| `WEAVE_HOST` | IP address the server listens on | `127.0.0.1` |
| `WEAVE_ATTACHMENTS` | Subdirectory inside `ZK_NOTEBOOK_DIR` serving static files under the same URL path (e.g. `media`) | (disabled) |

## License

MIT
