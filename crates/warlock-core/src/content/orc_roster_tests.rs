//! The orc tribe (#455): their stance toward the tribes they raid, and their weapons.

use crate::test_support::builtin;

/// Every orc type, and whether it is a beast (the wargs).
const ORCS: &[(&str, bool)] = &[
    ("orc_runt", false),
    ("orc_warrior", false),
    ("orc_archer", false),
    ("warg", true),
    ("orc_raider", false),
    ("orc_shaman", false),
    ("orc_berserker", false),
    ("orc_veteran", false),
    ("orc_bowman", false),
    ("orc_war_chief", false),
    ("dire_warg", true),
    ("orc_high_shaman", false),
    ("orc_warlord", false),
];

#[test]
fn the_orcs_hunt_the_player_and_leave_the_tribes_they_raid_be() {
    use super::Stance;
    let content = builtin();
    let stance = |a: &str, b: &str| {
        let id = |n: &str| content.creature_id(n).expect(n);
        content.stance(id(a), id(b))
    };
    for (a, b, want) in [
        ("orc_warrior", "player", Stance::Hostile),
        ("orc_warrior", "kobold", Stance::Neutral),
        ("orc_raider", "rat", Stance::Neutral),
        ("orc_veteran", "duergar_warrior", Stance::Neutral),
        ("orc_shaman", "skeleton", Stance::Neutral),
        ("warg", "orc_runt", Stance::Neutral),
        ("orc_warrior", "goblin", Stance::Neutral),
        ("orc_berserker", "ogre", Stance::Neutral),
        ("orc_archer", "giant_leech", Stance::Neutral),
        ("orc_bowman", "fire_salamander", Stance::Neutral),
    ] {
        assert_eq!(stance(a, b), want, "{a} and {b}");
    }
}

#[test]
fn the_orcs_are_shared_and_wield_their_weapons() {
    use crate::gear;
    use crate::rng::Rng;
    let content = builtin();
    for &(id, beast) in ORCS {
        let creature = content.creature_id(id).expect(id);
        assert_eq!(content.creature(creature).campaign, None, "{id}: shared");
        let mut armed = 0;
        let mut quivers = 0;
        for seed in 0..200 {
            let pack = gear::roll(&content, &mut Rng::gear(seed, "test"), creature, 10);
            let has =
                |object: &str| (pack.items.iter()).any(|s| content.object(s.object).id == object);
            if (has("short_bow") || has("composite_bow")) && has("arrow") {
                quivers += 1;
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
        // The racial weapon comes nine times in ten; the wargs carry nothing.
        if beast {
            assert_eq!(armed, 0, "{id}");
        } else {
            assert!(armed >= 160, "{id}: {armed} of 200 armed");
        }
        if id == "orc_archer" || id == "orc_bowman" {
            assert!(quivers >= 150, "{id}: {quivers} of 200 with bow and arrows");
        }
    }
}
