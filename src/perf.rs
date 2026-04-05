//! Performance metrics and in-app debug logging.
//!
//! Two systems:
//! 1. File logging: writes to `~/.chesstui/perf.log` when `CHESSTUI_PERF=1`
//! 2. In-app ring buffer: always active, feeds the debug panel (toggled with F3)

use std::collections::VecDeque;
use std::io::Write;
use std::sync::Mutex;
use std::time::Instant;

// ── File logging (opt-in via env var) ───────────────────────────────────

static FILE_LOG: Mutex<Option<std::fs::File>> = Mutex::new(None);
static FILE_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Call once at startup.
pub fn init() {
    init_ring();
    if std::env::var("CHESSTUI_PERF").unwrap_or_default() == "1" {
        FILE_ENABLED.store(true, std::sync::atomic::Ordering::Relaxed);
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let dir = std::path::Path::new(&home).join(".chesstui");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("perf.log");
        if let Ok(f) = std::fs::File::create(path) {
            *FILE_LOG.lock().unwrap() = Some(f);
        }
    }
}

pub fn file_enabled() -> bool {
    FILE_ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}

fn file_log(msg: &str) {
    if !file_enabled() { return; }
    if let Ok(mut guard) = FILE_LOG.lock() {
        if let Some(f) = guard.as_mut() {
            let _ = writeln!(f, "{}", msg);
        }
    }
}

// ── In-app debug ring buffer (always active) ───────────────────────────

const RING_CAPACITY: usize = 200;

static RING: Mutex<Option<VecDeque<String>>> = Mutex::new(None);

/// Initialize the ring buffer. Called from init().
fn init_ring() {
    *RING.lock().unwrap() = Some(VecDeque::with_capacity(RING_CAPACITY));
}

/// Push a message to the ring buffer.
fn ring_push(msg: String) {
    if let Ok(mut guard) = RING.lock() {
        if let Some(ring) = guard.as_mut() {
            if ring.len() >= RING_CAPACITY {
                ring.pop_front();
            }
            ring.push_back(msg);
        }
    }
}

/// Drain the ring buffer into a Vec for display. Returns recent entries.
pub fn drain_ring(max: usize) -> Vec<String> {
    if let Ok(guard) = RING.lock() {
        if let Some(ring) = guard.as_ref() {
            let skip = ring.len().saturating_sub(max);
            return ring.iter().skip(skip).cloned().collect();
        }
    }
    Vec::new()
}

// ── Combined logging ────────────────────────────────────────────────────

/// Log a timing measurement to both file and ring buffer.
pub fn log(section: &str, micros: u128) {
    let msg = format!("{:<28} {:>8}us", section, micros);
    ring_push(msg.clone());
    file_log(&msg);
}

/// Log a text message (non-timing) to the ring buffer.
pub fn log_msg(msg: &str) {
    ring_push(msg.to_string());
    file_log(msg);
}

/// Frame boundary separator.
pub fn frame_boundary(frame_micros: u128) {
    let msg = format!("── frame {:>8}us ──", frame_micros);
    ring_push(msg.clone());
    file_log(&msg);
}

pub fn enabled() -> bool {
    // Ring buffer is always active; this is for the perf_timer macro
    true
}

// ── RAII Timer ──────────────────────────────────────────────────────────

pub struct Timer {
    section: &'static str,
    start: Instant,
}

impl Timer {
    pub fn new(section: &'static str) -> Self {
        Self { section, start: Instant::now() }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_micros();
        log(self.section, elapsed);
    }
}

#[macro_export]
macro_rules! perf_timer {
    ($name:expr) => {
        let _timer = $crate::perf::Timer::new($name);
    };
}

// ── Frame stats (lightweight, no allocation) ────────────────────────────

use std::sync::atomic::{AtomicU64, Ordering};

static LAST_FRAME_US: AtomicU64 = AtomicU64::new(0);
static LAST_INPUT_LAG_US: AtomicU64 = AtomicU64::new(0);
static FRAME_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn set_frame_time(us: u64) {
    LAST_FRAME_US.store(us, Ordering::Relaxed);
    FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn set_input_lag(us: u64) {
    LAST_INPUT_LAG_US.store(us, Ordering::Relaxed);
}

pub fn frame_time_us() -> u64 {
    LAST_FRAME_US.load(Ordering::Relaxed)
}

pub fn input_lag_us() -> u64 {
    LAST_INPUT_LAG_US.load(Ordering::Relaxed)
}

pub fn frame_count() -> u64 {
    FRAME_COUNT.load(Ordering::Relaxed)
}
