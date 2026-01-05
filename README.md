## TinyAP v1.alpha
micro-blogging software built on ActivityPub, part of the Fediverse.

NoJS, Fast, Lightweight.

Demo: [@alice@tinyap.izkluxcvy.foo](https://tinyap.izkluxcvy.foo/@alice)

## Features

- Federate with remote users
- Create note
- Follow, Reply, Like, Boost, Undo them
- Tiny web UI
- Mastodon-compatible API

### Tested clients

- [Phanpy](https://phanpy.social/) for Web
- [Pinafore](https://pinafore.social/) for Web
- [Flare](https://flareapp.moe/) for Android, iOS
- [toot](https://github.com/ihabunek/toot) for CLI

## Requirements

- Rust
- SQLite3
- Http**S**

## Installation

Clone git repo.

```sh
$ git clone https://github.com/izkluxcvy/tinyap.git
$ cd tinyap
```

Create database.

```sh
$ sqlite3 tinyap.db < schema.sql
```

Configure your config.yaml.

```sh
$ vi config.yaml
```

Build and run

```sh
$ cargo build --release --features=web,api

$ mv target/release/tinyap ./
$ cargo clean

$ ./tinyap --help
$ ./tinyap serve
```

### Build feature flags:

- `web`: text-based tiny Web UI
- `api`: mastodon-compatible API
- `tls`: tinyap as a TLS termination

## Customizing Web UI

### templates/

HTML with [Jinja](https://en.wikipedia.org/wiki/Jinja_(template_engine)) template format.

Loaded once when server starts.

### static/

Static files like style.css.

Loaded on each access to /static/xxx.xx.