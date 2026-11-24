//! The rules of the code editor over a fake host: typing changes only the draft, save sends exactly one `edit`, discard goes back,
//! a locked text never writes. (The drawing needs a window and is shown by the picture of E5.)
//!
//! Pattern: Fake (a host that keeps the calls) — the rules run over the wiring without a window.

use std::sync::{Arc, Mutex};

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused, WriteValue};
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_words::Locale;

use super::View;
use super::wiring::Wiring;

/// A host that may or may not edit and keeps every call.
type Call = (String, Option<String>, Vec<WriteArg>);

#[derive(Default)]
struct Host {
    allowed: bool,
    refuse: bool,
    calls: Mutex<Vec<Call>>,
}

impl FormWrite for Host {
    fn can(&self, channel: &str, _record: Option<&str>) -> bool {
        self.allowed && channel == "edit"
    }

    fn send(
        &self,
        channel: &str,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        self.calls.lock().unwrap().push((
            channel.to_owned(),
            record.map(str::to_owned),
            args.to_vec(),
        ));
        if self.refuse {
            return Err(WriteRefused::new("conflict"));
        }
        Ok(WriteDone::default())
    }
}

fn input(content: &str, media_type: &str) -> FormInput {
    FormInput {
        form: "single-record@1.1.0".to_owned(),
        rows: vec![FormRow::of(
            "page-1",
            &[
                ("label", "Page"),
                ("content", content),
                ("media-type", media_type),
            ],
        )],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
    }
}

fn ctx(host: &Arc<Host>, now: &Arc<Mutex<FormInput>>) -> DrawCtx {
    let reread = Arc::clone(now);
    DrawCtx {
        look: (StyleId::Admin, false),
        say: Arc::new(str::to_owned),
        locale: Locale::english(),
        write: host.clone(),
        reread: Arc::new(move || reread.lock().unwrap().clone()),
    }
}

fn wiring(host: &Arc<Host>, content: &str, media_type: &str) -> (Wiring, Arc<Mutex<FormInput>>) {
    let now = Arc::new(Mutex::new(input(content, media_type)));
    let w = Wiring::new(input(content, media_type), &ctx(host, &now));
    (w, now)
}

fn host(allowed: bool) -> Arc<Host> {
    Arc::new(Host {
        allowed,
        ..Host::default()
    })
}

fn type_in(w: &Wiring, text: &str) {
    w.draft().editor().set(text);
    w.typed();
}

#[test]
fn the_view_is_the_widget_of_the_block() {
    assert_eq!(View.widget(), "CodeEditor");
}

#[test]
fn typing_changes_only_the_draft() {
    let h = host(true);
    let (w, _) = wiring(&h, "# one", "text/markdown");
    assert!(!w.draft().is_dirty());
    type_in(&w, "# two");
    assert!(w.draft().is_dirty());
    assert_eq!(w.draft().text(), "# two");
    assert!(h.calls.lock().unwrap().is_empty());
}

#[test]
fn save_sends_exactly_one_edit_with_the_whole_text_and_the_draft_follows_the_module() {
    let h = host(true);
    let (w, now) = wiring(&h, "a\nb", "text/markdown");
    type_in(&w, "a\nb\nc");
    // the module will say the new text at the next reading
    *now.lock().unwrap() = input("a\nb\nc", "text/markdown");
    w.save();
    w.save(); // clean now: no second call
    let calls = h.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "edit");
    assert_eq!(calls[0].1.as_deref(), Some("page-1"));
    assert_eq!(
        calls[0].2,
        vec![WriteArg {
            name: "content".to_owned(),
            value: WriteValue::Text("a\nb\nc".to_owned())
        }]
    );
    assert!(!w.draft().is_dirty());
}

#[test]
fn a_refused_save_keeps_the_draft_and_says_why() {
    let h = Arc::new(Host {
        allowed: true,
        refuse: true,
        ..Host::default()
    });
    let (w, _) = wiring(&h, "x", "text/plain");
    type_in(&w, "y");
    w.save();
    assert!(w.draft().is_dirty());
    assert_eq!(w.refusal().as_deref(), Some("code-editor.refused.conflict"));
    type_in(&w, "yy");
    assert_eq!(w.refusal(), None, "typing clears the old refusal");
}

#[test]
fn discard_goes_back_to_what_the_module_holds_and_writes_nothing() {
    let h = host(true);
    let (w, _) = wiring(&h, "keep", "text/plain");
    type_in(&w, "lost");
    w.discard();
    assert_eq!(w.draft().text(), "keep");
    assert!(!w.draft().is_dirty());
    assert!(h.calls.lock().unwrap().is_empty());
}

#[test]
fn a_locked_text_says_why_and_never_writes() {
    let h = host(false);
    let (w, _) = wiring(&h, "a", "text/plain");
    assert_eq!(w.locked().as_deref(), Some("code-editor.readonly.no-edit"));
    type_in(&w, "b");
    w.save();
    assert!(h.calls.lock().unwrap().is_empty());
    assert_eq!(w.refusal().as_deref(), Some("code-editor.refused.readonly"));
}

#[test]
fn an_editable_text_has_no_reason_and_markdown_has_a_preview() {
    let h = host(true);
    let (md, _) = wiring(&h, "a", "text/markdown");
    let (toml, _) = wiring(&h, "a", "application/toml");
    assert_eq!(md.locked(), None);
    assert!(md.has_preview());
    assert!(!toml.has_preview());
}

#[test]
fn a_new_reading_keeps_what_was_typed_and_a_clean_draft_follows() {
    let h = host(true);
    let (w, _) = wiring(&h, "one", "text/plain");
    w.load(input("two", "text/plain"));
    assert_eq!(w.draft().text(), "two");
    type_in(&w, "mine");
    w.load(input("three", "text/plain"));
    assert_eq!(w.draft().text(), "mine");
    w.discard();
    assert_eq!(w.draft().text(), "three");
}

#[test]
fn the_media_type_picks_the_colours_when_the_feature_is_on() {
    let h = host(true);
    let (w, _) = wiring(&h, "a", "text/markdown");
    assert_eq!(w.draft().media_type(), "text/markdown");
    assert_eq!(
        super::draft::language_of("text/markdown").is_some(),
        cfg!(feature = "syntect")
    );
    assert!(super::draft::language_of("image/png").is_none());
}

#[test]
fn the_language_is_one_message_with_a_parameter() {
    let h = host(true);
    let now = Arc::new(Mutex::new(input("a", "text/markdown")));
    let mut c = ctx(&h, &now);
    c.say = Arc::new(|key: &str| {
        if key == "code-editor.language" {
            "Language: {media-type}".to_owned()
        } else {
            key.to_owned()
        }
    });
    let w = Wiring::new(input("a", "text/markdown"), &c);
    assert_eq!(w.language_label(), "Language: text/markdown");
    // a host without the key: the bare media type, not a sentence put together here
    let (bare, _) = wiring(&h, "a", "application/toml");
    assert_eq!(bare.language_label(), "application/toml");
}
