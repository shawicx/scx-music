use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rodio::Source;
use rustfft::{num_complex::Complex, FftPlanner};
use tauri::ipc::Channel;

const FFT_SIZE: usize = 256;
const NUM_BINS: usize = 64;
const EMIT_INTERVAL_MS: u64 = 33;

pub type SampleBuffer = Arc<Mutex<Vec<f32>>>;

#[derive(Clone)]
pub struct AnalyzerHandle {
    pub buffer: SampleBuffer,
    running: Arc<AtomicBool>,
}

impl AnalyzerHandle {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(FFT_SIZE * 2))),
            running: Arc::new(AtomicBool::new(false)),
        }
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
    pub fn start(&self, channel: Channel<Vec<u8>>) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        let buffer = self.buffer.clone();
        let running = self.running.clone();

        thread::spawn(move || {
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(FFT_SIZE);
            let window: Vec<f32> = (0..FFT_SIZE)
                .map(|i| {
                    0.5 * (1.0
                        - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32).cos())
                })
                .collect();

            while running.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(EMIT_INTERVAL_MS));

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

                let bin_size = (FFT_SIZE / 2) / NUM_BINS;
                let mut bins = vec![0u8; NUM_BINS];

                for i in 0..NUM_BINS {
                    let mut sum = 0.0;
                    for j in 0..bin_size {
                        let idx = i * bin_size + j;
                        if idx < input.len() {
                            let mag =
                                (input[idx].re * input[idx].re + input[idx].im * input[idx].im)
                                    .sqrt();
                            sum += mag;
                        }
                    }
                    let avg = sum / bin_size as f32;
                    let scaled = (avg * 4.0).min(255.0) as u8;
                    bins[i] = scaled;
                }

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

impl<S> TeeSource<S> {
    pub fn new(source: S, analyzer: AnalyzerHandle) -> Self {
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
