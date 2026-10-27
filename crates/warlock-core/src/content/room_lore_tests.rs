//! A special room's `loot` and `train` (spec §7 "Special rooms", a scriptorium's): parsed and
//! checked, resolved once the objects and skills are known, refused when they name nothing, never
//! over a whole level, and `train` multiplied by [`Here::train`].

use super::room_lore::{LOOT_COUNT, TRAIN};
use super::{Content, ContentFiles, Here, ItemClassId, RoomEffectId, RoomEffects};
use crate::skills::SkillId;

fn effect(extra: &str) -> String {
    format!("[[effect]]\nid = \"x\"\nname = \"x\"\nmark = \"x\"\nenter = \"In.\"\n{extra}\n")
}

fn parse(extra: &str) -> Result<RoomEffects, String> {
    RoomEffects::parse("e.toml", &effect(extra), |_| None).map_err(|e| e.message)
}

#[test]
fn loot_and_train_are_checked_and_resolved() {
    let mut effects =
        parse("loot = { class = \"scrolls\", count = [2, 4] }\ntrain = { decipher_script = 200 }")
            .expect("valid");
    let def = effects.get(RoomEffectId(0));
    assert_eq!(
        (def.loot, def.train.as_slice()),
        (None, &[][..]),
        "until resolved"
    );
    assert!(!def.can_pervade());
    effects
        .resolve_lore(
            |n| (n == "scrolls").then_some(ItemClassId(7)),
            (
                |n: &str| (n == "decipher_script").then_some(SkillId(3)),
                true,
            ),
        )
        .expect("resolves");
    let def = effects.get(RoomEffectId(0));
    let loot = def.loot.expect("loot");
    assert_eq!((loot.class, loot.count), (ItemClassId(7), [2, 4]));
    assert_eq!(def.train, vec![(SkillId(3), 200)]);
    // The ends of the ranges parse.
    let (least, most) = (LOOT_COUNT.start(), LOOT_COUNT.end());
    assert!(
        parse(&format!(
            "loot = {{ class = \"c\", count = [{least}, {most}] }}"
        ))
        .is_ok()
    );
    for p in [TRAIN.start(), TRAIN.end()] {
        assert!(parse(&format!("train = {{ s = {p} }}")).is_ok(), "{p}");
    }
}

#[test]
fn bad_numbers_and_unknown_names_are_refused() {
    for bad in ["[0, 2]", "[3, 2]", "[1, 17]"] {
        let e = parse(&format!("loot = {{ class = \"c\", count = {bad} }}")).unwrap_err();
        assert!(e.contains("loot.count"), "{bad}: {e}");
    }
    for bad in [0, 1001] {
        let e = parse(&format!("train = {{ stealth = {bad} }}")).unwrap_err();
        assert!(e.contains(&format!("train.stealth {bad}")), "{e}");
    }
    let e = parse("level_chance = 5\npervade = \"All.\"\nloot = { class = \"c\", count = [1, 1] }")
        .unwrap_err();
    assert!(e.contains("level_chance"), "{e}");
    // Names that are no class or skill are refused once the content loads.
    for (from, to, says) in [
        (
            "class = \"scrolls\"",
            "class = \"tomes\"",
            "unknown item class `tomes`",
        ),
        (
            "decipher_script = 200",
            "calligraphy = 200",
            "unknown skill `calligraphy`",
        ),
    ] {
        let mut files = ContentFiles::builtin();
        files.edit_table("room_effects", |text| text.replace(from, to));
        let e = Content::load(&files).unwrap_err();
        assert!(e.message.contains(says), "{}", e.message);
    }
}

#[test]
fn train_multiplies_the_room_and_the_level() {
    let text = format!(
        "{}{}",
        effect("train = { decipher_script = 200 }"),
        effect("train = { decipher_script = 150, stealth = 50 }").replace("\"x\"", "\"y\"")
    );
    let mut effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
    let skill = |n: &str| match n {
        "decipher_script" => Some(SkillId(0)),
        "stealth" => Some(SkillId(1)),
        _ => None,
    };
    effects
        .resolve_lore(|_| None, (skill, true))
        .expect("resolves");
    let (a, b) = (effects.get(RoomEffectId(0)), effects.get(RoomEffectId(1)));
    let (decipher, stealth, other) = (SkillId(0), SkillId(1), SkillId(2));
    assert_eq!(Here::default().train(decipher), 100);
    let room = Here {
        room: Some(a),
        level: None,
    };
    assert_eq!((room.train(decipher), room.train(stealth)), (200, 100));
    let both = Here {
        room: Some(a),
        level: Some(b),
    };
    assert_eq!(
        (both.train(decipher), both.train(stealth), both.train(other)),
        (300, 50, 100)
    );
}

#[test]
fn the_scriptorium_is_built_in() {
    let content = Content::load(&ContentFiles::builtin()).expect("content");
    let id = content
        .room_effects()
        .id("scriptorium")
        .expect("scriptorium");
    let def = content.room_effect(id);
    assert_eq!((def.mark, def.weight, def.min_cells), ('r', 5, 9));
    assert_eq!(def.feature, content.terrain_id("shelves"));
    let loot = def.loot.expect("loot");
    assert_eq!(Some(loot.class), content.item_class_id("scrolls"));
    assert_eq!(loot.count, [2, 4]);
    let decipher = content.skill_id("decipher_script").expect("skill");
    assert_eq!(def.train, vec![(decipher, 200)]);
    assert!(def.near.is_some() && !def.can_pervade());
    let shelves = content.terrain(def.feature.expect("shelves"));
    assert!(!shelves.walkable && shelves.transparent);
}
