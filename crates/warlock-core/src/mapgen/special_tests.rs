//! The special-room draw (spec §7 "Special rooms", `special.rs`): the features' shares by their
//! weights, a dungeon's favoured and banned features, a feature added to the data turning up
//! where no dungeon names it, the whole-level version, and eligibility.

use super::special::draw;
use super::*;
use crate::content::ContentFiles;
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> =
    LazyLock::new(|| Content::load(&ContentFiles::builtin()).expect("builtin content"));

/// The built-in content with one more feature, `extra` (an `[[effect]]` body after its id, name,
/// mark, and enter line), at the end of `room_effects.toml`, and the shared `chance` at `chance`.
fn with_feature(extra: &str, chance: u8) -> Content {
    let mut files = ContentFiles::builtin();
    files.edit_table("room_effects", |text| {
        let text = text.replace("chance = 12\n", &format!("chance = {chance}\n"));
        format!(
            "{text}\n[[effect]]\nid = \"glittering\"\nname = \"glittering hall\"\nmark = \"g\"\n\
             enter = \"It glitters.\"\n{extra}"
        )
    });
    Content::load(&files).expect("fixture content")
}

/// How often each feature is drawn in `n` draws with `odds`, every feature eligible.
fn tally(content: &Content, odds: &RoomOdds, n: u32) -> Vec<u32> {
    let effects = content.room_effects();
    let mut rng = Rng::stream(7, "special_tests");
    let mut by = vec![0; effects.defs().len()];
    for _ in 0..n {
        let id = draw(effects, odds, &mut rng, |_| true).expect("an effect");
        by[usize::from(id.0)] += 1;
    }
    by
}

#[test]
fn features_are_drawn_by_their_weights() {
    let effects = CONTENT.room_effects();
    let total: u32 = effects.defs().iter().map(|e| u32::from(e.weight)).sum();
    let n = 20_000;
    let by = tally(&CONTENT, &RoomOdds::shared(effects), n);
    for (e, got) in effects.defs().iter().zip(&by) {
        let want = n * u32::from(e.weight) / total;
        assert!(
            got.abs_diff(want) * 100 < want * 8,
            "{}: {got} of {want}",
            e.id
        );
    }
}

#[test]
fn favoured_features_keep_their_shares_and_banned_ones_never_come() {
    let effects = CONTENT.room_effects();
    let id = |name: &str| effects.id(name).expect(name);
    let odds = RoomOdds {
        chance: 25,
        per_level: 2,
        favoured: vec![(id("hushed"), 15), (id("flooded"), 35)],
        banned: vec![id("sanctuary")],
    };
    let n = 20_000;
    let by = tally(&CONTENT, &odds, n);
    let percent = |name: &str| by[usize::from(id(name).0)] * 100 / n;
    assert!((33..=37).contains(&percent("flooded")), "{by:?}");
    assert!((13..=17).contains(&percent("hushed")), "{by:?}");
    assert_eq!(by[usize::from(id("sanctuary").0)], 0, "banned");
    // The other half by weight: shadowed 10 of the 51 left (echoing 8, draining 6, black 5,
    // hot 5, cold 5, silent 5, heavy 4, mirrors 3).
    assert!((9..=12).contains(&percent("shadowed")), "{by:?}");
    // Every eligible feature favoured: by their shares alone.
    let all = RoomOdds {
        favoured: vec![(id("hushed"), 10), (id("flooded"), 30)],
        ..odds
    };
    let mut rng = Rng::stream(7, "favoured");
    let only = |e: &crate::content::RoomEffectDef| ["hushed", "flooded"].contains(&e.id.as_str());
    let mut hushed = 0;
    for _ in 0..4000 {
        let e = draw(effects, &all, &mut rng, only).expect("an effect");
        hushed += u32::from(e == id("hushed"));
    }
    assert!((900..=1100).contains(&hushed), "{hushed} of 4000 hushed");
    // Nothing eligible: nothing drawn.
    assert_eq!(draw(effects, &all, &mut rng, |_| false), None);
}

#[test]
fn a_new_feature_turns_up_in_a_dungeon_that_never_names_it() {
    let content = with_feature("weight = 30\n", 12);
    let fen = content.dungeon("fennmynster").expect("Fennmynster");
    let odds = fen.room_odds(1).expect("odds");
    let glittering = content
        .room_effects()
        .id("glittering")
        .expect("the new feature");
    let flooded = content.room_effects().id("flooded").expect("flooded");
    let n = 20_000;
    let by = tally(&content, odds, n);
    let new = by[usize::from(glittering.0)];
    assert!(new > 0, "drawn in the fen");
    // The fen's own shares hold however many features are added.
    let floods = by[usize::from(flooded.0)] * 100 / n;
    assert!((33..=37).contains(&floods), "{floods} % flooded");
}

#[test]
fn a_feature_with_a_level_chance_sometimes_covers_the_whole_level() {
    let content = with_feature(
        "weight = 100\nlevel_chance = 40\npervade = \"Everything glitters.\"\nlight = 80\n",
        100,
    );
    let glittering = content
        .room_effects()
        .id("glittering")
        .expect("the new feature");
    let (mut whole, mut rooms) = (0, 0);
    for seed in 0..80 {
        let level =
            generate(&content, ("showcase", seed), "depths:1", Some("hub")).expect("generates");
        let in_room = level.rooms.iter().any(|r| r.effect == glittering);
        if level.pervading == Some(glittering) {
            whole += 1;
            assert!(!in_room, "seed {seed}: the room stays plain, one a level");
        }
        rooms += usize::from(in_room);
        assert!(level.pervading.is_none_or(|e| e == glittering));
    }
    // Most draws are the glittering hall (weight 100 of 145), 40 % of those the whole level.
    assert!((15..=45).contains(&whole), "{whole} of 80 levels covered");
    assert!(rooms > 15, "{rooms} levels with a glittering room");
    // Without a level chance nothing ever covers a level.
    for seed in 0..40 {
        let level =
            generate(&CONTENT, ("showcase", seed), "depths:2", Some("hub")).expect("generates");
        let pervades = |e| CONTENT.room_effect(e).level_chance > 0;
        assert!(level.pervading.is_none_or(pervades), "seed {seed}");
    }
}

#[test]
fn level_chance_comes_near_its_percent_over_many_draws() {
    // The extra draw alone, as `pick` makes it: 5 % of the draws.
    let mut rng = Rng::stream(3, "level_chance");
    let hits = (0..40_000).filter(|_| rng.chance(5)).count();
    assert!((1800..=2200).contains(&hits), "{hits} of 40 000");
}

#[test]
fn features_are_drawn_only_where_they_are_eligible() {
    let only = |extra: &str, campaign: &str, id: &str| {
        let content = with_feature(&format!("weight = 100\n{extra}"), 100);
        let glittering = content.room_effects().id("glittering").expect("glittering");
        (0..30)
            .filter(|seed| {
                let level = generate(&content, (campaign, *seed), id, None).expect("generates");
                level.rooms.iter().any(|r| r.effect == glittering)
            })
            .count()
    };
    // The Depths have no danger; Fennmynster does.
    assert_eq!(only("needs_danger = true\n", "showcase", "depths:1"), 0);
    // (The fen keeps 65 % of its draws for its own and a room special one time in four.)
    assert!(only("needs_danger = true\n", "ascmearc", "fennmynster:1") > 3);
    // The Depths' bottom, 8, has no level below.
    assert_eq!(only("needs_below = true\n", "showcase", "depths:8"), 0);
    assert!(only("needs_below = true\n", "showcase", "depths:7") > 10);
    // No room is that large, nor that small.
    assert_eq!(only("min_cells = 5000\n", "showcase", "depths:1"), 0);
    assert_eq!(only("max_cells = 2\n", "showcase", "depths:1"), 0);
    assert!(only("max_cells = 400\n", "showcase", "depths:1") > 10);
}
