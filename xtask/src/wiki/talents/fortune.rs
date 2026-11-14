//! What the fortune talents do, in the wiki's words (`warlock_core::talent::FortuneEffect`, #1495):
//! one arm a variant, beside the area's enum and sum in `warlock-core/src/talent/fortune.rs`.

use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::FortuneEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &FortuneEffect) -> String {
    let percent = |v: i32| format!("{} %", signed(v));
    match effect.clone() {
        FortuneEffect::Stipend => match content.stipend() {
            Some(rules) => format!(
                "each level you reach puts by {} plus {} gold, and ({} % of the time) one loot \
                 roll at that level with the odds of a magic or rare item {}{}, kept if magic, \
                 rare, a potion, a scroll, or a gem; your servant hands it all over when you \
                 talk to them (see [Stipend](class-aristocrat.md#stipend))",
                rules.each,
                rules.dice,
                rules.chance,
                percent(rules.magic),
                if rules.gems == 0 {
                    String::new()
                } else {
                    format!(" and of a gem {}", percent(rules.gems))
                },
            ),
            None => "nothing: the stipend is not in this game".to_string(),
        },
        FortuneEffect::StipendMore {
            gold,
            chance,
            magic,
        } => format!(
            "your [stipend](class-aristocrat.md#stipend) puts by {} gold, makes its loot roll {} \
             percentage points more often, and the odds of a magic or rare item in it are {}",
            percent(gold),
            signed(chance),
            percent(magic)
        ),
        FortuneEffect::Invest => match content.invest() {
            Some(rules) => format!(
                "your business partner invests the gold you put with them ({} % kept going in, \
                 {} % coming out): every week it changes by {} plus {} basis points (hundredths \
                 of a percent), {} more per quest of their town's people done and more for some \
                 quests (see [Invest](class-merchant.md#invest))",
                rules.fee_in, rules.fee_out, rules.base, rules.dice, rules.per_quest
            ),
            None => "nothing: investing is not in this game".to_string(),
        },
        FortuneEffect::Patron => match content.patron() {
            Some(rules) => format!(
                "a patron in town commissions Fine (or better) arms and armour from your trades, \
                 advances the stock, and pays {} % of the piece's worth (see \
                 [the patron](patron.md))",
                rules.pays
            ),
            None => "nothing: the patron is not in this game".to_string(),
        },
        FortuneEffect::StartingGold {
            percent,
            at_least,
            at_most,
        } => format!(
            "at creation, [gold coins](shops.md#money): {percent} % of those you start with more, at least \
             {at_least} and at most {at_most}"
        ),
        FortuneEffect::StartingGems { min, max, level } => {
            format!(
                "at creation, {min} to {max} loose [gems](gems.md), as found at \
                 [item level](magic-items.md#finding-them) {level}"
            )
        }
        FortuneEffect::Heirloom {
            min, max, level, ..
        } => format!(
            "at creation, your race's heirloom weapon (Heirlooms, below) with {min} to {max} \
             [affixes](affixes.md#prefixes) rolled at [item level](magic-items.md#finding-them) \
             {level} (two: [magic](magic-items.md); more: [rare](magic-items.md#rarity)), \
             [identified](magic-items.md#unidentified-items), readied in its own \
             [slot](equipment.md#what-fits-where) if that is free, else carried"
        ),
        FortuneEffect::LootMagicPercent { value } => format!(
            "the weights of the rarities better than a find's least {} (anything found: magic \
             and rare; what creatures drop, always magic: rare)",
            percent(value)
        ),
        FortuneEffect::QuestGoldPercent { value } => {
            format!("gold from quests {}", percent(value))
        }
        FortuneEffect::FavourPercent { value } => {
            format!("shop favour {} (before the cap)", percent(value))
        }
        FortuneEffect::GemWorth => "item descriptions show what each gem is worth".to_string(),
        FortuneEffect::IdentifyCarried { hours } => format!(
            "a magic item you carry for {hours} hours (counted from when you gain this) is \
             identified"
        ),
        FortuneEffect::GemFindPercent { value } => format!(
            "the chance a find is a loose gem {} (anything found, and what creatures drop)",
            percent(value)
        ),
        FortuneEffect::Distrusted => {
            "no one makes way for you, whatever else would have them (Noblesse too)".to_string()
        }
        FortuneEffect::MerchantsEye => format!(
            "active (on a thing you carry, for the time of reading a scroll, {} at normal pace; \
             no mana, cooldown, or check, as often as you like): the thing is identified as a \
             scroll of identify would identify it. On a thing you already know it is refused, \
             with no time spent",
            crate::wiki::util::at_normal_pace(warlock_core::content::price(
                content
                    .action_costs()
                    .cost(warlock_core::content::ActionKind::Read),
                warlock_core::content::BASE_TURN
            ))
        ),
    }
}
