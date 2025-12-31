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
    is_local INTEGER NOT NULL,
    note_count INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_users_is_local ON users(is_local);

CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    expires_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE follows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    follower_id INTEGER NOT NULL,
    followee_id INTEGER NOT NULL,
    pending INTEGER NOT NULL,
    UNIQUE(follower_id, followee_id),
    FOREIGN KEY (follower_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (followee_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE table notes (
    id BIGINT PRIMARY KEY,
    ap_url TEXT NOT NULL UNIQUE,
    author_id INTEGER NOT NULL,
    boosted_id BIGINT,
    boosted_username TEXT,
    boosted_created_at TEXT,
    content TEXT NOT NULL,
    attachments TEXT,
    parent_id BIGINT,
    parent_author_username TEXT,
    created_at TEXT NOT NULL,
    is_public INTEGER NOT NULL,
    like_count INTEGER NOT NULL DEFAULT 0,
    boost_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (boosted_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES notes(id) ON DELETE CASCADE
);
CREATE INDEX idx_notes_created_at ON notes(created_at);

CREATE TABLE likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    note_id BIGINT NOT NULL,
    UNIQUE(user_id, note_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
)

CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type INTEGER NOT NULL,
    sender_id INTEGER NOT NULL,
    recipient_id INTEGER NOT NULL,
    note_id BIGINT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (recipient_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
);
CREATE INDEX idx_notifications_created_at ON notifications(created_at);