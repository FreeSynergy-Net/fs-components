//! The shell in a window.
//!
//! Pattern: Facade over `WindowedApp`.

use blinc_app::prelude::WindowConfig;
use blinc_app::windowed::{WindowedApp, open_window_with};
use blinc_layout::prelude::div;
use fs_components_render::{bundle, register_fonts};

use crate::error::AppError;
use crate::styles::MIN_WINDOW;
use crate::widget::AppWidget;

/// Window settings. The title comes from the [`AppWidget`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowOptions {
    /// Initial width in logical pixels.
    pub width: u32,
    /// Initial height in logical pixels.
    pub height: u32,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
        }
    }
}

/// Keep Blinc's motion promotion on, as it was in `fs-blinc-3.1`.
///
/// Blinc `fs-blinc-3.2` (upstream cd3e196e) turned the motion promotion OFF by default (4e366fe1, variable
/// `BLINC_MOTION_PROMOTION`). With it off the block menu of the text editor is drawn under the text of the area (b4: "beta two"
/// shows through the menu; test `editor_bugs_pictures::b4`). The windowed runner has no setting for it but the variable, so a
/// program sets it when the person has not. Until Blinc has a `WindowConfig` field for it (a request is filed), every program
/// that opens a window calls this as the FIRST line of `main`. [`launch`] does not call it (and sets nothing): without the call the
/// window works, but the block menu of the editor is drawn under the text.
///
/// A variable the person set (`BLINC_MOTION_PROMOTION=0` or `1`) is left alone.
///
/// ```compile_fail
/// // Needs `unsafe`: it changes the environment of the whole program.
/// fs_components_app::enable_motion_promotion();
/// ```
///
/// # Safety
/// Changing the environment is unsound while another thread reads or writes it (glibc). Call this as the first line of `main`,
/// before this program (or a library it uses) has started any thread, and not later.
pub unsafe fn enable_motion_promotion() {
    if std::env::var_os(MOTION_PROMOTION).is_none() {
        // SAFETY: the caller promises that no other thread exists yet (see `# Safety`).
        unsafe { std::env::set_var(MOTION_PROMOTION, "1") };
    }
}

/// The variable Blinc reads for the motion promotion.
pub const MOTION_PROMOTION: &str = "BLINC_MOTION_PROMOTION";

/// Open a window with this shell and run until it is closed.
///
/// Sets nothing in the environment. A program calls [`enable_motion_promotion`] as the first line of `main` (before any thread);
/// without it the block menu of the text editor is drawn under the text (Blinc `fs-blinc-3.2`).
///
/// # Errors
/// [`AppError::Window`] if the window or the graphics cannot be started.
pub fn launch(options: WindowOptions, app: AppWidget) -> Result<(), AppError> {
    register_fonts();
    // `FS_FRAME_LOG=<path>`: one CSV row per frame (see `frame_log`); not set: nothing happens.
    let frame_log = crate::frame_log::install_from_env();
    #[cfg(feature = "test-access")]
    fs_components_widgets::test_access::start_from_env();
    let config = WindowConfig {
        title: app.with_shell(|shell| shell.title().to_owned()),
        width: options.width,
        height: options.height,
        min_size: Some(MIN_WINDOW),
        ..Default::default()
    };
    let css = app.styles();
    WindowedApp::run_with_theme(config, bundle(&css), color_scheme(), move |ctx| {
        // The window's own way to close: the event loop ends, the program's tidying up (drops) runs.
        let close = ctx.close_callback();
        app.set_quit(move || close());
        if frame_log {
            crate::frame_log::note_size(ctx.width, ctx.height);
        }
        app.build(ctx.width, ctx.height)
    })
    .map_err(|e| AppError::Window(e.to_string()))
}

/// Pattern: RAII guard – a value whose `Drop` tells that a window is gone, so the one who opened it learns of the close.
/// Abweichung: it is not tied to the window by an id (Blinc gives none back) but by being owned by the window's builder, which
/// Blinc drops when it removes the window (`secondary_windows.remove` on `CloseRequested`).
struct CloseWatch(Option<Box<dyn FnOnce() + Send>>);

impl CloseWatch {
    fn new(on_closed: impl FnOnce() + Send + 'static) -> Self {
        Self(Some(Box::new(on_closed)))
    }
}

impl Drop for CloseWatch {
    fn drop(&mut self) {
        if let Some(on_closed) = self.0.take() {
            on_closed();
        }
    }
}

/// Pattern: RAII guard – what the builder of a further window holds; `held` drops first, then the watch tells it is done.
/// Abweichung: the field order is the contract (Rust drops fields in order), so `on_closed` never runs while `held` is alive.
struct Owned<T> {
    held: T,
    _watch: CloseWatch,
}

/// Open a further window in the running program (Blinc `open_window_with`), in addition to the one [`launch`] opened.
///
/// Call it after [`launch`] has started the event loop (from a command or an event handler of the program), never before: Blinc
/// then has no loop to queue the window on, logs a warning, and nothing opens (`on_closed` does not run).
/// The window shows `app` with the same styles as [`launch`]. When the person closes the window (or `app.quit()` is called,
/// which `open_more` wires to this window when it is first built), Blinc drops the builder: first `app` and with it everything
/// it holds, then `on_closed` runs, once, on the thread of the event loop. Keep `on_closed` short.
///
/// Blinc gives back no window id, so the window cannot be found again from outside; the program keeps its own handle (the
/// `app` it passed, or what `on_closed` clears).
///
/// # Errors
/// None today; the result is kept so a later Blinc, which can refuse a window, does not change the signature.
#[allow(clippy::unnecessary_wraps)]
pub fn open_more(
    options: WindowOptions,
    app: AppWidget,
    on_closed: impl FnOnce() + Send + 'static,
) -> Result<(), AppError> {
    let config = WindowConfig {
        title: app.with_shell(|shell| shell.title().to_owned()),
        width: options.width,
        height: options.height,
        min_size: Some(MIN_WINDOW),
        ..Default::default()
    };
    let owned = Owned {
        held: app,
        _watch: CloseWatch::new(on_closed),
    };
    let mut wired = false;
    open_window_with(config, move |ctx| {
        // Use `owned` as a whole (closures capture single fields): else the watch would drop when `open_more` returns.
        let owned = &owned;
        let app = &owned.held;
        if !wired {
            // The window's own way to close, taken at the first build (as `launch` does).
            let close = ctx.close_callback();
            app.set_quit(move || close());
            wired = true;
        }
        div()
            .w(ctx.width)
            .h(ctx.height)
            .child(app.build(ctx.width, ctx.height))
    });
    Ok(())
}

/// The colour scheme the window starts with, named explicitly instead of left to Blinc's detection:
/// `FS_COLOR_SCHEME=light|dark` wins; else, on a KDE desktop, a KDE colour scheme whose name says
/// "Dark" gives dark; everything else is what the system reports. (A KDE scheme without "dark" in its
/// name says nothing: it is left to the system, so a leftover `kdeglobals` cannot turn a dark desktop
/// light.)
fn color_scheme() -> blinc_theme::ColorScheme {
    let kde_desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .is_ok_and(|d| d.to_ascii_uppercase().split(':').any(|part| part == "KDE"));
    scheme_from(
        std::env::var("FS_COLOR_SCHEME").ok().as_deref(),
        kde_desktop.then(kde_scheme_name).flatten().as_deref(),
    )
    .unwrap_or_else(blinc_theme::detect_system_color_scheme)
}

fn scheme_from(env: Option<&str>, kde: Option<&str>) -> Option<blinc_theme::ColorScheme> {
    match env.map(str::to_ascii_lowercase).as_deref() {
        Some("dark") => return Some(blinc_theme::ColorScheme::Dark),
        Some("light") => return Some(blinc_theme::ColorScheme::Light),
        _ => {}
    }
    kde.is_some_and(|name| name.to_ascii_lowercase().contains("dark"))
        .then_some(blinc_theme::ColorScheme::Dark)
}

/// An absolute directory from the environment; an empty or relative value is ignored (XDG).
fn absolute_dir(value: Option<std::ffi::OsString>) -> Option<std::path::PathBuf> {
    value
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_absolute())
}

/// `ColorScheme=` of `[General]` in `kdeglobals`, if there is a KDE configuration and the name is not empty.
fn kde_scheme_name() -> Option<String> {
    let home = absolute_dir(std::env::var_os("XDG_CONFIG_HOME"))
        .or_else(|| absolute_dir(std::env::var_os("HOME")).map(|h| h.join(".config")))?;
    let text = std::fs::read_to_string(home.join("kdeglobals")).ok()?;
    general_color_scheme(&text)
}

fn general_color_scheme(text: &str) -> Option<String> {
    let mut in_general = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_general = line == "[General]";
        } else if in_general && let Some((key, value)) = line.split_once('=') {
            let value = value.trim();
            if key.trim() == "ColorScheme" && !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use blinc_theme::ColorScheme::{Dark, Light};

    #[test]
    fn the_environment_wins_a_dark_kde_name_gives_dark_everything_else_is_the_systems() {
        assert_eq!(scheme_from(Some("DARK"), Some("BreezeLight")), Some(Dark));
        assert_eq!(scheme_from(Some("light"), Some("BreezeDark")), Some(Light));
        assert_eq!(scheme_from(None, Some("BreezeDark")), Some(Dark));
        assert_eq!(
            scheme_from(None, Some("BreezeLight")),
            None,
            "left to the system"
        );
        assert_eq!(scheme_from(Some("auto"), None), None);
        assert_eq!(scheme_from(None, None), None);
    }

    #[test]
    fn kdeglobals_is_read_where_it_says_so() {
        let text = "[Other]\r\nColorScheme=Wrong\r\n[General]\r\nColorScheme = BreezeDark\r\n";
        assert_eq!(general_color_scheme(text).as_deref(), Some("BreezeDark"));
        assert_eq!(
            general_color_scheme("[General]\nColorScheme=\n"),
            None,
            "empty is nothing"
        );
        assert_eq!(general_color_scheme("[Other]\nColorScheme=X\n"), None);
    }

    #[test]
    fn an_empty_or_relative_directory_is_ignored() {
        use std::ffi::OsString;
        assert_eq!(absolute_dir(Some(OsString::new())), None);
        assert_eq!(absolute_dir(Some(OsString::from("rel/dir"))), None);
        assert!(absolute_dir(Some(OsString::from("/abs"))).is_some());
        assert_eq!(absolute_dir(None), None);
    }
}

#[cfg(test)]
mod close_tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// What a builder of Blinc is: a boxed `FnMut + Send`, dropped by `secondary_windows.remove`.
    type Builder = Box<dyn FnMut() + Send>;

    struct Note(&'static str, Arc<Mutex<Vec<&'static str>>>);
    impl Drop for Note {
        fn drop(&mut self) {
            self.1.lock().unwrap().push(self.0);
        }
    }

    fn builder(log: &Arc<Mutex<Vec<&'static str>>>) -> Builder {
        let owned = Owned {
            held: Note("held", Arc::clone(log)),
            _watch: CloseWatch::new({
                let log = Arc::clone(log);
                move || log.lock().unwrap().push("closed")
            }),
        };
        // The closure captures `owned` as the real builder does; Blinc calls it each frame.
        Box::new(move || {
            let owned = &owned;
            let _ = &owned.held;
        })
    }

    #[test]
    fn dropping_the_builder_drops_what_the_window_holds_and_then_tells_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut windows: std::collections::HashMap<u32, Builder> = std::collections::HashMap::new();
        windows.insert(7, builder(&log));
        windows.get_mut(&7).unwrap()(); // a frame
        assert!(log.lock().unwrap().is_empty(), "open: nothing told");
        // `CloseRequested`: `secondary_windows.remove(&wid)`
        windows.remove(&7);
        assert_eq!(
            *log.lock().unwrap(),
            ["held", "closed"],
            "held first, then the notice, once"
        );
    }

    #[test]
    fn a_window_that_is_still_open_tells_nothing_and_another_close_is_separate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut windows: Vec<Builder> = vec![builder(&log), builder(&log)];
        drop(windows.remove(0));
        assert_eq!(
            log.lock().unwrap().len(),
            2,
            "only the first window is gone"
        );
        drop(windows);
        assert_eq!(log.lock().unwrap().len(), 4);
    }
}
