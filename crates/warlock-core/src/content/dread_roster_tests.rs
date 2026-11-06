//! The dread sort (docs/design/monster-dread.md §6, §8; #1124): which types are dread, each at
//! its tier, and the tier moves of §8.3. Dread is a flag, not a tier: a dread type keeps the
//! rarity it had, but for the moves listed here.

use super::Content;
use std::collections::BTreeSet;

/// Every dread type and its rarity tier (§6's rows marked dread; §8.3's moves applied).
const DREAD: &[(&str, &str)] = &[
    // Undead
    ("shadow", "rare"),
    ("greater_shadow", "rare"),
    ("deathshade", "very_rare"),
    ("wraith", "rare"),
    ("dread_wraith", "rare"),
    ("wraith_lord", "rare"),
    ("spectre", "rare"),
    ("gibbering_shade", "rare"),
    ("white_lady", "rare"),
    ("washer_at_the_ford", "rare"),
    ("night_mare", "rare"),
    ("ghost_lord", "rare"),
    ("banshee", "rare"),
    ("vampire_spawn", "rare"),
    ("vampire", "rare"),
    ("nosferatu", "rare"),
    ("vampire_lord", "rare"),
    ("elder_vampire", "very_rare"),
    ("eldest_vampire", "very_rare"),
    ("lich", "rare"),
    ("archlich", "very_rare"),
    ("wormhost", "uncommon"),
    ("wormlord", "rare"),
    ("charnel_heap", "uncommon"),
    ("nightworm", "very_rare"),
    // Demons and devils
    ("chasme", "uncommon"),
    ("hezrou", "uncommon"),
    ("balor", "very_rare"),
    ("barbed_devil", "common"),
    ("pit_fiend", "very_rare"),
    // Yugoloths, fiends, hags
    ("ultroloth", "very_rare"),
    ("succubus", "rare"),
    ("incubus", "rare"),
    ("rakshasa", "very_rare"),
    ("sea_hag", "rare"),
    // Fey
    ("pixie", "very_rare"),
    ("darkling", "uncommon"),
    ("blink_dog", "uncommon"),
    ("nymph", "rare"),
    // Vermin, marsh things
    ("will_o_wisp", "very_rare"),
    ("basilisk", "very_rare"),
    // The peoples
    ("drow_assassin", "rare"),
];

/// §8.3's moves of types not flagged dread, with the tier each moved to.
const MOVED: &[(&str, &str)] = &[
    ("fetch", "rare"),
    ("ember_mote", "uncommon"),
    ("frost_mote", "uncommon"),
    ("spark_mote", "uncommon"),
    ("dust_devil", "uncommon"),
    ("water_mote", "uncommon"),
    ("stone_mote", "uncommon"),
];

fn rarity(content: &Content, id: &str) -> String {
    let def = content.creature(content.creature_id(id).unwrap_or_else(|| panic!("{id}")));
    content.creature_rarity(def.rarity).id.clone()
}

#[test]
fn the_dread_types_are_the_sorts_at_their_tiers() {
    let content = Content::builtin().expect("content");
    let flagged: BTreeSet<&str> = (content.creature_defs().iter())
        .filter(|c| c.dread)
        .map(|c| c.id.as_str())
        .collect();
    let listed: BTreeSet<&str> = DREAD.iter().map(|&(id, _)| id).collect();
    assert_eq!(flagged, listed, "the dread flags against the sort");
    for &(id, tier) in DREAD {
        assert_eq!(rarity(&content, id), tier, "{id}: rarity");
    }
}

#[test]
fn the_sorts_other_tier_moves_hold_and_leave_them_unflagged() {
    let content = Content::builtin().expect("content");
    for &(id, tier) in MOVED {
        assert_eq!(rarity(&content, id), tier, "{id}: rarity");
        let def = content.creature(content.creature_id(id).expect(id));
        assert!(!def.dread, "{id}: not dread (§8.3)");
    }
}

#[test]
fn no_unique_is_dread() {
    // A unique (the layer keepers, the demilich) is placed by its own rules, never dread-drawn.
    let content = Content::builtin().expect("content");
    for def in content.creature_defs() {
        assert!(
            !(def.unique && def.dread),
            "{}: a unique is never dread",
            def.id
        );
    }
}
