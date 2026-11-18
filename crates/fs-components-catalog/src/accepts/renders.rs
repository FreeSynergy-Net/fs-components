//! What a building block RENDERS (block contract 0.4, key `renders`): the basic type, the optional targets of a reference, and
//! the roles it draws. The catalog chooses the block of a field from these entries ([`super::BlockCatalog::choose`]).
//!
//! Pattern: Value object + Specification (`clash`: two entries that would both claim the same field in the same role).
//! The words are the words of the files (closed schema; an unknown key, word or role is a problem, never ignored).

use fs_types::BasicType;
use toml::{Table, Value};

use super::codes::{AcceptsCode, keys};
use super::slot::{Role, basic_from_key};
use fs_types::coded::Coded;

/// The keys `renders` may have.
const RENDERS_KEYS: [&str; 3] = ["basic", "target", "roles"];

/// One `renders` entry of a block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Renders {
    /// The basic type the block draws.
    pub basic: BasicType,
    /// For a `reference`: the data types it draws; empty = any target (the general block of the type).
    pub target: Vec<String>,
    /// The roles it draws (every role it is chosen for).
    pub roles: Vec<Role>,
}

impl Renders {
    /// Whether this entry draws `role`.
    #[must_use]
    pub fn has_role(&self, role: Role) -> bool {
        self.roles.contains(&role)
    }

    /// Whether this entry names `target` (an empty `target` names none, it is the general one).
    #[must_use]
    pub fn names_target(&self, target: &str) -> bool {
        !target.is_empty() && self.target.iter().any(|t| t == target)
    }

    /// Whether two entries claim the same field in `role`: same basic type, and both general or sharing a target.
    #[must_use]
    pub fn clash(&self, other: &Self, role: Role) -> bool {
        self.basic == other.basic
            && self.has_role(role)
            && other.has_role(role)
            && ((self.target.is_empty() && other.target.is_empty())
                || self.target.iter().any(|t| other.target.contains(t)))
    }
}

/// The `renders` of one block body (`None` = the block has none, which is allowed); problems are pushed.
pub(super) fn read_renders(
    block: &str,
    body: &Table,
    problems: &mut Vec<Coded<AcceptsCode>>,
) -> Option<Renders> {
    let value = body.get("renders")?;
    let bad = |field: &str| {
        Coded::new(AcceptsCode::RendersInvalid)
            .with_text(keys::BLOCK, block)
            .with_text(keys::FIELD, field)
    };
    let Some(t) = value.as_table() else {
        problems.push(bad("renders"));
        return None;
    };
    let before = problems.len();
    problems.extend(
        t.keys()
            .filter(|k| !RENDERS_KEYS.contains(&k.as_str()))
            .map(|k| bad(k)),
    );
    let basic = if let Some(word) = t.get("basic").and_then(Value::as_str) {
        let found = basic_from_key(word);
        if found.is_none() {
            problems.push(
                Coded::new(AcceptsCode::RendersUnknownBasic)
                    .with_text(keys::BLOCK, block)
                    .with_text(keys::WORD, word),
            );
        }
        found
    } else {
        problems.push(bad("basic"));
        None
    };
    let target = words(t.get("target"), true);
    if target.is_none() {
        problems.push(bad("target"));
    }
    // `words` refuses a duplicate (the one place: test `c7b_a_duplicate_role_is_refused`); the role words are then unique.
    let roles: Option<Vec<Role>> = words(t.get("roles"), false)
        .and_then(|w| w.iter().map(|word| Role::from_key(word)).collect());
    if roles.is_none() {
        problems.push(bad("roles"));
    }
    if problems.len() != before {
        return None;
    }
    Some(Renders {
        basic: basic?,
        target: target?,
        roles: roles?,
    })
}

/// A list of words: absent = empty (when `may_be_absent`), otherwise a non-empty list of unique strings.
fn words(value: Option<&Value>, may_be_absent: bool) -> Option<Vec<String>> {
    let Some(v) = value else {
        return may_be_absent.then(Vec::new);
    };
    let list: Vec<String> = v
        .as_array()?
        .iter()
        .map(|x| x.as_str().map(str::to_owned))
        .collect::<Option<_>>()?;
    let unique = list.iter().enumerate().all(|(i, a)| !list[..i].contains(a));
    (unique && !list.is_empty()).then_some(list)
}
