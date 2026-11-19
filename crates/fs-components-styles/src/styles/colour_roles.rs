//! The colours that belong to a style but are not a palette colour: the colour of a kind of file (folder, picture, sheet ...), the
//! shade, paper and ink of an icon, the tiles of the people of a plan, the ink of an exported icon. They are style tokens
//! (ADR 0066): a widget asks the style, no colour is written in a widget (the conformance test of `fs-components-widgets` reads the
//! sources for colour literals outside this crate).
//!
//! Pattern: Registry by exhaustive `match` (`tint` names every [`StyleId`]) over one base table; a style moves the base colours
//! toward white or black, so a style change shows in the icons as well. Admin is the base (its values are the table).

use crate::color::Rgba;
use crate::style::StyleId;

/// A kind of file or folder that has a colour of its own.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconKind {
    /// A folder.
    Folder,
    /// A picture.
    Image,
    /// A text.
    Text,
    /// An archive.
    Archive,
    /// Sound.
    Audio,
    /// Film.
    Video,
    /// A PDF.
    Pdf,
    /// A spreadsheet.
    Sheet,
    /// A document.
    Doc,
    /// Slides.
    Slides,
    /// Code.
    Code,
    /// A file of an unknown kind.
    Unknown,
    /// The folder of the desktop.
    Desktop,
    /// The folder of downloads.
    Downloads,
    /// The folder of documents.
    Documents,
    /// The folder of pictures.
    Pictures,
    /// The folder of music.
    Music,
    /// The folder of videos.
    Videos,
}

impl IconKind {
    /// All kinds.
    pub const ALL: [IconKind; 18] = [
        Self::Folder,
        Self::Image,
        Self::Text,
        Self::Archive,
        Self::Audio,
        Self::Video,
        Self::Pdf,
        Self::Sheet,
        Self::Doc,
        Self::Slides,
        Self::Code,
        Self::Unknown,
        Self::Desktop,
        Self::Downloads,
        Self::Documents,
        Self::Pictures,
        Self::Music,
        Self::Videos,
    ];

    /// The colour of the kind in the base table (the one of Admin).
    const fn base(self) -> u32 {
        match self {
            Self::Folder => 0x00F1_B531,
            Self::Image | Self::Downloads => 0x002F_9E6E,
            Self::Text => 0x006B_7A8C,
            Self::Archive => 0x00A0_6B2C,
            Self::Audio => 0x00B0_5BB5,
            Self::Music => 0x00A0_5BB5,
            Self::Video | Self::Videos => 0x00C0_504D,
            Self::Pdf => 0x00D0_4437,
            Self::Sheet => 0x002E_8B57,
            Self::Doc => 0x003B_6FC4,
            Self::Slides => 0x00D9_7A2B,
            Self::Code => 0x004A_86C8,
            Self::Unknown => 0x008A_949E,
            Self::Desktop => 0x003B_82C4,
            Self::Documents => 0x00C4_8A2A,
            Self::Pictures => 0x0045_8F8A,
        }
    }
}

/// How a style moves the base colours: per mille toward white, per mille toward black. One of the two is 0.
const fn tint(id: StyleId) -> (u16, u16) {
    match id {
        StyleId::Admin => (0, 0),
        // few large things, high contrast: darker
        StyleId::Simple => (0, 120),
        // light, airy colours
        StyleId::Apple => (140, 0),
        StyleId::Windows => (0, 40),
        // a dark launcher: brighter so the colours stand on the dark ground
        StyleId::Craft => (220, 0),
    }
}

/// `c` moved `per_mille` toward `to` (0 or 255).
fn toward(c: u8, to: u8, per_mille: u16) -> u8 {
    let (from, to, p) = (i32::from(c), i32::from(to), i32::from(per_mille));
    u8::try_from((from + (to - from) * p / 1000).clamp(0, 255)).unwrap_or(c)
}

fn moved(id: StyleId, base: u32) -> Rgba {
    let c = Rgba::hex(base);
    let (white, black) = tint(id);
    let step = |v: u8| toward(toward(v, 255, white), 0, black);
    Rgba {
        r: step(c.r),
        g: step(c.g),
        b: step(c.b),
        a: 255,
    }
}

/// The colour of `kind` in the style `id`.
pub fn kind_colour(id: StyleId, kind: IconKind) -> Rgba {
    moved(id, kind.base())
}

/// The shadow under a folder (`strong` false) or a sheet (`strong` true): black, a little more for a sheet.
pub const fn icon_shade(strong: bool) -> Rgba {
    Rgba::hex(0).alpha(if strong { 61 } else { 56 })
}

/// The paper of a sheet icon.
pub const fn icon_paper(_id: StyleId) -> Rgba {
    Rgba::hex(0x00FA_FCFF)
}

/// The ink (white or black) for a sign or a text laid on `grounds`: the one whose LEAST contrast over all the grounds is higher.
/// Not a fixed white: a style that lightens its colours (Apple, Craft) would leave white initials on a pale tile. Black and white
/// together reach 4.58:1 on any single ground (the two contrasts meet at 4.58:1 at their worst), so a text on a plain ground is
/// always readable; a sign on a ground that fades (a folder) is measured on both ends.
pub fn readable_on(grounds: &[Rgba]) -> Rgba {
    let white = Rgba::hex(0x00FF_FFFF);
    let black = Rgba::hex(0);
    let least = |ink: Rgba| {
        grounds
            .iter()
            .map(|g| ink.contrast(*g))
            .fold(f32::MAX, f32::min)
    };
    if least(white) >= least(black) {
        white
    } else {
        black
    }
}

/// The lightest part of the front of a folder icon: the ground fades from this (22 percent toward white) to the kind colour.
pub fn folder_top(kind: Rgba) -> Rgba {
    Rgba {
        r: toward(kind.r, 255, 220),
        g: toward(kind.g, 255, 220),
        b: toward(kind.b, 255, 220),
        a: 255,
    }
}

/// The colour of the sign laid on the front of the pinned folder `kind` in the style `id`: white or black, whichever is readable
/// on the whole ground of the folder (a graphic needs 3:1, WCAG 1.4.11).
pub fn pinned_sign(id: StyleId, kind: IconKind) -> Rgba {
    let ground = kind_colour(id, kind);
    readable_on(&[ground, folder_top(ground)])
}

/// The colour of the soft shadow of a small menu: black, a little over a tenth.
pub const fn menu_shadow(_id: StyleId) -> Rgba {
    Rgba::hex(0).alpha(31)
}

/// The ink of an icon written to a file (the exported icon theme): neutral dark grey, the colour of text on a light panel.
pub const fn export_ink(_id: StyleId) -> Rgba {
    Rgba::hex(0x0044_4444)
}

/// The base colours of the seven tiles of a plan (the people).
const PLAN_TILES: [u32; 7] = [
    0x002F_7BF5,
    0x00E8_573F,
    0x008B_5CF6,
    0x00F2_A516,
    0x0012_A58A,
    0x00E0_368F,
    0x007B_B51C,
];

/// The colour of tile `n` (0 to 6) of a plan in the style `id`.
pub fn plan_tile(id: StyleId, n: usize) -> Rgba {
    moved(id, PLAN_TILES[n % PLAN_TILES.len()])
}

/// The colour of the initials on tile `n` of a plan: white or black, the one that reads on the tile of this style.
pub fn plan_tile_text(id: StyleId, n: usize) -> Rgba {
    readable_on(&[plan_tile(id, n)])
}

/// The names of the theme variables [`sheet`] gives (without the leading `--`): every style gives all of them.
pub const NAMES: [&str; 15] = [
    "plan-tile-0",
    "plan-tile-1",
    "plan-tile-2",
    "plan-tile-3",
    "plan-tile-4",
    "plan-tile-5",
    "plan-tile-6",
    "plan-tile-text-0",
    "plan-tile-text-1",
    "plan-tile-text-2",
    "plan-tile-text-3",
    "plan-tile-text-4",
    "plan-tile-text-5",
    "plan-tile-text-6",
    "menu-shadow",
];

/// The theme variables of these colours as one `:root` rule: `--plan-tile-N` and `--plan-tile-text-N` (N 0 to 6), `--menu-shadow`.
pub fn sheet(id: StyleId) -> String {
    use std::fmt::Write;
    let mut css = String::from(":root {");
    for n in 0..PLAN_TILES.len() {
        let _ = write!(css, " --plan-tile-{n}: {};", plan_tile(id, n).css());
    }
    for n in 0..PLAN_TILES.len() {
        let _ = write!(
            css,
            " --plan-tile-text-{n}: {};",
            plan_tile_text(id, n).css()
        );
    }
    // the soft shadow of a small menu that opens over text (the slash menu of the wiki): not the shadow of a dialog
    let _ = writeln!(
        css,
        " --menu-shadow: 0px 4px 14px 0px {}; }}",
        menu_shadow(id).css()
    );
    css
}
