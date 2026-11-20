//! The style sheet of the feed.
//!
//! Pattern: none — only theme variables (constants, no behaviour).

/// CSS of the feed: only theme variables. The article with the cursor has an edge (not colour alone: the edge is a line).
pub const STYLES: &str = r"
.fs-feed { display: flex; flex-direction: column; gap: var(--space-2); }
.fs-feed:focus { outline-width: 2px; outline-color: var(--border-focus); outline-offset: 2px; }
.fs-article { display: flex; flex-direction: column; gap: var(--space-1); padding: var(--space-3); border-radius: var(--radius-default); border-width: 2px; border-color: var(--border); background: var(--surface); }
.fs-article-title { color: var(--text-primary); font-size: var(--text-base); font-weight: 600; }
.fs-article-body { color: var(--text-primary); font-size: var(--text-sm); }
.fs-article-current { border-color: var(--border-focus); }
";
