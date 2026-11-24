//! The draft: the text a person is typing and the text the module said at the last reading.
//!
//! Pattern: Memento — `loaded` is what the module holds; the editor state is only the draft and never the truth.

use std::sync::Arc;

use fs_components_editor::Language;

use crate::view::SourceEditor;

/// The colours of `media_type`: the language of the feature `syntect`, or none (plain text) without the feature or for a media
/// type no language knows.
pub(super) fn language_of(media_type: &str) -> Option<Arc<dyn Language>> {
    #[cfg(feature = "syntect")]
    if let Some(language) = fs_components_editor::SyntectLanguage::for_media_type(media_type) {
        return Some(Arc::new(language));
    }
    let _ = media_type;
    None
}

/// The text of one record being edited.
pub(super) struct Draft {
    editor: SourceEditor,
    loaded: String,
    media_type: String,
}

impl Draft {
    /// A draft of `content` coloured for `media_type`.
    pub(super) fn new(content: &str, media_type: &str) -> Self {
        let editor = language_of(media_type)
            .map_or_else(SourceEditor::new, |l| SourceEditor::new().with_language(l));
        editor.set(content);
        Self {
            editor,
            loaded: content.to_owned(),
            media_type: media_type.to_owned(),
        }
    }

    /// The editor state (drawn by the view; typing changes only this).
    pub(super) fn editor(&self) -> &SourceEditor {
        &self.editor
    }

    /// The text as it is now.
    pub(super) fn text(&self) -> String {
        self.editor.text()
    }

    /// Is the draft different from what the module holds?
    pub(super) fn is_dirty(&self) -> bool {
        self.editor.text() != self.loaded
    }

    /// Back to what the module holds.
    pub(super) fn discard(&self) {
        self.editor.set(&self.loaded);
    }

    /// The module now says `content` of `media_type`. A clean draft follows; a typed draft stays (nothing typed is lost) and only
    /// the text to go back to changes. A new media type means new colours, so the editor is made anew.
    pub(super) fn adopt(&mut self, content: &str, media_type: &str) {
        let typed = self.is_dirty().then(|| self.text());
        if media_type == self.media_type {
            content.clone_into(&mut self.loaded);
            if typed.is_none() {
                self.editor.set(content);
            }
        } else {
            *self = Self::new(content, media_type);
        }
        if let Some(text) = typed {
            self.editor.set(&text);
        }
    }

    /// The module accepted `text`: it is what it holds now (so the draft is clean until the next reading says otherwise).
    pub(super) fn saved(&mut self, text: &str) {
        text.clone_into(&mut self.loaded);
    }

    /// The media type the colours were chosen for (a test asks).
    #[cfg(test)]
    pub(super) fn media_type(&self) -> &str {
        &self.media_type
    }
}
