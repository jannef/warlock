//! The `background` system's talents (#705): Favoured child's doubled odds, Hard-won's marks for
//! the points a story took, Storyteller's favour by the story's rare and fateful lines, and the
//! creation-only refusal.

use super::*;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, StoryTier};
use crate::replay::Replay;
use crate::rng::Rng;
use crate::stats::{PerStat, Stat, Stats};
use std::collections::BTreeMap;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("built-in content")));

/// The built-in content with `tiers` (the `[tiers]` table's body) and its chapters replaced by
/// `chapters` (`(file name, text)`).
fn stories(tiers: &str, chapters: &[(&str, &str)]) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/background.toml",
        &format!(
            "[tiers]\n{tiers}\n\n[points]\nstat = 2\nskill = 1\nweapon_marks = 1\ngold_per = 25\n\
             item = 1\nmagic_any = 2\nmagic = 3\nmagic_rare = 5\ntalent = 4\nflaw = 3\n\
             alignment = 0\n"
        ),
    );
    files.remove("data/background/");
    files.remove("campaigns/ascmearc/data/background/");
    files.remove("campaigns/showcase/data/background/");
    for (name, text) in chapters {
        files.insert(&format!("data/background/{name}"), text);
    }
    Arc::new(Content::load(&files).expect("valid stories"))
}

/// A kitted run of the showcase as `character` with `talents` named first, the creation picks
/// left filled with the first talents offered outside the Heritage tree (so no Random pick
/// takes a heritage talent by chance).
fn start(
    content: &Arc<Content>,
    character: &str,
    talents: &[&str],
    seed: u64,
) -> Result<World, WorldError> {
    let spec: CharacterSpec = character.parse().expect("a character");
    let draft = World::draft(content.clone(), "showcase", Some(seed), &spec)?;
    let book = content.talents();
    let heritage = book.tree_id("heritage").expect("heritage");
    let mut picks: Vec<String> = talents.iter().map(|t| t.to_string()).collect();
    loop {
        let (left, offers) = draft
            .creation_offers(&picks)
            .map_err(WorldError::Character)?;
        if left == 0 {
            break;
        }
        let (id, ways) = (offers.iter())
            .find(|(id, _)| book.get(*id).tree != heritage)
            .expect("a talent to pick");
        let id = &book.get(*id).id;
        picks.push(crate::world::pick_text(id, ways[0].as_deref()));
    }
    World::start_run(
        content.clone(),
        "showcase",
        Some(seed),
        Some(&spec.with_talents(picks)),
        None,
        true,
    )
}

fn story(w: &World) -> Vec<String> {
    let c = w.state().character.as_ref().expect("a character");
    c.story.iter().map(|t| t.id()).collect()
}

#[test]
fn favoured_child_doubles_the_luckier_odds() {
    // The tiers as each chapter's stream draws them, at twice the odds: uncommon, rare, and
    // fateful within a tenth of 24 %, 6 %, 1 % (20 000 seeds, every shared chapter).
    let b = BUILTIN.background().expect("stories");
    let spec: CharacterSpec = "human,female,warrior".parse().expect("a character");
    let mut counts: BTreeMap<StoryTier, u32> = BTreeMap::new();
    let mut draws = 0;
    for seed in 0..20_000 {
        for chapter in b.chapters_for("") {
            let roll = Rng::background(seed, &spec, &chapter.id).below(1000);
            *counts.entry(b.tiers.of(roll, 200)).or_default() += 1;
            draws += 1;
        }
    }
    for tier in [StoryTier::Uncommon, StoryTier::Rare, StoryTier::Fateful] {
        let expected = 2.0 * f64::from(b.tiers.per_mille(tier)) * f64::from(draws) / 1000.0;
        let got = f64::from(counts.get(&tier).copied().unwrap_or(0));
        assert!(
            (got - expected).abs() <= expected / 10.0,
            "{}: {got} of {draws}, expected {expected}",
            tier.id()
        );
    }
    // In a run: with fateful odds of a half, the talent makes them certain; without it the
    // chapter is plain about half the time, and the story stream is the same as before.
    let lucky = "[chapter]\nid = \"luck\"\nname = \"Luck\"\nparagraph = 1\n\n\
        [[line]]\nid = \"plain\"\ntier = \"common\"\nweight = 1\ntext = \"Nothing.\"\n\n\
        [[line]]\nid = \"star\"\ntier = \"fateful\"\nweight = 1\ntext = \"A star.\"\n\
        effects = [{ stat = { strength = 2, dexterity = 2 } }]\n";
    let content = stories(
        "uncommon = 0\nrare = 0\nfateful = 500",
        &[("01-luck.toml", lucky)],
    );
    let character = "human,male,warrior";
    let mut plain = 0;
    for seed in 0..40 {
        let favoured = start(&content, character, &["favoured_child"], seed).expect("starts");
        assert_eq!(story(&favoured), ["luck/star"], "seed {seed}");
        let w = start(&content, character, &[], seed).expect("starts");
        let spec: CharacterSpec = character.parse().expect("a character");
        let roll = Rng::background(seed, &spec, "luck").below(1000);
        let expected = match roll < 500 {
            true => "luck/star",
            false => "luck/plain",
        };
        assert_eq!(story(&w), [expected], "seed {seed}");
        plain += usize::from(expected == "luck/plain");
    }
    assert!((10..=30).contains(&plain), "{plain} plain of 40");
}

#[test]
fn hard_won_marks_every_point_a_story_took() {
    let trade = "[chapter]\nid = \"trade\"\nname = \"Trade\"\nparagraph = 1\n\n\
        [[line]]\nid = \"plain\"\ntier = \"common\"\nweight = 1\ntext = \"Nothing.\"\n\n\
        [[line]]\nid = \"told\"\ntier = \"common\"\nweight = 1\ntext = \"A trade.\"\nonly = true\n\
        effects = [{ stat = { strength = -2, dexterity = 1 } }]\n";
    let content = stories(
        "uncommon = 120\nrare = 30\nfateful = 5",
        &[("01-trade.toml", trade)],
    );
    let character = "human,male,warrior";
    let plain = start(&content, character, &[], 3).expect("starts");
    let won = start(&content, character, &["hard_won"], 3).expect("starts");
    assert_eq!(story(&won), ["trade/told"]);
    let (p, w) = (plain.state().player(), won.state().player());
    // Three marks a point taken; none for the point given; without the talent, none at all.
    assert_eq!(w.stat_marks[Stat::Strength], 6);
    assert_eq!(w.stat_marks[Stat::Dexterity], 0);
    assert_eq!(p.stat_marks[Stat::Strength], 0);
}

#[test]
fn storyteller_favours_a_tale_worth_hearing() {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let mut w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    w.player_mut().stats = Stats::new(PerStat::splat(12), PerStat::splat(40));
    let shops = w.content().shops().expect("shops").clone();
    let tale = [
        StoryTier::Common,
        StoryTier::Rare,
        StoryTier::Uncommon,
        StoryTier::Rare,
        StoryTier::Fateful,
    ];
    let favour = |w: &World, story: &[StoryTier]| {
        super::shop::favour(w.content(), &shops, w.state.player(), story)
    };
    let (none, told) = (favour(&w, &[]), favour(&w, &tale));
    assert_eq!(none, told, "a story alone moves nothing");
    w.player_mut().talents.take("storyteller", None);
    assert_eq!(favour(&w, &[]), none + 5);
    assert_eq!(
        favour(&w, &[StoryTier::Common, StoryTier::Uncommon]),
        none + 5
    );
    // Two rare lines and a fateful one: 5 + 2 × 5 + 10.
    assert_eq!(favour(&w, &tale), none + 25);
    // The world reads the player's own story: with none, the plain +5.
    assert_eq!(w.player_favour(), none + 5);
}

#[test]
fn the_story_talents_of_creation_are_refused_at_a_level() {
    let mut world = start(&BUILTIN, "human,male,warrior", &[], 1).expect("starts");
    for action in crate::action::parse_actions("xp:2431").expect("actions") {
        world.apply(&action);
    }
    for (id, name) in [
        ("favoured_child", "Favoured child"),
        ("hard_won", "Hard-won"),
    ] {
        for action in crate::action::parse_actions(&format!("talent:{id}")).expect("actions") {
            world.apply(&action);
        }
        let said = &world.messages().iter().last().expect("a refusal").text;
        assert_eq!(
            *said,
            format!("You cannot take {name}: you need to be a new character.")
        );
        assert!(!world.state().player().talents.has(id, 1));
    }
}

#[test]
fn a_favoured_child_replays() {
    let mut world =
        start(&BUILTIN, "human,female,warrior", &["favoured_child"], 11).expect("starts");
    assert!(world.state().player().talents.has("favoured_child", 1));
    let mut replay = Replay::new(&world);
    for action in crate::action::parse_actions("wait wait").expect("actions") {
        world.apply(&action);
        replay.push(&action, &world);
    }
    let (again, divergence) = replay.play(BUILTIN.clone()).expect("plays");
    assert!(divergence.is_none(), "{divergence:?}");
    assert_eq!(story(&again), story(&world));
    assert_eq!(again.state_hash(), world.state_hash());
}
