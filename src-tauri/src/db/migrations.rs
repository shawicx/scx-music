const V1_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS songs (
    id           TEXT PRIMARY KEY,
    title        TEXT NOT NULL,
    artist       TEXT NOT NULL DEFAULT '',
    album        TEXT NOT NULL DEFAULT '',
    duration     TEXT NOT NULL,
    duration_secs REAL NOT NULL,
    quality      TEXT NOT NULL DEFAULT '',
    file_path    TEXT NOT NULL UNIQUE,
    art_gradient TEXT NOT NULL DEFAULT '',
    created_at   TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS playlists (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS playlist_songs (
    playlist_id TEXT NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    song_id     TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (playlist_id, song_id)
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_songs_artist ON songs(artist);
CREATE INDEX IF NOT EXISTS idx_songs_album ON songs(album);
CREATE INDEX IF NOT EXISTS idx_playlist_songs_playlist ON playlist_songs(playlist_id);

INSERT OR IGNORE INTO playlists (id, name, sort_order) VALUES
    ('fav', '我喜欢的', 0);
";

pub fn run_migrations(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    conn.execute_batch(V1_SCHEMA)?;
    Ok(())
}
