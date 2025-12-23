## tinyap
micro-blogging platform built on ActivityPub, part of the Fediverse.

NoJS, Fast, Lightweight.

Demo: [@alice@tinyap.izkluxcvy.foo](https://tinyap.izkluxcvy.foo/@alice)

## Features

- Fetch remote user by accessing /@user@example.com
- Follow, Like, Boost, Undo them
- Create and deliver note
- Reply
- Timeline

## Requirements

- OpenSSL ([more info](https://docs.rs/openssl/latest/openssl/#automatic))
- SQLite3
- Rust
- Http**S**

## Installation

Clone git repo.

```sh
$ git clone https://github.com/izkluxcvy/tinyap.git
$ cd tinyap
```

Create database first. (Building first will result in failure.)

```sh
$ sqlite3 tinyap.db < scheme.sql
```

Configure your .env file

```sh
$ vi .env
```

CERT_PATH and KEY_PATH is only required when building with `tls` feature.

```sh
SITE_NAME=TinyAP
DATABASE_URL=sqlite://./tinyap.db
HOST=127.0.0.1
PORT=8443
CERT_PATH=./server.crt
KEY_PATH=./server.key
DOMAIN=example.com
TIMEZONE=Asia/Tokyo
SESSION_TTL_DAYS=90
SESSION_MAX_PER_USER=5
ALLOW_SIGNUP=true
MAX_TIMELINE_NOTES=50
MAX_NOTE_CHARS=2000
```

Build and Run

```sh
$ cargo build --release
$ # OR
$ cargo build --release --features=tls
$ # for using tinyap as a TLS termination

$ mv target/release/tinyap ./
$ cargo clean

$ ./tinyap
```

## Benchmark

AMD Ryzen 7 5700X (16) @ 4.66 GHz

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