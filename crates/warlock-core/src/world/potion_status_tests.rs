//! Potions that lend a status for a while (spec §8 "Potions", #567): boosts, visions, quickness,
//! and resistances drunk through `eat:`; how they stack and wear off; Lasting draughts and
//! Cauldron-wise; the content checks of `vision` and `resist` on statuses.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::stats::Stat;

/// The test content: a dark hall (no light at all), the player at `@` carrying `pack`; `more`
/// adds or replaces files.
fn content_with(pack: &str, more: &[(&str, &str)]) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/draught_hall.toml",
        "name = \"Hall\"\nambient = [0, 0, 0, 0]\nmap = '''\n############\n#@.........#\n\
         ############\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/draught_hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"draught_hall\"\nseed = 7\nhp = 1000\ncarry = {pack}\n"
        ),
    );
    for (path, text) in more {
        files.insert(path, text);
    }
    Content::load(&files).map_err(|e| e.to_string())
}

fn world_with(pack: &str, more: &[(&str, &str)]) -> World {
    let content = Arc::new(content_with(pack, more).expect("test content is valid"));
    World::new(content, "draught_hall", Some(7)).expect("world")
}

fn world(pack: &str) -> World {
    world_with(pack, &[])
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

/// The player waits until `minutes` of game time have passed.
fn pass(w: &mut World, minutes: u64) {
    let until = w.state.time + Duration::minutes(minutes);
    while w.state.time < until {
        run(w, "wait");
    }
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn severity(w: &World, status: &str) -> Option<u16> {
    let status = w.content.status_id(status).expect("status");
    w.state.player().effects.get(status).map(|e| e.severity)
}

fn score(w: &World, stat: Stat) -> u8 {
    w.state.player().effective_stats(&w.content).scores()[stat]
}

#[test]
fn a_potion_of_might_raises_strength_for_ten_minutes_and_says_so() {
    let mut w = world("[\"potion_of_might\"]");
    let before = score(&w, Stat::Strength);
    run(&mut w, "eat:a");
    assert_eq!(severity(&w, "fortified"), Some(1));
    assert_eq!(score(&w, Stat::Strength), before + 2);
    let lines = said(&w);
    assert!(lines.iter().any(|m| m == "You feel stronger."), "{lines:?}");
    assert_eq!(w.state.player().stat_marks[Stat::Strength], 15);
    pass(&mut w, 9);
    assert_eq!(severity(&w, "fortified"), Some(1), "still on");
    pass(&mut w, 2);
    assert_eq!(severity(&w, "fortified"), None, "worn off");
    assert_eq!(score(&w, Stat::Strength), before);
}

#[test]
fn a_greater_boost_raises_twice_as_much_and_eases_to_a_common_one() {
    let mut w = world("[\"greater_potion_of_wit\"]");
    let before = score(&w, Stat::Intelligence);
    run(&mut w, "eat:a");
    assert_eq!(severity(&w, "keen"), Some(2));
    assert_eq!(score(&w, Stat::Intelligence), before + 4);
    pass(&mut w, 11);
    assert_eq!(severity(&w, "keen"), Some(1));
    assert_eq!(score(&w, Stat::Intelligence), before + 2);
}

#[test]
fn boosts_keep_the_stronger_dose() {
    let mut w = world("[\"greater_potion_of_grace\", \"potion_of_grace\"]");
    let before = score(&w, Stat::Dexterity);
    run(&mut w, "eat:a");
    run(&mut w, "eat:a");
    assert_eq!(severity(&w, "nimble"), Some(2), "highest, never added");
    assert_eq!(score(&w, Stat::Dexterity), before + 4);
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("Nothing more comes of it.")
    );
    // A common one first, then a greater: it grows, and the line is said again.
    let mut w = world("[\"potion_of_grace\", \"greater_potion_of_grace\"]");
    run(&mut w, "eat:a eat:a");
    assert_eq!(severity(&w, "nimble"), Some(2));
    let nimble = said(&w).iter().filter(|m| *m == "You feel nimble.").count();
    assert_eq!(nimble, 2);
}

#[test]
fn a_vision_potion_lends_its_eyes_in_the_dark() {
    let mut w = world("[\"potion_of_owls_eyes\", \"potion_of_moths_eyes\"]");
    assert_eq!(w.eyes().darkvision, 0, "human eyes");
    let far = crate::geom::Pos::new(4, 1);
    assert!(!w.perception().is_visible(far), "the dark hides it");
    run(&mut w, "eat:a");
    assert_eq!(severity(&w, "owl_eyed"), Some(1));
    assert!(w.eyes().darkvision > 0);
    assert!(w.perception().is_visible(far), "the dark hall is seen");
    assert!(!w.eyes().sees_uv);
    run(&mut w, "eat:a");
    assert!(w.eyes().sees_uv, "and the moth's");
    assert!(w.eyes().darkvision > 0, "each part the better");
    assert_eq!(w.state.player().stat_marks[Stat::Willpower], 30);
    pass(&mut w, 31);
    assert_eq!(w.eyes().darkvision, 0);
    assert!(!w.eyes().sees_uv);
}

#[test]
fn a_resistance_potion_resists_as_gear_never_immunity() {
    let mut w = world("[\"potion_of_warmth\"]");
    let fire = w.content.damage_type_id("fire").expect("fire");
    let player = w.state.player;
    assert_eq!(w.resistances_of(player).get(Some(fire)), 0);
    run(&mut w, "eat:a");
    assert_eq!(severity(&w, "warmed"), Some(1));
    assert_eq!(w.resistances_of(player).get(Some(fire)), 50);
    // With a talent's ward too, it adds up as gear does, capped short of immunity.
    let mut w = world("[\"potion_of_warmth\"]");
    w.player_mut().talents.take("kindled", None);
    run(&mut w, "eat:a");
    let player = w.state.player;
    let resisted = w.resistances_of(player).get(Some(fire));
    assert_eq!(resisted, 75);
    assert!(resisted < crate::damage::IMMUNE);
}

#[test]
fn a_potion_of_quickness_hastes() {
    let mut w = world("[\"potion_of_quickness\"]");
    run(&mut w, "eat:a");
    assert_eq!(severity(&w, "hasted"), Some(1));
    let lines = said(&w);
    assert!(
        lines.iter().any(|m| m == "The world slows around you."),
        "{lines:?}"
    );
    assert_eq!(w.state.player().stat_marks[Stat::Dexterity], 15);
    pass(&mut w, 2);
    assert_eq!(severity(&w, "hasted"), None);
}

#[test]
fn lasting_draughts_makes_boosts_and_visions_last_half_again() {
    let mut w = world("[\"potion_of_vigour\", \"potion_of_cats_eyes\", \"potion_of_frost\"]");
    w.player_mut().talents.take("lasting_draughts", None);
    run(&mut w, "eat:a eat:a eat:a");
    pass(&mut w, 14);
    assert_eq!(severity(&w, "hale"), Some(1), "15 minutes, not 10");
    pass(&mut w, 2);
    assert_eq!(severity(&w, "hale"), None);
    pass(&mut w, 25);
    assert_eq!(severity(&w, "cat_eyed"), Some(1), "45 minutes, not 30");
    assert_eq!(severity(&w, "chill_proof"), None, "a resistance as ever");
    pass(&mut w, 60);
    assert_eq!(severity(&w, "cat_eyed"), None);
}

#[test]
fn the_save_holds_a_lengthened_easing() {
    let mut w = world("[\"potion_of_vigour\"]");
    w.player_mut().talents.take("lasting_draughts", None);
    run(&mut w, "eat:a");
    pass(&mut w, 12);
    let status = w.content.status_id("hale").expect("hale");
    let wear = w.state.player().effects.get(status).expect("hale").wear;
    assert!(wear >= 600, "past the status's own pace: {wear}");
    let content = w.content.clone();
    let talents = w.state.player().talent_mods(&content);
    assert!(w.state.player().effects.is_valid_for(&content, &talents));
}

#[test]
fn potions_of_every_new_kind_describe_themselves_in_words() {
    let mut w = world("[\"potion_of_might\", \"potion_of_owls_eyes\", \"potion_of_earthing\"]");
    run(&mut w, "identify");
    let text = |w: &World, i: usize| {
        let stack = &w.state.player().inventory[i];
        crate::magic::describe::appraised(stack, w.lore(), Default::default())
            .iter()
            .map(|l| l.text.clone())
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert!(text(&w, 0).contains("Makes you stronger for a while"));
    assert!(text(&w, 1).contains("Lets you see in the dark for a while"));
    assert!(text(&w, 2).contains("Wards you against electricity for a while"));
}

#[test]
fn drinks_must_lend_the_status_their_kind_says() {
    let potion = |drink: &str| {
        format!(
            "[[object]]\nid = \"odd\"\nname = \"odd potion\"\nplural = \"odd potions\"\n\
             glyph = \"!\"\nfg = \"#d89a30\"\nclass = \"potions\"\nmaterials = [\"glass\"]\n\
             weight = 250\nvalue = 1\nstackable = true\n[object.drink]\ncolour = \"orange\"\n\
             {drink}\n"
        )
    };
    for drink in [
        // A boost that raises nothing, eyes that lend none, a ward that resists nothing.
        "kind = \"boost\"\nstatus = \"poisoned\"\nseverity = 1",
        "kind = \"vision\"\nstatus = \"fortified\"",
        "kind = \"resistance\"\nstatus = \"owl_eyed\"",
        // Past the status's max, and fields of other kinds.
        "kind = \"boost\"\nstatus = \"fortified\"\nseverity = 3",
        "kind = \"haste\"\nseverity = 1\nstatus = \"hasted\"",
        "kind = \"vision\"\nstatus = \"owl_eyed\"\ndice = \"1d4\"",
    ] {
        let text = potion(drink);
        let err = content_with("[]", &[("data/objects/odd.toml", &text)]);
        assert!(err.is_err(), "{drink}");
    }
    let fine = potion("kind = \"boost\"\nstatus = \"fortified\"\nseverity = 2");
    assert!(content_with("[]", &[("data/objects/odd.toml", &fine)]).is_ok());
}
