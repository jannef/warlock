//! Scrolls (spec §8 "Scrolls").

use super::text::capital;
use super::traps::{ice_text, sound_text};
use std::fmt::Write as _;
use warlock_core::content::{Cast, Content, Reading};

/// The scrolls: what reading each does, its damage dice, how far it reaches, what it sounds like,
/// from which item level (depth) loot holds it, and what it is worth. By name.
pub(super) fn scrolls(content: &Content) -> String {
    let mut out = String::from(
        "| Scroll | What reading it does | Damage | Reach | Sound | Found from | Value |\n\
         |---|---|:---:|---:|---|---|---:|\n",
    );
    let mut scrolls: Vec<_> = (content.object_defs().iter())
        .filter_map(|o| o.read.as_ref().map(|r| (o, r)))
        .collect();
    scrolls.sort_by_key(|(o, _)| o.name.to_lowercase());
    for (o, reading) in scrolls {
        let (what, damage, reach, sound) = match reading {
            Reading::Identify => (
                "Identifies one unidentified item you carry: you know its magic from then on, or \
                 what kind of scroll an unknown one is"
                    .to_string(),
                "—".to_string(),
                "—".to_string(),
                "Silent".to_string(),
            ),
            Reading::Spell(spell) => {
                let what = match &spell.cast {
                    Cast::Burst(b) => {
                        let walls = if b.blast > 0 {
                            "; it blows the softer walls it reaches apart"
                        } else {
                            ""
                        };
                        format!(
                            "Flies at the cell you aim at, or the first creature in its way, and \
                             bursts {} cells around it (walls stop it): everyone in it, you too, \
                             takes the damage, half at the edge; no roll to hit{walls}{}",
                            b.radius,
                            ice_text(b.ice)
                        )
                    }
                    Cast::Bolt(b) => format!(
                        "Runs from you through the cell you aim at and on, up to {} cells or a \
                         wall: everyone on it takes the damage; no roll to hit",
                        b.length
                    ),
                };
                let damage = match &spell.cast {
                    Cast::Burst(b) => super::damage::typed_dice(content, b.damage, b.kind),
                    Cast::Bolt(b) => super::damage::typed_dice(content, b.damage, b.kind),
                };
                (
                    what,
                    damage,
                    format!("{} cells", spell.range),
                    sound_text(spell.noise),
                )
            }
        };
        let found = match o.frequency {
            0 => "not as loot".to_string(),
            f => format!("item level {}, frequency {f}", o.level),
        };
        let _ = writeln!(
            out,
            "| {} `{}` | {what} | {damage} | {reach} | {sound} | {found} | {} gold |",
            capital(&o.name),
            o.glyph,
            o.value
        );
    }
    out
}
