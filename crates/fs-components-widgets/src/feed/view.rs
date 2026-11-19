//! The feed as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the pure `State`; a change draws the articles again through `Live`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::model::{Outcome, State};
use crate::Element;
use crate::keys::{Handled, KeyEvent, bind_keys};
use crate::live::Live;

type OnMove = Arc<dyn Fn(&str) + Send + Sync>;

/// One article: a key, a title and what its body shows.
#[non_exhaustive]
#[derive(Clone)]
pub struct Article {
    /// What the caller gets back; part of the ids.
    pub key: String,
    /// The title (names the article for a screen reader).
    pub title: String,
    body: Arc<dyn Fn() -> Div + Send + Sync>,
}

impl Article {
    /// An article; `body` is built each time the feed is drawn.
    pub fn new(
        key: impl Into<String>,
        title: impl Into<String>,
        body: impl Fn() -> Div + Send + Sync + 'static,
    ) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            body: Arc::new(body),
        }
    }
}

/// A feed. Clones share the articles, the state and the callback.
#[derive(Clone)]
pub struct Feed {
    id: String,
    name: Option<String>,
    articles: Arc<Mutex<Vec<Article>>>,
    state: Arc<Mutex<State>>,
    live: Live,
    on_move: Option<OnMove>,
}

impl Feed {
    /// A feed of these articles. Ids: `<id>`, `<id>-article-<key>`, `<id>-title-<key>`.
    pub fn new(articles: Vec<Article>) -> Self {
        let state = State::new(articles.len());
        Self {
            id: "feed".to_owned(),
            name: None,
            articles: Arc::new(Mutex::new(articles)),
            state: Arc::new(Mutex::new(state)),
            live: Live::new(),
            on_move: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// What the feed is called (`aria-label`): "News".
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// The size of the whole feed when it is known (`aria-setsize`); without it the feed says `-1`, as the APG asks.
    #[must_use]
    pub fn total(self, total: usize) -> Self {
        self.slot().set_total(Some(total));
        self
    }

    /// Call `handler` with the key of the article the cursor moved to (the host scrolls it into view).
    #[must_use]
    pub fn on_move(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_move = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn list(&self) -> MutexGuard<'_, Vec<Article>> {
        self.articles.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The key of the article the cursor is on.
    pub fn current(&self) -> Option<String> {
        let at = self.slot().cursor()?;
        self.list().get(at).map(|a| a.key.clone())
    }

    /// More articles are loading (`aria-busy`) or not.
    pub fn busy(&self, busy: bool) {
        self.slot().set_busy(busy);
        self.live.refresh_now();
    }

    /// Articles arrived at the end.
    pub fn push(&self, more: Vec<Article>) {
        self.slot().grow(more.len());
        self.list().extend(more);
        self.live.refresh_now();
    }

    /// A press of the keyboard.
    pub fn press(&self, event: KeyEvent) -> Handled {
        let outcome = self.slot().press(event.key);
        let Outcome::Moved(to) = outcome else {
            return false;
        };
        self.live.refresh_now();
        let key = self.list().get(to).map(|a| a.key.clone());
        self.said(key);
        true
    }

    fn click(&self, at: usize) {
        if self.slot().go(at) {
            self.live.refresh_now();
            let key = self.list().get(at).map(|a| a.key.clone());
            self.said(key);
        }
    }

    fn said(&self, key: Option<String>) {
        if let (Some(key), Some(handler)) = (key, &self.on_move) {
            handler(&key);
        }
    }

    /// The feed, to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (inner, keys) = (self.clone(), self.clone());
        let mut root = div().id(&self.id).class("fs-feed").attr("role", "feed");
        if let Some(name) = &self.name {
            root = root.attr("aria-label", name);
        }
        bind_keys(
            root.child(self.live.region("articles", move || inner.articles())),
            Arc::new(move |event: KeyEvent| keys.press(event)),
        )
    }

    fn articles(&self) -> Div {
        let (cursor, size, busy) = {
            let s = self.slot();
            (s.cursor(), s.set_size(), s.busy())
        };
        let mut column = div()
            .id(format!("{}-articles", self.id))
            .flex_col()
            .attr("aria-busy", if busy { "true" } else { "false" });
        for (i, a) in self.list().iter().enumerate() {
            let title_id = format!("{}-title-{}", self.id, a.key);
            let mut one = div()
                .id(format!("{}-article-{}", self.id, a.key))
                .class("fs-article")
                .attr("role", "article")
                .attr("aria-posinset", (i + 1).to_string())
                .attr("aria-setsize", size.to_string())
                .attr("aria-labelledby", &title_id);
            if cursor == Some(i) {
                one = one.class("fs-article-current");
            }
            let this = self.clone();
            column = column.child(
                one.on_click(move |_| this.click(i))
                    .child(text(a.title.clone()).id(title_id).class("fs-article-title"))
                    .child(div().class("fs-article-body").child((a.body)())),
            );
        }
        column
    }
}
