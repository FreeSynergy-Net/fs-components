//! The text rules of the contract's plain edits, for editors that hold text: Backspace and the task box.
//!
//! Pattern: none — pure functions over a string, shared by the editors that edit the Markdown text itself (the source editor,
//! the block editor) so that they agree with the contract and with each other.

/// The text of a document of blocks (one blank line between) after Backspace at `at` bytes into the block that starts at `start`:
/// the new text and the byte offset of the cursor. At the start of a block (not the first) the blank line goes, so the block is
/// joined to the one above; elsewhere the character before goes. `None` when there is nothing before to remove.
#[must_use]
pub fn backspace_text(text: &str, start: usize, at: usize) -> Option<(String, usize)> {
    let place = (start + at).min(text.len());
    let from = if at == 0 {
        // The blank line between the blocks: two line breaks.
        (start >= 2 && text[..start].ends_with("\n\n")).then(|| start - 2)?
    } else {
        text[..place].char_indices().next_back()?.0
    };
    Some((format!("{}{}", &text[..from], &text[place..]), from))
}

/// The block with the task item on the line `at` is on ticked or unticked; `None` if that line is no task item.
#[must_use]
pub fn toggle_task_line(block: &str, at: usize) -> Option<String> {
    let at = at.min(block.len());
    let start = block[..at].rfind('\n').map_or(0, |i| i + 1);
    let line = &block[start..];
    let end = start + line.find('\n').unwrap_or(line.len());
    let line = &block[start..end];
    let indent = line.len() - line.trim_start().len();
    let rest = &line[indent..];
    let marker = ["- ", "* ", "+ "]
        .iter()
        .find(|m| rest.starts_with(*m))
        .map(|m| m.len())
        .or_else(|| {
            let digits = rest.chars().take_while(char::is_ascii_digit).count();
            (digits > 0 && rest[digits..].starts_with(". ")).then_some(digits + 2)
        })?;
    let box_at = start + indent + marker;
    let flipped = match block.get(box_at..box_at + 3)? {
        "[ ]" => "[x]",
        "[x]" | "[X]" => "[ ]",
        _ => return None,
    };
    Some(format!(
        "{}{flipped}{}",
        &block[..box_at],
        &block[box_at + 3..]
    ))
}
