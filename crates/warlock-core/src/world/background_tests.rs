//! Background stories in a run (`world/background.rs`): each effect lands where it should, the
//! story comes after the picks, who has one, and its ids through replays and saves.

use super::*;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Rarity};
use crate::replay::Replay;
use crate::stats::Stat;
use crate::world::{Actor, WorldError};
use std::sync::Arc;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("built-in content")));

static STORYLESS: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(crate::background::storyless()));

/// The built-in content with its chapters replaced by `chapters` (`(file name, text)`).
fn only_chapters(chapters: &[(&str, &str)]) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.remove("data/background/");
    files.remove("campaigns/ascmearc/data/background/");
    files.remove("campaigns/showcase/data/background/");
    for (name, text) in chapters {
        files.insert(&format!("data/background/{name}"), text);
    }
    Arc::new(Content::load(&files).expect("valid stories"))
}

/// A chapter `id` of `paragraph` whose one forced line (`only`, asking nothing else) has
/// `body` (its `effects` and the rest).
fn forced(id: &str, paragraph: u8, body: &str) -> String {
    format!(
        "[chapter]\nid = \"{id}\"\nname = \"{id}\"\nparagraph = {paragraph}\n\n\
         [[line]]\nid = \"plain\"\ntier = \"common\"\nweight = 1\ntext = \"Nothing.\"\n\n\
         [[line]]\nid = \"told\"\ntier = \"common\"\nweight = 1\ntext = \"Told of {id}.\"\nonly = true\n{body}\n"
    )
}

fn kitted(content: &Arc<Content>, character: &str, talents: &[&str]) -> Result<World, WorldError> {
    let spec: CharacterSpec = character.parse().expect("a character");
    let spec = spec.with_talents(talents.iter().map(|t| t.to_string()).collect());
    World::start_run(
        content.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
}

fn gold(w: &World) -> u32 {
    let coin = w.content().object_id(crate::content::GOLD).expect("gold");
    crate::world::shop::count_of(&w.state().player().inventory, coin)
}

fn story(w: &World) -> Vec<String> {
    let c = w.state().character.as_ref().expect("a character");
    c.story.iter().map(|t| t.id()).collect()
}

#[test]
fn each_effect_lands_where_it_should() {
    let content = only_chapters(&[
        (
            "01-stats.toml",
            &forced(
                "stats",
                1,
                "effects = [{ stat = { strength = 2, dexterity = -2 } }]",
            ),
        ),
        (
            "02-gold.toml",
            &forced(
                "gold",
                1,
                "effects = [{ gold = 10 }, { skill = { heal = \"2d4\" } }]",
            ),
        ),
        (
            "03-marks.toml",
            &forced(
                "marks",
                2,
                "effects = [{ weapon_marks = { spears = 3 } }, { alignment = 50 }, \
                 { item = { wear = \"iron_ring\" } }]",
            ),
        ),
        (
            "04-magic.toml",
            &forced(
                "magic",
                2,
                "effects = [{ item = { carry = \"long_sword\", magic = \"magic\" } }, \
                 { stat = { intelligence = -1 } }]",
            ),
        ),
        (
            "05-scroll.toml",
            &forced("scroll", 3, "effects = [{ item = \"scroll_of_light\" }]"),
        ),
    ]);
    let character = "human,female,warrior";
    let before = kitted(&STORYLESS, character, &[]).expect("starts");
    let after = kitted(&content, character, &[]).expect("starts");
    assert_eq!(
        story(&after),
        [
            "stats/told",
            "gold/told",
            "marks/told",
            "magic/told",
            "scroll/told"
        ]
    );
    assert!(story(&before).is_empty());
    let (was, is) = (before.state().player(), after.state().player());
    // The picks are the same: the story comes after them.
    assert_eq!(was.talents, is.talents);
    // The score and its potential together.
    for (stat, by) in [
        (Stat::Strength, 2),
        (Stat::Dexterity, -2),
        (Stat::Intelligence, -1),
    ] {
        let moved = |s: u8, b: i32| (i32::from(s) + b) as u8;
        assert_eq!(
            is.stats.score(stat),
            moved(was.stats.score(stat), by),
            "{stat:?}"
        );
        assert_eq!(
            is.stats.potential(stat),
            moved(was.stats.potential(stat), by)
        );
    }
    assert_eq!(
        is.hp,
        is.max_hp(after.content()),
        "hit points to the new maximum"
    );
    assert_eq!(is.mana, is.max_mana(after.content()));
    assert_eq!(gold(&after), gold(&before) + 10);
    let heal = BUILTIN.skill_id("heal").expect("heal");
    let gained = is.skills.value(heal).unwrap_or(0) - was.skills.value(heal).unwrap_or(0);
    assert!((2..=8).contains(&gained), "{gained}");
    let spears = BUILTIN.weapon_skills().id("spears").expect("spears");
    assert_eq!(
        is.weapon_skills.rank(spears),
        was.weapon_skills.rank(spears),
        "never a rank"
    );
    assert_eq!(
        is.weapon_skills.get(spears).marks,
        was.weapon_skills.get(spears).marks + 3
    );
    let score = |a: &Actor| a.alignment.expect("an alignment").value();
    assert_eq!(score(is), score(was) + 50);
    // The story's own, after the kit's (a warrior's long sword).
    let find = |name: &str| {
        let object = BUILTIN.object_id(name).expect(name);
        (is.inventory.iter().rposition(|s| s.object == object)).expect(name)
    };
    assert!(
        is.equipment.holds(find("iron_ring")),
        "worn, a ring finger free"
    );
    let sword = &is.inventory[find("long_sword")];
    assert!(
        sword.is_identified() && sword.rarity() >= Rarity::Magic,
        "{sword:?}"
    );
    let scroll = BUILTIN.object_id("scroll_of_light").expect("scroll");
    assert!(after.state().known_scrolls.contains(&scroll));
    assert!(!before.state().known_scrolls.contains(&scroll));
    // What the kit wore and wielded is worn and wielded still.
    for (slot, item) in was.equipment.iter() {
        let now = is.equipment.get(slot).expect("still held");
        assert_eq!(is.inventory[now].object, was.inventory[item].object);
    }
}

#[test]
fn a_story_takes_gold_down_to_none() {
    let content = only_chapters(&[(
        "01-loss.toml",
        &forced("loss", 1, "effects = [{ gold = -100 }]"),
    )]);
    let before = kitted(&STORYLESS, "human,female,warrior", &[]).expect("starts");
    let after = kitted(&content, "human,female,warrior", &[]).expect("starts");
    assert!(
        gold(&before) > 0 && gold(&before) < 100,
        "{}",
        gold(&before)
    );
    assert_eq!(gold(&after), 0);
    let (was, is) = (before.state().player(), after.state().player());
    assert_eq!(
        is.inventory.len() + 1,
        was.inventory.len(),
        "the coins' stack is gone"
    );
    for (slot, item) in was.equipment.iter() {
        let now = is.equipment.get(slot).expect("still held");
        assert_eq!(is.inventory[now].object, was.inventory[item].object);
    }
}

#[test]
fn the_story_comes_after_the_picks() {
    // A story's Toughness point is no help to a pick that needs it (Hardy wants 11): the picks
    // are made first, and the story reads them.
    let content = only_chapters(&[(
        "01-hardy.toml",
        &forced(
            "hardy",
            1,
            "when = { talents = [\"hardy\"] }\neffects = [{ stat = { toughness = 1 } }]",
        ),
    )]);
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let date = content.scenario("showcase").expect("showcase").start;
    let toughness = |seed| {
        let s = spec.start(&content, seed, date).expect("starts");
        s.stats.score(Stat::Toughness)
    };
    let picks = |w: &World| {
        w.state()
            .character
            .as_ref()
            .expect("c")
            .spec
            .talents
            .clone()
    };
    let tough = |w: &World| w.state().player().stats.score(Stat::Toughness);
    let seed = (0..200)
        .find(|&s| toughness(s) == 10)
        .expect("a seed of Toughness 10");
    let run = |content: &Arc<Content>, talents: &[&str]| {
        let spec = spec
            .clone()
            .with_talents(talents.iter().map(|t| t.to_string()).collect());
        World::start_run(
            content.clone(),
            "showcase",
            Some(seed),
            Some(&spec),
            None,
            true,
        )
    };
    let err = run(&content, &["hardy"])
        .map(|_| ())
        .expect_err("Hardy refused");
    assert!(err.to_string().contains("talent `hardy`"), "{err}");
    // Picked by a character tough enough, the line that asks for it is told.
    let seed11 = (0..200)
        .find(|&s| toughness(s) == 11)
        .expect("a seed of Toughness 11");
    let spec11 = spec
        .clone()
        .with_talents(vec!["hardy".into(), "lean".into(), "keen_ears".into()]);
    let w = World::start_run(
        content.clone(),
        "showcase",
        Some(seed11),
        Some(&spec11),
        None,
        true,
    )
    .expect("starts");
    assert_eq!(story(&w), ["hardy/told"]);
    assert!(picks(&w).contains(&"hardy".to_string()));
    assert!(tough(&w) >= 12, "11 and the story's point");
    let plain = run(&content, &["lean", "keen_ears", "long_strider"]).expect("starts");
    assert_eq!(story(&plain), ["hardy/plain"]);
}

#[test]
fn only_a_chosen_character_with_the_kit_has_a_story() {
    let spec: CharacterSpec = "human,female,thief".parse().expect("a character");
    for (with, without) in [
        (
            World::new(BUILTIN.clone(), "showcase", None).expect("world"),
            World::new(STORYLESS.clone(), "showcase", None).expect("world"),
        ),
        (
            World::with_character(BUILTIN.clone(), "showcase", Some(3), Some(&spec))
                .expect("world"),
            World::with_character(STORYLESS.clone(), "showcase", Some(3), Some(&spec))
                .expect("world"),
        ),
    ] {
        assert!(story(&with).is_empty());
        assert_eq!(
            with.state_hash(),
            without.state_hash(),
            "nothing drawn, nothing saved"
        );
        let json = serde_json::to_string(with.state()).expect("json");
        assert!(
            !json.contains("\"story\""),
            "left out of the save while empty"
        );
    }
    let kitted = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(3),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    assert_eq!(story(&kitted).len(), 8, "{:?}", story(&kitted));
    assert_eq!(
        story(&kitted)
            .last()
            .map(String::as_str)
            .map(|l| l.starts_with("road/")),
        Some(true)
    );
}

#[test]
fn the_player_perceives_their_story_as_paragraphs() {
    let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
    let kitted = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(3),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let view = kitted.perception();
    let c = view.character().expect("a character");
    let told = crate::background::retell(kitted.content(), kitted.campaign(), c);
    assert_eq!(view.story(), told);
    assert!((1..=4).contains(&told.len()), "{told:?}");
    assert!(told.iter().all(|p| !p.is_empty() && !p.contains('{')));
    let plain = World::new(BUILTIN.clone(), "showcase", None).expect("world");
    assert!(plain.perception().story().is_empty(), "a scenario's own");
}

#[test]
fn a_replay_and_a_save_keep_the_story() {
    let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
    let mut world = World::start_run(
        BUILTIN.clone(),
        "ascmearc",
        Some(11),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut replay = Replay::new(&world);
    for action in crate::action::parse_actions("wait wait").expect("actions") {
        world.apply(&action);
        replay.push(&action, &world);
    }
    let (again, divergence) = replay.play(BUILTIN.clone()).expect("plays");
    assert!(divergence.is_none(), "{divergence:?}");
    assert_eq!(story(&again), story(&world));
    assert_eq!(again.state_hash(), world.state_hash());
    // A save keeps the ids and tells the text again from the content.
    let bytes = crate::save::encode(&world);
    let loaded = crate::save::decode(&bytes, BUILTIN.clone()).expect("loads");
    assert_eq!(loaded.state(), world.state());
    let character = world.state().character.clone().expect("a character");
    let told = crate::background::retell(&BUILTIN, "ascmearc", &character);
    assert_eq!(told.len(), 4, "{told:?}");
    // An edited sentence reaches the saved character; a line gone is skipped, and logged.
    let first = character.story[0].clone();
    let chapter = BUILTIN
        .background()
        .and_then(|b| b.chapter("ascmearc", &first.chapter))
        .expect("chapter");
    let text = chapter.line(&first.line).expect("line").text.clone();
    let mut files = ContentFiles::builtin();
    files.edit_table("background", |t| {
        t.replace(&text, "You were someone else entirely.")
    });
    let edited = Arc::new(Content::load(&files).expect("valid"));
    let loaded = crate::save::decode(&bytes, edited.clone()).expect("loads");
    let c = loaded.state().character.as_ref().expect("a character");
    assert!(
        crate::background::retell(&edited, "ascmearc", c)[0]
            .starts_with("You were someone else entirely.")
    );
    let mut files = ContentFiles::builtin();
    let id = format!("id = \"{}\"", first.line);
    files.edit_table("background", |t| t.replace(&id, "id = \"renamed_line\""));
    let renamed = Arc::new(Content::load(&files).expect("valid"));
    let loaded = crate::save::decode(&bytes, renamed.clone()).expect("loads");
    let c = loaded.state().character.as_ref().expect("a character");
    assert!(
        !crate::background::retell(&renamed, "ascmearc", c)
            .concat()
            .contains(&text)
    );
    let log = format!("story line `{}` is not in the content: skipped", first.id());
    assert!(loaded.retiered().contains(&log), "{:?}", loaded.retiered());
}

#[test]
fn the_fateful_lines_at_pinned_seeds() {
    // Found by a search over seeds (a human warrior on Beorggeat, the Random picks): a pool edit
    // that moves them fails this, as the samples.
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let run = |seed| {
        World::start_run(
            BUILTIN.clone(),
            "ascmearc",
            Some(seed),
            Some(&spec),
            None,
            true,
        )
        .expect("starts")
    };
    // The flux: Hardy, kept among the boons, and the page says the story gave it.
    let w = run(107);
    assert!(
        story(&w).contains(&"trial/the_flux".to_string()),
        "{:?}",
        story(&w)
    );
    let me = w.state().player();
    assert_eq!(me.talents.rank_of("hardy", None), 1);
    assert!(me.boons.iter().any(|b| b.id == "hardy"));
    assert!(talent_rows(&w).contains(&"Hardy 1/3 (story)".to_string()));
    // The named blade: a rare weapon, identified.
    let w = run(398);
    assert!(
        story(&w).contains(&"fortune/the_named_blade".to_string()),
        "{:?}",
        story(&w)
    );
    let me = w.state().player();
    let weapons = [
        "long_sword",
        "rapier",
        "scimitar",
        "battle_axe",
        "war_hammer",
        "quarterstaff",
    ];
    let blade = (me.inventory.iter())
        .find(|s| {
            s.rarity() == Rarity::Rare && weapons.contains(&BUILTIN.object(s.object).id.as_str())
        })
        .expect("a rare weapon");
    assert!(blade.is_identified(), "{blade:?}");
}

/// Creation picks no test of a story talent reads (a human's three).
const PICKS: [&str; 3] = ["keen_ears", "lean", "healthy"];

fn talent_rows(w: &World) -> Vec<String> {
    crate::dump::layer_rows(w, crate::dump::Layer::Talents)
}

#[test]
fn a_story_talent_is_given_at_rank_one_and_kept() {
    let content = only_chapters(&[(
        "01-blessed.toml",
        &forced(
            "blessed",
            1,
            "effects = [{ talent = \"hardy\" }, { stat = { presence = -1 } }]",
        ),
    )]);
    let character = "human,male,warrior";
    let before = kitted(&STORYLESS, character, &PICKS).expect("starts");
    let w = kitted(&content, character, &PICKS).expect("starts");
    assert_eq!(story(&w), ["blessed/told"]);
    let me = w.state().player();
    assert_eq!(me.talents.rank_of("hardy", None), 1);
    let boons: Vec<&str> = me.boons.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(boons, ["hardy"]);
    // What its rank hands out (Hardy's hit points at level 1), and the pool full.
    let was = before.state().player().max_hp(&STORYLESS);
    assert!(me.max_hp(&content) > was, "{} > {was}", me.max_hp(&content));
    assert_eq!(me.hp, me.max_hp(&content));
    assert!(talent_rows(&w).contains(&"Hardy 1/3 (story)".to_string()));
    let own = w.perception().own_talents();
    let hardy = own.iter().find(|t| t.name == "Hardy").expect("held");
    assert!(hardy.story && !hardy.boon && !hardy.granted);
    // A save and a restore keep it.
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let again = loaded.state().player();
    assert_eq!(again.talents.rank_of("hardy", None), 1);
    assert_eq!(again.max_hp(&content), me.max_hp(&content));
    assert!(talent_rows(&loaded).contains(&"Hardy 1/3 (story)".to_string()));
}

#[test]
fn a_held_talent_is_never_given_by_a_story() {
    let content = only_chapters(&[
        (
            "01-first.toml",
            &forced(
                "first",
                1,
                "effects = [{ talent = \"lean\" }, { stat = { presence = -1 } }]",
            ),
        ),
        (
            "02-again.toml",
            &forced(
                "again",
                1,
                "effects = [{ talent = \"lean\" }, { stat = { presence = -1 } }]",
            ),
        ),
    ]);
    let character = "human,male,warrior";
    // Given once: the second line, giving it again, does not fit, and its chapter is plain.
    let w = kitted(
        &content,
        character,
        &["keen_ears", "healthy", "long_strider"],
    )
    .expect("starts");
    assert_eq!(story(&w), ["first/told", "again/plain"]);
    assert_eq!(w.state().player().boons.len(), 1);
    // Picked at creation: neither line is told.
    let picked = kitted(&content, character, &PICKS).expect("starts");
    assert_eq!(story(&picked), ["first/plain", "again/plain"]);
    assert!(picked.state().player().boons.is_empty());
    assert_eq!(picked.state().player().talents.rank_of("lean", None), 1);
}

#[test]
fn a_class_table_naming_a_story_talent_later_gives_nothing_in_its_place() {
    let content = only_chapters(&[(
        "01-nimble.toml",
        &forced(
            "nimble",
            1,
            "effects = [{ talent = \"evasion\" }, { stat = { presence = -1 } }]",
        ),
    )]);
    let mut w = kitted(&content, "human,male,thief", &PICKS).expect("starts");
    assert_eq!(story(&w), ["nimble/told"]);
    let held = |w: &World| {
        (w.state().player().talents.iter())
            .filter(|t| t.id == "evasion")
            .count()
    };
    assert_eq!(held(&w), 1);
    // The thief's table gives Evasion at level 18: held already, it gives nothing.
    let spec = w.state().character.as_ref().map(|c| &c.spec);
    let factor = crate::content::Factor::of(w.content(), spec);
    let need = w.content().experience().threshold_for(18, factor);
    let more = need - w.player_xp();
    for action in crate::action::parse_actions(&format!("xp:{more}")).expect("actions") {
        w.apply(&action);
    }
    assert_eq!(w.player_level(), 18);
    assert_eq!(held(&w), 1);
    let me = w.state().player();
    assert_eq!(me.talents.rank_of("evasion", None), 1);
    assert_eq!(me.boons.len(), 1);
    assert!(talent_rows(&w).contains(&"Evasion (story)".to_string()));
}

#[test]
fn a_story_flaw_is_a_sign_flaw_marked_as_the_storys() {
    let content = only_chapters(&[(
        "01-sickly.toml",
        &forced("sickly", 1, "effects = [{ flaw = \"thin_skinned\" }]"),
    )]);
    let w = kitted(&content, "human,female,warrior", &PICKS).expect("starts");
    assert_eq!(story(&w), ["sickly/told"]);
    let me = w.state().player();
    assert_eq!(me.talents.rank_of("thin_skinned", None), 1);
    assert_eq!(me.boons.len(), 1);
    let own = w.perception().own_talents();
    let frail = own.iter().find(|t| t.name == "Thin-skinned").expect("held");
    assert!(frail.story && frail.flaw && !frail.boon);
    assert!(talent_rows(&w).contains(&"Thin-skinned (story, flaw)".to_string()));
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
}
