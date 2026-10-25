//! The campaign's creature waves (docs/design/campaign.md §10): the shipped types load where
//! their data says, on the XP guideline, with the factions they fight in and the gear they
//! carry. Each wave adds its types to [`WAVES`].

use crate::content::{Content, Stance};
use crate::gear;
use crate::rng::Rng;
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> = LazyLock::new(|| Content::builtin().expect("content"));

/// Each wave's types, with the campaign that owns each (`None`: shared, `data/creatures/`).
const WAVES: &[(&str, Option<&str>)] = &[
    // The barrow wave (Cealdhlāw, #333).
    ("grave_worm", None),
    ("barrow_spider", None),
    ("goblin", None),
    ("goblin_archer", None),
    ("skeleton", None),
    ("zombie", None),
    ("giant_centipede", None),
    ("barrow_ghoul", None),
    ("goblin_chief", None),
    // Campaign-owned, but shared until the Ascmearc's directory exists (#332): then
    // `Some("ascmearc")`, its file moved to `campaigns/ascmearc/data/creatures/`.
    ("barrow_wight", None),
];

#[test]
fn the_waves_load_where_they_belong_on_the_xp_guideline() {
    for &(id, campaign) in WAVES {
        let def = CONTENT.creature(CONTENT.creature_id(id).expect(id));
        assert_eq!(def.campaign.as_deref(), campaign, "{id}");
        // The creature XP guideline (data/experience.toml): 5 × level + 10.
        assert_eq!(def.xp, 5 * u32::from(def.level) + 10, "{id}");
    }
}

#[test]
fn goblins_and_the_undead_fight_the_kobolds_and_each_other() {
    let stance = |a: &str, b: &str| {
        let id = |n: &str| CONTENT.creature_id(n).expect(n);
        CONTENT.stance(id(a), id(b))
    };
    for (a, b, want) in [
        ("goblin", "player", Stance::Hostile),
        ("goblin", "kobold", Stance::Hostile),
        ("goblin", "rat", Stance::Hostile),
        ("goblin", "goblin_chief", Stance::Neutral),
        ("skeleton", "player", Stance::Hostile),
        ("zombie", "kobold", Stance::Hostile),
        ("barrow_ghoul", "goblin", Stance::Hostile),
        ("barrow_wight", "skeleton", Stance::Neutral),
        // The undead leave the vermin of their barrows alone.
        ("zombie", "grave_worm", Stance::Neutral),
        ("barrow_spider", "player", Stance::Hostile),
    ] {
        assert_eq!(stance(a, b), want, "{a} and {b}");
    }
}

#[test]
fn the_barrow_dead_resist_by_their_nature() {
    let resist = |id: &str, kind: &str| {
        let def = CONTENT.creature(CONTENT.creature_id(id).expect(id));
        let kind = CONTENT.damage_type_id(kind).expect(kind);
        (def.resist.iter())
            .find(|(k, _)| *k == kind)
            .map_or(0, |(_, r)| *r)
    };
    assert_eq!(resist("skeleton", "piercing"), 50);
    assert_eq!(resist("skeleton", "bludgeoning"), -50);
    assert_eq!(resist("barrow_wight", "cold"), 50);
    for id in ["skeleton", "barrow_wight"] {
        let def = CONTENT.creature(CONTENT.creature_id(id).expect(id));
        assert!(!def.corpse && def.blood.is_none(), "{id}");
    }
}

/// Over many seeds, what each humanoid of the waves carries: its racial weapon (or another
/// weapon) is wielded whenever carried, armour worn, and an archer with a bow holds arrows ready
/// nearly always.
#[test]
fn the_barrow_humanoids_wield_their_weapons() {
    for id in [
        "goblin",
        "goblin_archer",
        "goblin_chief",
        "skeleton",
        "barrow_wight",
    ] {
        let creature = CONTENT.creature_id(id).expect(id);
        let mut armed = 0;
        let mut quivers = 0;
        for seed in 0..200 {
            let pack = gear::roll(&CONTENT, &mut Rng::gear(seed, "test"), creature, 3);
            let has =
                |object: &str| (pack.items.iter()).any(|s| CONTENT.object(s.object).id == object);
            if has("short_bow") && has("arrow") {
                quivers += 1;
            }
            for (i, stack) in pack.items.iter().enumerate() {
                let object = CONTENT.object(stack.object);
                if object.slot.is_some() {
                    assert!(
                        pack.equipment.holds(i),
                        "{id} {seed}: {} not held",
                        object.id
                    );
                }
                if object.attack.is_some() || object.launches.is_some() {
                    armed += 1;
                }
            }
        }
        // The goblins' racial weapons come nine times in ten; the dead are armed less often.
        let least = match id {
            "skeleton" => 40,
            "barrow_wight" => 90,
            _ => 160,
        };
        assert!(armed >= least, "{id}: {armed} of 200 armed");
        if id == "goblin_archer" {
            assert!(quivers >= 150, "{quivers} of 200 with bow and arrows");
        }
    }
}
