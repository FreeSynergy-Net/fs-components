//! K10-K17 of ENTWURF section 5: the check of bindings against the block, the type catalog and the shapes.
mod support;

use fs_components_catalog::{AcceptsCode, Binding, ContractRange, Provider, Role, Source, check};
use fs_types::CommandKind;
use semver::Version;
use support::tasks_board::{tasks_provider, tasks_range};
use support::tasks_catalog::TASK;
use support::util::*;

fn range() -> ContractRange {
    tasks_range().unwrap()
}

fn bind(slot: &str, source: Source) -> Binding {
    Binding::new(slot, &range(), TASK, source)
}

fn field(slot: &str, name: &str, role: Role) -> Binding {
    bind(
        slot,
        Source::Field {
            path: vec![name.to_owned()],
            role,
        },
    )
}

fn board_check(bindings: &[Binding]) -> Vec<String> {
    let c = catalog();
    show(&check(
        c.need("board").unwrap(),
        bindings,
        &[tasks_provider()],
    ))
}

/// K10: unknown slot, a slot bound twice, the fifth fact of four, a wrong kind.
#[test]
fn k10_slots() {
    let cards = bind("cards", Source::Records);
    let got = board_check(&[cards.clone(), bind("nope", Source::Records)]);
    assert!(
        got.contains(&"SlotUnknown[block=board,slot=nope]".to_owned()),
        "{got:?}"
    );
    let two = board_check(&[
        cards.clone(),
        field("card-title", "title", Role::Display),
        field("card-title", "title", Role::Display),
    ]);
    assert!(
        two.contains(&"BoundTwice[block=board,slot=card-title]".to_owned()),
        "{two:?}"
    );
    let facts: Vec<Binding> = std::iter::once(cards.clone())
        .chain((0..5).map(|_| field("card-facts", "due", Role::Compact)))
        .collect();
    let five = board_check(&facts);
    assert_eq!(
        five.iter().filter(|s| s.starts_with("BoundTwice")).count(),
        1,
        "{five:?}"
    );
    let four = board_check(&facts[..5]);
    assert!(
        four.iter().all(|s| !s.starts_with("BoundTwice")),
        "{four:?}"
    );
    let kind = board_check(&[cards, bind("card-title", Source::Grouping("board".into()))]);
    assert!(
        kind.contains(&"KindMismatch[block=board,slot=card-title,kind=grouping]".to_owned()),
        "{kind:?}"
    );
}

fn only_range(r: &str, providers: &[Provider]) -> Vec<String> {
    let c = catalog();
    let range = ContractRange::parse(r).unwrap();
    let b = Binding::new("cards", &range, TASK, Source::Records);
    let list = check(c.need("board").unwrap(), &[b], providers);
    show(&list)
        .into_iter()
        .filter(|s| !s.starts_with("SlotUnbound"))
        .collect()
}

/// K11: only 0.3 on offer for `>=0.2.0, <0.3.0`; the highest version in `>=0.2.0, <1.0.0` is chosen (0.2 has no `Task`, 0.5 has).
#[test]
fn k11_contract_missing() {
    let mut p3 = tasks_provider();
    p3.version = Version::new(0, 3, 0);
    assert_eq!(
        only_range("tasks@>=0.2.0, <0.3.0", &[p3]),
        ["ContractMissing[capability=tasks,range=tasks@>=0.2.0, <0.3.0,have=0.3.0]"]
    );
    let mut p2 = tasks_provider();
    p2.version = Version::new(0, 2, 0);
    p2.types.clear();
    let mut p5 = tasks_provider();
    p5.version = Version::new(0, 5, 0);
    assert_eq!(
        only_range("tasks@>=0.2.0, <1.0.0", &[p2, p5]),
        Vec::<String>::new()
    );
}

/// K12: unknown type, unknown field, a path of two steps.
#[test]
fn k12_type_and_field() {
    let cards = bind("cards", Source::Records);
    let nope = Binding::new("cards", &range(), "Nope", Source::Records);
    let got = board_check(&[nope]);
    assert!(
        got.contains(&"TypeUnknown[capability=tasks,type=Nope]".to_owned()),
        "{got:?}"
    );
    let got = board_check(&[cards.clone(), field("card-title", "nope", Role::Display)]);
    assert!(
        got.contains(&"FieldUnknown[type=Task,path=nope]".to_owned()),
        "{got:?}"
    );
    let two = Binding::new(
        "card-title",
        &range(),
        TASK,
        Source::Field {
            path: vec!["title".into(), "x".into()],
            role: Role::Display,
        },
    );
    let got = board_check(&[cards, two]);
    assert!(
        got.contains(&"FieldUnknown[type=Task,path=title/x]".to_owned()),
        "{got:?}"
    );
}

/// K13: wrong basic type, wrong role, `multiple` into a single slot.
#[test]
fn k13_basic_role_cardinality() {
    let cards = bind("cards", Source::Records);
    let got = board_check(&[cards.clone(), field("card-title", "due", Role::Display)]);
    assert!(
        got.contains(&"BasicMismatch[block=board,slot=card-title,basic=dateTime]".to_owned()),
        "{got:?}"
    );
    let got = board_check(&[cards.clone(), field("card-title", "title", Role::Compact)]);
    assert!(
        got.contains(&"RoleMismatch[block=board,slot=card-title,role=compact]".to_owned()),
        "{got:?}"
    );
    let blocks = "[block.t]\naccepts = [\n  { slot = \"r\", kind = \"records\" },\n  { slot = \"v\", kind = \"value\", basic = [\"any\"], role = \"display\", within = \"r\" },\n]\n";
    let c = catalog_of(blocks);
    let list = check(
        c.need("t").unwrap(),
        &[
            bind("r", Source::Records),
            field("v", "participants", Role::Display),
        ],
        &[tasks_provider()],
    );
    assert_eq!(
        show(&list),
        ["CardinalityMismatch[block=t,slot=v,path=participants]"]
    );
}

/// K14: an input slot refuses a read-only field.
#[test]
fn k14_read_only() {
    let c = catalog();
    let b = [
        bind("record", Source::Records),
        field("value", "uid", Role::Input),
    ];
    let list = check(c.need("form-field").unwrap(), &b, &[tasks_provider()]);
    assert_eq!(show(&list), ["FieldReadOnly[type=Task,path=uid]"]);
    let ok = [
        bind("record", Source::Records),
        field("value", "title", Role::Input),
    ];
    assert_eq!(
        check(c.need("form-field").unwrap(), &ok, &[tasks_provider()]).len(),
        0
    );
}

/// K15: names of grouping, order, command; a provider without `find` or without a shape is not listable.
#[test]
fn k15_names_and_listable() {
    let cards = bind("cards", Source::Records);
    for (slot, source) in [
        ("columns", Source::Grouping("nope".into())),
        ("order", Source::Order("nope".into())),
        ("move", Source::Command("nope".into())),
    ] {
        let got = board_check(&[cards.clone(), bind(slot, source)]);
        assert!(
            got.contains(&"NameUnknown[type=Task,name=nope]".to_owned()),
            "{slot}: {got:?}"
        );
    }
    let c = catalog();
    let mut no_find = tasks_provider();
    no_find.types[0]
        .commands
        .retain(|x| x.kind != CommandKind::Find);
    let mut no_shape = tasks_provider();
    no_shape.shapes.clear();
    for p in [no_find, no_shape] {
        let list = check(c.need("board").unwrap(), std::slice::from_ref(&cards), &[p]);
        assert!(show(&list).contains(&"NotListable[type=Task]".to_owned()));
    }
}

/// K16: a slot `within` `cards` must use the same contract range as `cards`.
#[test]
fn k16_within() {
    let other = ContractRange::parse("tasks@>=0.2.0, <0.4.0").unwrap();
    let title = Binding::new(
        "card-title",
        &other,
        TASK,
        Source::Field {
            path: vec!["title".into()],
            role: Role::Display,
        },
    );
    let got = board_check(&[bind("cards", Source::Records), title]);
    assert!(
        got.contains(&"WithinMismatch[block=board,slot=card-title,within=cards]".to_owned()),
        "{got:?}"
    );
}

/// K17: ALL problems come, in order, then the unbound required slots (never only the first).
#[test]
fn k17_all_problems() {
    let got = board_check(&[
        bind("cards", Source::Records),
        bind("nope", Source::Records),
        field("card-title", "due", Role::Display),
        bind("move", Source::Command("nope".into())),
    ]);
    assert_eq!(
        got,
        [
            "SlotUnknown[block=board,slot=nope]",
            "BasicMismatch[block=board,slot=card-title,basic=dateTime]",
            "NameUnknown[type=Task,name=nope]",
            "SlotUnbound[block=board,slot=columns]",
        ]
    );
    let _ = AcceptsCode::SlotUnbound;
}
