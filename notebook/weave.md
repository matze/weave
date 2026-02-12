# Weave

Weave is a **self-hosted**, **single-user** and **web-based** frontend to view
and edit [zk](https://github.com/zk-org/zk) notes. It is lightweight, quick and
opinionated. It features

- a single binary with a built-in zk re-implementation (no `zk` binary needed)
- fuzzy search across all note titles and tags
- note editing
- syntax highlighting of code blocks
- real-time file watching (external edits show up immediately)
- light and dark mode
- special tags for access control and ordering


## Quickstart

To start a server that hosts an editable version of this notebook, check out the
code and run

```bash
ZK_NOTEBOOK_DIR="$(pwd)/notebook" WEAVE_PASSWORD="secret" cargo run --release
```

This starts the server on [http://localhost:8000](http://localhost:8000). You
should be able to see this page under
[http://localhost:8000/note/weave](http://localhost:8000/note/weave).


## Further topics

- [Installation](install)
- [Usage](using)


## License

MIT

#public #pin
