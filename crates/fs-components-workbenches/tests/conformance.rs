//! The conformance collection of the editor contract, run against every building block of this crate.
//!
//! Tests: model tests, no window: the text areas of the editors are made and edited in memory.

use fs_components_workbenches::contract::{TextEditor, conformance};
use fs_components_workbenches::{BlockEditor, SourceEditor};

#[test]
fn the_source_editor_passes_the_collection() {
    conformance::check(&|| -> Box<dyn TextEditor> { Box::new(SourceEditor::new()) });
}

#[test]
fn the_block_editor_passes_the_collection() {
    conformance::check(&|| -> Box<dyn TextEditor> { Box::new(BlockEditor::new(1, "", "")) });
}

#[test]
fn the_source_editor_with_a_language_passes_the_collection() {
    use fs_components_editor::language::FakeLanguage;
    conformance::check(&|| -> Box<dyn TextEditor> {
        Box::new(SourceEditor::new().with_language(std::sync::Arc::new(FakeLanguage)))
    });
}
