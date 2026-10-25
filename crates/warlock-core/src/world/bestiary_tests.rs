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
    ("barrow_wight", Some("ascmearc")),
    // The fen wave (Fennmynster, #334).
    ("giant_leech", None),
    ("giant_frog", None),
    ("drowned", None),
    ("crocodile", None),
    ("cultist", Some("ascmearc")),
    ("will_o_wisp", None),
    ("ghost", None),
    ("mud_golem", None),
    ("cult_priest", Some("ascmearc")),
    ("marsh_troll", None),
    ("bog_hag", Some("ascmearc")),
    // The delf wave (Fyrdelf, #335).
    ("ember_beetle", None),
    ("duergar_miner", None),
    ("duergar_warrior", None),
    ("ogre", None),
    ("fire_salamander", None),
    ("troll", None),
    ("duergar_smith", None),
    ("magma_crawler", None),
    ("ash_wraith", None),
    ("basilisk", None),
    ("cinder_elemental", None),
    ("fire_drake", Some("ascmearc")),
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

#[test]
fn the_cult_fights_the_marsh_and_leaves_the_dead_alone() {
    let stance = |a: &str, b: &str| {
        let id = |n: &str| CONTENT.creature_id(n).expect(n);
        CONTENT.stance(id(a), id(b))
    };
    for (a, b, want) in [
        ("giant_leech", "player", Stance::Hostile),
        ("crocodile", "player", Stance::Hostile),
        ("bog_hag", "player", Stance::Hostile),
        ("cultist", "player", Stance::Hostile),
        ("cultist", "marsh_troll", Stance::Hostile),
        ("cult_priest", "bog_hag", Stance::Hostile),
        ("cultist", "cult_priest", Stance::Neutral),
        ("cultist", "drowned", Stance::Neutral),
        ("giant_frog", "bog_hag", Stance::Neutral),
        ("will_o_wisp", "mud_golem", Stance::Neutral),
        // The fen's dead are the barrows' kin: undead, hostile to the goblins, not to the fen.
        ("drowned", "goblin", Stance::Hostile),
        ("ghost", "skeleton", Stance::Neutral),
        ("ghost", "crocodile", Stance::Neutral),
    ] {
        assert_eq!(stance(a, b), want, "{a} and {b}");
    }
}

#[test]
fn the_duergar_fight_the_goblins_kin_and_leave_the_fire_be() {
    let stance = |a: &str, b: &str| {
        let id = |n: &str| CONTENT.creature_id(n).expect(n);
        CONTENT.stance(id(a), id(b))
    };
    for (a, b, want) in [
        ("duergar_miner", "player", Stance::Hostile),
        ("duergar_warrior", "ogre", Stance::Hostile),
        ("duergar_smith", "troll", Stance::Hostile),
        ("duergar_warrior", "ash_wraith", Stance::Hostile),
        ("duergar_miner", "duergar_smith", Stance::Neutral),
        ("duergar_warrior", "fire_salamander", Stance::Neutral),
        ("duergar_miner", "kobold", Stance::Neutral),
        ("fire_drake", "player", Stance::Hostile),
        ("cinder_elemental", "ember_beetle", Stance::Neutral),
        ("magma_crawler", "goblin", Stance::Neutral),
        ("ogre", "player", Stance::Hostile),
        ("basilisk", "player", Stance::Hostile),
    ] {
        assert_eq!(stance(a, b), want, "{a} and {b}");
    }
}

#[test]
fn the_fen_dead_and_the_mud_shrug_off_blows() {
    let def = |id: &str| CONTENT.creature(CONTENT.creature_id(id).expect(id));
    let resist = |id: &str, kind: &str| {
        let kind = CONTENT.damage_type_id(kind).expect(kind);
        (def(id).resist.iter())
            .find(|(k, _)| *k == kind)
            .map_or(0, |(_, r)| *r)
    };
    for kind in ["slashing", "piercing", "bludgeoning"] {
        assert_eq!(resist("ghost", kind), 50, "{kind}");
    }
    assert_eq!(resist("drowned", "poison"), 100);
    let hide = |id: &str| def(id).natural.as_ref().map(|h| (h.defence, h.protection));
    assert_eq!(hide("mud_golem"), Some((0, 5)));
    assert_eq!(hide("crocodile"), Some((2, 3)));
    for id in ["ghost", "mud_golem", "will_o_wisp"] {
        assert!(!def(id).corpse && def(id).blood.is_none(), "{id}");
    }
    assert!(def("will_o_wisp").light.is_some());
    // Only the bog hag comes once a run (of the fen's types).
    for id in [
        "giant_leech",
        "giant_frog",
        "drowned",
        "crocodile",
        "cultist",
        "will_o_wisp",
    ] {
        assert!(!def(id).unique, "{id}");
    }
    assert!(def("bog_hag").unique);
}

/// Over many seeds, the cult's daggers and the hag's staff: wielded whenever carried, and about
/// as often as their lines say; the cult's scrolls come now and then.
#[test]
fn the_cult_bears_its_daggers_and_the_hag_her_staff() {
    for (id, weapon, least, scrolls_least) in [
        ("cultist", "dagger", 160, 10),
        ("cult_priest", "dagger", 160, 50),
        ("bog_hag", "quarterstaff", 140, 0),
    ] {
        let creature = CONTENT.creature_id(id).expect(id);
        let (mut armed, mut scrolls) = (0, 0);
        for seed in 0..200 {
            let pack = gear::roll(&CONTENT, &mut Rng::gear(seed, "test"), creature, 6);
            for (i, stack) in pack.items.iter().enumerate() {
                let object = CONTENT.object(stack.object);
                if object.id == weapon {
                    assert!(pack.equipment.holds(i), "{id} {seed}: {weapon} not held");
                    armed += 1;
                }
                if object.id.starts_with("scroll_of_") {
                    scrolls += 1;
                }
            }
        }
        assert!(armed >= least, "{id}: {armed} of 200 armed");
        assert!(scrolls >= scrolls_least, "{id}: {scrolls} scrolls");
    }
}

#[test]
fn the_fire_things_shrug_off_fire_and_the_cold_cracks_them() {
    let resist = |id: &str, kind: &str| {
        let def = CONTENT.creature(CONTENT.creature_id(id).expect(id));
        let kind = CONTENT.damage_type_id(kind).expect(kind);
        (def.resist.iter())
            .find(|(k, _)| *k == kind)
            .map_or(0, |(_, r)| *r)
    };
    assert_eq!(resist("ember_beetle", "fire"), 50);
    assert_eq!(resist("fire_salamander", "fire"), 100);
    assert_eq!(resist("cinder_elemental", "fire"), 100);
    assert_eq!(resist("fire_drake", "fire"), 100);
    assert!(resist("magma_crawler", "cold") < 0);
    assert!(resist("troll", "fire") < 0);
    assert_eq!(resist("ash_wraith", "cold"), 50);
    for id in ["magma_crawler", "ash_wraith", "cinder_elemental"] {
        let def = CONTENT.creature(CONTENT.creature_id(id).expect(id));
        assert!(!def.corpse, "{id}");
    }
    let drake = CONTENT.creature(CONTENT.creature_id("fire_drake").expect("drake"));
    assert!(drake.unique);
}

/// Over many seeds, the delf's humanoids: the warriors' axes and the smiths' hammers in hand
/// nine times in ten or more, the ogres' mauls four in five, a miner's pick-axe held as a tool,
/// and whatever they wear worn.
#[test]
fn the_delf_humanoids_wield_their_weapons() {
    for (id, weapon, least) in [
        ("duergar_miner", "pick_axe", 160),
        ("duergar_warrior", "hand_axe", 160),
        ("duergar_smith", "war_hammer", 175),
        ("ogre", "maul", 140),
    ] {
        let creature = CONTENT.creature_id(id).expect(id);
        let mut held = 0;
        for seed in 0..200 {
            let pack = gear::roll(&CONTENT, &mut Rng::gear(seed, "test"), creature, 10);
            for (i, stack) in pack.items.iter().enumerate() {
                let object = CONTENT.object(stack.object);
                if object.slot.is_some() {
                    assert!(
                        pack.equipment.holds(i),
                        "{id} {seed}: {} not held",
                        object.id
                    );
                }
                if object.id == weapon {
                    held += 1;
                }
            }
        }
        assert!(held >= least, "{id}: {weapon} held {held} of 200");
    }
}
