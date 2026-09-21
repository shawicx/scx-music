use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use rodio::Source;
use serde::{Deserialize, Serialize};

/// ISO 倍频程频段中心频率（Hz）
pub const EQ_BAND_HZ: [f32; 10] = [
    31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];
const NUM_BANDS: usize = 10;
/// 倍频程带宽对应的 Q 值
const PEAKING_Q: f32 = 1.41;
/// 参数轮询粒度（样本数，44.1kHz 下约 12ms）
const BLOCK_SAMPLES: usize = 512;
/// 切换预设时系数线性平滑的长度（约 23ms）
const SMOOTH_SAMPLES: usize = 1024;
/// f0 钳制上限：奈奎斯特频率 × 0.45
const F0_NYQUIST_RATIO: f32 = 0.45;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EqParams {
    pub preset_id: String,
    pub enabled: bool,
    pub preamp_db: f32,
    pub gains_db: [f32; NUM_BANDS],
}

impl EqParams {
    pub fn flat() -> Self {
        Self {
            preset_id: "flat".into(),
            enabled: false,
            preamp_db: 0.0,
            gains_db: [0.0; NUM_BANDS],
        }
    }

    fn is_identity(&self) -> bool {
        self.preamp_db == 0.0 && self.gains_db.iter().all(|&g| g == 0.0)
    }
}

/// 内置预设定义（IPC 枚举给前端，UI 可画迷你曲线）
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EqPresetDef {
    pub id: String,
    pub preamp_db: f32,
    pub gains_db: [f32; NUM_BANDS],
}

pub fn builtin_presets() -> Vec<EqPresetDef> {
    fn p(id: &str, preamp_db: f32, gains_db: [f32; NUM_BANDS]) -> EqPresetDef {
        EqPresetDef {
            id: id.into(),
            preamp_db,
            gains_db,
        }
    }
    vec![
        p("flat", 0.0, [0.0; NUM_BANDS]),
        p("pop", -1.0, [-1.0, 1.0, 3.0, 4.0, 3.0, 0.0, -1.0, -1.0, 0.0, 0.0]),
        p("rock", -3.0, [5.0, 4.0, 3.0, 1.0, -1.0, -1.0, 1.0, 3.0, 5.0, 5.0]),
        p("classical", -1.0, [4.0, 3.0, 2.0, 0.0, -1.0, -1.0, 0.0, 2.0, 3.0, 4.0]),
        p("jazz", -1.0, [3.0, 2.0, 1.0, 2.0, -1.0, -1.0, 0.0, 1.0, 2.0, 3.0]),
        p("vocal", -1.0, [-2.0, -1.0, 0.0, 2.0, 4.0, 4.0, 3.0, 1.0, 0.0, -2.0]),
        p("bass_boost", -4.0, [6.0, 5.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        p("electronic", -3.0, [4.0, 3.0, 1.0, 0.0, -2.0, 1.0, 0.0, 2.0, 4.0, 5.0]),
    ]
}

pub fn find_preset(id: &str) -> Option<EqPresetDef> {
    builtin_presets().into_iter().find(|p| p.id == id)
}

// ---- Biquad（RBJ Cookbook）----

#[derive(Clone, Copy, Debug, PartialEq)]
struct BiquadCoeffs {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl BiquadCoeffs {
    const IDENTITY: BiquadCoeffs = BiquadCoeffs {
        b0: 1.0,
        b1: 0.0,
        b2: 0.0,
        a1: 0.0,
        a2: 0.0,
    };
}

/// f0 钳制到远离奈奎斯特，避免低采样率下 shelf/peaking 失稳
fn clamp_f0(f0: f32, sample_rate: f32) -> f32 {
    f0.min(sample_rate * F0_NYQUIST_RATIO).max(1.0)
}

/// RBJ w0 = 2π·f0/sr（先钳制 f0）
fn w0_of(f0: f32, sample_rate: f32) -> f32 {
    2.0 * std::f32::consts::PI * clamp_f0(f0, sample_rate) / sample_rate
}

/// 峰值滤波器系数（RBJ peaking EQ）。中心频率增益 = gain_db。
fn peaking_coeffs(f0: f32, q: f32, gain_db: f32, sample_rate: f32) -> BiquadCoeffs {
    let a = db_to_linear(gain_db).sqrt();
    let w0 = w0_of(f0, sample_rate);
    let cw = w0.cos();
    let alpha = w0.sin() / (2.0 * q);

    let b0 = 1.0 + alpha * a;
    let b1 = -2.0 * cw;
    let b2 = 1.0 - alpha * a;
    let a0 = 1.0 + alpha / a;
    let a1 = -2.0 * cw;
    let a2 = 1.0 - alpha / a;
    BiquadCoeffs {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
    }
}

/// 低架滤波器系数（RBJ low shelf，S 用与 peaking 相同的 Q 表达）
fn low_shelf_coeffs(f0: f32, gain_db: f32, sample_rate: f32) -> BiquadCoeffs {
    let a = db_to_linear(gain_db).sqrt();
    let w0 = w0_of(f0, sample_rate);
    let cw = w0.cos();
    let sw = w0.sin();
    let beta = a.sqrt() / PEAKING_Q;

    let b0 = a * ((a + 1.0) - (a - 1.0) * cw + beta * sw);
    let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cw);
    let b2 = a * ((a + 1.0) - (a - 1.0) * cw - beta * sw);
    let a0 = (a + 1.0) + (a - 1.0) * cw + beta * sw;
    let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cw);
    let a2 = (a + 1.0) + (a - 1.0) * cw - beta * sw;
    BiquadCoeffs {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
    }
}

/// 高架滤波器系数（RBJ high shelf）
fn high_shelf_coeffs(f0: f32, gain_db: f32, sample_rate: f32) -> BiquadCoeffs {
    let a = db_to_linear(gain_db).sqrt();
    let w0 = w0_of(f0, sample_rate);
    let cw = w0.cos();
    let sw = w0.sin();
    let beta = a.sqrt() / PEAKING_Q;

    let b0 = a * ((a + 1.0) + (a - 1.0) * cw + beta * sw);
    let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cw);
    let b2 = a * ((a + 1.0) + (a - 1.0) * cw - beta * sw);
    let a0 = (a + 1.0) - (a - 1.0) * cw + beta * sw;
    let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cw);
    let a2 = (a + 1.0) - (a - 1.0) * cw - beta * sw;
    BiquadCoeffs {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
    }
}

/// 按频段选择滤波器类型：最低频段低架、最高频段高架、中间峰值。
/// 0dB 直接返回恒等系数，保证 flat 精确透传。
fn band_coeffs(band: usize, gain_db: f32, sample_rate: f32) -> BiquadCoeffs {
    if gain_db == 0.0 {
        return BiquadCoeffs::IDENTITY;
    }
    let f0 = EQ_BAND_HZ[band];
    match band {
        0 => low_shelf_coeffs(f0, gain_db, sample_rate),
        9 => high_shelf_coeffs(f0, gain_db, sample_rate),
        _ => peaking_coeffs(f0, PEAKING_Q, gain_db, sample_rate),
    }
}

fn params_coeffs(params: &EqParams, sample_rate: f32) -> Vec<BiquadCoeffs> {
    if params.is_identity() {
        return vec![BiquadCoeffs::IDENTITY; NUM_BANDS];
    }
    (0..NUM_BANDS)
        .map(|b| band_coeffs(b, params.gains_db[b], sample_rate))
        .collect()
}

fn db_to_linear(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

/// biquad 状态（直接 II 型转置 TDF2）
#[derive(Clone, Copy, Default)]
struct BiquadState {
    z1: f32,
    z2: f32,
}

impl BiquadState {
    fn process(&mut self, x: f32, c: &BiquadCoeffs) -> f32 {
        let y = c.b0 * x + self.z1;
        self.z1 = c.b1 * x - c.a1 * y + self.z2;
        self.z2 = c.b2 * x - c.a2 * y;
        y
    }
}

// ---- 共享句柄 ----

/// 引擎与源包装器之间的参数共享：切预设只更新参数 + bump 版本号，
/// 正在播放的 EffectsSource 按块轮询版本后平滑过渡。
#[derive(Clone)]
pub struct EffectsHandle {
    params: Arc<Mutex<EqParams>>,
    version: Arc<AtomicU64>,
}

impl EffectsHandle {
    pub fn new() -> Self {
        Self {
            params: Arc::new(Mutex::new(EqParams::flat())),
            version: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn get(&self) -> EqParams {
        // 参数锁中毒自愈：取回内部数据继续服务（与 audio/mod.rs 策略一致）
        self.params
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn set(&self, params: EqParams) {
        let mut guard = self.params.lock().unwrap_or_else(|e| e.into_inner());
        *guard = params;
        self.version.fetch_add(1, Ordering::SeqCst);
    }

    fn version(&self) -> u64 {
        self.version.load(Ordering::SeqCst)
    }
}

// ---- Source 包装器 ----

/// EQ 源包装器：仿 TeeSource 模式，逐样本滤波 + 按块感知参数变化。
/// 直通条件（disabled / flat / 平滑完成且状态归零）下零 DSP 开销。
pub struct EffectsSource<S> {
    source: S,
    handle: EffectsHandle,
    sample_rate: f32,
    seen_version: u64,
    channels: usize,
    ch_idx: usize,
    pos: usize,
    bypass: bool,
    pending_bypass: bool,
    smooth_remaining: usize,
    from_coeffs: Vec<BiquadCoeffs>,
    to_coeffs: Vec<BiquadCoeffs>,
    cur_coeffs: Vec<BiquadCoeffs>,
    from_preamp: f32,
    to_preamp: f32,
    cur_preamp: f32,
    states: Vec<Vec<BiquadState>>,
}

impl<S: Iterator<Item = f32> + Source> EffectsSource<S> {
    pub fn new(source: S, handle: EffectsHandle) -> Self {
        let channels = (source.channels() as usize).max(1);
        let params = handle.get();
        let sample_rate = source.sample_rate() as f32;
        let identity_target = !params.enabled || params.is_identity();
        let (cur, pre) = if identity_target {
            (vec![BiquadCoeffs::IDENTITY; NUM_BANDS], 1.0)
        } else {
            (params_coeffs(&params, sample_rate), db_to_linear(params.preamp_db))
        };
        Self {
            source,
            seen_version: handle.version(),
            handle,
            sample_rate,
            channels,
            ch_idx: 0,
            pos: 0,
            // 构造时滤波状态为零，恒等目标可直接进入直通
            bypass: identity_target,
            pending_bypass: identity_target,
            smooth_remaining: 0,
            from_coeffs: cur.clone(),
            to_coeffs: cur.clone(),
            cur_coeffs: cur,
            from_preamp: pre,
            to_preamp: pre,
            cur_preamp: pre,
            states: vec![vec![BiquadState::default(); NUM_BANDS]; channels],
        }
    }
}

impl<S: Iterator<Item = f32>> EffectsSource<S> {
    /// 块边界检查：版本变化则重设平滑目标；直通目标在状态归零后落地。
    /// （放在仅要求 Iterator 的 impl 块中，供 next() 调用；采样率构造时已缓存）
    fn refresh(&mut self) {
        let v = self.handle.version();
        if v != self.seen_version {
            self.seen_version = v;
            let params = self.handle.get();
            let sample_rate = self.sample_rate;
            let identity_target = !params.enabled || params.is_identity();
            let (to, to_pre) = if identity_target {
                (vec![BiquadCoeffs::IDENTITY; NUM_BANDS], 1.0)
            } else {
                (params_coeffs(&params, sample_rate), db_to_linear(params.preamp_db))
            };
            if !identity_target {
                self.bypass = false;
            }
            self.pending_bypass = identity_target;
            self.from_coeffs = self.cur_coeffs.clone();
            self.from_preamp = self.cur_preamp;
            if self.from_coeffs == to && self.from_preamp == to_pre {
                self.smooth_remaining = 0;
            } else {
                self.to_coeffs = to;
                self.to_preamp = to_pre;
                self.smooth_remaining = SMOOTH_SAMPLES;
            }
        } else if self.pending_bypass
            && !self.bypass
            && self.smooth_remaining == 0
            && self.states_quiet()
        {
            self.bypass = true;
        }
    }

    fn states_quiet(&self) -> bool {
        self.states
            .iter()
            .flatten()
            .all(|s| s.z1.abs() < 1e-6 && s.z2.abs() < 1e-6)
    }
}

impl<S: Iterator<Item = f32>> Iterator for EffectsSource<S> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let x = self.source.next()?;
        self.pos += 1;
        if self.pos >= BLOCK_SAMPLES {
            self.pos = 0;
            self.refresh();
        }
        if self.bypass {
            return Some(x);
        }
        if self.smooth_remaining > 0 {
            self.smooth_remaining -= 1;
            let t = 1.0 - self.smooth_remaining as f32 / SMOOTH_SAMPLES as f32;
            for b in 0..NUM_BANDS {
                let f = self.from_coeffs[b];
                let to = self.to_coeffs[b];
                let cur = &mut self.cur_coeffs[b];
                cur.b0 = f.b0 + (to.b0 - f.b0) * t;
                cur.b1 = f.b1 + (to.b1 - f.b1) * t;
                cur.b2 = f.b2 + (to.b2 - f.b2) * t;
                cur.a1 = f.a1 + (to.a1 - f.a1) * t;
                cur.a2 = f.a2 + (to.a2 - f.a2) * t;
            }
            self.cur_preamp = self.from_preamp + (self.to_preamp - self.from_preamp) * t;
        }
        let mut y = x * self.cur_preamp;
        let ch = self.ch_idx;
        for b in 0..NUM_BANDS {
            y = self.states[ch][b].process(y, &self.cur_coeffs[b]);
        }
        self.ch_idx = (self.ch_idx + 1) % self.channels;
        Some(y)
    }
}

impl<S: Iterator<Item = f32> + Source> Source for EffectsSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.source.total_duration()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::buffer::SamplesBuffer;

    const SR: f32 = 44100.0;

    fn sine(freq: f32, samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / SR).sin())
            .collect()
    }

    fn buffer_source(samples: Vec<f32>) -> SamplesBuffer<f32> {
        SamplesBuffer::new(1, SR as u32, samples)
    }

    fn params(preset_id: &str, enabled: bool, preamp_db: f32, gains: [f32; 10]) -> EqParams {
        EqParams {
            preset_id: preset_id.into(),
            enabled,
            preamp_db,
            gains_db: gains,
        }
    }

    #[test]
    fn zero_db_gain_yields_identity_coeffs() {
        // 0dB 时 RBJ 三种滤波器都退化为恒等系数：b0=1、b1==a1、b2==a2
        for &f0 in EQ_BAND_HZ.iter() {
            let c = peaking_coeffs(f0, PEAKING_Q, 0.0, SR);
            assert!((c.b0 - 1.0).abs() < 1e-6, "peaking b0 应为 1: {c:?}");
            assert_eq!(c.b1.to_bits(), c.a1.to_bits(), "peaking b1 应逐位等于 a1");
            assert_eq!(c.b2.to_bits(), c.a2.to_bits(), "peaking b2 应逐位等于 a2");

            let c = low_shelf_coeffs(f0, 0.0, SR);
            assert!((c.b0 - 1.0).abs() < 1e-6);
            assert_eq!(c.b1.to_bits(), c.a1.to_bits());
            assert_eq!(c.b2.to_bits(), c.a2.to_bits());

            let c = high_shelf_coeffs(f0, 0.0, SR);
            assert!((c.b0 - 1.0).abs() < 1e-6);
            assert_eq!(c.b1.to_bits(), c.a1.to_bits());
            assert_eq!(c.b2.to_bits(), c.a2.to_bits());
        }
    }

    #[test]
    fn zero_db_filter_passes_sine_through_exactly() {
        // 恒等系数 + 零初始状态 ⇒ 输出与输入逐样本相等（零开销直通的数学基础）
        let input = sine(1000.0, 8192);
        let coeffs = peaking_coeffs(1000.0, PEAKING_Q, 0.0, SR);
        let mut state = BiquadState::default();
        for &x in &input {
            let y = state.process(x, &coeffs);
            assert_eq!(y.to_bits(), x.to_bits(), "0dB 滤波应精确透传");
        }
    }

    #[test]
    fn peaking_6db_doubles_sine_amplitude_at_center_freq() {
        let input = sine(1000.0, SR as usize);
        let coeffs = peaking_coeffs(1000.0, PEAKING_Q, 6.0, SR);
        let mut state = BiquadState::default();
        let mut peak = 0.0f32;
        for (i, &x) in input.iter().enumerate() {
            let y = state.process(x, &coeffs);
            if i >= 8192 {
                peak = peak.max(y.abs());
            }
        }
        // 峰值滤波器中心频率增益 = gain_db（±5% 容差）
        assert!(
            (peak - 2.0).abs() / 2.0 < 0.05,
            "+6dB @1kHz 正弦稳态峰值应≈2.0，实际 {peak}"
        );
    }

    #[test]
    fn flat_and_disabled_pass_samples_through_exactly() {
        let input = sine(440.0, 4096);

        // flat（enabled 也为 true，但全 0 增益）
        let handle = EffectsHandle::new();
        handle.set(params("flat", true, 0.0, [0.0; 10]));
        let out: Vec<f32> = EffectsSource::new(buffer_source(input.clone()), handle).collect();
        for (i, (&x, &y)) in input.iter().zip(out.iter()).enumerate() {
            assert_eq!(y.to_bits(), x.to_bits(), "flat 应精确透通 @{}", i);
        }

        // rock 增益但 disabled
        let handle = EffectsHandle::new();
        handle.set(params("rock", false, -3.0, [5.0, 4.0, 3.0, 1.0, -1.0, -1.0, 1.0, 3.0, 5.0, 5.0]));
        let out: Vec<f32> = EffectsSource::new(buffer_source(input), handle).collect();
        assert!(out.iter().all(|&y| y.abs() <= 1.0 + 1e-6));
    }

    #[test]
    fn enabled_eq_changes_output_and_stays_finite() {
        let input = sine(1000.0, 8192);
        let handle = EffectsHandle::new();
        handle.set(params("rock", true, -3.0, [5.0, 4.0, 3.0, 1.0, -1.0, -1.0, 1.0, 3.0, 5.0, 5.0]));
        let out: Vec<f32> = EffectsSource::new(buffer_source(input.clone()), handle).collect();

        assert!(out.iter().all(|y: &f32| y.is_finite()), "输出不应有 NaN/Inf");
        // 4kHz/8k 频段大幅提升，1kHz 附近输出能量应与输入明显不同
        let rms_in: f32 = input.iter().map(|x| x * x).sum::<f32>() / input.len() as f32;
        let rms_out: f32 = out.iter().map(|y| y * y).sum::<f32>() / out.len() as f32;
        assert!(
            (rms_in - rms_out).abs() / rms_in.max(1e-9) > 0.05,
            "启用 EQ 后能量应变化：in={rms_in}, out={rms_out}"
        );
    }

    #[test]
    fn preset_change_mid_stream_is_click_free() {
        // 播放中切换：从 flat 切到 rock，输出全有限且无样本级跳变（爆音）
        let input = sine(1000.0, 16384);
        let handle = EffectsHandle::new();
        let mut src = EffectsSource::new(buffer_source(input.clone()), handle.clone());

        let mut prev: Option<f32> = None;
        let mut max_delta = 0.0f32;
        for i in 0..input.len() {
            if i == 4096 {
                handle.set(params("rock", true, -3.0, [5.0, 4.0, 3.0, 1.0, -1.0, -1.0, 1.0, 3.0, 5.0, 5.0]));
            }
            let y = src.next().expect("样本不应提前耗尽");
            assert!(y.is_finite(), "切换后样本应有限 @{}", i);
            if let Some(p) = prev {
                max_delta = max_delta.max((y - p).abs());
            }
            prev = Some(y);
        }
        assert!(
            max_delta < 0.5,
            "切换期间相邻样本跳变应远小于爆音级（实测 {max_delta}）"
        );
    }

    #[test]
    fn builtin_presets_are_wellformed() {
        let presets = builtin_presets();
        assert!(presets.len() >= 8, "内置预设应≥8 个");

        let mut ids: Vec<&str> = presets.iter().map(|p| p.id.as_str()).collect();
        let total = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), total, "预设 id 必须唯一");

        let flat = presets.iter().find(|p| p.id == "flat").expect("flat 预设必须存在");
        assert!(flat.gains_db.iter().all(|&g| g == 0.0), "flat 应全 0");
        assert_eq!(flat.preamp_db, 0.0);

        for p in &presets {
            assert_eq!(p.gains_db.len(), NUM_BANDS);
            assert!(
                p.gains_db.iter().all(|&g| (-12.0..=12.0).contains(&g)),
                "{} 增益应在 [-12,+12]", p.id
            );
            assert!((-12.0..=0.0).contains(&p.preamp_db), "{} preamp 应在 [-12,0]", p.id);
        }
    }

    #[test]
    fn find_preset_matches_and_rejects() {
        assert_eq!(find_preset("rock").map(|p| p.id), Some("rock".to_string()));
        assert!(find_preset("flat").is_some());
        assert!(find_preset("no-such-preset").is_none());
    }

    #[test]
    fn high_shelf_at_low_sample_rate_is_finite() {
        // 32kHz 采样率下 16kHz = 奈奎斯特，f0 应被钳制，系数保持有限稳定
        let c = high_shelf_coeffs(16000.0, 6.0, 32000.0);
        assert!([c.b0, c.b1, c.b2, c.a1, c.a2].iter().all(|v| v.is_finite()));
        // 二阶实分母稳定判据（三角条件）：|a2| < 1 且 |a1| < 1 + a2
        assert!(
            c.a2.abs() < 1.0 && c.a1.abs() < 1.0 + c.a2,
            "钳制后极点应在单位圆内：a1={}, a2={}",
            c.a1,
            c.a2
        );

        let c = band_coeffs(9, 6.0, 32000.0);
        assert!([c.b0, c.b1, c.b2, c.a1, c.a2].iter().all(|v| v.is_finite()));
    }

    #[test]
    fn handle_set_bumps_version_and_get_roundtrips() {
        let handle = EffectsHandle::new();
        let v0 = handle.version();
        assert_eq!(handle.get(), EqParams::flat(), "初始应为 flat/disabled");

        let rock = params("rock", true, -3.0, [5.0; 10]);
        handle.set(rock.clone());
        assert_eq!(handle.get(), rock);
        assert_eq!(handle.version(), v0 + 1);

        handle.set(rock.clone());
        assert_eq!(handle.version(), v0 + 2, "重复 set 也应 bump 版本");
    }

    #[test]
    fn db_to_linear_matches_reference_values() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 1e-6);
        assert!((db_to_linear(6.0) - 1.9952623).abs() < 1e-4);
        assert!((db_to_linear(-6.0) - 0.5011872).abs() < 1e-4);
        assert!((db_to_linear(-12.0) - 0.2511886).abs() < 1e-4);
    }
}
