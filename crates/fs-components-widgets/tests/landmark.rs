//! `landmark`: the role of each kind, the name, the rule that a region and a form need one, the page check (once, told apart).
//!
//! Tests: model tests over the rule table and `problems`; attribute reads on the element.

mod common;

use blinc_layout::div::div;
use common::{attr, layout, serial};
use fs_components_widgets::landmark::{Landmark, Problem, problems};

const ALL: [(Landmark, &str); 8] = [
    (Landmark::Banner, "banner"),
    (Landmark::Navigation, "navigation"),
    (Landmark::Main, "main"),
    (Landmark::Aside, "complementary"),
    (Landmark::Footer, "contentinfo"),
    (Landmark::Search, "search"),
    (Landmark::Form, "form"),
    (Landmark::Region, "region"),
];

#[test]
fn every_kind_has_its_aria_role() {
    for (kind, role) in ALL {
        assert_eq!(kind.role(), role);
    }
}

#[test]
fn an_element_carries_the_role_and_the_name() {
    let _g = serial();
    let named = Landmark::Navigation.element(Some("Places")).expect("named");
    let tree = layout(div().child(named.id("nav")), 200.0, 100.0);
    assert_eq!(attr(&tree, "nav", "role").as_deref(), Some("navigation"));
    assert_eq!(attr(&tree, "nav", "aria-label").as_deref(), Some("Places"));
    let bare = Landmark::Main.element(None).expect("main needs no name");
    let tree = layout(div().child(bare.id("m")), 200.0, 100.0);
    assert_eq!(attr(&tree, "m", "role").as_deref(), Some("main"));
    assert_eq!(attr(&tree, "m", "aria-label"), None);
}

#[test]
fn a_region_and_a_form_are_landmarks_only_with_a_name() {
    assert!(Landmark::Region.element(None).is_none());
    assert!(Landmark::Region.element(Some("  ")).is_none(), "blanks are no name");
    assert!(Landmark::Form.element(None).is_none());
    assert!(Landmark::Region.element(Some("Filters")).is_some());
    for (kind, _) in ALL {
        assert_eq!(kind.needs_name(), matches!(kind, Landmark::Form | Landmark::Region));
        assert_eq!(kind.once(), matches!(kind, Landmark::Banner | Landmark::Main | Landmark::Footer));
    }
}

#[test]
fn a_good_page_has_no_problem() {
    let page = [
        (Landmark::Banner, None),
        (Landmark::Navigation, Some("Places")),
        (Landmark::Navigation, Some("Path")),
        (Landmark::Main, None),
        (Landmark::Region, Some("Filters")),
    ];
    assert_eq!(problems(&page), Vec::<Problem>::new());
}

#[test]
fn a_nameless_region_two_mains_and_two_unnamed_navigations_are_found() {
    assert_eq!(problems(&[(Landmark::Region, None)]), [Problem::NameMissing(Landmark::Region)]);
    assert_eq!(
        problems(&[(Landmark::Main, None), (Landmark::Main, None)]),
        [Problem::NotOnce(Landmark::Main)]
    );
    assert_eq!(
        problems(&[(Landmark::Navigation, None), (Landmark::Navigation, Some("Path"))]),
        [Problem::NotTold(Landmark::Navigation)]
    );
    assert_eq!(
        problems(&[(Landmark::Aside, Some("Help")), (Landmark::Aside, Some("Help"))]),
        [Problem::NotTold(Landmark::Aside)],
        "the same name twice does not tell them apart"
    );
    assert!(problems(&[(Landmark::Aside, None)]).is_empty(), "one unnamed side is fine");
}
