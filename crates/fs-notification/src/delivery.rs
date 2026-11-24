//! Showing a notification: the rule chooses the forms, the outlets take it, a fallback keeps a modal from being lost.
//!
//! Pattern: Strategy ([`crate::Policy`]) + Chain (form by form, outlet by outlet).

use fs_types::coded::Coded;

use crate::codes::{NotificationCode, keys};
use crate::events::{Event, Open};
use crate::message::NoteId;
use crate::notification::Notification;
use crate::notifications::{Notifications, lock};
use crate::outlet::{Form, Window};
use crate::remember::Offer;

impl Notifications {
    // Known and open (review r1 S7): the replace check and the swap run under two locks (the forms are shown between them, outside a
    // lock, so a foreign Outlet cannot block the state). A notification that ends in between comes back under its id, and its later
    // end is the newest entry for `outcome`. A rare race: the answer stays in the log and its events went out, but `outcome` then shows the later end, and the
    // returned notification can be answered a second time (S7, still open); to be closed with a generation number in a later cut.
    pub(crate) fn show(
        &self,
        notification: Notification,
        offer: Option<Offer>,
        now_ms: u64,
    ) -> Result<NoteId, Coded<NotificationCode>> {
        // the fields are public, so the buttons are checked again here: a question without a button could never be answered
        notification.check_actions()?;
        let (chosen, id, lifetime, notification, offer) = {
            let mut st = lock(&self.inner.state);
            st.now_ms = now_ms;
            // without a window nothing can draw a box (and no toast comes): the box is dropped, the question stays what it was
            let (notification, offer) = if offer.is_some() && st.situation.window == Window::Absent
            {
                (notification.without_remember(), None)
            } else {
                (notification, offer)
            };
            let mut chosen = st.policy.choose(&notification, &st.situation);
            if offer.is_some() {
                // the system cannot draw a box (G-Q8): with a window the question is a modal or a toast, never only the system form
                chosen.retain(|f| *f != Form::System);
            }
            // only an id that is open here may be replaced; an id from outside that was never issued is ignored
            let id = match &notification.replaces {
                Some(r) if st.open.iter().any(|o| &o.id == r) => r.clone(),
                _ => {
                    let id = NoteId::issued(&st.instance, st.next_id);
                    st.next_id += 1;
                    id
                }
            };
            let lifetime = st.policy.lifetime_ms(&notification);
            (chosen, id, lifetime, notification, offer)
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
            offer,
        };
        let before = {
            let mut st = lock(&self.inner.state);
            if let Some(at) = st.open.iter().position(|o| o.id == id) {
                std::mem::replace(&mut st.open[at], open).forms
            } else {
                st.open.push(open);
                Vec::new()
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
