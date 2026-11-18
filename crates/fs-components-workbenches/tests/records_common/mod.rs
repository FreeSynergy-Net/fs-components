//! A provider in memory for the records adapter tests: it is NOT `Send` (an `Rc` inside, as a capnp client is), it is told what
//! to answer from the test thread through a shared script, and it counts the calls.
#![allow(dead_code, clippy::pedantic)]

use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use fs_components_workbenches::records::{
    Fault, FieldValue, Group, GroupSet, Hidden, HiddenReason, Page, Pending, RecordQuery,
    RecordRow, RecordsBinding, RecordsRead, Role, Slot, Worker, block_on_std,
};

#[derive(Default)]
pub struct Script {
    pub generation: u64,
    pub groups: Vec<Group>,
    pub rows: Vec<RecordRow>,
    pub page_len: usize,
    pub fail: Option<String>,
    pub finds: u32,
    pub generations: u32,
    pub queries: Vec<RecordQuery>,
}

pub type Shared = Arc<Mutex<Script>>;

pub fn script(s: &Shared) -> MutexGuard<'_, Script> {
    s.lock().unwrap_or_else(PoisonError::into_inner)
}

pub struct Mem {
    s: Shared,
    _not_send: Rc<()>,
}

impl Mem {
    pub fn new(s: Shared) -> Self {
        Self {
            s,
            _not_send: Rc::new(()),
        }
    }
}

impl RecordsRead for Mem {
    fn find(&self, q: &RecordQuery) -> Pending<'_, Page> {
        let after = q.after.clone();
        let q = q.clone();
        Box::pin(async move {
            let mut s = script(&self.s);
            s.finds += 1;
            s.queries.push(q);
            if let Some(code) = &s.fail {
                return Err(Fault::new(code.clone()));
            }
            let start: usize = after.map_or(0, |a| a.parse().unwrap());
            let len = if s.page_len == 0 {
                s.rows.len()
            } else {
                s.page_len
            };
            let end = (start + len).min(s.rows.len());
            Ok(Page {
                items: s.rows[start..end].to_vec(),
                next: (end < s.rows.len()).then(|| end.to_string()),
                total: Some(s.rows.len() as u64),
            })
        })
    }

    fn groups(&self, _t: &str, _g: &str, _scope: Option<&str>) -> Pending<'_, GroupSet> {
        Box::pin(async move {
            let s = script(&self.s);
            match &s.fail {
                Some(code) => Err(Fault::new(code.clone())),
                None => Ok(GroupSet {
                    revision: 1,
                    groups: s.groups.clone(),
                }),
            }
        })
    }

    fn generation(&self, _t: &str, _scope: Option<&str>) -> Pending<'_, u64> {
        Box::pin(async move {
            let mut s = script(&self.s);
            s.generations += 1;
            match &s.fail {
                Some(code) => Err(Fault::new(code.clone())),
                None => Ok(s.generation),
            }
        })
    }
}

pub fn group(key: &str, title: Option<&str>, limit: Option<u32>) -> Group {
    Group {
        key: key.into(),
        title: title.map(Into::into),
        hint_limit: limit,
    }
}

pub fn row(id: &str, group: &str, fields: &[(&str, &[&str])]) -> RecordRow {
    RecordRow {
        id: id.into(),
        values: fields
            .iter()
            .map(|(f, v)| FieldValue {
                field: (*f).into(),
                values: v.iter().map(|x| (*x).into()).collect(),
            })
            .collect(),
        hidden: Vec::new(),
        lock: None,
        may: Vec::new(),
        group: Some(group.into()),
    }
}

pub fn hide(mut r: RecordRow, field: &str, why: HiddenReason) -> RecordRow {
    r.hidden.push(Hidden {
        field: field.into(),
        reason: why,
    });
    r
}

pub fn binding() -> RecordsBinding {
    RecordsBinding::new("Task", "board", "title")
        .subtitle("note")
        .slot(Slot::new("due", "Due", Role::Compact))
        .slot(Slot::new("priority", "Priority", Role::Compact))
        .slot(Slot::new("description", "Description", Role::Display))
        .slot(Slot::new("secret", "Secret", Role::Display))
        .slot(Slot::new("title", "Title", Role::Cell))
        .slot(Slot::new("due", "Due", Role::Cell))
        .commands("tasks.")
}

pub fn start(s: &Shared, b: RecordsBinding) -> Arc<Worker> {
    let s = Arc::clone(s);
    let w = Arc::new(Worker::spawn(move || Mem::new(s), b, None, block_on_std));
    until(|| w.version() > 0, "first reading");
    w
}

pub fn until(mut cond: impl FnMut() -> bool, what: &str) {
    let end = Instant::now() + Duration::from_secs(10);
    while !cond() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

pub fn filled() -> Shared {
    let s = Shared::default();
    {
        let mut g = script(&s);
        g.generation = 7;
        g.groups = vec![
            group("todo", Some("To do"), None),
            group("doing", None, Some(2)),
        ];
        g.rows = vec![
            row(
                "t1",
                "todo",
                &[
                    ("title", &["Write"]),
                    ("note", &["soon"]),
                    ("due", &["2026-11-20"]),
                    ("priority", &["2"]),
                    ("description", &["Long text"]),
                ],
            ),
            row("t2", "doing", &[("title", &["Read"])]),
            row("t3", "todo", &[("title", &["Sleep"])]),
        ];
    }
    s
}
