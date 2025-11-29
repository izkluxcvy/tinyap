## tinyap
micro-blogging platform built on ActivityPub, part of the Fediverse.

NoJS, Fast, Lightweight.

Demo: [@alice@tinyap.izkluxcvy.foo](https://tinyap.izkluxcvy.foo/@alice)

## Requirements

- openssl ([more info](https://docs.rs/openssl/latest/openssl/#automatic))
- sqlite3
- Rust
- HTTP**S** for federation

## Installation

Clone git repo.

```sh
$ git clone https://github.com/izkluxcvy/tinyap.git
$ cd tinyap
```

Create database first. (Building first will result in failure.)

```sh
$ sqlite3 tinyap.db
```

```sql
sqlite> CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT,
    actor_id TEXT UNIQUE NOT NULL,
    private_key TEXT,
    public_key TEXT,
    display_name TEXT NOT NULL,
    bio TEXT DEFAULT '',
    created_at TEXT NOT NULL,
    is_local INTEGER NOT NULL
);
CREATE TABLE sessions (
    session_id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);
CREATE TABLE follows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    object_actor TEXT NOT NULL,
    pending INTEGER NOT NULL,
    UNIQUE(user_id, object_actor)
);
CREATE TABLE followers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    actor TEXT NOT NULL,
    inbox TEXT NOT NULL,
    UNIQUE(user_id, actor)
);
CREATE TABLE notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT UNIQUE NOT NULL,
    ap_id TEXT UNIQUE NOT NULL,
    user_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    in_reply_to TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (in_reply_to) REFERENCES notes(ap_id)
);
CREATE TABLE likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    note_apid TEXT NOT NULL,
    actor TEXT NOT NULL,
    UNIQUE(note_apid, actor),
    FOREIGN KEY (note_apid) REFERENCES notes(ap_id)
);
CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    type TEXT NOT NULL,
    actor TEXT NOT NULL,
    note_uuid TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (username) REFERENCES users(username),
    FOREIGN KEY (note_uuid) REFERENCES notes(uuid)
);

sqlite> .exit
```

Configure your .env file

```sh
$ vi .env
```

```sh
DATABASE_URL=sqlite://./tinyap.db
HOST=127.0.0.1
PORT=8080
DOMAIN=example.com
SESSION_TTL_DAYS=90
SESSION_MAX_PER_USER=5
ALLOW_SIGNUP=true
MAX_TIMELINE_NOTES=50
```

Build and Run

```sh
$ cargo build --release
$ mv target/release/tinyap ./
$ cargo clean

$ ./tinyap
```

## Benchmark

```sh
$ curl http://localhost:8080/local -w "%{time_total}\n" -o /dev/null -sS
0.002244
$ wrk -t16 -c400 -d10s http://localhost:8080/local
Running 10s test @ http://localhost:8080/local
  16 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    18.49ms   10.21ms 278.14ms   97.04%
    Req/Sec     1.41k   485.19    17.05k    98.56%
  224539 requests in 10.10s, 399.58MB read
Requests/sec:  22231.70
Transfer/sec:     39.56MB

$ curl http://localhost:8080/@alice -w "%{time_total}\n" -o /dev/null -sS
0.002189
$ wrk -t16 -c400 -d10s http://localhost:8080/@alice
Running 10s test @ http://localhost:8080/@alice
  16 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    35.11ms   63.01ms 816.95ms   96.95%
    Req/Sec     0.98k   373.83    11.59k    95.87%
  153806 requests in 10.06s, 237.92MB read
Requests/sec:  15289.49
Transfer/sec:     23.65MB
```