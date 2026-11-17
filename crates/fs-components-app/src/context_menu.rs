//! The context menu in Blinc's overlay stack (M3): right click, menu key, Shift+F10; flip and clamp; submenu; two stages.
//!
//! Pattern: Facade over the pure state machine `ContextState` of `fs-components-commands` + Adapter (its lines to the panels
//! of `menu_view`). The state machine decides everything that is a rule (target = selection, stage, once, locked line, Escape); this
//! file only draws it and reports what it says. The panel and the panel of the submenu are two overlays, as for the menus of the
//! bar (see `menu_overlay.rs`). The place comes from [`place_at_point`] (right of and below the point; flipped to the left and
//! upwards at the edges; shortened if neither side fits). A chosen command runs through the **one** handler of
//! `AppWidget::set_menus` (as a click on a line of the bar and as a shortcut do), and `AppWidget::context_target` says what it
//! acts on. The shell's own `app.quit` is answered by the shell. The lock of the runtime is never held while the overlay stack is
//! touched (the stack runs closures that take it). Blinc does not close an overlay on a press outside, so the shell asks for it
//! (and again at every full rebuild).

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::click_outside;
use blinc_layout::overlay_state::overlay_stack;
use blinc_layout::widgets::overlay_stack::{OverlayBuilder, OverlayHandle};
use fs_components_commands::standard::QUIT;
use fs_components_commands::{Choice, CommandId, ContextState, Point, Resolved, Target, Trigger};
use fs_components_widgets::dropdown::place_at_point;

use crate::context_size::{estimate_height, panel_width};
use crate::menu::MenuEntry;
use crate::menu_model::{ModelTexts, context_entries};
use crate::menu_overlay::{sub_position, window_width};
use crate::menu_view::{Anchor, Callbacks, Look, arrows_left, content_width, one_panel};
use crate::shell::Status;
use crate::widget::{AppWidget, locked};

/// The element id of the panel of the context menu.
pub(crate) const PANEL_ID: &str = "fs-app-context-panel";
/// The element id of the panel of its submenu.
pub(crate) const SUB_PANEL_ID: &str = "fs-app-context-sub-panel";
/// The key of the one registration for a press outside.
const OUTSIDE_KEY: &str = "fs-app-context-outside";
/// The menu the lines report (the title bar's menus are numbered from 0; this is none of them).
const CONTEXT_INDEX: usize = usize::MAX;

/// What the window keeps of the context menu.
#[derive(Default)]
pub(crate) struct Runtime {
    state: ContextState,
    /// The lines of the first level, one per item of `state`.
    entries: Vec<MenuEntry>,
    overlay: Option<u64>,
    sub_overlay: Option<u64>,
    /// What the command that ran last acted on (read by the program inside its handler).
    target: Option<Target>,
}

/// Where the runtime of the context menu is kept; clones share it.
pub(crate) type ContextSlot = Arc<Mutex<Runtime>>;

fn runtime(slot: &ContextSlot) -> MutexGuard<'_, Runtime> {
    slot.lock().unwrap_or_else(PoisonError::into_inner)
}

impl AppWidget {
    /// Open the context menu for `trigger` on `target`, made of `items` (resolved items of the arrangement `context:<kind>`).
    /// The stage follows the trigger (Shift+right click opens the second stage); a menu that was open closes first. Lines that
    /// are chosen run through the handler of [`AppWidget::set_menus`] (set the menus first); read what they act on with
    /// [`AppWidget::context_target`]. A placeholder is skipped. Call it on the interface thread.
    pub fn open_context_menu(
        &self,
        trigger: &Trigger,
        target: Target,
        items: &[Resolved],
        texts: &ModelTexts<'_>,
    ) {
        self.close_context_menu();
        let items: Vec<Resolved> = items
            .iter()
            .filter(|i| !matches!(i, Resolved::Placeholder(_)))
            .cloned()
            .collect();
        let (staged, at) = {
            let mut rt = runtime(&self.context);
            rt.state.open(trigger, target, &items);
            (rt.state.items().to_vec(), trigger.point())
        };
        let entries = context_entries(&staged, texts);
        runtime(&self.context).entries.clone_from(&entries);
        if entries.is_empty() {
            runtime(&self.context).state.press_outside();
            return;
        }
        let raw = self.show_panel(entries, at);
        runtime(&self.context).overlay = Some(raw);
        self.watch_context_outside();
    }

    /// Close the context menu, if it is open.
    pub fn close_context_menu(&self) {
        let (overlay, sub) = {
            let mut rt = runtime(&self.context);
            rt.state.press_outside();
            (rt.overlay.take(), rt.sub_overlay.take())
        };
        click_outside::unregister_click_outside(OUTSIDE_KEY);
        for raw in [sub, overlay].into_iter().flatten() {
            OverlayHandle::from_raw(raw).close();
        }
    }

    /// Is the context menu open?
    pub fn context_menu_open(&self) -> bool {
        runtime(&self.context).state.is_open()
    }

    /// What the context menu acts on: while it is open, its target; after a line ran, the target of that run (so the handler of the
    /// program can read it). `None` before the first menu.
    pub fn context_target(&self) -> Option<Target> {
        let rt = runtime(&self.context);
        rt.state.target().cloned().or_else(|| rt.target.clone())
    }

    /// Ask again for a press outside while a context menu is open (Blinc forgets it at every full rebuild).
    pub(crate) fn watch_context_outside(&self) {
        if !runtime(&self.context).state.is_open() {
            return;
        }
        let this = self.clone();
        click_outside::register_click_outside_multi(
            OUTSIDE_KEY,
            &[PANEL_ID.to_owned(), SUB_PANEL_ID.to_owned()],
            move || this.close_context_menu(),
        );
    }

    /// The shell answers `app.quit` itself; every other command goes to the handler of the program.
    pub(crate) fn builtin(&self, id: &str) -> bool {
        if id == QUIT {
            self.quit();
            true
        } else {
            false
        }
    }

    fn callbacks_for_context(&self) -> Callbacks {
        let (choose, sub) = (self.clone(), self.clone());
        Callbacks {
            choose: Arc::new(move |_, id| choose.context_choose(id)),
            sub: Arc::new(move |_, position, anchor| sub.context_sub(position, anchor)),
        }
    }

    fn show_panel(&self, entries: Vec<MenuEntry>, at: Point) -> u64 {
        let calls = self.callbacks_for_context();
        let (window_w, window_h) = overlay_stack()
            .lock()
            .map_or((0.0, 0.0), |s| s.viewport());
        let width = panel_width(&entries, window_w);
        let placed = place_at_point((at.x, at.y), (width, estimate_height(&entries)), (window_w, window_h));
        let left = placed.rect.x;
        let arrows = arrows_left(&entries, (left, width), window_w);
        let raw = overlay_stack()
            .lock()
            .map_or(0, |stack| stack.peek_next_handle_id());
        let (slot, reader) = (Arc::clone(&self.context), Arc::clone(&self.context));
        OverlayBuilder::dropdown()
            .at(placed.rect.x, placed.rect.y)
            .dismissable_by_escape(true)
            .on_close(move |_reason| {
                // Escape comes this way. A menu opened since then is not this one.
                let sub = {
                    let mut rt = runtime(&slot);
                    if rt.overlay != Some(raw) {
                        return;
                    }
                    rt.state.press_outside();
                    rt.overlay = None;
                    rt.sub_overlay.take()
                };
                click_outside::unregister_click_outside(OUTSIDE_KEY);
                if let Some(sub) = sub {
                    OverlayHandle::from_raw(sub).close();
                }
            })
            .content(move || {
                // Drawn again when the submenu opens or closes, so the line that opens it says `aria-expanded`.
                let open_sub = runtime(&reader).state.open_sub();
                let look = Look {
                    open_sub,
                    width,
                    ..Look::default()
                };
                one_panel(PANEL_ID, CONTEXT_INDEX, &entries, (look, &arrows), &calls)
            })
            .show()
            .raw()
    }

    /// A line or a symbol was pressed: the state machine says what it leads to.
    fn context_choose(&self, id: &str) {
        let choice = runtime(&self.context).state.choose(&CommandId::new(id));
        match choice {
            Choice::Run { id, target } => {
                runtime(&self.context).target = Some(target);
                self.release_context_overlays();
                // The menu is gone first: what the handler opens must not go down with it.
                if !self.builtin(id.as_str()) {
                    let handler = locked(&self.shell).handler();
                    if let Some(handler) = handler {
                        handler(id.as_str());
                    }
                }
            }
            Choice::Locked { id, .. } => {
                let reason = runtime(&self.context)
                    .entries
                    .iter()
                    .flat_map(|e| std::iter::once(e).chain(&e.row).chain(&e.children))
                    .find(|e| e.id == id.as_str())
                    .and_then(|e| e.reason.clone());
                if let Some(reason) = reason {
                    self.set_status(Some(Status::Info(reason)));
                }
            }
            _ => {}
        }
    }

    /// The line at `position` opens its submenu beside it (`anchor`), or closes it if it was open.
    fn context_sub(&self, position: usize, anchor: Anchor) {
        let (children, old) = {
            let mut rt = runtime(&self.context);
            if !rt.state.toggle_sub(position) {
                return;
            }
            let children = rt
                .state
                .open_sub()
                .and_then(|p| rt.entries.get(p))
                .map(|e| e.children.clone());
            (children, rt.sub_overlay.take())
        };
        // The old one goes first; its close callback sees that it is no longer the runtime's and does nothing.
        if let Some(raw) = old {
            OverlayHandle::from_raw(raw).close();
        }
        let Some(children) = children else { return };
        let window = window_width();
        let width = content_width(&children, window);
        let at = sub_position(anchor, width, window);
        let raw = self.show_sub(children, (at, width));
        runtime(&self.context).sub_overlay = Some(raw);
        self.watch_context_outside();
    }

    fn show_sub(&self, entries: Vec<MenuEntry>, ((x, y), width): ((f32, f32), f32)) -> u64 {
        let calls = self.callbacks_for_context();
        let raw = overlay_stack()
            .lock()
            .map_or(0, |stack| stack.peek_next_handle_id());
        let slot = Arc::clone(&self.context);
        OverlayBuilder::dropdown()
            .at(x, y)
            .dismissable_by_escape(true)
            .on_close(move |_reason| {
                // Escape closes this level only; the menu stays.
                let mut rt = runtime(&slot);
                if rt.sub_overlay == Some(raw) {
                    rt.sub_overlay = None;
                    rt.state.escape();
                }
            })
            .content(move || {
                let look = Look {
                    width,
                    ..Look::default()
                };
                one_panel(SUB_PANEL_ID, CONTEXT_INDEX, &entries, (look, &[]), &calls)
            })
            .show()
            .raw()
    }

    /// Take both overlays out of the runtime and close them (the menu is already closed in the state).
    fn release_context_overlays(&self) {
        let (overlay, sub) = {
            let mut rt = runtime(&self.context);
            (rt.overlay.take(), rt.sub_overlay.take())
        };
        click_outside::unregister_click_outside(OUTSIDE_KEY);
        for raw in [sub, overlay].into_iter().flatten() {
            OverlayHandle::from_raw(raw).close();
        }
    }
}
