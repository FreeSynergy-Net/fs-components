//! The typed block of the rich editor and its two-way link to the Markdown source of one block of a [`crate::Doc`].
//!
//! Pattern: Adapter — a block of Markdown source becomes a typed block (kind plus plain text) for editing; it becomes
//! Markdown source again on the way out. A block the editor has not touched keeps its source and is given back byte for
//! byte; a block it cannot edit as a typed block is [`Kind::Raw`] and stays text.

/// What a block is. The kind is a property of the block, not a mark: the heading level and the task box are not characters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Running text.
    Paragraph,
    /// A heading of level 1 to 6.
    Heading(u8),
    /// A bullet list item.
    Bullet,
    /// A numbered list item with its number.
    Numbered(u32),
    /// A task item: `true` when it is done.
    Task(bool),
    /// A quotation.
    Quote,
    /// A fenced code block with its language hint (may be empty); the text holds the lines between the fences.
    Code(String),
    /// A horizontal rule.
    Rule,
    /// Something the rich editor does not edit as a typed block (a table, a multi-line list): shown and kept as source text.
    Raw,
}

/// One block: its kind, its plain text, and the source it came from while nobody has changed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichBlock {
    /// What the block is.
    pub kind: Kind,
    /// The text without the Markdown marker (for [`Kind::Raw`] the whole source).
    pub text: String,
    /// The Markdown source the block was read from; `Some` only while the block is unchanged.
    pub(crate) source: Option<String>,
}

impl RichBlock {
    /// A new block of `kind` holding `text`, with no source (it is written in the canonical form).
    #[must_use]
    pub fn new(kind: Kind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
            source: None,
        }
    }

    /// Read one block of Markdown source (without the blank lines around it).
    #[must_use]
    pub fn parse(source: &str) -> Self {
        let keep = |kind, text: &str| Self {
            kind,
            text: text.to_owned(),
            source: Some(source.to_owned()),
        };
        if let Some((lang, code)) = fenced(source) {
            return keep(Kind::Code(lang), &code);
        }
        if source.contains('\n') {
            let marked = source
                .lines()
                .any(|l| marker(l).is_some() || is_rule(l) || l.starts_with('|'));
            return keep(if marked { Kind::Raw } else { Kind::Paragraph }, source);
        }
        if is_rule(source) {
            return keep(Kind::Rule, "");
        }
        match marker(source) {
            Some((kind, len)) => keep(kind, &source[len..]),
            None if source.starts_with('|') => keep(Kind::Raw, source),
            None => keep(Kind::Paragraph, source),
        }
    }

    /// The marker the canonical form writes before the text (for code the opening fence and its line break).
    fn prefix(&self) -> String {
        match &self.kind {
            Kind::Paragraph | Kind::Raw | Kind::Rule => String::new(),
            Kind::Heading(n) => format!("{} ", "#".repeat(usize::from(*n))),
            Kind::Bullet => "- ".to_owned(),
            Kind::Numbered(n) => format!("{n}. "),
            Kind::Task(done) => format!("- [{}] ", if *done { 'x' } else { ' ' }),
            Kind::Quote => "> ".to_owned(),
            Kind::Code(lang) => format!("```{lang}\n"),
        }
    }

    /// The Markdown source of the block: the original while unchanged, else the canonical form of its kind.
    #[must_use]
    pub fn to_source(&self) -> String {
        if let Some(s) = &self.source {
            return s.clone();
        }
        match &self.kind {
            Kind::Rule => "---".to_owned(),
            Kind::Code(_) => format!("{}{}\n```", self.prefix(), self.text),
            _ => format!("{}{}", self.prefix(), self.text),
        }
    }

    /// The length in bytes of what the source shows before the text (an offset in the source minus this is an offset in the text).
    #[must_use]
    pub fn marker_len(&self) -> usize {
        match (&self.source, &self.kind) {
            (_, Kind::Paragraph | Kind::Raw | Kind::Rule) => 0,
            (Some(s), Kind::Code(_)) => s.find('\n').map_or(0, |n| n + 1),
            (Some(s), _) => s.len().saturating_sub(self.text.len()),
            (None, _) => self.prefix().len(),
        }
    }
}

fn is_rule(line: &str) -> bool {
    let t = line.trim();
    t.len() >= 3
        && ["-", "*", "_"]
            .iter()
            .any(|c| t.chars().all(|x| c.starts_with(x)))
}

/// The marker at the start of a one-line block: its kind and its length in bytes.
fn marker(line: &str) -> Option<(Kind, usize)> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if (1..=6).contains(&hashes) && line[hashes..].starts_with(' ') {
        return Some((Kind::Heading(u8::try_from(hashes).ok()?), hashes + 1));
    }
    for (head, done) in [
        ("- [ ] ", false),
        ("* [ ] ", false),
        ("- [x] ", true),
        ("- [X] ", true),
        ("* [x] ", true),
    ] {
        if line.starts_with(head) {
            return Some((Kind::Task(done), head.len()));
        }
    }
    if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
        return Some((Kind::Bullet, 2));
    }
    if line.starts_with("> ") {
        return Some((Kind::Quote, 2));
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    if digits > 0 && digits < 10 && line[digits..].starts_with(". ") {
        return Some((Kind::Numbered(line[..digits].parse().ok()?), digits + 2));
    }
    None
}

/// A source that is one fenced code block: the language hint and the lines between the fences.
fn fenced(source: &str) -> Option<(String, String)> {
    let first = source.lines().next()?;
    let fence = ["```", "~~~"].into_iter().find(|f| first.starts_with(f))?;
    let rest = source[first.len()..].strip_prefix('\n').unwrap_or("");
    let body = rest
        .strip_suffix(fence)
        .and_then(|b| b.strip_suffix('\n').or(Some(b).filter(|b| b.is_empty())))?;
    Some((first[fence.len()..].trim().to_owned(), body.to_owned()))
}
