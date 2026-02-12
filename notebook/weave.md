# Weave

Weave is a **self-hosted**, **single-user** and **web-based** frontend to view
and edit [zk](https://github.com/zk-org/zk) notes. It is lightweight, quick and
opinionated and features

- a single deployed binary via `zk` re-implementation
- basic note editing
- syntax highlighting of code blocks


## Quickstart

To start a server that hosts an editable version of this notebook, check out the
code, run

```bash
ZK_NOTEBOOK_DIR="$(pwd)/notebook" WEAVE_PASSWORD="secret" cargo run
```

and go to [http://localhost:8000](http://localhost:8000).


## Further topics

- [Installing weave](install)
- [Special tags](tags)


## License

MIT

#public #pin
