CREATE TABLE users (
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
    boosted_username TEXT,
    boosted_created_at TEXT,
    content TEXT NOT NULL,
    in_reply_to TEXT,
    reply_to_author TEXT,
    created_at TEXT NOT NULL,
    is_public INTEGER NOT NULL DEFAULT 1,
    like_count INTEGER NOT NULL DEFAULT 0,
    boost_count INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_notes_created_at ON notes(created_at);
CREATE INDEX idx_notes_inreplyto ON notes(in_reply_to);
CREATE TABLE likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    note_apid TEXT NOT NULL,
    actor TEXT NOT NULL,
    UNIQUE(note_apid, actor)
);
CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    type TEXT NOT NULL,
    actor TEXT NOT NULL,
    note_uuid TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_notifications_username_created_at ON notifications(username, created_at);
