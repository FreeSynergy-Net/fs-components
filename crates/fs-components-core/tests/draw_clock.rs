//! Pattern: none (test). `DrawCtx` carries the clock: the system clock without a host, the injected one when the host says so.

use std::sync::Arc;

use fs_components_core::clock::{Clock, FixedClock};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, PlaceInfo};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_words::Locale;

fn input() -> FormInput {
    FormInput {
        form: "x@1.0.0".to_owned(),
        rows: Vec::new(),
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx() -> DrawCtx {
    DrawCtx::new(
        PlaceInfo::default(),
        (StyleId::Admin, false),
        Arc::new(|k: &str| k.to_owned()),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(input),
    )
}

#[test]
fn without_a_host_clock_the_context_reads_the_system_clock() {
    // 2026-01-01 is day 20454; the system clock of any machine running this is later.
    assert!(ctx().today_days() >= 20_454, "{}", ctx().today_days());
}

#[test]
fn an_injected_fixed_clock_gives_its_day_whatever_the_system_says() {
    let c = ctx().with_clock(Arc::new(FixedClock::at_day(20_402)));
    assert_eq!(c.today_days(), 20_402);
    assert_eq!(c.clock.now_unix(), 20_402 * 86_400 + 43_200);
}

#[test]
fn the_day_before_1970_rounds_down_not_towards_zero() {
    assert_eq!(FixedClock::new(-1).today_days(), -1);
    assert_eq!(FixedClock::new(86_399).today_days(), 0);
    assert_eq!(FixedClock::new(86_400).today_days(), 1);
}
