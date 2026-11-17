//! The translation of the filters (media types and patterns to portal filters) and of `file://` URIs to paths.
//!
//! Pattern: Model tests over pure functions (they run everywhere, portal or not). Mutants: media types not lower-cased; an invalid media type
//! kept; duplicates kept; a filter with nothing in it kept; `localhost` refused / another host accepted; `%2F`-style escapes
//! not decoded; a query part kept in the path.

use std::path::PathBuf;

use fs_components_filepicker::Filter;
use fs_components_filepicker::filters::{PortalFilter, to_portal};
use fs_components_filepicker::portal_uri::path_of_uri;

fn one(filter: &Filter) -> Option<PortalFilter> {
    to_portal(std::slice::from_ref(filter)).into_iter().next()
}

#[test]
fn media_types_and_patterns_become_one_portal_filter() {
    let f = Filter::new("Contacts")
        .pattern("*.vcf")
        .media_type("text/vcard")
        .media_type("application/jscontact+json");
    let p = one(&f).unwrap();
    assert_eq!(p.label, "Contacts");
    assert_eq!(p.globs, ["*.vcf"]);
    assert_eq!(p.media_types, ["text/vcard", "application/jscontact+json"]);
}

#[test]
fn media_types_are_lower_cased_and_must_look_like_a_media_type() {
    let p = one(&Filter::new("x")
        .media_type(" Text/VCard ")
        .media_type("nonsense")
        .media_type("a/"))
    .unwrap();
    assert_eq!(p.media_types, ["text/vcard"]);
}

#[test]
fn duplicates_go_and_order_stays() {
    let p = one(&Filter::new("x")
        .pattern("*.a")
        .pattern("*.b")
        .pattern("*.a")
        .media_type("text/a")
        .media_type("TEXT/A"))
    .unwrap();
    assert_eq!(p.globs, ["*.a", "*.b"]);
    assert_eq!(p.media_types, ["text/a"]);
}

#[test]
fn a_filter_with_nothing_in_it_is_dropped() {
    assert_eq!(one(&Filter::new("Empty")), None);
    assert_eq!(
        one(&Filter::new("Bad").media_type("nonsense").pattern("  ")),
        None
    );
}

#[test]
fn an_empty_name_falls_back_to_the_kinds() {
    let p = one(&Filter::new("  ").pattern("*.vcf").media_type("text/vcard")).unwrap();
    assert_eq!(p.label, "*.vcf, text/vcard");
}

#[test]
fn the_order_of_the_filters_is_kept() {
    let all = to_portal(&[
        Filter::new("B").pattern("*.b"),
        Filter::new("A").pattern("*.a"),
    ]);
    let names: Vec<&str> = all.iter().map(|f| f.label.as_str()).collect();
    assert_eq!(names, ["B", "A"]);
}

#[test]
fn local_file_uris_become_paths() {
    assert_eq!(
        path_of_uri("file:///home/x/a.vcf"),
        Some(PathBuf::from("/home/x/a.vcf"))
    );
    assert_eq!(
        path_of_uri("file://localhost/tmp/a"),
        Some(PathBuf::from("/tmp/a"))
    );
    assert_eq!(
        path_of_uri("file:///tmp/with%20space%C3%A4"),
        Some(PathBuf::from("/tmp/with spaceä"))
    );
    assert_eq!(
        path_of_uri("file:///tmp/a%3Fb%23c"),
        Some(PathBuf::from("/tmp/a?b#c"))
    );
    assert_eq!(
        path_of_uri("file:///tmp/a?query"),
        Some(PathBuf::from("/tmp/a"))
    );
}

#[test]
fn everything_else_is_refused_not_guessed() {
    for bad in [
        "http://example.org/a",
        "file://other-host/tmp/a",
        "file:relative",
        "file://",
        "file:///tmp/%zz",
        "file:///tmp/%4",
        "file:///tmp/%ff",
        "/plain/path",
        "",
    ] {
        assert_eq!(path_of_uri(bad), None, "{bad}");
    }
}
