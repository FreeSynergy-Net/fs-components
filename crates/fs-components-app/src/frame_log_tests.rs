//! Tests of the frame log.
//!
//! Pattern: none (tests).

use super::*;
use fs_components_primitives::frame_probe;
use std::sync::MutexGuard;

/// The probe counters are process-wide: tests that write a row or touch them run one after the other.
fn serial() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    frame_probe::switch_on();
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn temp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("fs-frame-log-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn one_frame() {
    tracing::trace!(
        target: "blinc_app::frame_timing",
        total_us = 12_345_u64,
        p1_rebuild_check_us = 10_u64,
        p2_rebuild_us = 9_000_u64,
        p3_tick_us = 20_u64,
        p4_render_us = 3_000_u64,
        p5_chain_us = 5_u64,
        did_rebuild = true,
        dirty_springs = 0_usize,
        "frame"
    );
}

#[test]
fn a_set_variable_creates_the_file_with_a_header_and_a_row_per_frame() {
    let _serial = serial();
    let path = temp("on.csv");
    let _ = std::fs::remove_file(&path);
    let log = FrameLog::from_value(Some(path.clone().into_os_string()))
        .unwrap()
        .expect("log is on");
    tracing::subscriber::with_default(log, one_frame);
    let text = std::fs::read_to_string(&path).unwrap();
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some(HEADER));
    let row: Vec<&str> = lines.next().expect("one row").split(',').collect();
    assert_eq!(row.len(), HEADER.split(',').count());
    assert_eq!(row[2], "12.345", "total");
    assert_eq!(row[4], "9.000", "rebuild");
    assert_eq!(row[6], "3.000", "render");
    assert!(lines.next().is_none());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn no_variable_or_an_empty_one_creates_nothing() {
    let _serial = serial();
    assert!(FrameLog::from_value(None).unwrap().is_none());
    assert!(
        FrameLog::from_value(Some(OsString::new()))
            .unwrap()
            .is_none()
    );
}

#[test]
fn an_existing_file_is_never_overwritten() {
    let _serial = serial();
    let path = temp("foreign.csv");
    std::fs::write(&path, "mine").unwrap();
    let err = FrameLog::create(&path).err().expect("refused");
    assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "mine");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn the_first_frame_after_a_resize_is_told_as_resize() {
    let _serial = serial();
    let path = temp("resize.csv");
    let _ = std::fs::remove_file(&path);
    let log = FrameLog::create(&path).unwrap();
    tracing::subscriber::with_default(log, || {
        note_size(100.0, 100.0);
        one_frame();
        one_frame();
        note_size(120.0, 100.0);
        one_frame();
        tracing::trace!(target: "blinc_app::frame_timing", source = "window_resized", "invalidate_render_cache");
        one_frame();
    });
    let text = std::fs::read_to_string(&path).unwrap();
    let kinds: Vec<&str> = text
        .lines()
        .skip(1)
        .map(|l| l.split(',').nth(1).unwrap())
        .collect();
    assert_eq!(kinds, ["resize", "rebuild", "resize", "resize"]);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn other_targets_are_not_listened_to() {
    let _serial = serial();
    let path = temp("target.csv");
    let _ = std::fs::remove_file(&path);
    let log = FrameLog::create(&path).unwrap();
    tracing::subscriber::with_default(log, || {
        tracing::trace!(target: "somewhere_else", total_us = 1_u64, "frame");
    });
    assert_eq!(std::fs::read_to_string(&path).unwrap().lines().count(), 1);
    let _ = std::fs::remove_file(&path);
}

/// One frame with `rebuild_us` of rebuild, written to a fresh file; returns its single row as fields.
fn row_with(name: &str, rebuild_us: u64, report: impl FnOnce()) -> Vec<String> {
    let path = temp(name);
    let _ = std::fs::remove_file(&path);
    let log = FrameLog::create(&path).unwrap();
    let _ = frame_probe::take();
    report();
    tracing::subscriber::with_default(log, || {
        tracing::trace!(
            target: "blinc_app::frame_timing",
            total_us = 20_000_u64,
            p2_rebuild_us = rebuild_us,
            did_rebuild = true,
            "frame"
        );
    });
    let text = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    text.lines()
        .nth(1)
        .expect("one row")
        .split(',')
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_header_keeps_the_ten_old_columns_first_and_appends_five() {
    let _serial = serial();
    let cols: Vec<&str> = HEADER.split(',').collect();
    assert_eq!(
        &cols[..10],
        [
            "t_ms",
            "kind",
            "total_ms",
            "check_ms",
            "rebuild_ms",
            "tick_ms",
            "render_ms",
            "chain_ms",
            "width",
            "height"
        ]
    );
    assert_eq!(
        &cols[10..],
        ["build_ms", "blinc_ms", "cards_ms", "cards_n", "stateful_n"]
    );
}

#[test]
fn build_ms_is_the_time_of_our_build_closure() {
    let _serial = serial();
    let row = row_with("build.csv", 9_000, || {
        frame_probe::time_build(|| std::thread::sleep(std::time::Duration::from_millis(20)));
    });
    let build: f64 = row[10].parse().unwrap();
    assert!((20.0..200.0).contains(&build), "build_ms {build}");
}

#[test]
fn blinc_ms_is_rebuild_minus_build_and_never_negative() {
    let _serial = serial();
    // Rebuild 30 ms, our build at least 20 ms: Blinc's share is at most 10 ms, and clearly smaller than rebuild.
    let row = row_with("blinc.csv", 30_000, || {
        frame_probe::time_build(|| std::thread::sleep(std::time::Duration::from_millis(20)));
    });
    let (build, blinc): (f64, f64) = (row[10].parse().unwrap(), row[11].parse().unwrap());
    assert!((blinc - (30.0 - build)).abs() < 0.01, "{blinc} vs {build}");
    // Our build longer than the whole rebuild (a clock difference): zero, not wrapped around.
    let row = row_with("blinc0.csv", 1_000, || {
        frame_probe::time_build(|| std::thread::sleep(std::time::Duration::from_millis(20)));
    });
    assert_eq!(row[11], "0.000");
    // No build reported: the whole rebuild is Blinc's.
    let row = row_with("blinc1.csv", 7_000, || {});
    assert_eq!((row[10].as_str(), row[11].as_str()), ("0.000", "7.000"));
}

#[test]
fn cards_ms_and_cards_n_tell_the_cards_created() {
    let _serial = serial();
    let row = row_with("cards.csv", 9_000, || {
        let made = frame_probe::time_cards(|| {
            std::thread::sleep(std::time::Duration::from_millis(15));
            vec![1, 2, 3]
        });
        assert_eq!(made.len(), 3);
    });
    let ms: f64 = row[12].parse().unwrap();
    assert!((15.0..200.0).contains(&ms), "cards_ms {ms}");
    assert_eq!(row[13], "3", "cards_n");
    assert_eq!(row[14], "0", "stateful_n stays apart");
}

#[test]
fn stateful_n_counts_the_stateful_widgets_of_the_frame_and_starts_again_after_a_row() {
    let _serial = serial();
    let row = row_with("stateful.csv", 9_000, || {
        for _ in 0..7 {
            frame_probe::count_stateful();
        }
    });
    assert_eq!(row[14], "7");
    assert_eq!((row[12].as_str(), row[13].as_str()), ("0.000", "0"));
    // The row emptied the counters: the next frame starts at zero.
    let row = row_with("stateful2.csv", 9_000, || {});
    assert_eq!(row[14], "0");
}
