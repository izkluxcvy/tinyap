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
    is_local BIGINT NOT NULL,
    note_count BIGINT NOT NULL DEFAULT 0,
    following_count BIGINT NOT NULL DEFAULT 0,
    follower_count BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX idx_users_is_local ON users(is_local);

CREATE TABLE follows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    follower_id BIGINT NOT NULL,
    followee_id BIGINT NOT NULL,
    pending BIGINT NOT NULL,
    UNIQUE(follower_id, followee_id),
    FOREIGN KEY (follower_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (followee_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE table notes (
    id BIGINT PRIMARY KEY,
    ap_url TEXT NOT NULL UNIQUE,
    author_id BIGINT NOT NULL,
    boosted_id BIGINT,
    boosted_username TEXT,
    boosted_created_at TEXT,
    content TEXT NOT NULL,
    attachments TEXT,
    parent_id BIGINT,
    parent_author_username TEXT,
    created_at TEXT NOT NULL,
    is_public BIGINT NOT NULL,
    like_count BIGINT NOT NULL DEFAULT 0,
    boost_count BIGINT NOT NULL DEFAULT 0,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (boosted_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES notes(id) ON DELETE CASCADE
);
CREATE INDEX idx_notes_author_id ON notes(author_id);
CREATE INDEX idx_notes_boosted_id ON notes(boosted_id);
CREATE INDEX idx_notes_parent_id ON notes(parent_id);
CREATE INDEX idx_notes_created_at_id ON notes(created_at, id);

CREATE TABLE likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id BIGINT NOT NULL,
    note_id BIGINT NOT NULL,
    UNIQUE(user_id, note_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
);

CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type BIGINT NOT NULL,
    sender_id BIGINT NOT NULL,
    recipient_id BIGINT NOT NULL,
    note_id BIGINT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (recipient_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
);
CREATE INDEX idx_notifications_recipient_id ON notifications(recipient_id);
CREATE INDEX idx_notifications_created_at ON notifications(created_at);

-- Web session
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL UNIQUE,
    user_id BIGINT NOT NULL,
    expires_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- API OAuth
CREATE TABLE oauth_apps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_name TEXT NOT NULL,
    redirect_uri TEXT NOT NULL,
    client_id BIGINT NOT NULL UNIQUE,
    client_secret TEXT NOT NULL
);

CREATE TABLE oauth_authorizations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id BIGINT NOT NULL,
    client_id BIGINT NOT NULL,
    code TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (client_id) REFERENCES oauth_apps(client_id) ON DELETE CASCADE
);

CREATE TABLE oauth_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id BIGINT NOT NULL,
    client_id BIGINT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (client_id) REFERENCES oauth_apps(client_id) ON DELETE CASCADE
);
