//! A measuring hook for the window loop: `FS_FRAME_LOG=<path>` writes one CSV row per frame.
//!
//! Pattern: Observer — Blinc reports the time of each frame as a `tracing` event (target
//! `blinc_app::frame_timing`); this subscriber listens to that target only and writes the numbers down.
//!
//! Without the variable nothing is installed: one environment check at start, no subscriber, no file, no
//! cost per frame (Blinc's own `trace!` stays disabled). The path comes from the variable and nowhere else.
//! The file is created new (never an existing file, never through a link: `create_new`), mode 0600 on Unix.
//! A file that cannot be created is told on standard error once and the program runs on without a log.
//!
//! Columns: `t_ms,kind,total_ms,check_ms,rebuild_ms,tick_ms,render_ms,chain_ms,width,height`.
//! `t_ms` is the time since the log was opened; `kind` is `resize` for the first frame after the window
//! size changed, `rebuild` for another frame that rebuilt the tree, else `redraw`. The five phases are
//! Blinc's own: `check` (does the tree need a rebuild), `rebuild` (our build closure, tree, style,
//! layout), `tick` (animations), `render` (paint on the GPU), `chain` (what follows the paint). Blinc
//! does not time layout or the swapchain present on their own; `total_ms` is the whole frame.
//! `width`/`height` are the window size in logical pixels the build closure was last called with.

use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

/// The variable that switches the log on.
pub const VARIABLE: &str = "FS_FRAME_LOG";

/// The tracing target of Blinc's frame times.
const TARGET: &str = "blinc_app::frame_timing";

/// The first line of the file.
pub const HEADER: &str =
    "t_ms,kind,total_ms,check_ms,rebuild_ms,tick_ms,render_ms,chain_ms,width,height";

/// Window size last seen by the build closure, `width << 32 | height` (whole logical pixels).
static SIZE: AtomicU64 = AtomicU64::new(0);

/// Remember the window size the build closure is called with. Called only when the log is on.
pub fn note_size(width: f32, height: f32) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let packed = (u64::from(width.max(0.0) as u32) << 32) | u64::from(height.max(0.0) as u32);
    SIZE.store(packed, Ordering::Relaxed);
}

/// The subscriber: writes a row for each Blinc frame event.
pub struct FrameLog {
    file: Mutex<File>,
    start: Instant,
    /// Size of the last row, to tell the first frame after a size change.
    last_size: AtomicU64,
    /// Blinc dropped its render caches because of a window resize since the last row.
    resized: AtomicBool,
}

impl FrameLog {
    /// Create the file (new, mode 0600) and write the header.
    ///
    /// # Errors
    /// The file exists already, or cannot be created or written.
    pub fn create(path: &Path) -> io::Result<Self> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let mut file = options.open(path)?;
        writeln!(file, "{HEADER}")?;
        Ok(Self {
            file: Mutex::new(file),
            start: Instant::now(),
            last_size: AtomicU64::new(SIZE.load(Ordering::Relaxed)),
            resized: AtomicBool::new(false),
        })
    }

    /// The log the environment asks for: `None` when the variable is not set or empty (nothing is
    /// created), else the new file.
    ///
    /// # Errors
    /// As [`FrameLog::create`].
    pub fn from_value(value: Option<OsString>) -> io::Result<Option<Self>> {
        match value {
            Some(path) if !path.is_empty() => Self::create(Path::new(&path)).map(Some),
            _ => Ok(None),
        }
    }

    fn row(&self, frame: &Frame) {
        let size = SIZE.load(Ordering::Relaxed);
        let size_changed = self.last_size.swap(size, Ordering::Relaxed) != size;
        let kind = if self.resized.swap(false, Ordering::Relaxed) || size_changed {
            "resize"
        } else if frame.rebuilt {
            "rebuild"
        } else {
            "redraw"
        };
        let line = format_row(
            self.start.elapsed().as_micros(),
            kind,
            frame,
            (size >> 32, size & 0xFFFF_FFFF),
        );
        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

/// Install the log if the environment asks for it. Returns whether it is on. Call once, before the window.
pub fn install_from_env() -> bool {
    match FrameLog::from_value(std::env::var_os(VARIABLE)) {
        Ok(None) => false,
        Ok(Some(log)) => tracing::subscriber::set_global_default(log).is_ok(),
        Err(e) => {
            eprintln!("{VARIABLE}: no frame log written: {e}");
            false
        }
    }
}

/// One frame as Blinc reports it (microseconds).
#[derive(Default)]
struct Frame {
    total: u64,
    check: u64,
    rebuild: u64,
    tick: u64,
    render: u64,
    chain: u64,
    rebuilt: bool,
}

fn format_row(t_us: u128, kind: &str, frame: &Frame, size: (u64, u64)) -> String {
    let ms = |us: u64| format!("{}.{:03}", us / 1000, us % 1000);
    let mut line = String::with_capacity(96);
    let _ = writeln!(
        line,
        "{}.{:03},{kind},{},{},{},{},{},{},{},{}",
        t_us / 1000,
        t_us % 1000,
        ms(frame.total),
        ms(frame.check),
        ms(frame.rebuild),
        ms(frame.tick),
        ms(frame.render),
        ms(frame.chain),
        size.0,
        size.1,
    );
    line
}

/// Reads the fields of one event.
#[derive(Default)]
struct Reader {
    frame: Frame,
    message: String,
    source: String,
}

impl Visit for Reader {
    fn record_u64(&mut self, field: &Field, value: u64) {
        match field.name() {
            "total_us" => self.frame.total = value,
            "p1_rebuild_check_us" => self.frame.check = value,
            "p2_rebuild_us" => self.frame.rebuild = value,
            "p3_tick_us" => self.frame.tick = value,
            "p4_render_us" => self.frame.render = value,
            "p5_chain_us" => self.frame.chain = value,
            _ => {}
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == "did_rebuild" {
            self.frame.rebuilt = value;
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "source" {
            value.clone_into(&mut self.source);
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }
}

impl Subscriber for FrameLog {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.target() == TARGET
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut reader = Reader::default();
        event.record(&mut reader);
        match reader.message.as_str() {
            "frame" => self.row(&reader.frame),
            "invalidate_render_cache" if reader.source == "window_resized" => {
                self.resized.store(true, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}

#[cfg(test)]
#[path = "frame_log_tests.rs"]
mod tests;
