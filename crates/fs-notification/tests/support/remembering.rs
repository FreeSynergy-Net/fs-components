//! Helpers of the remembering tests: the setting `fs.open.when-open`, real `Settings` over a `MemoryStore`, a facade over a recorder.
#![allow(dead_code)]

use std::sync::Arc;

use fs_notification::{ActionId, Notification, Notifications, Outlet, Policy, Remember, Remembers};
use fs_settings::{
    Catalog, Declaration, GroupId, Kind, LockRights, MemoryStore, NoSystem, Owner, OwnerKind,
    ProgramName, SettingKey, Settings, Target, Value,
};

use super::{Recorder, act, msg};

pub struct Yes;
impl LockRights for Yes {
    fn can_lock(&self, _key: &SettingKey) -> bool {
        true
    }
}

pub fn key() -> SettingKey {
    SettingKey::new(
        &Owner::new(OwnerKind::Framework, "open").unwrap(),
        "when-open",
    )
    .unwrap()
}

pub fn catalog(human_only: bool) -> Catalog {
    let mut d = Declaration::new(
        key(),
        Kind::Choice {
            keys: ["ask", "switch", "tab", "window"]
                .map(String::from)
                .to_vec(),
        },
        Value::Choice("ask".into()),
        GroupId::new("open").unwrap(),
    );
    if human_only {
        d = d.human_only();
    }
    Catalog::of(vec![d]).unwrap()
}

pub fn settings(store: &MemoryStore, human_only: bool) -> Arc<Settings> {
    Arc::new(Settings::open(
        ProgramName::new("demo").unwrap(),
        catalog(human_only),
        Arc::new(store.clone()),
        Arc::new(NoSystem),
    ))
}

pub fn facade(out: &Arc<Recorder>, r: Arc<dyn Remembers>) -> Notifications {
    let o: Arc<dyn Outlet> = out.clone();
    Notifications::with_instance(Policy::default(), vec![o], "n")
        .unwrap()
        .with_remembers(r)
}

pub fn ask_open() -> Notification {
    Notification::ask(
        msg("open-where"),
        vec![act("tab"), act("window"), act("cancel")],
    )
    .unwrap()
    .remember(Remember::new(key(), Target::AllPrograms))
}

pub fn titles(n: &Notifications) -> Vec<String> {
    n.open()
        .iter()
        .map(|o| o.notification.title.key().to_owned())
        .collect()
}

pub fn id(s: &str) -> ActionId {
    ActionId::new(s).unwrap()
}
