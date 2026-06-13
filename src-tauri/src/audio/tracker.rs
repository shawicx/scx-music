use std::time::Instant;

pub struct PlaySession {
    pub song_id: String,
    pub song_duration_secs: f64,
    pub started_at: Option<Instant>,
    pub accumulated_secs: f64,
    pub total_listened_secs: f64,
}

impl PlaySession {
    pub fn new(song_id: String, song_duration_secs: f64) -> Self {
        Self {
            song_id,
            song_duration_secs,
            started_at: Some(Instant::now()),
            accumulated_secs: 0.0,
            total_listened_secs: 0.0,
        }
    }

    pub fn pause(&mut self) {
        if let Some(t) = self.started_at.take() {
            let elapsed = t.elapsed().as_secs_f64();
            self.accumulated_secs += elapsed;
            self.total_listened_secs += elapsed;
        }
    }

    pub fn resume(&mut self) {
        self.started_at = Some(Instant::now());
    }

    pub fn total_secs(&self) -> f64 {
        self.accumulated_secs
            + self
                .started_at
                .map(|t| t.elapsed().as_secs_f64())
                .unwrap_or(0.0)
    }
}

const MIN_RECORD_SECS: f64 = 5.0;

/// Write play duration to database (history + total_play_duration).
/// Does NOT touch play_count — use finalize_session for that.
pub fn flush_session(
    conn: &rusqlite::Connection,
    session: &PlaySession,
) -> rusqlite::Result<bool> {
    let secs = session.total_secs();
    if secs < MIN_RECORD_SECS {
        return Ok(false);
    }

    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "INSERT INTO play_history (song_id, duration_secs) VALUES (?1, ?2)",
        rusqlite::params![session.song_id, secs],
    )?;

    tx.execute(
        "UPDATE songs SET total_play_duration = total_play_duration + ?1 WHERE id = ?2",
        rusqlite::params![secs, session.song_id],
    )?;

    tx.commit()?;
    Ok(true)
}

/// Finalize a session: flush remaining duration, then check if total listened
/// time >= 50% of song duration to increment play_count.
pub fn finalize_session(
    conn: &rusqlite::Connection,
    session: &PlaySession,
) -> rusqlite::Result<bool> {
    let flushed = flush_session(conn, session)?;

    // Use total_listened_secs (never reset by auto-flush) for the 50% check
    let total = session.total_listened_secs
        + session
            .started_at
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0);

    if session.song_duration_secs > 0.0 && total >= session.song_duration_secs * 0.5 {
        conn.execute(
            "UPDATE songs SET play_count = play_count + 1 WHERE id = ?1",
            rusqlite::params![session.song_id],
        )?;
    }

    Ok(flushed)
}
