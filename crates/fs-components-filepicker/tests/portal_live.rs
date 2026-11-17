//! The real portal: only reachability of the FileChooser interface, in the environment of the agent account itself. Never run by
//! default and never against the desktop of a person (it needs the session bus of the account that runs it).
//!
//! Pattern: Integration test, ignored. The dialog itself needs a person and is NOT covered by any automatic test: the portal path is
//! proven only as far as translation, mapping and the no-bus case (see `portal_map.rs`). Run: `cargo test -p fs-components-filepicker
//! --test portal_live -- --ignored` where a portal is running.
#![cfg(all(feature = "portal", target_os = "linux"))]

use ashpd::desktop::file_chooser::FileChooserProxy;

#[test]
#[ignore = "portal: needs a running xdg-desktop-portal in the session of the account that runs the test; headless CI has none"]
fn the_file_chooser_interface_is_reachable() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let proxy = runtime
        .block_on(FileChooserProxy::new())
        .expect("the portal must be reachable here");
    assert!(proxy.version() >= 1);
}
