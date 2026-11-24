//! Play a described input against the shell, offscreen, and draw the result (feature `play`).
//!
//! Pattern: Facade over `fs-components-replay` (the plan and `perform`) and `blinc_recorder::Headless` (the surface): one
//! function keeps the order mount, play, settle, draw. Deviation: `picture` draws through `draw_to_png`, this goes through
//! `Headless`, because only `Headless` takes input.
//!
//! After the last step the surface settles before it is drawn. A step of the program that asks for a new drawing
//! (`request_rebuild`, as `fs_components_widgets::plan::redraw` does after each answer of a board) is taken up there: `Headless`
//! builds the mounted element again when the clock runs, so a card moved by the host is drawn in its new column and not where it stood.

use std::path::Path;
use std::sync::{Mutex, PoisonError};

use blinc_recorder::Headless;
use fs_components_render::{RenderError, bundle, picture_scheme, register_fonts};
use fs_components_replay::{Driver, HeadlessDriver, Plan, perform};

use crate::error::AppError;
use crate::widget::AppWidget;

/// One surface at a time: Blinc keeps theme and reactive state in process-wide singletons.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

impl AppWidget {
    /// Play `plan` against the shell as it is now, on a surface of `width` x `height` without a window, then draw it to a PNG.
    /// The look is that of [`AppWidget::picture`] (the scheme of `FS_RENDER_SCHEME`/`FS_COLOR_SCHEME`, light when unset). No
    /// picture is written before every step has been played; a plan the surface cannot play is refused before the first step. Not while a
    /// window of this process is open: it would reset that window's theme. Call it on the interface thread.
    ///
    /// # Errors
    /// [`AppError::Picture`] if the size is 0 or there is no graphics adapter; [`AppError::Replay`] with the code of the
    /// failure (`steps-unsupported`, `steps-element-missing`, ... or `steps-driver` when the picture cannot be written).
    pub fn play(&self, plan: &Plan, width: u32, height: u32, path: &Path) -> Result<(), AppError> {
        if width == 0 || height == 0 {
            return Err(AppError::Picture(RenderError::EmptyCanvas));
        }
        let _one = ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner);
        if !Headless::available() {
            return Err(no_graphics("no graphics adapter"));
        }
        register_fonts();
        let mut ui = Headless::new(width, height).map_err(|e| no_graphics(&e.to_string()))?;
        let scheme = picture_scheme(
            std::env::var("FS_RENDER_SCHEME").ok().as_deref(),
            std::env::var("FS_COLOR_SCHEME").ok().as_deref(),
        );
        ui.theme(bundle(&self.styles()), scheme);
        let app = self.clone();
        #[allow(clippy::cast_precision_loss)] // sizes are far below 2^23
        let (w, h) = (width as f32, height as f32);
        ui.mount(move |_| app.build(w, h));
        ui.settle();
        let mut driver = HeadlessDriver::new(&mut ui);
        perform(plan, &mut driver).map_err(AppError::Replay)?;
        // The rebuild that the answers of the program asked for lands here, before the picture.
        ui.settle();
        HeadlessDriver::new(&mut ui)
            .snapshot(path)
            .map_err(AppError::Replay)
    }
}

fn no_graphics(why: &str) -> AppError {
    AppError::Picture(RenderError::NoGraphics(why.to_owned()))
}
