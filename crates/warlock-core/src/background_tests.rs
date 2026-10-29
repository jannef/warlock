//! Telling a story (`background.rs`): the tiers' odds, the floor, the fallback, `only`, slots,
//! each condition, and determinism.

use super::*;
use crate::character::{CharacterSpec, Start};
use crate::content::{ContentFiles, StoryTier, When};
use std::collections::BTreeMap;
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> = LazyLock::new(|| Content::builtin().expect("built-in content"));

fn start(content: &Content, character: &str, seed: u64) -> Start {
    let spec: CharacterSpec = character.parse().expect("a character");
    let date = content.scenario("ascmearc").expect("ascmearc").start;
    spec.start(content, seed, date).expect("starts")
}

/// What a character's story may read, as `start` made them, holding `talents`.
struct Seen {
    start: Start,
    talents: Talents,
    trained: Vec<WeaponSkillId>,
}

impl Seen {
    fn new(content: &Content, character: &str, seed: u64, talents: &[&str]) -> Self {
        let start = start(content, character, seed);
        let mut held = Talents::default();
        for t in talents {
            held.take(t, None);
        }
        let trained = (start.weapon_skills.trained())
            .filter(|(_, t)| t.rank > 0)
            .map(|(id, _)| id)
            .collect();
        Self {
            start,
            talents: held,
            trained,
        }
    }

    fn witness<'a>(&'a self, content: &Content, campaign: &'a str) -> Witness<'a> {
        let date = content.scenario("ascmearc").expect("ascmearc").start;
        let c = &self.start.character;
        Witness {
            character: c,
            age: c.age_on(date),
            lucky: c.lucky(content).is_some(),
            band: content.alignment().map(|r| r.band(self.start.alignment)),
            skills: &self.start.skills,
            talents: &self.talents,
            trained: &self.trained,
            campaign,
            luck: 100,
        }
    }
}

fn ids(story: &Story) -> Vec<String> {
    story.told.iter().map(Told::id).collect()
}

#[test]
fn tier_odds_hold_over_many_seeds() {
    // Each chapter's own stream, its first draw: the tiers come as `[tiers]` says, within a
    // tenth of each (20 000 seeds, every shared chapter).
    let b = CONTENT.background().expect("stories");
    let spec: CharacterSpec = "human,female,warrior".parse().expect("a character");
    let mut counts: BTreeMap<StoryTier, u32> = BTreeMap::new();
    let mut draws = 0;
    for seed in 0..20_000 {
        for chapter in b.chapters_for("") {
            let roll = Rng::background(seed, &spec, &chapter.id).below(1000);
            *counts.entry(b.tiers.of(roll, 100)).or_default() += 1;
            draws += 1;
        }
    }
    for tier in StoryTier::ALL {
        let expected = f64::from(b.tiers.per_mille(tier)) * f64::from(draws) / 1000.0;
        let got = f64::from(counts.get(&tier).copied().unwrap_or(0));
        assert!(
            (got - expected).abs() <= expected / 10.0,
            "{}: {got} of {draws}, expected {expected}",
            tier.id()
        );
    }
}

#[test]
fn no_score_falls_below_one() {
    // Every race, gender, and class, on a thousand stories each: a story's stat points stop at
    // the floor, however they fall (no caps, one floor; gold's is `world/background_tests.rs`).
    for race in &CONTENT.races().list {
        for class in CONTENT.classes() {
            for gender in crate::character::Gender::ALL {
                let character = format!("{},{},{}", race.id, gender.id(), class.id);
                let seen = Seen::new(&CONTENT, &character, 1, &[]);
                let witness = seen.witness(&CONTENT, "ascmearc");
                for seed in 0..1000 {
                    let mut stats = seen.start.stats;
                    for effect in tell(&CONTENT, seed, &witness).effects {
                        if let Rolled::Stat(stat, by) = effect {
                            stats.gift(stat, by);
                        }
                    }
                    for stat in Stat::ALL {
                        assert!(stats.score(stat) >= 1, "{character} {seed}");
                        assert!(stats.potential(stat) >= stats.score(stat));
                    }
                }
            }
        }
    }
}

/// The built-in content with its chapters replaced by `chapters` (`(file name, text)`), every
/// campaign's too.
fn only_chapters(chapters: &[(&str, &str)]) -> Content {
    let mut files = ContentFiles::builtin();
    files.remove("data/background/");
    files.remove("campaigns/ascmearc/data/background/");
    files.remove("campaigns/showcase/data/background/");
    for (name, text) in chapters {
        files.insert(&format!("data/background/{name}"), text);
    }
    Content::load(&files).expect("valid stories")
}

const FALLBACK: &str = r#"
[chapter]
id = "one"
name = "One"
paragraph = 1
defaults = { guardian = "your aunt", home = "home", trade = "trade" }

[[line]]
id = "plain"
tier = "common"
weight = 1
text = "Nothing happened."

[[line]]
id = "kobolds_luck"
tier = "rare"
weight = 1
text = "Something did."
when = { race = ["kobold"] }
effects = [{ stat = { dexterity = 2 } }]
"#;

#[test]
fn an_empty_tier_falls_to_the_plain_line() {
    let content = only_chapters(&[("01-one.toml", FALLBACK)]);
    for (character, told) in [
        ("human,male,warrior", "one/plain"),
        ("kobold,male,warrior", "one/kobolds_luck"),
    ] {
        let seen = Seen::new(&content, character, 1, &[]);
        // Luck enough that every draw is fateful: nothing fateful fits, so the rare line, else
        // the common one.
        let witness = Witness {
            luck: 1_000_000,
            ..seen.witness(&content, "")
        };
        for seed in 0..20 {
            assert_eq!(ids(&tell(&content, seed, &witness)), [told], "{character}");
        }
    }
}

#[test]
fn the_heritage_talents_force_their_lines() {
    let seen = |talents: &[&str]| Seen::new(&CONTENT, "human,female,aristocrat", 3, talents);
    let (wealth, sword, neither) = (
        seen(&["family_wealth"]),
        seen(&["fathers_sword"]),
        seen(&[]),
    );
    for seed in 0..300 {
        let told = |s: &Seen| ids(&tell(&CONTENT, seed, &s.witness(&CONTENT, "ascmearc")));
        let rich = told(&wealth);
        assert!(rich.contains(&"parents/old_money".to_string()), "{rich:?}");
        assert!(
            !rich.contains(&"fortune/the_bequest".to_string()),
            "no second inheritance"
        );
        let heir = told(&sword);
        assert!(
            heir.contains(&"fortune/fathers_blade_told".to_string()),
            "{heir:?}"
        );
        let plain = told(&neither);
        assert!(
            !plain
                .iter()
                .any(|l| l == "parents/old_money" || l == "fortune/fathers_blade_told")
        );
    }
}

#[test]
fn texts_fill_their_slots() {
    let mut c = start(&CONTENT, "dwarf,female,warrior", 1).character;
    c.name = "Brakka".into();
    let story = |lines: &[&str]| -> Vec<Told> {
        (lines.iter())
            .map(|l| {
                let (chapter, line) = l.split_once('/').expect("chapter/line");
                Told {
                    chapter: chapter.into(),
                    line: line.into(),
                }
            })
            .collect()
    };
    // A slot nothing set reads the chapter's default; a set one, what was set, capitalised at a
    // sentence's start.
    c.story = story(&["parents/plain_folk", "birth/quiet_birth"]);
    let told = retell(&CONTENT, "ascmearc", &c);
    assert_eq!(told.len(), 1, "one paragraph: {told:?}");
    assert!(
        told[0].ends_with("which your family counted a mercy."),
        "{told:?}"
    );
    c.story = story(&[
        "parents/smiths_child",
        "birth/quiet_birth",
        "trial/the_debt",
    ]);
    let told = retell(&CONTENT, "ascmearc", &c);
    assert!(
        told[0].contains("which your father counted a mercy"),
        "{told:?}"
    );
    assert!(told[1].starts_with("Your father died owing"), "{told:?}");
    // A line's own `set` is read from the next line on.
    c.story = story(&["parents/foundling", "birth/quiet_birth"]);
    assert!(retell(&CONTENT, "ascmearc", &c)[0].contains("which the sisters counted"));
    // The name and the gender's pronouns.
    let words = BTreeMap::new();
    assert_eq!(
        fill("{They} told {them} {their} name, {name}.", &c, &words),
        "She told her her name, Brakka."
    );
    c.spec.gender = Gender::Other;
    assert_eq!(fill("{They} and {them}", &c, &words), "They and them");
    // A line the content no longer has is skipped, and said so for the log.
    c.story = story(&[
        "parents/plain_folk",
        "birth/gone_line",
        "childhood/ordinary_child",
    ]);
    let told = retell(&CONTENT, "ascmearc", &c);
    assert_eq!(told.len(), 2);
    assert!(!told[0].contains("mercy"));
    assert_eq!(
        missing(&CONTENT, "ascmearc", &c),
        ["story line `birth/gone_line` is not in the content: skipped"]
    );
    // The road is the campaign's: outside the Ascmearc its line is not there.
    c.story = story(&["road/the_last_cart"]);
    assert_eq!(retell(&CONTENT, "ascmearc", &c).len(), 1);
    assert!(retell(&CONTENT, "", &c).is_empty());
}

/// A line asking `when`, and nothing else.
fn asking(when: When) -> StoryLine {
    StoryLine {
        id: "test".into(),
        tier: StoryTier::Common,
        weight: 1,
        text: String::new(),
        when,
        only: false,
        effects: Vec::new(),
        tags: Vec::new(),
        set: BTreeMap::new(),
    }
}

#[test]
fn every_condition_reads_the_witness() {
    let seen = Seen::new(&CONTENT, "dwarf,female,warrior", 1, &["hardy"]);
    let w = seen.witness(&CONTENT, "ascmearc");
    let c = &seen.start.character;
    let sign = c.sign.clone().expect("a sign");
    let no_tags = BTreeSet::new();
    let fits = |when: When| w.fits(&asking(when), &no_tags);
    let ids = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert!(fits(When::default()));
    assert!(fits(When {
        race: ids(&["elf", "dwarf"]),
        ..When::default()
    }));
    assert!(!fits(When {
        race: ids(&["elf"]),
        ..When::default()
    }));
    assert!(fits(When {
        class: ids(&["warrior"]),
        ..When::default()
    }));
    assert!(!fits(When {
        class: ids(&["thief"]),
        ..When::default()
    }));
    assert!(fits(When {
        gender: vec![Gender::Female],
        ..When::default()
    }));
    assert!(!fits(When {
        gender: vec![Gender::Male, Gender::Other],
        ..When::default()
    }));
    assert!(fits(When {
        animal: vec![sign.animal.clone()],
        ..When::default()
    }));
    assert!(!fits(When {
        animal: ids(&["no_such"]),
        ..When::default()
    }));
    assert!(fits(When {
        element: vec![sign.element.clone()],
        ..When::default()
    }));
    assert!(!fits(When {
        element: ids(&["no_such"]),
        ..When::default()
    }));
    assert!(fits(When {
        lucky: Some(w.lucky),
        ..When::default()
    }));
    assert!(!fits(When {
        lucky: Some(!w.lucky),
        ..When::default()
    }));
    let band = w.band.expect("a band");
    let at = crate::alignment::Requirement {
        min: Some(band),
        max: Some(band),
    };
    assert!(fits(When {
        band: Some(at),
        ..When::default()
    }));
    let worse = crate::alignment::Requirement {
        min: None,
        max: Some(crate::alignment::Band::Wicked),
    };
    assert!(
        band == crate::alignment::Band::Wicked
            || !fits(When {
                band: Some(worse),
                ..When::default()
            })
    );
    assert!(fits(When {
        talents: ids(&["hardy"]),
        ..When::default()
    }));
    assert!(!fits(When {
        talents: ids(&["hardy", "lean"]),
        ..When::default()
    }));
    assert!(!fits(When {
        not_talents: ids(&["hardy"]),
        ..When::default()
    }));
    assert!(fits(When {
        not_talents: ids(&["lean"]),
        ..When::default()
    }));
    let trained = seen
        .start
        .skills
        .iter()
        .next()
        .map(|(id, _)| id)
        .expect("a skill");
    let untrained = (0..CONTENT.skills().len() as u8)
        .map(crate::skills::SkillId)
        .find(|&id| seen.start.skills.value(id).is_none())
        .expect("an untrained skill");
    assert!(fits(When {
        skills: vec![trained],
        ..When::default()
    }));
    assert!(!fits(When {
        skills: vec![trained, untrained],
        ..When::default()
    }));
    let tags: BTreeSet<String> = ["orphan".to_string()].into();
    let with_tags = |when: When| w.fits(&asking(when), &tags);
    assert!(with_tags(When {
        tags: ids(&["orphan"]),
        ..When::default()
    }));
    assert!(!fits(When {
        tags: ids(&["orphan"]),
        ..When::default()
    }));
    assert!(!with_tags(When {
        not_tags: ids(&["orphan"]),
        ..When::default()
    }));
    let age = |min, max| Some(crate::content::AgeRange { min, max });
    assert!(fits(When {
        age: age(Some(w.age), Some(w.age)),
        ..When::default()
    }));
    assert!(!fits(When {
        age: age(Some(w.age + 1), None),
        ..When::default()
    }));
    assert!(!fits(When {
        age: age(None, Some(w.age - 1)),
        ..When::default()
    }));
    assert!(fits(When {
        campaign: ids(&["ascmearc"]),
        ..When::default()
    }));
    assert!(!fits(When {
        campaign: ids(&["showcase"]),
        ..When::default()
    }));
}

#[test]
fn the_same_seed_tells_the_same_story_and_another_seed_another() {
    let seen = Seen::new(&CONTENT, "human,male,thief", 1, &[]);
    let w = seen.witness(&CONTENT, "ascmearc");
    let stories: Vec<Story> = (0..10).map(|seed| tell(&CONTENT, seed, &w)).collect();
    for (seed, story) in stories.iter().enumerate() {
        assert_eq!(*story, tell(&CONTENT, seed as u64, &w));
        assert_eq!(story.told.len(), 8, "every chapter tells a line");
    }
    let distinct: BTreeSet<Vec<String>> = stories.iter().map(ids).collect();
    assert!(distinct.len() > 5, "{distinct:?}");
    // Content without stories tells none.
    let none = crate::background::storyless();
    assert_eq!(tell(&none, 1, &w), Story::default());
}
