CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT,
    ap_url TEXT NOT NULL UNIQUE,
    inbox_url TEXT NOT NULL,
    private_key TEXT,
    public_key TEXT,
    display_name TEXT NOT NULL,
    bio TEXT DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_local INTEGER NOT NULL
);