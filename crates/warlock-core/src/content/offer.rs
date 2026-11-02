//! A trader's one offer (spec §8 "Dialogue": "An offer"; the pedlar of the Ascmearc's roads,
//! #808, docs/design/overworld-encounters.md §15 Q7): a dialogue's `[dialogue.offer]`. Asked for
//! like a topic (its `id` in the speaker's topic namespace), it is one item at a fixed price,
//! taken or left (`world/wares.rs`). The item is a favoured roll
//! ([`crate::magic::loot::favoured`]: rare, with `affixes` of them at the player's level plus
//! `level.bonus`), or, by `uniques.chance` in basis points, one of `uniques.items` not yet
//! offered in the run, which no other content may place.
//!
//! ```toml
//! [dialogue.offer]
//! id = "wares"
//! name = "Your wares"
//! reply = "Pleasure doin' business."
//! sold = "That was the only one I 'ad."
//! level = { bonus = 4, least = 4 }
//! affixes = [[4, 50], [5, 35], [6, 15]]
//! price = { affix = 15, unique = 1500 }
//! uniques = { chance = 200, items = ["brosinga_mene"] }
//! ```

use super::dialogue::{MAX_LINE, MAX_REPLY, check_lines, is_id};
use super::{Content, ContentError, MAX_ITEM_LEVEL, MAX_PER_KIND};
use serde::Deserialize;

/// A trader's one offer, as data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TradeOffer {
    /// Its id in replays (`ask:E:wares`), unique among the speaker's topics and services.
    pub id: String,
    /// What the player picks: "Your wares" (the screen adds the price).
    pub name: String,
    /// What the speaker says when the item is bought.
    pub reply: String,
    /// What the speaker says when asked again after the sale.
    pub sold: String,
    /// The item level of the favoured roll: the player's level plus this.
    pub level_bonus: u8,
    /// The least item level of the favoured roll.
    pub level_least: u8,
    /// How many affixes the favoured item has: `(count, weight)` pairs.
    pub affixes: Vec<(u8, u32)>,
    /// What the favoured item costs on top of its value: gold per affix per item level.
    pub affix_price: u32,
    /// What a unique item costs.
    pub unique_price: u32,
    /// The chance, in basis points, that the offer is one of [`TradeOffer::uniques`].
    pub unique_chance: u32,
    /// The unique items it may be (ids in the uniques table), each at most once a run.
    pub uniques: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OfferSpec {
    id: String,
    name: String,
    reply: String,
    sold: String,
    level: LevelSpec,
    affixes: Vec<(u8, u32)>,
    price: PriceSpec,
    uniques: Option<UniquesSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelSpec {
    bonus: u8,
    #[serde(default = "one")]
    least: u8,
}

fn one() -> u8 {
    1
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PriceSpec {
    affix: u32,
    #[serde(default)]
    unique: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UniquesSpec {
    chance: u32,
    items: Vec<String>,
}

/// Every unique an offer names is placed only through it: no other content names it (checked
/// once everything that places objects has loaded).
pub(super) fn check_placements(content: &Content) -> Result<(), ContentError> {
    for d in content.dialogues() {
        let Some(offer) = &d.offer else { continue };
        if let Some(id) = (offer.uniques.iter()).find(|id| content.uniques.is_placed(id)) {
            return Err(ContentError::new(
                "data/uniques",
                format!(
                    "`{}{id}` is placed by content, but offer `{}` names it: \
                     a trader's unique is placed only through the offer",
                    super::unique::UNIQUE,
                    offer.id
                ),
            ));
        }
    }
    Ok(())
}

/// The most affixes an item has: three prefixes and three suffixes.
const MOST_AFFIXES: u8 = 2 * MAX_PER_KIND as u8;

/// Resolves the offer of a dialogue of `campaign` against `content` (its uniques and shops);
/// `taken` says whether an id is already a topic's or a service's.
pub(super) fn resolve(
    spec: OfferSpec,
    campaign: Option<&str>,
    taken: impl Fn(&str) -> bool,
    content: &Content,
) -> Result<TradeOffer, String> {
    let bad = |m: String| format!("offer `{}`: {m}", spec.id);
    if !is_id(&spec.id) {
        return Err(bad(
            "ids are lower-case letters, digits, and underscores".into()
        ));
    }
    if taken(&spec.id) {
        return Err(bad("its id is a topic's or a service's".into()));
    }
    if content.shops().is_none() {
        return Err(bad(
            "it is paid in the shops' currency: data/shops.toml is needed".into(),
        ));
    }
    check_lines([&spec.name], MAX_LINE)
        .and_then(|()| check_lines([&spec.reply, &spec.sold], MAX_REPLY))
        .map_err(bad)?;
    if spec.level.least == 0 || spec.level.least > MAX_ITEM_LEVEL {
        return Err(bad(format!("`level.least` is 1 to {MAX_ITEM_LEVEL}")));
    }
    let counts_ok = (spec.affixes.iter()).all(|&(n, _)| (1..=MOST_AFFIXES).contains(&n));
    if !counts_ok || spec.affixes.iter().all(|&(_, w)| w == 0) {
        return Err(bad(format!(
            "`affixes` are [count, weight] pairs, counts 1 to {MOST_AFFIXES}, some weight above 0"
        )));
    }
    if spec.price.affix == 0 {
        return Err(bad("`price.affix` is at least 1 gold".into()));
    }
    let (chance, uniques) = match spec.uniques {
        Some(u) => (u.chance, u.items),
        None => (0, Vec::new()),
    };
    if chance > 10_000 {
        return Err(bad("`uniques.chance` is in basis points, 0 to 10000".into()));
    }
    if !uniques.is_empty() && spec.price.unique == 0 {
        return Err(bad("`price.unique` is at least 1 gold".into()));
    }
    for (i, id) in uniques.iter().enumerate() {
        let def = (content.unique(id)).ok_or_else(|| bad(format!("unknown unique item `{id}`")))?;
        if def.campaign.is_some() && def.campaign.as_deref() != campaign {
            return Err(bad(format!("`{id}` is another campaign's")));
        }
        if uniques[..i].contains(id) {
            return Err(bad(format!("`{id}` is named twice")));
        }
    }
    Ok(TradeOffer {
        id: spec.id,
        name: spec.name,
        reply: spec.reply,
        sold: spec.sold,
        level_bonus: spec.level.bonus,
        level_least: spec.level.least,
        affixes: spec.affixes,
        affix_price: spec.price.affix,
        unique_price: spec.price.unique,
        unique_chance: chance,
        uniques,
    })
}
