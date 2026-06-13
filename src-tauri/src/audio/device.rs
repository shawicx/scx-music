use rodio::cpal::traits::HostTrait;
use rodio::{OutputStream, OutputStreamHandle};
use tauri::{AppHandle, Emitter};

use super::types::*;
use super::AudioState;

/// Try to create an OutputStream for the given device.
/// Falls back to enumerating supported configs if default_output_config() fails
/// (fixes "Invalid property value" on some CoreAudio devices like Mac mini speakers).
pub(super) fn try_output_stream_for_device(
    device: &rodio::cpal::Device,
) -> Result<(OutputStream, OutputStreamHandle), rodio::StreamError> {
    match OutputStream::try_from_device(device) {
        Ok(s) => return Ok(s),
        Err(_) => {}
    }

    use rodio::cpal::traits::DeviceTrait;
    match device.supported_output_configs() {
        Ok(configs) => {
            let configs: Vec<_> = configs.collect();
            for cfg in &configs {
                let config = cfg.with_max_sample_rate();
                match OutputStream::try_from_device_config(device, config) {
                    Ok(s) => return Ok(s),
                    Err(_) => {}
                }
            }
            try_build_hardcoded_stream(device)
        }
        Err(_) => {
            try_build_hardcoded_stream(device)
        }
    }
}

/// Build an output stream with hardcoded standard configs as a last resort.
pub(super) fn try_build_hardcoded_stream(
    device: &rodio::cpal::Device,
) -> Result<(OutputStream, OutputStreamHandle), rodio::StreamError> {
    use rodio::cpal::{SampleFormat, SampleRate, SupportedStreamConfig, SupportedBufferSize};

    let configs_to_try: Vec<(u32, u16, SampleFormat)> = vec![
        (48000, 2, SampleFormat::F32),
        (44100, 2, SampleFormat::F32),
        (48000, 2, SampleFormat::I16),
        (44100, 2, SampleFormat::I16),
        (96000, 2, SampleFormat::F32),
        (48000, 1, SampleFormat::F32),
        (44100, 1, SampleFormat::F32),
    ];

    for (rate, channels, fmt) in configs_to_try {
        let supported = SupportedStreamConfig::new(
            channels,
            SampleRate(rate),
            SupportedBufferSize::Unknown,
            fmt,
        );
        match OutputStream::try_from_device_config(device, supported) {
            Ok(s) => return Ok(s),
            Err(_) => {}
        }
    }

    Err(rodio::StreamError::NoDevice)
}

pub(super) fn find_device_by_name(name: &str) -> Result<rodio::cpal::Device, String> {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};
    let host = rodio::cpal::default_host();

    if let Some(default) = host.default_output_device() {
        if default.name().ok().as_deref() == Some(name) {
            return Ok(default);
        }
    }

    let mut all_devices = host
        .devices()
        .map_err(|e| format!("Device enumeration error: {}", e))?;
    all_devices
        .find(|d| d.name().ok().as_deref() == Some(name))
        .ok_or_else(|| format!("Device '{}' not found", name))
}

// ---- Tauri Commands ----

#[tauri::command]
pub fn player_get_output_devices() -> Result<AudioDevicesResponse, String> {
    use rodio::cpal::traits::DeviceTrait;
    let host = rodio::cpal::default_host();
    let default_name = host.default_output_device().and_then(|d| d.name().ok());

    let all_devices = host
        .devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

    let mut seen = std::collections::HashSet::new();
    let result: Vec<AudioDeviceInfo> = all_devices
        .filter_map(|d| {
            let name = d.name().ok()?;
            if !seen.insert(name.clone()) {
                return None;
            }
            let is_default = default_name.as_ref() == Some(&name);
            Some(AudioDeviceInfo { name, is_default })
        })
        .collect();

    Ok(AudioDevicesResponse {
        devices: result,
        default_device_name: default_name,
    })
}

#[tauri::command]
pub fn player_set_output_device(
    app: AppHandle,
    state: tauri::State<'_, AudioState>,
    db: tauri::State<'_, crate::db::Db>,
    device_name: Option<String>,
) -> Result<(), String> {
    if let Some(ref name) = device_name {
        let device = find_device_by_name(name)?;
        match try_output_stream_for_device(&device) {
            Ok((_stream, _handle)) => {
                drop(_stream);
                drop(_handle);
            }
            Err(e) => {
                return Err(format!("无法使用设备「{}」: {}", name, e));
            }
        }
    }

    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let value = device_name.as_deref().unwrap_or("");
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params!["output_device", value],
        )
        .map_err(|e| e.to_string())?;
    }

    let arc: AudioState = (*state).clone();
    let payload;
    {
        let mut s = arc.lock().map_err(|e| e.to_string())?;
        s.rebuild_engine_with_device(device_name)?;
        payload = s.get_state_payload();
    }
    let _ = app.emit("audio:state_change", &payload);
    Ok(())
}

#[tauri::command]
pub fn player_get_current_device(
    state: tauri::State<'_, AudioState>,
) -> Result<Option<String>, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    Ok(s.output_device_name.clone())
}
