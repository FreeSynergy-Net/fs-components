//! The pure part of the help: the block, the source, the chain and the outcome. No Blinc here.
//!
//! Pattern: Strategy + Chain of Responsibility (see `mod.rs`).

use std::sync::Arc;

/// One block of help text as the interface `texts` answers it (ADR 0072): plain text, no executable content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelpBlock {
    /// The stable id of the block.
    pub id: String,
    /// The language the text is in (BCP 47).
    pub lang: String,
    /// The text; paragraphs are separated by an empty line.
    pub text: String,
    /// The edition of the block (higher wins when two sources hold the same block).
    pub rev: u32,
    /// Set when the block is NOT in the first language asked for: the language that was wished.
    pub fallback_from: Option<String>,
    /// A translation older than its original (ADR 0057): shown as such, not hidden.
    pub outdated: bool,
}

/// A source that cannot answer (the program with the texts is not installed, it does not reply).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelpError(pub String);

/// Where help blocks come from: the host's adapter to the interface `texts`, a copy in the package, the catalog.
pub trait HelpSource: Send + Sync {
    /// The block of `purpose` (`help.<topic>`) of `program` in the first of `langs` that has one; `Ok(None)` when the source answers
    /// and has none, `Err` when it cannot answer at all.
    ///
    /// # Errors
    /// [`HelpError`] when the source is not there or does not reply.
    fn block(
        &self,
        program: &str,
        purpose: &str,
        langs: &[String],
    ) -> Result<Option<HelpBlock>, HelpError>;
}

/// What the person gets for a topic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// An entry with content.
    Found(HelpBlock),
    /// The source answered and has no entry (or an empty one): the fallback sentence is shown.
    Missing,
    /// No source answered: the fallback sentence is shown.
    Unavailable(String),
}

/// The purpose key of a topic (`help.versions`), the vocabulary of ADR 0072.
pub fn purpose_of(topic: &str) -> String {
    format!("help.{topic}")
}

/// Ask `source` for `topic` and say what came back; never "nothing".
pub fn resolve(source: &dyn HelpSource, program: &str, topic: &str, langs: &[String]) -> Outcome {
    match source.block(program, &purpose_of(topic), langs) {
        Ok(Some(block)) if !paragraphs(&block.text).is_empty() => Outcome::Found(block),
        Ok(_) => Outcome::Missing,
        Err(error) => Outcome::Unavailable(error.0),
    }
}

/// The sources in the order of ADR 0072 point 4: the first answer with the highest `rev` wins. A source that cannot answer is
/// skipped; when none can, the chain cannot either.
#[derive(Clone, Default)]
pub struct Chain(Vec<Arc<dyn HelpSource>>);

impl Chain {
    /// An empty chain (it answers `Ok(None)`: nothing to read from).
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a source behind the ones already there.
    #[must_use]
    pub fn then(mut self, source: Arc<dyn HelpSource>) -> Self {
        self.0.push(source);
        self
    }
}

impl HelpSource for Chain {
    fn block(
        &self,
        program: &str,
        purpose: &str,
        langs: &[String],
    ) -> Result<Option<HelpBlock>, HelpError> {
        let mut best: Option<HelpBlock> = None;
        let (mut answered, mut failed) = (false, None);
        for source in &self.0 {
            match source.block(program, purpose, langs) {
                Ok(Some(block)) => {
                    answered = true;
                    if best.as_ref().is_none_or(|b| block.rev > b.rev) {
                        best = Some(block);
                    }
                }
                Ok(None) => answered = true,
                Err(error) => {
                    failed.get_or_insert(error);
                }
            }
        }
        match (best, answered, failed) {
            (Some(block), _, _) => Ok(Some(block)),
            (None, false, Some(error)) => Err(error),
            _ => Ok(None),
        }
    }
}

/// The paragraphs of a block: control characters (except a line break) removed, split at empty lines, trimmed, none empty. A block
/// is text, so nothing in it is ever run or drawn as anything else (ADR 0072 point 4).
pub fn paragraphs(text: &str) -> Vec<String> {
    let clean: String = text
        .chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .collect();
    clean
        .replace("\r\n", "\n")
        .split("\n\n")
        .map(|p| p.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|p| !p.is_empty())
        .collect()
}
