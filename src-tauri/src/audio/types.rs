use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackMode {
    Sequential,
    RepeatAll,
    RepeatOne,
    Shuffle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedSong {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f64,
    pub quality: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatePayload {
    pub current_song: Option<QueuedSong>,
    pub state: PlaybackState,
    pub volume: f64,
    pub mode: PlaybackMode,
    pub progress: f64,
    pub duration: f64,
    pub queue_length: usize,
    pub queue_index: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevicesResponse {
    pub devices: Vec<AudioDeviceInfo>,
    pub default_device_name: Option<String>,
}
