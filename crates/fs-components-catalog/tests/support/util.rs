//! Small helpers of the tests: the draft data, parameters of codes, a catalog read from texts.
#![allow(dead_code)]

use fs_components_catalog::{AcceptsCode, BlockCatalog};
use fs_types::coded::{Coded, Param};

pub const CONTRACT: &str = include_str!("../data/contract.toml");
pub const BLOCKS: &str = include_str!("../data/blocks-accepts.toml");

/// The parameter `key` of a code as text (numbers as text).
pub fn param(c: &Coded<AcceptsCode>, key: &str) -> Option<String> {
    c.params().find(|(k, _)| *k == key).map(|(_, v)| match v {
        Param::Text(t) => t.clone(),
        Param::Number(n) => n.to_string(),
        _ => String::new(),
    })
}

/// `code[key=value,...]` of every problem, for readable asserts.
pub fn show(list: &[Coded<AcceptsCode>]) -> Vec<String> {
    list.iter()
        .map(|c| {
            let ps: Vec<String> = c
                .params()
                .map(|(k, _)| format!("{k}={}", param(c, k).unwrap_or_default()))
                .collect();
            format!("{:?}[{}]", c.code(), ps.join(","))
        })
        .collect()
}

pub fn codes(list: &[Coded<AcceptsCode>]) -> Vec<AcceptsCode> {
    list.iter().map(Coded::code).collect()
}

/// The 12 draft blocks as a catalog.
pub fn catalog() -> BlockCatalog {
    BlockCatalog::read(CONTRACT, &[("blocks-accepts.toml", BLOCKS)]).expect("draft data reads")
}

/// A catalog of one block text.
pub fn catalog_of(blocks: &str) -> BlockCatalog {
    BlockCatalog::read(CONTRACT, &[("t.toml", blocks)]).expect("test blocks read")
}
