//! Background stories' data: the built-in set, the hidden-number rule, and every refusal.

use super::*;
use crate::content::ContentFiles;
use crate::test_support::CONTENT;

/// A chapter file of paragraph 4 with a plain line and `lines`.
fn chapter(id: &str, lines: &str) -> String {
    format!(
        "[chapter]\nid = \"{id}\"\nname = \"Test\"\nparagraph = 4\n\n\
         [[line]]\nid = \"plain\"\ntier = \"common\"\nweight = 1\ntext = \"Nothing.\"\n\n{lines}"
    )
}

/// The built-in content with `files` added (`(path, text)`), loaded.
fn with(files: &[(&str, &str)]) -> Result<Content, String> {
    let mut all = ContentFiles::builtin();
    for (path, text) in files {
        all.insert(path, text);
    }
    Content::load(&all).map_err(|e| e.to_string())
}

/// The error a test chapter with `lines` gives.
fn refused(lines: &str) -> String {
    with(&[("data/background/99-test.toml", &chapter("test", lines))]).expect_err(lines)
}

/// A line `x` with `body` (its `tier` and the rest).
fn line(body: &str) -> String {
    format!("[[line]]\nid = \"x\"\nweight = 1\ntext = \"A thing.\"\n{body}\n")
}

#[test]
fn the_built_in_stories_load_with_every_effect_kind_and_tier() {
    let b = CONTENT.background().expect("background stories");
    assert_eq!(b.tiers.per_mille(Tier::Common), 845);
    let shared: Vec<&str> = (b.chapters.iter())
        .filter(|c| c.campaign.is_none())
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(
        shared,
        [
            "parents",
            "birth",
            "childhood",
            "youth",
            "trial",
            "fortune",
            "leaving",
            "companion"
        ]
    );
    let road: Vec<&str> = b.chapters_for("ascmearc").map(|c| c.id.as_str()).collect();
    assert_eq!(road.len(), 9);
    assert_eq!(road.last(), Some(&"road"));
    assert_eq!(
        b.chapters_for("").count(),
        8,
        "a story outside every campaign"
    );
    let lines: Vec<&Line> = b.chapters.iter().flat_map(|c| &c.lines).collect();
    for tier in Tier::ALL {
        assert!(lines.iter().any(|l| l.tier == tier), "{}", tier.id());
    }
    let effects: Vec<&Effect> = lines.iter().flat_map(|l| &l.effects).collect();
    let has = |f: fn(&Effect) -> bool| effects.iter().any(|e| f(e));
    assert!(has(|e| matches!(e, Effect::Stat(_, by) if *by > 0)));
    assert!(has(|e| matches!(e, Effect::Stat(_, by) if *by < 0)));
    assert!(has(|e| matches!(e, Effect::Gold(d) if d.min() > 0)));
    assert!(has(|e| matches!(e, Effect::Gold(d) if d.max() < 0)));
    assert!(has(|e| matches!(e, Effect::Item(i) if i.magic.is_none())));
    assert!(has(
        |e| matches!(e, Effect::Item(i) if i.magic.is_some() && i.prefer_trained)
    ));
    assert!(has(|e| matches!(e, Effect::Skill(..))));
    assert!(has(|e| matches!(e, Effect::WeaponMarks(Marks::Best, _))));
    assert!(has(|e| matches!(
        e,
        Effect::WeaponMarks(Marks::Skill(_), _)
    )));
    assert!(has(|e| matches!(e, Effect::Alignment(_))));
    assert!(has(
        |e| matches!(e, Effect::Talent(id, None) if id == "hardy")
    ));
    // The heritage talents force their chapters (design §6.2).
    let forced: Vec<&str> = (lines.iter())
        .filter(|l| l.only)
        .flat_map(|l| &l.when.talents)
        .map(String::as_str)
        .collect();
    assert_eq!(forced, ["family_wealth", "fathers_sword"]);
}

#[test]
fn texts_hide_the_numbers() {
    // Words only (design §8.3): no stat's name, no number, nothing of how a story is drawn.
    let b = CONTENT.background().expect("background stories");
    let banned = [
        "seed",
        "rare",
        "tier",
        "talent",
        "bonus",
        "rolled",
        "generated",
    ];
    for line in b.chapters.iter().flat_map(|c| &c.lines) {
        let text = line.text.to_lowercase();
        let words: Vec<&str> = text.split(|c: char| !c.is_alphanumeric()).collect();
        for stat in Stat::ALL {
            let name = stat.name().to_lowercase();
            assert!(!text.contains(&name), "{}: {name}", line.id);
        }
        for word in banned {
            assert!(!words.contains(&word), "{}: `{word}`", line.id);
        }
        assert!(
            !text.chars().any(|c| c.is_ascii_digit()),
            "{}: a number",
            line.id
        );
        assert!(!text.contains(['+', '−']), "{}: a sign", line.id);
    }
}

#[test]
fn slots_are_the_known_ones() {
    assert_eq!(
        slots_of("{They} met {guardian} at {home}."),
        Ok(vec!["They", "guardian", "home"])
    );
    assert!(
        slots_of("{uncle}")
            .unwrap_err()
            .contains("unknown slot `{uncle}`")
    );
    assert!(slots_of("{home").unwrap_err().contains("without its `}`"));
    assert!(slots_of("home}").unwrap_err().contains("without its `{`"));
}

#[test]
fn bad_lines_are_refused() {
    for (body, why) in [
        ("tier = \"legendary\"", "unknown variant"),
        ("tier = \"common\"\ntext2 = 1", "unknown field"),
        (
            "tier = \"common\"\nwhen = { talents = [\"no_such\"] }",
            "unknown talent `no_such`",
        ),
        (
            "tier = \"common\"\nwhen = { race = [\"orc\"] }",
            "unknown race `orc`",
        ),
        (
            "tier = \"common\"\nwhen = { animal = [\"dragon\"] }",
            "unknown `dragon`",
        ),
        (
            "tier = \"common\"\nwhen = { skills = [\"juggling\"] }",
            "unknown skill `juggling`",
        ),
        (
            "tier = \"common\"\nwhen = { campaign = [\"nowhere\"] }",
            "unknown campaign `nowhere`",
        ),
        (
            "tier = \"common\"\neffects = [{ talent = \"no_such\" }]",
            "unknown talent `no_such`",
        ),
        (
            "tier = \"common\"\neffects = [{ talent = \"weapon_focus\" }]",
            "is chosen for a weapon skill",
        ),
        (
            "tier = \"common\"\neffects = [{ talent = \"weapon_focus:lances\" }]",
            "`lances` is no choice",
        ),
        (
            "tier = \"common\"\neffects = [{ talent = \"hardy:axes\" }]",
            "takes no choice",
        ),
        (
            "tier = \"common\"\neffects = [{ gold = 5, alignment = 5 }]",
            "names one kind",
        ),
        (
            "tier = \"common\"\neffects = [{ item = \"spork\" }]",
            "unknown object `spork`",
        ),
        (
            "tier = \"common\"\neffects = [{ weapon_marks = { lances = 2 } }]",
            "unknown weapon skill",
        ),
        (
            "tier = \"common\"\nset = { uncle = \"Bob\" }",
            "not a slot a line sets",
        ),
    ] {
        let err = refused(&line(body));
        assert!(err.contains(why), "{body}: {err}");
    }
    let unknown_slot = "[[line]]\nid = \"x\"\ntier = \"common\"\nweight = 1\ntext = \"{uncle}\"\n";
    assert!(refused(unknown_slot).contains("unknown slot"));
    let twice = format!("{}{}", line("tier = \"common\""), line("tier = \"common\""));
    assert!(refused(&twice).contains("used twice in the chapter"));
}

#[test]
fn chapters_are_checked_together() {
    // A shared chapter's id, in a campaign or twice.
    let path = "campaigns/ascmearc/data/background/09-again.toml";
    let err = with(&[(path, &chapter("youth", ""))]).expect_err("shared id");
    assert!(err.contains("used twice"), "{err}");
    let err = with(&[("data/background/99-again.toml", &chapter("youth", ""))]).expect_err("twice");
    assert!(err.contains("used twice"), "{err}");
    // Two campaigns may each have their own `road`.
    let roads = (CONTENT.background().expect("stories").chapters.iter())
        .filter(|c| c.id == "road")
        .count();
    assert_eq!(roads, 2);
    // Without the settings file, no story and nothing checked.
    let mut files = ContentFiles::builtin();
    files.remove(SETTINGS);
    files.insert("data/background/99-t.toml", "nonsense = 1\n");
    let content = Content::load(&files).expect("no stories");
    assert!(content.background().is_none());
}

#[test]
fn worth_follows_the_points() {
    let b = CONTENT.background().expect("stories");
    let p = b.points;
    let gold = |text: &str| Effect::Gold(text.parse().expect("dice")).worth(&p);
    assert_eq!(gold("4d6+10"), (1, 0)); // a mean of 24
    assert_eq!(gold("6d6+10"), (2, 0)); // 31
    assert_eq!(gold("-15"), (0, 1));
    assert_eq!(gold("-100"), (0, 4));
    assert_eq!(Effect::Stat(Stat::Strength, -2).worth(&p), (0, 4));
    assert_eq!(b.tiers.of(0), Tier::Fateful);
    assert_eq!(b.tiers.of(4), Tier::Fateful);
    assert_eq!(b.tiers.of(5), Tier::Rare);
    assert_eq!(b.tiers.of(35), Tier::Uncommon);
    assert_eq!(b.tiers.of(155), Tier::Common);
}
