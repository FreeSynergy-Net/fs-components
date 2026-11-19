//! The meter as an element.
//!
//! Pattern: Builder — described by chained calls, drawn by `build`; it holds no state after that.

use blinc_layout::div::div;
use blinc_layout::prelude::text;

use super::labels::Labels;
use super::zone::{Zone, zone};
use crate::Element;
use crate::slider::{Scale, number};

/// A meter.
#[must_use]
pub struct Meter {
    id: String,
    name: String,
    scale: Scale,
    value: f64,
    cuts: Option<(f64, f64, f64)>,
    labels: Labels,
}

impl Meter {
    /// A meter named `name` ("Disk space") showing `value` on `scale` (the step does not matter here).
    pub fn new(scale: Scale, name: &str, value: f64) -> Self {
        Self {
            id: "meter".to_owned(),
            name: name.to_owned(),
            scale,
            value: value.clamp(scale.min(), scale.max()),
            cuts: None,
            labels: Labels::default(),
        }
    }

    /// The prefix of the ids: `<id>`, `<id>-fill`.
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Cut the range at `low` and `high` and name the best place `optimum`; without it the zone is always good.
    pub fn zones(mut self, low: f64, high: f64, optimum: f64) -> Self {
        self.cuts = Some((low, high, optimum));
        self
    }

    /// The words of the zones.
    pub fn labels(mut self, labels: &Labels) -> Self {
        self.labels = labels.clone();
        self
    }

    /// The zone of the value now.
    pub fn zone(&self) -> Zone {
        self.cuts.map_or(Zone::Good, |(low, high, optimum)| {
            zone(self.value, low, high, optimum)
        })
    }

    /// The meter, `width` pixels wide for the bar, to place in a layout.
    pub fn build(&self, width: f32) -> impl Element + use<> {
        let zone = self.zone();
        let word = self.labels.of(zone).to_owned();
        #[allow(clippy::cast_possible_truncation)]
        let fill = self.scale.fraction(self.value) as f32 * width;
        let class = match zone {
            Zone::Good => "fs-meter-good",
            Zone::Fair => "fs-meter-fair",
            Zone::Poor => "fs-meter-poor",
        };
        div()
            .id(&self.id)
            .class("fs-meter")
            .class(class)
            .attr("role", "meter")
            .attr("aria-label", &self.name)
            .attr("aria-valuenow", number(self.value))
            .attr("aria-valuemin", number(self.scale.min()))
            .attr("aria-valuemax", number(self.scale.max()))
            .attr("aria-valuetext", format!("{}, {word}", number(self.value)))
            .child(
                div().class("fs-meter-track").w(width).child(
                    div()
                        .id(format!("{}-fill", self.id))
                        .class("fs-meter-fill")
                        .w(fill),
                ),
            )
            .child(text(format!("{} ({word})", number(self.value))).class("fs-meter-text"))
    }
}
