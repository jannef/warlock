//! Animated types generated at load (spec §8 "Animated objects", #483): their ids and order,
//! their numbers by the tier's formulas, what every one of them is, the same on every load, and
//! the files refused.

use super::{Content, ContentFiles, ID_PREFIX};
use crate::stats::Stat;

fn builtin() -> Content {
    Content::builtin().expect("content")
}

fn load_with(path: &str, text: &str) -> Result<Content, super::ContentError> {
    let mut files = ContentFiles::builtin();
    files.insert(path, text);
    Content::load(&files)
}

#[test]
fn one_type_per_body_after_the_shared_files_before_every_campaigns() {
    let content = builtin();
    let defs = content.creature_defs();
    let animated: Vec<usize> = (0..defs.len())
        .filter(|&i| defs[i].animated.is_some())
        .collect();
    let bodies = &content.animated().bodies;
    assert_eq!(animated.len(), bodies.len());
    assert_eq!(bodies.len(), 16);
    // In a row, in body (path) order, each its own family.
    for (n, &i) in animated.iter().enumerate() {
        assert_eq!(i, animated[0] + n, "in a row");
        let def = &defs[i];
        assert_eq!(def.id, format!("{ID_PREFIX}{}", bodies[n].spec.id));
        let a = def.animated.expect("animated");
        assert_eq!((a.body, usize::from(a.family.0)), (n, i));
    }
    assert_eq!(defs[animated[0]].id, "animated_bellows");
    // Every shared type before them, every campaign's after.
    assert!(defs[..animated[0]].iter().all(|d| d.campaign.is_none()));
    let after = &defs[animated[animated.len() - 1] + 1..];
    assert!(!after.is_empty() && after.iter().all(|d| d.campaign.is_some()));
}

#[test]
fn the_numbers_follow_the_tiers_formulas() {
    let content = builtin();
    let def = |id: &str| content.creature(content.creature_id(id).expect(id));
    let scores = |id: &str| Stat::ALL.map(|s| def(id).stats.score(s));
    // St Dx To In Wi Pr: the base, then small's −4 St, +4 Dx, −4 To.
    assert_eq!(scores("animated_broom"), [7, 14, 8, 2, 16, 2]);
    assert_eq!(scores("animated_coat_stand"), [10, 16, 11, 2, 18, 2]);
    let broom = def("animated_broom");
    assert_eq!((broom.level, broom.base_hp, broom.xp), (1, 1, 15));
    let damage = |id: &str| {
        def(id)
            .attack
            .as_ref()
            .expect("attack")
            .attack
            .damage
            .to_string()
    };
    assert_eq!(damage("animated_broom"), "1d2");
    let coat_stand = def("animated_coat_stand");
    assert_eq!(
        (coat_stand.level, coat_stand.base_hp, coat_stand.xp),
        (8, 10, 50)
    );
    assert_eq!(damage("animated_coat_stand"), "1d8");
    // Speeds, verbs, and the rug's hold, as their files say.
    assert_eq!(
        (
            broom.speed,
            broom.attack.as_ref().expect("attack").verb.as_str()
        ),
        (1200, "swats")
    );
    let rug = def("animated_rug");
    assert_eq!(rug.attack.as_ref().expect("attack").on_hit.len(), 1);
    // The cutlery's name is its own.
    assert_eq!(def("animated_cutlery").name, "clatter of cutlery");
    assert_eq!(def("animated_cutlery").plural, "clatters of cutlery");
    assert_eq!(def("animated_candlestick").plural, "animated candlesticks");
}

#[test]
fn a_bodys_material_group_gives_its_hide_resistances_and_death() {
    let content = builtin();
    let def = |id: &str| content.creature(content.creature_id(id).expect(id));
    let resist = |id: &str, kind: &str| {
        let kind = content.damage_type_id(kind).expect(kind);
        (def(id).resist.iter())
            .find(|(k, _)| *k == kind)
            .map_or(0, |&(_, v)| v)
    };
    // Brass is a metal: Protection 2, weak to lightning, clatters.
    let candlestick = def("animated_candlestick");
    let hide = candlestick.natural.as_ref().expect("a hide");
    assert_eq!((hide.protection, hide.name.as_str()), (2, "brass"));
    assert_eq!(resist("animated_candlestick", "electricity"), -25);
    assert_eq!(
        candlestick.death_verb(),
        "topples and lies still",
        "its own"
    );
    assert_eq!(def("animated_teapot").death_verb(), "clatters to the floor");
    // Wood burns and splinters; wool burns and falls in a heap; paper has no group.
    assert_eq!(resist("animated_chair", "fire"), -50);
    assert_eq!(def("animated_chair").death_verb(), "splinters");
    assert_eq!(resist("animated_rug", "slashing"), -25);
    assert!(def("animated_rug").natural.is_none());
    assert!(def("animated_book").resist.is_empty() && def("animated_book").natural.is_none());
    assert_eq!(def("animated_book").death_verb(), "falls still");
}

#[test]
fn every_animated_type_is_a_bloodless_construct_of_its_faction() {
    let content = builtin();
    let faction = content.factions().id("animated").expect("the faction");
    let immune: Vec<_> = ["diseased", "poisoned", "bleeding"]
        .map(|s| content.status_id(s).expect(s))
        .to_vec();
    for def in content
        .creature_defs()
        .iter()
        .filter(|d| d.animated.is_some())
    {
        assert_eq!(def.faction, Some(faction), "{}", def.id);
        assert_eq!(def.tags, ["animated", "construct"], "{}", def.id);
        assert!(!def.corpse && def.blood.is_none(), "{}", def.id);
        assert!(
            content
                .corpse_of(content.creature_id(&def.id).expect("id"))
                .is_none()
        );
        assert_eq!(def.immune, immune, "{}", def.id);
        assert_eq!(
            (def.weight, def.loot),
            (1, 0),
            "{}: its body is its weight",
            def.id
        );
        let body = content
            .animated()
            .body_of(def.animated.as_ref())
            .expect("a body");
        assert_eq!(def.glyph, content.object(body.object).glyph, "{}", def.id);
    }
    // The faction is hostile to the player only.
    let player = content.factions().id("player").expect("player");
    let undead = content.factions().id("undead").expect("undead");
    let stance = |a, b| content.factions().stance(a, b);
    assert_eq!(stance(faction, player), super::Stance::Hostile);
    assert_eq!(stance(faction, undead), super::Stance::Neutral);
}

#[test]
fn generation_is_the_same_on_every_load() {
    let (a, b) = (builtin(), builtin());
    let ids = |c: &Content| {
        c.creature_defs()
            .iter()
            .map(|d| d.id.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&a), ids(&b));
    assert_eq!(a.fingerprint_for("ascmearc"), b.fingerprint_for("ascmearc"));
}

#[test]
fn a_colliding_id_and_bad_bodies_are_refused() {
    let rat = include_str!("../../../../assets/data/creatures/rat.toml");
    let err = load_with(
        "data/creatures/zz_broom.toml",
        &rat.replace("id = \"rat\"", "id = \"animated_broom\""),
    )
    .expect_err("a collision");
    assert!(
        err.message
            .contains("duplicate creature id `animated_broom`"),
        "{}",
        err.message
    );
    let body = include_str!("../../../../assets/data/animated/stool.toml");
    for (from, to, says) in [
        ("level = 1", "level = 9", "outside its tier"),
        (
            "object = \"stool\"",
            "object = \"dagger\"",
            "not of class `household`",
        ),
        (
            "object = \"stool\"",
            "object = \"nothing\"",
            "unknown object",
        ),
        ("speed = 900", "speed = 900\ncount = [2, 3]", "stackable"),
    ] {
        let err =
            load_with("data/animated/stool.toml", &body.replace(from, to)).expect_err("refused");
        assert!(err.message.contains(says), "{to}: {}", err.message);
    }
}
