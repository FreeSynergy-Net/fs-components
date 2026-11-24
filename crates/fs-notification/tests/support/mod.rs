//! Helpers of the tests: a recording outlet and short constructors.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use fs_notification::{
    Action, ActionId, Event, Form, Message, NoteId, Notification, NotificationCode, Notifications,
    Outlet, Policy,
};
use fs_types::coded::Coded;

/// What an outlet was told, in order: (`show` or `close`, form, id, notification title key).
pub type Log = Arc<Mutex<Vec<(String, Form, String, String)>>>;

pub struct Recorder {
    pub forms: Vec<Form>,
    pub log: Log,
    pub fail: bool,
    /// The last notification handed to each `show`.
    pub seen: Mutex<Vec<(Form, Notification)>>,
}

impl Recorder {
    pub fn new(forms: &[Form]) -> Arc<Self> {
        Arc::new(Self {
            forms: forms.to_vec(),
            log: Arc::default(),
            fail: false,
            seen: Mutex::default(),
        })
    }

    pub fn failing(forms: &[Form]) -> Arc<Self> {
        Arc::new(Self {
            forms: forms.to_vec(),
            log: Arc::default(),
            fail: true,
            seen: Mutex::default(),
        })
    }

    pub fn lines(&self) -> Vec<String> {
        self.log
            .lock()
            .unwrap()
            .iter()
            .map(|(what, form, id, _)| format!("{what} {} {id}", form.as_str()))
            .collect()
    }
}

impl Outlet for Recorder {
    fn forms(&self) -> &[Form] {
        &self.forms
    }

    fn show(
        &self,
        form: Form,
        id: &NoteId,
        n: &Notification,
    ) -> Result<(), Coded<NotificationCode>> {
        if self.fail {
            return Err(Coded::new(NotificationCode::ShowFailed));
        }
        self.log.lock().unwrap().push((
            "show".into(),
            form,
            id.as_str().into(),
            n.title.key().into(),
        ));
        self.seen.lock().unwrap().push((form, n.clone()));
        Ok(())
    }

    fn close(&self, form: Form, id: &NoteId) {
        self.log
            .lock()
            .unwrap()
            .push(("close".into(), form, id.as_str().into(), String::new()));
    }
}

pub fn msg(key: &str) -> Message {
    Message::new(key).unwrap()
}

pub fn act(id: &str) -> Action {
    Action::new(ActionId::new(id).unwrap(), msg(&format!("button-{id}")))
}

pub fn question(title: &str) -> Notification {
    Notification::ask(msg(title), vec![act("now"), act("later")]).unwrap()
}

pub fn all_forms() -> Arc<Recorder> {
    Recorder::new(&[Form::Toast, Form::Modal, Form::System])
}

pub fn facade(outlet: &Arc<Recorder>) -> Notifications {
    let o: Arc<dyn Outlet> = outlet.clone();
    Notifications::with_instance(Policy::default(), vec![o], "n").unwrap()
}

/// Collects every event.
pub fn collect(n: &Notifications) -> (fs_notification::Subscription, Arc<Mutex<Vec<Event>>>) {
    let got: Arc<Mutex<Vec<Event>>> = Arc::default();
    let sink = Arc::clone(&got);
    let sub = n.subscribe(Arc::new(move |e: &Event| {
        sink.lock().unwrap().push(e.clone());
    }));
    (sub, got)
}
pub mod remembering;
