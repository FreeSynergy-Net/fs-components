//! The source editor as a building block behind the editor contract: plain text (Markdown, code, configuration) in Blinc's code editor.
//!
//! Pattern: Strategy (one of the editors) — [`TextEditor`]: a document goes in, only operations of the model come out. This
//! is the editor of the classic and the Confluence-like layout; it never reformats: what is in the editor is what the
//! person wrote. Blinc's `code_editor` (line numbers, colours by a [`Language`]) is behind it; the text of the editor
//! state is only a view, the draft of the person (the module keeps the truth).
//!
//! Limits: the editor holds the whole text, so the operations are found by comparing the blocks before and after
//! ([`diff`]); a person typing in a block makes a replace of that block, a blank line inside a block makes a replace
//! and an insert. Folding and search are off (the design of the editor wants them off for now).

use std::sync::Arc;

use blinc_layout::div::Div;
use blinc_layout::syntax::SyntaxConfig;
use blinc_layout::widgets::code::{
    Code, CodeEditor, SharedCodeEditorState, code, code_editor, code_editor_state, code_minimap,
};
use fs_components_editor::{
    Doc, Edit, Language, Op, TextEditor, backspace_text, diff, toggle_task_line,
};

use super::source_highlight::{BlincHighlight, SyntaxPalette};

/// The code editor state of one edit, its language and the document as it was when the edit began.
pub struct SourceEditor {
    state: SharedCodeEditorState,
    language: Option<Arc<dyn Language>>,
    loaded: Doc,
}

impl Default for SourceEditor {
    fn default() -> Self {
        Self {
            state: code_editor_state(""),
            language: None,
            loaded: Doc::default(),
        }
    }
}

impl SourceEditor {
    /// A source editor with an empty text and no colours.
    pub fn new() -> Self {
        Self::default()
    }

    /// The same editor, its text coloured by `language` (the theme must be initialised when the widget is made).
    #[must_use]
    pub fn with_language(mut self, language: Arc<dyn Language>) -> Self {
        self.language = Some(language);
        self
    }

    /// The editor state (the layouts draw it with [`widget`](Self::widget) and [`minimap`](Self::minimap)).
    pub fn state(&self) -> &SharedCodeEditorState {
        &self.state
    }

    /// The code editor: line numbers on, folding and search off, coloured by the language when there is one.
    pub fn widget(&self) -> CodeEditor {
        let editor = code_editor(&self.state)
            .line_numbers(true)
            .code_folding(false)
            .search_enabled(false);
        let Some(language) = &self.language else {
            return editor;
        };
        let high = BlincHighlight::new(Arc::clone(language), SyntaxPalette::from_theme());
        let editor = editor.syntax(SyntaxConfig::new(high));
        // `syntax` fixes ground, ink and line numbers to the highlighter's; the theme's rules (`.blinc-code`) are better.
        let mut data = self.lock();
        data.config.bg_color = None;
        data.config.text_color = None;
        data.config.line_number_color = None;
        drop(data);
        editor
    }

    /// The text as it stands, drawn locked (line numbers, the same colours): for a person who may not edit. Blinc's `code_editor`
    /// has no read-only mode, so the read-only `code` block takes its place.
    pub fn read_only(&self) -> Code {
        let block = code(self.text()).line_numbers(true);
        match &self.language {
            Some(language) => {
                let high = BlincHighlight::new(Arc::clone(language), SyntaxPalette::from_theme());
                block.syntax(SyntaxConfig::new(high))
            }
            None => block,
        }
    }

    /// The overview of the whole text, to be put next to [`widget`](Self::widget) (it is built once: the parent builds it again
    /// after a change).
    pub fn minimap(&self) -> Div {
        code_minimap(&self.state)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, blinc_layout::widgets::code::CodeEditorData> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn text(&self) -> String {
        self.lock().value()
    }

    /// Replace the whole text. Blinc has no `set_value`: the selection is deleted (this empties the colour cache), then the
    /// lines are put in and the history is dropped.
    pub(crate) fn set(&self, text: &str) {
        let mut data = self.lock();
        data.select_all();
        data.delete_selection();
        data.lines = if text.is_empty() {
            vec![String::new()]
        } else {
            text.split('\n').map(str::to_owned).collect()
        };
        data.cursor.line = 0;
        data.cursor.column = 0;
        data.selection_start = None;
        data.undo_stack.clear();
        data.redo_stack.clear();
    }

    /// Where block `i` starts in the text as it stands (blocks are one blank line apart).
    fn start_of(&self, i: usize) -> usize {
        Doc::parse(&self.text())
            .blocks
            .iter()
            .take(i)
            .map(|b| b.len() + 2)
            .sum()
    }
}

impl TextEditor for SourceEditor {
    fn name(&self) -> &'static str {
        "source"
    }

    fn load(&mut self, doc: &Doc) {
        self.set(&doc.to_markdown());
        self.loaded = doc.clone();
    }

    fn document(&self) -> Doc {
        Doc::parse(&self.text())
    }

    fn edit(&mut self, edit: &Edit) {
        let text = self.text();
        let (block, at, insert) = match edit {
            Edit::Type { block, at, text } => (*block, *at, text.as_str()),
            Edit::Enter { block, at } => (*block, *at, "\n"),
            Edit::Backspace { block, at } => {
                if let Some((now, _)) = backspace_text(&text, self.start_of(*block), *at) {
                    self.set(&now);
                }
                return;
            }
            Edit::ToggleTask { block, at } => {
                let start = self.start_of(*block);
                if let Some(one) = Doc::parse(&text).blocks.get(*block)
                    && let Some(now) = toggle_task_line(one, *at)
                {
                    self.set(&format!(
                        "{}{now}{}",
                        &text[..start],
                        &text[start + one.len()..]
                    ));
                }
                return;
            }
        };
        let place = (self.start_of(block) + at).min(text.len());
        self.set(&format!("{}{insert}{}", &text[..place], &text[place..]));
    }

    fn take_ops(&mut self) -> Vec<Op> {
        let now = self.document();
        let ops = diff(&self.loaded, &now);
        self.loaded = now;
        ops
    }
}
