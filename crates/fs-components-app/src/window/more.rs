//! A second window beside the one [`super::launch`] opened.
//!
//! Pattern: RAII guard (the close watch).

use blinc_app::prelude::WindowConfig;
use blinc_app::windowed::open_window_with;
use blinc_layout::prelude::div;

use super::WindowOptions;
use crate::error::AppError;
use crate::styles::MIN_WINDOW;
use crate::widget::AppWidget;

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
