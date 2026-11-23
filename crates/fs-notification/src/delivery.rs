//! Showing a notification: the rule chooses the forms, the outlets take it, a fallback keeps a modal from being lost.
//!
//! Pattern: Strategy ([`crate::Policy`]) + Chain (form by form, outlet by outlet).

use fs_types::coded::Coded;

use crate::codes::{NotificationCode, keys};
use crate::events::{Event, Open};
use crate::message::NoteId;
use crate::notification::Notification;
use crate::notifications::{Notifications, lock};
use crate::outlet::Form;

impl Notifications {
    pub(crate) fn show(
        &self,
        notification: Notification,
        now_ms: u64,
    ) -> Result<NoteId, Coded<NotificationCode>> {
        let (chosen, id, lifetime) = {
            let mut st = lock(&self.inner.state);
            let chosen = st.policy.choose(&notification, &st.situation);
            // only an id that is open here may be replaced; an id from outside that was never issued is ignored
            let id = match &notification.replaces {
                Some(r) if st.open.iter().any(|o| &o.id == r) => r.clone(),
                _ => {
                    let id = NoteId::issued(st.next_id);
                    st.next_id += 1;
                    id
                }
            };
            let lifetime = st.policy.lifetime_ms(&notification);
            (chosen, id, lifetime)
        };
        if chosen.is_empty() {
            return Err(Coded::new(NotificationCode::NoForm));
        }
        let shown = self.deliver(&chosen, &id, &notification)?;
        let open = Open {
            id: id.clone(),
            notification,
            forms: shown.clone(),
            deadline_ms: lifetime.map(|ms| now_ms.saturating_add(ms)),
        };
        let before = {
            let mut st = lock(&self.inner.state);
            match st.open.iter().position(|o| o.id == id) {
                Some(at) => std::mem::replace(&mut st.open[at], open).forms,
                None => {
                    st.open.push(open);
                    Vec::new()
                }
            }
        };
        // a replaced notification leaves the forms it does not have any more (a modal that was a question, the system form)
        let gone: Vec<Form> = before.into_iter().filter(|f| !shown.contains(f)).collect();
        self.close_forms(&gone, &id);
        self.notify(&[Event::Shown {
            id: id.clone(),
            forms: shown,
        }]);
        Ok(id)
    }

    /// Hands it to the outlets, form by form. A modal that no outlet can show falls back to a toast. Returns the forms that took it.
    fn deliver(
        &self,
        chosen: &[Form],
        id: &NoteId,
        notification: &Notification,
    ) -> Result<Vec<Form>, Coded<NotificationCode>> {
        let mut shown = Vec::new();
        let mut failure = None;
        let mut want: Vec<Form> = chosen.to_vec();
        if want.contains(&Form::Modal)
            && !self.has_form(Form::Modal)
            && !want.contains(&Form::Toast)
        {
            want.insert(0, Form::Toast);
        }
        for form in want {
            if form == Form::Modal && !self.has_form(Form::Modal) {
                continue;
            }
            let view = if form == Form::System {
                notification.redacted()
            } else {
                notification.clone()
            };
            let mut took = false;
            for outlet in self
                .inner
                .outlets
                .iter()
                .filter(|o| o.forms().contains(&form))
            {
                match outlet.show(form, id, &view) {
                    Ok(()) => took = true,
                    Err(e) => failure = Some(e),
                }
            }
            if took {
                shown.push(form);
            }
        }
        if shown.is_empty() {
            return Err(failure.unwrap_or_else(|| {
                let names: Vec<&str> = chosen.iter().map(|f| f.as_str()).collect();
                Coded::new(NotificationCode::NoOutlet).with_text(keys::FORMS, names.join("+"))
            }));
        }
        Ok(shown)
    }

    fn has_form(&self, form: Form) -> bool {
        self.inner.outlets.iter().any(|o| o.forms().contains(&form))
    }
}
