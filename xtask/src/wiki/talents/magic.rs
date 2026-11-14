//! What the magic talents do, in the wiki's words (`warlock_core::talent::MagicEffect`, #1495): one
//! arm a variant, beside the area's enum and sum in `warlock-core/src/talent/magic.rs`.

use super::{spell_mod_text, spell_text};
use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::MagicEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &MagicEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    match effect.clone() {
        MagicEffect::ManaRegenPercent { value } => {
            format!("mana regeneration pace {}", percent(value))
        }
        MagicEffect::MaxManaPercent { value } => {
            format!(
                "maximum [mana](mana.md#how-much-mana-you-have) {}",
                percent(value)
            )
        }
        MagicEffect::MaxManaPerLevels {
            per_levels,
            at_least,
        } => format!("+1 maximum mana per {per_levels} levels, at least +{at_least}"),
        MagicEffect::ManaRestingPercent { value } => {
            format!("mana regeneration pace while resting {}", percent(value))
        }
        MagicEffect::ManaCostPercent { value } => format!(
            "abilities' mana costs {} (rounded to the nearest, at least 1)",
            percent(value)
        ),
        MagicEffect::RodCostPercent { value } => format!(
            "rods' mana costs {} (rounded to the nearest, at least 1), before other cuts",
            percent(value)
        ),
        MagicEffect::BloodMagic { hp, mana } => format!(
            "active (the time of a wait): give {hp} hit points (never your last) for {mana} \
             mana, up to your maximum; not while your mana is full"
        ),
        MagicEffect::ReadPercent { value } => format!("reading time {}", percent(value)),
        MagicEffect::Study {
            bonus,
            percent: time,
            overreach_percent,
        } => {
            let mut parts = Vec::new();
            if bonus != 0 {
                parts.push(format!("study check {}", signed(bonus)));
            }
            if time != 0 {
                parts.push(format!("study time {}", percent(time)));
            }
            if overreach_percent != 0 {
                parts.push(format!(
                    "the levels a spell is above your learning count {} in the study \
                     check, rounded down (a spell beyond your learning is that much less beyond \
                     you)",
                    percent(overreach_percent)
                ));
            }
            parts.join(", ")
        }
        MagicEffect::ScrollKept { chance } => {
            format!("a {chance} % chance a scroll you read is not used up")
        }
        MagicEffect::SparesSelf => {
            "your own bursts, a scroll's or a spell's, never catch you".to_string()
        }
        MagicEffect::PotionKept { chance } => {
            format!("a {chance} % chance a potion you drink is not used up")
        }
        MagicEffect::Cackle { degrees, noise } => {
            let hexes: Vec<String> = (content.hex_statuses().iter())
                .map(|&s| content.status(s).name.clone())
                .collect();
            format!(
                "active (the time of a spell; no mana, cooldown, or check, as often as you like): \
                 a cackle, a voice of loudness {noise}; every hex you laid ({}) on a creature you \
                 see grows {degrees} degree{}, up to its maximum, and its degree's time starts \
                 again (one at its maximum only starts again). Another's hex and a creature out \
                 of sight are untouched. With no creature in sight under a hex of yours it is \
                 refused, with no time spent",
                hexes.join(", "),
                if degrees == 1 { "" } else { "s" },
            )
        }
        MagicEffect::Spell { spell } => (content.spell_id(&spell))
            .map(|id| content.spell(id))
            .map_or_else(|| spell.clone(), |def| spell_text(content, def)),
        MagicEffect::SpellMod(spec) => spell_mod_text(content, &spec, false),
        MagicEffect::SpellType { arc_chance, when } => format!(
            "your {} spells arc {arc_chance} % of the time",
            when.join(", ")
        ),
        MagicEffect::SpellManaPercent { value } => {
            format!("your spells' mana {} (at least 1)", percent(value))
        }
        MagicEffect::HexTithe { per_levels } => format!(
            "when a creature dies under a hex you laid, whoever killed it, your mana rises by its \
             level divided by {per_levels} (rounded down, at least 1), never past your maximum"
        ),
        MagicEffect::Knock { mana } => format!(
            "active (aimed in a direction, for {mana} mana, the time of a spell): the lock of \
             the locked door or shut chest there springs open; nothing opens and no trap \
             springs. Where nothing locked stands it is refused, with no time or mana spent"
        ),
        MagicEffect::Scribe { mana_times } => format!(
            "active (a spell you know, from the scribe list): on the first [blank \
             scroll](scrolls.md#writing-scrolls) you carry, a scroll of the spell is written, for \
             {mana_times} times the spell's mana (what casting it would cost before Frugal \
             caster's cut; Frugal caster then cuts the whole) and as many turns of writing as a \
             study of the spell takes (Quick study does not shorten it). The mana is spent and \
             the blank used up only at the end; interrupted, nothing is spent. Refused, with no \
             time spent, without a blank scroll or the mana"
        ),
    }
}
