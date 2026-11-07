//! The duergar (#518): their stance toward the others, and their weapons and ammunition.

use super::{Combat, Content, Stance};

/// The duergar who wield their racial weapons or shoot (the miners, warriors, and smiths of the
/// delf are `bestiary_tests`').
const ARMED: &[&str] = &[
    "duergar_slinger",
    "duergar_crossbowman",
    "duergar_veteran",
    "duergar_priest",
    "duergar_hulk",
    "duergar_spider_rider",
    "duergar_mindbreaker",
    "duergar_thane",
    "duergar_king",
];

#[test]
fn the_duergar_hunt_the_player_and_leave_their_old_foes_be() {
    let content = Content::builtin().expect("content");
    let stance = |a: &str, b: &str| {
        let id = |n: &str| content.creature_id(n).expect(n);
        content.stance(id(a), id(b))
    };
    for (a, b, want) in [
        ("duergar_slinger", "player", Stance::Hostile),
        ("duergar_crossbowman", "goblin", Stance::Neutral),
        ("duergar_veteran", "orc_warrior", Stance::Neutral),
        ("duergar_priest", "skeleton", Stance::Neutral),
        ("duergar_hulk", "hill_giant", Stance::Neutral),
        ("duergar_spider_rider", "troll", Stance::Neutral),
        ("duergar_thane", "dretch", Stance::Neutral),
        ("duergar_king", "duergar_miner", Stance::Neutral),
        ("duergar_mindbreaker", "cultist", Stance::Neutral),
        ("duergar_priest", "kobold", Stance::Neutral),
        ("duergar_veteran", "fire_salamander", Stance::Neutral),
        ("duergar_slinger", "duergar_trader", Stance::Neutral),
    ] {
        assert_eq!(stance(a, b), want, "{a} and {b}");
    }
}

#[test]
fn the_duergar_wield_their_weapons_and_the_shooters_their_ammunition() {
    use crate::gear;
    use crate::rng::Rng;
    let content = Content::builtin().expect("content");
    for &id in ARMED {
        let creature = content.creature_id(id).expect(id);
        let def = content.creature(creature);
        let mut armed = 0;
        let mut loaded = 0;
        for seed in 0..200 {
            let pack = gear::roll(&content, &mut Rng::gear(seed, "test"), creature, 10);
            let has =
                |object: &str| (pack.items.iter()).any(|s| content.object(s.object).id == object);
            if (has("sling") && has("sling_stone")) || (has("crossbow") && has("bolt")) {
                loaded += 1;
            }
            for (i, stack) in pack.items.iter().enumerate() {
                let object = content.object(stack.object);
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
        if def.gear.racial_weapon.is_some() {
            assert!(armed >= 160, "{id}: {armed} of 200 armed");
        } else {
            assert_eq!(armed, 0, "{id}: fights bare");
        }
        if def.combat == Combat::Ranged {
            assert!(loaded >= 150, "{id}: {loaded} of 200 with ammunition");
        }
    }
}
