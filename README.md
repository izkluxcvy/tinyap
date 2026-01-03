## TinyAP v1.alpha
micro-blogging software built on ActivityPub, part of the Fediverse.

NoJS, Fast, Lightweight.

Demo: [@alice@tinyap.izkluxcvy.foo](https://tinyap.izkluxcvy.foo/@alice)

## Features

- Federate with remote users
- Create note
- Follow, Reply, Like, Boost, Undo them
- Tiny web UI
- (coming soon) Mastodon-compatible API

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

Build and Run

```sh
$ cargo build --release

$ mv target/release/tinyap ./
$ cargo clean

$ ./tinyap serve
```