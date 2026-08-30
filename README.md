## TinyAP
micro-blogging software built on ActivityPub, part of the Fediverse.

NoJS, Fast, Lightweight.

Demo: [@alice@tinyap.izkluxcvy.foo](https://tinyap.izkluxcvy.foo/@alice)

## Features

- Federate with remote users
- Create text note
- Follow, Reply, Mention, Like, Boost, Undo them
- Block domain
- Tiny memory usage
- Tiny web UI
- Mastodon-compatible API

### Memory usage

Real memory usage of `tinyap.izkluxcvy.foo`

```sh
$ watch -n 360 'echo $(date "+%F %T"),$(cat /sys/fs/cgroup/system.slice/tinyap.service/memory.current) >> memory.csv'
```

![memory usage](memory_usage.webp)

(Note that Argon2 password hasher costs 19MB memory)

### Tested clients

- [Phanpy](https://phanpy.social/) for Web
- [Pinafore](https://pinafore.social/) for Web
- [Tokodon](https://apps.kde.org/tokodon/) for Linux, Windows
- [Tuba](https://tuba.geopjr.dev/) for Linux, Windows
- [Ice Cubes](https://apps.apple.com/us/app/ice-cubes-for-mastodon/id6444915884) for iOS, Mac
- [Fedicat](https://fedicat.com/) for iOS
- [Tusky](https://tusky.app/) for Android
- [Toot](https://github.com/ihabunek/toot) for CLI, TUI

## Requirements

- Rust (for building from source)
- SQLite or PostgreSQL
- Http**S**

## Installation

### Debian package

Download .deb package(features: sqlite, web, api) from [Releases](https://github.com/izkluxcvy/tinyap/releases/)

Install TinyAP

```sh
$ sudo apt install ./tinyap_amd64.deb
$ tinyap --version
```

Configure and run

```sh
$ sudo vi /etc/tinyap/config.yaml
$ sudo systemctl enable --now tinyap.service
```

### Build from source

Clone git repo.

```sh
$ git clone --depth 1 https://github.com/izkluxcvy/tinyap.git
$ cd tinyap
```

Create database.

```sh
$ # for SQLite
$ sqlite3 tinyap.db < schema.sql

$ # for PostgreSQL
$ psql -U postgres -c "CREATE DATABASE tinyap"
$ sed -e "s/INTEGER PRIMARY KEY AUTOINCREMENT/BIGSERIAL PRIMARY KEY/g" schema.sql | psql -U postgres -d tinyap
```

Configure your config.yaml.

```sh
$ vi config.yaml
```

Build and run

```sh
$ cargo build --release --features=sqlite,web,api

$ mv target/release/tinyap ./
$ cargo clean

$ ./tinyap --help
$ ./tinyap serve
```

For Linux/glibc, `MALLOC_MMAP_THRESHOLD_=131072` environment variable can suppress memory fragmentation.

### Build feature flags:

- `mimalloc`: use mimalloc for memory allocator instead of system allocator
- `sqlite`: use SQLite for DB (must be exclusive with postgres)
- `postgres`: use PostgreSQL for DB
- `web`: text-based tiny Web UI
- `api`: mastodon-compatible API
- `tls`: tinyap as a TLS termination

### Config file path

You can place in `./config.yaml` or `/etc/tinyap/config.yaml` or `$TINYAP_CONFIG`

```sh
$ TINYAP_CONFIG=/path/to/config.yaml ./tinyap serve
```

### Setup full text search

For SQLite (FTS5):

```sh
$ sqlite3 tinyap.db
..
sqlite> .exit
```

```sql
CREATE VIRTUAL TABLE notes_fts USING fts5("content", content='notes', content_rowid='id', tokenize="unicode61 tokenchars '#'");
CREATE TRIGGER notes_ai AFTER INSERT ON notes BEGIN INSERT INTO notes_fts(rowid, content) VALUES (new.id, new.content); END;
CREATE TRIGGER notes_ad AFTER DELETE ON notes BEGIN INSERT INTO notes_fts(notes_fts, rowid, content) VALUES ('delete', old.id, old.content); END;
INSERT INTO notes_fts(rowid, content) SELECT id, content FROM notes;
```

For PostgreSQL:

```sh
$ psql -U postgres -d tinyap
..
tinyap=# \q
```

```sql
ALTER TABLE notes ADD COLUMN search_vector tsvector GENERATED ALWAYS AS (to_tsvector('english', coalesce(content, ''))) STORED;
CREATE INDEX idx_notes_search_vector ON notes USING GIN (search_vector);
```

## Customizing Web UI

### templates/

HTML with [Jinja](https://en.wikipedia.org/wiki/Jinja_(template_engine)) template format.

Loaded once when server starts.

### static/

Static files like style.css.

Loaded on each access to /static/xxx.xx.
