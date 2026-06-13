const INIT_SCHEMA: &str = "
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
    genre        TEXT NOT NULL DEFAULT '',
    file_size    INTEGER NOT NULL DEFAULT 0,
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

CREATE TABLE IF NOT EXISTS lyrics (
    song_id     TEXT PRIMARY KEY,
    raw_lrc     TEXT,
    source      TEXT NOT NULL,
    offset_secs REAL NOT NULL DEFAULT 0.0
);

CREATE INDEX IF NOT EXISTS idx_songs_artist ON songs(artist);
CREATE INDEX IF NOT EXISTS idx_songs_album ON songs(album);
CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title);
CREATE INDEX IF NOT EXISTS idx_songs_created_at ON songs(created_at);
CREATE INDEX IF NOT EXISTS idx_songs_genre ON songs(genre);
CREATE INDEX IF NOT EXISTS idx_playlist_songs_playlist ON playlist_songs(playlist_id);
CREATE INDEX IF NOT EXISTS idx_playlist_songs_playlist_position ON playlist_songs(playlist_id, sort_order);

INSERT OR IGNORE INTO playlists (id, name, sort_order) VALUES ('fav', '我喜欢的', 0);
";

const V6_PLAY_HISTORY: &str = "
CREATE TABLE IF NOT EXISTS play_history (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    song_id       TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    played_at     TEXT NOT NULL DEFAULT (datetime('now')),
    duration_secs REAL NOT NULL DEFAULT 0.0
);

CREATE INDEX IF NOT EXISTS idx_play_history_song ON play_history(song_id);
CREATE INDEX IF NOT EXISTS idx_play_history_played_at ON play_history(played_at);
";

pub fn run_migrations(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    conn.execute_batch(INIT_SCHEMA)?;

    // V6: play_history table + songs cache fields
    let play_history_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='play_history'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !play_history_exists {
        conn.execute_batch(V6_PLAY_HISTORY)?;

        // Add cache columns to songs table (idempotent via try-catch style)
        let _ = conn.execute_batch(
            "ALTER TABLE songs ADD COLUMN play_count INTEGER NOT NULL DEFAULT 0;",
        );
        let _ = conn.execute_batch(
            "ALTER TABLE songs ADD COLUMN total_play_duration REAL NOT NULL DEFAULT 0.0;",
        );
    }

    Ok(())
}
