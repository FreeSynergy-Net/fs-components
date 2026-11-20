//! Which tokens a block of contract 0.2 reads.
//!
//! Pattern: none (one constant table; the names are those of `block_tokens::NAMES`).

/// Which tokens a block of 0.2 reads: `(block of the contract, tokens)`. A block of 0.2 that is not here, or a token that no
/// style gives, is red in `tests/block_tokens.rs`.
pub const BLOCK_TOKENS: &[(&str, &[&str])] = &[
    (
        "dialog",
        &[
            "scrim",
            "dialog-width",
            "dialog-shadow",
            "dialog-radius",
            "dialog-pad",
            "dialog-danger-edge",
            "dialog-head-line",
            "dialog-head-gap",
            "danger-bg",
            "danger-bg-hover",
            "danger-bg-active",
            "danger-text",
            "danger-line",
            "danger-line-width",
        ],
    ),
    (
        "confirm-dialog",
        &[
            "dialog-width",
            "dialog-shadow",
            "dialog-danger-edge",
            "dialog-head-line",
            "dialog-head-gap",
            "danger-bg",
            "danger-bg-hover",
            "danger-bg-active",
            "danger-text",
            "danger-line",
            "danger-line-width",
        ],
    ),
    ("choice-dialog", &["dialog-width", "dialog-shadow"]),
    ("input-dialog", &["dialog-width", "dialog-shadow"]),
    ("license-dialog", &["dialog-width", "dialog-shadow"]),
    (
        "toast",
        &[
            "toast-width",
            "toast-edge",
            "dialog-shadow",
            "message-radius",
            "message-pad",
            "message-tint",
        ],
    ),
    (
        "alert",
        &[
            "toast-edge",
            "message-radius",
            "message-pad",
            "message-tint",
            "alert-info-bg",
            "alert-ok-bg",
            "alert-warning-bg",
            "alert-error-bg",
        ],
    ),
    (
        "partial-notice",
        &[
            "toast-edge",
            "message-radius",
            "message-pad",
            "message-tint",
            "alert-warning-bg",
        ],
    ),
    ("empty-state", &[]),
    (
        "skeleton",
        &["skeleton-base", "skeleton-radius", "skeleton-height"],
    ),
    (
        "switch",
        &[
            "switch-width",
            "switch-height",
            "switch-on-track",
            "switch-on-thumb",
            "switch-off-track",
            "switch-off-thumb",
            "switch-disabled-opacity",
        ],
    ),
    (
        "segmented",
        &[
            "segment-height",
            "segment-pad",
            "segment-on-bg",
            "segment-on-line",
            "segment-on-line-width",
            "segment-on-text",
        ],
    ),
];
