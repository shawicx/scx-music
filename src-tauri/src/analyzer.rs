use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rodio::Source;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use tauri::ipc::Channel;

const FFT_SIZE: usize = 1024;
const NUM_BINS: usize = 64;
const EMIT_INTERVAL_MS: u64 = 16;
const F_MIN: f32 = 20.0;
const F_MAX: f32 = 16000.0;
/// dB 动态范围下限：-60dB 映射为 0.0，0dB（满幅）映射为 1.0
const DB_FLOOR: f32 = -60.0;

pub type SampleBuffer = Arc<Mutex<Vec<f32>>>;

/// 对数分箱：把 [F_MIN, min(F_MAX, 奈奎斯特)] 按几何级数划分为 num_bins 个频段。
/// 人耳对频率是对数感知，线性分箱会让低频占满可视宽度、高频挤在一角。
pub(crate) fn log_bin_edges(sample_rate: f32, num_bins: usize) -> Vec<(f32, f32)> {
    let f_max = F_MAX.min(sample_rate / 2.0);
    let ratio = (f_max / F_MIN).powf(1.0 / num_bins as f32);
    (0..num_bins)
        .map(|i| {
            let lo = F_MIN * ratio.powi(i as i32);
            let hi = if i + 1 == num_bins {
                f_max
            } else {
                F_MIN * ratio.powi((i + 1) as i32)
            };
            (lo, hi)
        })
        .collect()
}

/// FFT 幅值 → [0,1]。参考幅值为 Hann 窗满幅正弦的相干增益（FFT_SIZE/4），
/// -60dB 以下归零，0dB 以上削顶。
pub(crate) fn magnitude_to_normalized(mag: f32) -> f32 {
    let ref_mag = FFT_SIZE as f32 / 4.0;
    let db = 20.0 * (mag / ref_mag).log10();
    ((db - DB_FLOOR) / -DB_FLOOR).clamp(0.0, 1.0)
}

/// FFT 输出 → 64 个对数频段能量（0..1）。
/// 每个频段取范围内 FFT bin 的最大幅值（峰值聚合），稀疏单音不会被平均稀释。
pub(crate) fn spectrum_from_fft(input: &[Complex<f32>], sample_rate: f32) -> Vec<f32> {
    let edges = log_bin_edges(sample_rate, NUM_BINS);
    let usable = (FFT_SIZE / 2).min(input.len());
    let bin_hz = sample_rate / FFT_SIZE as f32;

    edges
        .iter()
        .map(|&(lo, hi)| {
            let first = (lo / bin_hz).ceil() as usize;
            let last = (hi / bin_hz).floor() as usize;
            if first >= usable || last < first {
                // 频段窄于单个 FFT bin：取中心频率最近的 bin
                let idx = ((lo + hi) * 0.5 / bin_hz) as usize;
                if idx < usable {
                    magnitude_to_normalized(input[idx].norm())
                } else {
                    0.0
                }
            } else {
                let mag = (first..=last.min(usable - 1))
                    .map(|i| input[i].norm())
                    .fold(0.0f32, f32::max);
                magnitude_to_normalized(mag)
            }
        })
        .collect()
}

#[derive(Clone)]
pub struct AnalyzerHandle {
    pub buffer: SampleBuffer,
    sample_rate: Arc<AtomicU32>,
    running: Arc<AtomicBool>,
}

impl AnalyzerHandle {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(FFT_SIZE * 2))),
            sample_rate: Arc::new(AtomicU32::new(44100)),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_sample_rate(&self, sr: u32) {
        self.sample_rate.store(sr, Ordering::Relaxed);
    }

    pub fn push_samples(&self, new_samples: &[f32]) {
        let mut buf = self.buffer.lock().unwrap();
        buf.extend_from_slice(new_samples);
        if buf.len() > FFT_SIZE * 2 {
            let drain_count = buf.len() - FFT_SIZE;
            buf.drain(0..drain_count);
        }
    }

    /// 启动 FFT 线程,通过 `channel` 点对点推送频谱数据。
    /// 生命周期:channel 随调用方(前端 webview)销毁而失效,
    /// `send` 失败时退出线程并置 running=false,无需前端手动 stop。
    pub fn start(&self, channel: Channel<Vec<f32>>) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        let buffer = self.buffer.clone();
        let running = self.running.clone();
        let sample_rate = self.sample_rate.clone();

        thread::spawn(move || {
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(FFT_SIZE);
            let window: Vec<f32> = (0..FFT_SIZE)
                .map(|i| {
                    0.5 * (1.0
                        - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32).cos())
                })
                .collect();

            let mut cached_rate = 0.0f32;
            let mut edges: Vec<(f32, f32)> = Vec::new();

            while running.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(EMIT_INTERVAL_MS));

                let sr = sample_rate.load(Ordering::Relaxed) as f32;
                if sr != cached_rate {
                    edges = log_bin_edges(sr, NUM_BINS);
                    cached_rate = sr;
                }

                let samples: Vec<f32> = {
                    let mut buf = buffer.lock().unwrap();
                    if buf.len() < FFT_SIZE {
                        continue;
                    }
                    let start = buf.len() - FFT_SIZE;
                    let s = buf[start..].to_vec();
                    buf.clear();
                    s
                };

                let mut input: Vec<Complex<f32>> = samples
                    .iter()
                    .zip(window.iter())
                    .map(|(&s, &w)| Complex::new(s * w, 0.0))
                    .collect();

                fft.process(&mut input);

                let bins = spectrum_from_fft(&input, sr);

                // 点对点推送:channel 销毁或前端断开时 send 返回 Err,退出线程
                if channel.send(bins).is_err() {
                    break;
                }
            }
            running.store(false, Ordering::SeqCst);
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

pub struct TeeSource<S> {
    source: S,
    analyzer: AnalyzerHandle,
    batch: Vec<f32>,
}

impl<S: Iterator<Item = f32> + Source> TeeSource<S> {
    pub fn new(source: S, analyzer: AnalyzerHandle) -> Self {
        // 对数分箱按真实采样率计算频率边界，创建时同步一次
        analyzer.set_sample_rate(source.sample_rate());
        Self {
            source,
            analyzer,
            batch: Vec::with_capacity(1024),
        }
    }
}

impl<S: Iterator<Item = f32>> Iterator for TeeSource<S> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.source.next()?;
        self.batch.push(sample);
        if self.batch.len() >= 1024 {
            self.analyzer.push_samples(&self.batch);
            self.batch.clear();
        }
        Some(sample)
    }
}

impl<S: Iterator<Item = f32> + Source> Source for TeeSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_bin_edges_covers_range_with_geometric_progression() {
        let edges = log_bin_edges(44100.0, NUM_BINS);
        assert_eq!(edges.len(), NUM_BINS);
        assert!((edges[0].0 - F_MIN).abs() < 0.01, "起始频率应为 20Hz");
        assert!(
            (edges[NUM_BINS - 1].1 - F_MAX).abs() < 0.5,
            "终止频率应为 16kHz"
        );

        // 单调递增、频段不重叠
        for w in edges.windows(2) {
            assert!(w[0].1 <= w[1].0 + f32::EPSILON, "频段不应重叠");
        }

        // 几何级数：相邻频段宽度比恒定
        let ratio_first = edges[0].1 / edges[0].0;
        let ratio_mid = edges[32].1 / edges[32].0;
        assert!(
            (ratio_first - ratio_mid).abs() / ratio_first < 0.01,
            "对数分箱应为等比数列"
        );
    }

    #[test]
    fn log_bin_edges_clamps_to_nyquist() {
        let edges = log_bin_edges(22050.0, NUM_BINS);
        let nyquist = 22050.0 / 2.0;
        assert!(
            (edges[NUM_BINS - 1].1 - nyquist).abs() < 0.5,
            "低采样率时终止频率应收敛到奈奎斯特"
        );
        assert!(edges.iter().all(|&(_, hi)| hi <= nyquist + 0.5));
    }

    #[test]
    fn magnitude_to_normalized_maps_db_range() {
        let ref_mag = FFT_SIZE as f32 / 4.0; // Hann 窗满幅正弦的相干增益

        assert_eq!(magnitude_to_normalized(0.0), 0.0, "静音应为 0");
        assert!(
            (magnitude_to_normalized(ref_mag) - 1.0).abs() < 1e-3,
            "满幅应为 1"
        );
        assert_eq!(magnitude_to_normalized(ref_mag * 1000.0), 1.0, "超满幅应削顶为 1");

        // -30dB（幅值 1/31.62）应落在 0.5 附近
        let minus_30db = magnitude_to_normalized(ref_mag / 31.623);
        assert!((minus_30db - 0.5).abs() < 0.01, "-30dB 应映射到 0.5");

        assert!(
            magnitude_to_normalized(ref_mag) >= magnitude_to_normalized(ref_mag / 10.0),
            "映射应单调不减"
        );
    }

    #[test]
    fn spectrum_from_fft_peaks_into_containing_bin() {
        let sr = 44100.0;
        let bin_hz = sr / FFT_SIZE as f32; // ≈43Hz
        let mut input = vec![Complex::new(0.0, 0.0); FFT_SIZE];
        let ref_mag = FFT_SIZE as f32 / 4.0;
        input[100] = Complex::new(ref_mag, 0.0); // ≈4306Hz 单音

        let out = spectrum_from_fft(&input, sr);
        assert_eq!(out.len(), NUM_BINS);

        let (idx, &peak) = out
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap();
        let freq = 100.0 * bin_hz;
        let edges = log_bin_edges(sr, NUM_BINS);
        let (lo, hi) = edges[idx];
        assert!(
            freq >= lo - bin_hz && freq <= hi + bin_hz,
            "峰值所在频段 [{lo}, {hi}] 应包含单音频率 {freq}"
        );
        assert!(peak > 0.9, "单音能量应接近 1，实际 {peak}");

        assert!(out[0] < 0.01, "低频远端应安静");
    }
}
