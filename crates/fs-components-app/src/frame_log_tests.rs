//! Tests of the frame log.
//!
//! Pattern: none (tests).

use super::*;

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
    assert!(FrameLog::from_value(None).unwrap().is_none());
    assert!(
        FrameLog::from_value(Some(OsString::new()))
            .unwrap()
            .is_none()
    );
}

#[test]
fn an_existing_file_is_never_overwritten() {
    let path = temp("foreign.csv");
    std::fs::write(&path, "mine").unwrap();
    let err = FrameLog::create(&path).err().expect("refused");
    assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "mine");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn the_first_frame_after_a_resize_is_told_as_resize() {
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
    let path = temp("target.csv");
    let _ = std::fs::remove_file(&path);
    let log = FrameLog::create(&path).unwrap();
    tracing::subscriber::with_default(log, || {
        tracing::trace!(target: "somewhere_else", total_us = 1_u64, "frame");
    });
    assert_eq!(std::fs::read_to_string(&path).unwrap().lines().count(), 1);
    let _ = std::fs::remove_file(&path);
}
