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

CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    expires_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE table notes (
    id BIGINT PRIMARY KEY,
    ap_url TEXT NOT NULL UNIQUE,
    author_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    attachments TEXT,
    created_at TEXT NOT NULL,
    is_public INTEGER NOT NULL,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX idx_notes_author_id ON notes(author_id);