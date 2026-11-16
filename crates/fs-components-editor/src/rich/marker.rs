//! The marker rules of the reader and the writer: what a line starts with (heading, list, rule, fence, HTML block, task box)
//! and the escapes that keep a paragraph a paragraph.
//!
//! Pattern: Strategy — small pure predicates that tell the reader and the writer how a line is to be taken.

/// A thematic break: three or more of one of `-`, `*`, `_`, with spaces or tabs between them (`_ _ _`, `* * *`, `- - -`).
pub(super) fn is_rule(line: &str) -> bool {
    let mut marks = line.chars().filter(|c| !matches!(c, ' ' | '\t'));
    let Some(first) = marks.next().filter(|c| matches!(c, '-' | '*' | '_')) else {
        return false;
    };
    let rest: Vec<char> = marks.collect();
    rest.len() >= 2 && rest.iter().all(|&c| c == first)
}

/// The tags that start an HTML block of kind 6 (`CommonMark` 4.6), and the four of kind 1.
const HTML_BLOCK_TAGS: &[&str] = &[
    "address",
    "article",
    "aside",
    "base",
    "basefont",
    "blockquote",
    "body",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dialog",
    "dir",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hr",
    "html",
    "iframe",
    "legend",
    "li",
    "link",
    "main",
    "menu",
    "menuitem",
    "nav",
    "noframes",
    "ol",
    "optgroup",
    "option",
    "p",
    "param",
    "search",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
    "script",
    "pre",
    "style",
    "textarea",
];

/// Does `line` start an HTML block of kind 1 to 6 (`<!--`, `<?`, `<!X`, `<![CDATA[`, `<script`, `<pre`, `<style`, `<textarea`,
/// a tag of the block list, open or closing)? Kind 7 (any complete tag alone on a line) is not asked: it is the same as an
/// inline tag in the text and passes through on purpose.
pub(super) fn html_block_start(line: &str) -> bool {
    let Some(rest) = line.strip_prefix('<') else {
        return false;
    };
    if rest.starts_with(['!', '?']) {
        return true;
    }
    let rest = rest.strip_prefix('/').unwrap_or(rest);
    let name: String = rest
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_ascii_lowercase();
    let after = &rest[name.len()..];
    HTML_BLOCK_TAGS.contains(&name.as_str())
        && (after.is_empty() || after.starts_with([' ', '\t', '>', '/']))
}

/// A link reference definition may start here (`[a]: b`): a line that opens a bracket and closes it with a colon.
pub(super) fn link_definition(line: &str) -> bool {
    line.starts_with('[') && line.contains("]:")
}

/// A closing sequence of `#` at the end of a heading text (`x ##`), which a reader takes off: the number of backslashes in
/// front of the last `#` when the line would have one (`x ##` is 0, `x #\#` is 1).
pub(super) fn closing_hashes(text: &str) -> Option<usize> {
    let rest = text.trim_end_matches([' ', '\t']).strip_suffix('#')?;
    let slashes = rest.chars().rev().take_while(|&c| c == '\\').count();
    let before = rest[..rest.len() - slashes].trim_end_matches('#');
    (before.is_empty() || before.ends_with([' ', '\t'])).then_some(slashes)
}

/// The heading text with one more backslash in front of its last `#` when it ends like a closing sequence, so that no reader
/// takes the `#` off.
pub(super) fn mask_closing(text: &str) -> String {
    if closing_hashes(text).is_none() {
        return text.to_owned();
    }
    let at = text.trim_end_matches([' ', '\t']).len() - 1;
    format!("{}\\{}", &text[..at], &text[at..])
}

/// The inverse of [`mask_closing`].
pub(super) fn unmask_closing(text: &str) -> String {
    match closing_hashes(text) {
        Some(k) if k > 0 => {
            let at = text.trim_end_matches([' ', '\t']).len() - 1;
            format!("{}{}", &text[..at - 1], &text[at..])
        }
        _ => text.to_owned(),
    }
}

/// The heading a line is: level and the length of the marker.
pub(super) fn heading(line: &str) -> Option<(u8, usize)> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    let after = &line[hashes..];
    ((1..=6).contains(&hashes) && marker_end(after)).then(|| {
        (
            u8::try_from(hashes).unwrap_or(6),
            hashes + usize::from(!after.is_empty()),
        )
    })
}

/// Does a marker end here: a space, a tab or the end of the line (`CommonMark` takes `#`, `-` and `1.` alone on a line and
/// followed by a tab as a marker as well)?
pub(super) fn marker_end(after: &str) -> bool {
    after.is_empty() || after.starts_with([' ', '\t'])
}

/// The list marker a line starts with: ordered, the number, and the length of the marker.
pub(super) fn list_marker(line: &str) -> Option<(bool, u32, usize)> {
    if line.starts_with(['-', '*', '+']) && marker_end(&line[1..]) {
        return Some((false, 1, 1 + usize::from(line.len() > 1)));
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    let after = &line[digits..];
    (digits > 0 && digits < 10 && after.starts_with(['.', ')']) && marker_end(&after[1..])).then(
        || {
            let gap = usize::from(after.len() > 1);
            (true, line[..digits].parse().unwrap_or(1), digits + 1 + gap)
        },
    )
}

/// The task box at the start of an item's text: its state and the length (with the space after it).
pub(super) fn task_box(text: &str) -> Option<(bool, usize)> {
    [("[ ] ", false), ("[x] ", true), ("[X] ", true)]
        .iter()
        .find_map(|(b, done)| text.starts_with(b).then_some((*done, b.len())))
}

/// Would `line` be read as something other than a paragraph at the start of a block (heading, list, quote, table, rule, fence,
/// task box)?
pub(super) fn is_block_marker(line: &str) -> bool {
    heading(line).is_some()
        || list_marker(line).is_some()
        || line.starts_with(['>', '|'])
        || is_rule(line)
        || ["```", "~~~"].iter().any(|f| line.starts_with(f))
        || task_box(line).is_some()
        || html_block_start(line)
        || link_definition(line)
}

/// Is `line` the text of a paragraph that begins with a backslash in front of what would be a marker (`\## x`)? The writer puts
/// that backslash there so that the text stays a paragraph; `escaped(line, f)` asks for the marker kind `f`.
pub(super) fn escaped(line: &str, marker: fn(&str) -> bool) -> bool {
    line.strip_prefix('\\')
        .is_some_and(|rest| marker(rest.trim_start_matches('\\')))
}

/// The number a line starts with, written with `k` backslashes in front of the dot or bracket (`1. x`, `1\. x`, `1\\. x`,
/// `1) x`, `1.` alone, `1.<tab>x`): the length of the digits and `k`. The writer keeps a number from being a list marker by one more backslash there, which is the valid
/// `CommonMark` escape (`1\. x`; a backslash in front of the digit is none).
pub(super) fn number_escape(line: &str) -> Option<(usize, usize)> {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    let k = line[digits..].chars().take_while(|&c| c == '\\').count();
    let after = &line[digits + k..];
    (digits > 0 && digits < 10 && after.starts_with(['.', ')']) && marker_end(&after[1..]))
        .then_some((digits, k))
}

/// The fence a line opens: its run of three or more backticks or tildes (the closing line is exactly that run).
pub(super) fn fence(line: &str) -> Option<&str> {
    let c = line.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let n = line.chars().take_while(|&x| x == c).count();
    (n >= 3).then(|| &line[..n])
}
