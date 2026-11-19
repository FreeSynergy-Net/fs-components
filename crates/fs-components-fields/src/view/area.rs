//! The look of a text area: its ink comes from a style sheet rule, not from the builder.
//!
//! Pattern: Adapter over Blinc's `text_area`: it has no colour setter (its text takes Blinc's theme colour, which is not the style's:
//! white on the light ground, black on the dark one), only the rules `color`, `caret-color`, `::placeholder` and `::selection` of a
//! class. So every style, light and dark, has a class `fs-field-area-<style>-<light|dark>` with concrete colours of its palette.

use fs_components_styles::StyleId;

use super::Look;

/// The class of the text area of `look`.
pub(super) fn area_class(look: &Look) -> String {
    format!(
        "fs-field-area-{}-{}",
        look.style.id.name(),
        if look.dark { "dark" } else { "light" }
    )
}

/// The rules of the classes of every style, light and dark: install them with the other style sheets of the window
/// (`fs_components_widgets::styles()`), or the text of a many-line field has Blinc's colour, not the style's.
pub fn area_styles() -> String {
    let mut css = String::new();
    for id in StyleId::ALL {
        for dark in [false, true] {
            let look = Look::new(id, dark);
            let (pal, class) = (look.palette(), area_class(&look));
            css.push_str(&format!(
                ".{class} {{ color: {}; caret-color: {}; }}\n.{class}::placeholder {{ color: {}; }}\n.{class}::selection {{ background: {}; }}\n",
                pal.text.css(),
                pal.accent_text.css(),
                pal.text_weak.css(),
                pal.selected.css(),
            ));
        }
    }
    css
}
