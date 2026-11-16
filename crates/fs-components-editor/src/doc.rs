//! The document the editors work on: blocks of Markdown source.
//!
//! Pattern: none — plain data. A block is a run of lines between blank lines; a fenced code block is one block whatever
//! it holds. Text goes in and out in one canonical form: one blank line between blocks, one line break at the end.

/// A document: its blocks, each the Markdown source of one block, without the blank lines around it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Doc {
    /// The blocks in order.
    pub blocks: Vec<String>,
}

/// The fence at the start of `line`: its character (backtick or tilde) and the length of its run, three at least.
fn fence_run(line: &str) -> Option<(char, usize)> {
    let marker = line.trim_start();
    let c = marker.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let n = marker.chars().take_while(|&x| x == c).count();
    (n >= 3).then_some((c, n))
}

impl Doc {
    /// A document of these blocks.
    #[must_use]
    pub fn new(blocks: Vec<String>) -> Self {
        Self { blocks }
    }

    /// Cut `text` into blocks at blank lines; a fenced code block is one block whatever it contains.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut out: Vec<String> = Vec::new();
        let mut current: Vec<&str> = Vec::new();
        let mut fence: Option<(char, usize)> = None;
        for line in text.lines() {
            let tick = fence_run(line);
            match (fence, tick) {
                (None, Some(open)) => fence = Some(open),
                (Some((c, n)), Some((tc, tn))) if c == tc && tn >= n => fence = None,
                _ => {}
            }
            if fence.is_none() && line.trim().is_empty() && tick.is_none() {
                if !current.is_empty() {
                    out.push(current.join("\n"));
                    current.clear();
                }
            } else {
                current.push(line);
            }
        }
        if !current.is_empty() {
            out.push(current.join("\n"));
        }
        Self { blocks: out }
    }

    /// The blocks put together again: one blank line between, a line break at the end (none for no blocks).
    #[must_use]
    pub fn to_markdown(&self) -> String {
        if self.blocks.is_empty() {
            return String::new();
        }
        let mut text = self.blocks.join("\n\n");
        text.push('\n');
        text
    }

    /// Is `text` in the canonical form, so that it goes through [`Doc::parse`] and [`Doc::to_markdown`] unchanged?
    #[must_use]
    pub fn is_canonical(text: &str) -> bool {
        Self::parse(text).to_markdown() == text
    }
}

#[cfg(test)]
mod tests {
    use super::Doc;

    fn blocks(v: &[&str]) -> Vec<String> {
        v.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn a_longer_fence_is_not_closed_by_a_shorter_line() {
        let saved = Doc::new(blocks(&["````\na\n```\n````", "b", "## c"]));
        let loaded = Doc::parse(&saved.to_markdown());
        assert_eq!(loaded, saved, "three blocks, the heading kept");
    }

    #[test]
    fn a_fence_is_closed_by_its_own_character_only() {
        let saved = Doc::new(blocks(&["~~~\na\n```\n\nb\n~~~", "## c"]));
        assert_eq!(Doc::parse(&saved.to_markdown()), saved);
        let plain = Doc::new(blocks(&["```\na\n\nb\n```", "## c"]));
        assert_eq!(Doc::parse(&plain.to_markdown()), plain);
    }
}
