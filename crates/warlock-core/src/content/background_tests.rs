//! Background stories' data: the built-in set, the hidden-number rule, and every refusal.

use super::*;
use crate::content::ContentFiles;
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> = LazyLock::new(|| Content::builtin().expect("built-in content"));

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
            "leaving"
        ]
    );
    let road: Vec<&str> = b.chapters_for("ascmearc").map(|c| c.id.as_str()).collect();
    assert_eq!(road.len(), 8);
    assert_eq!(road.last(), Some(&"road"));
    assert_eq!(
        b.chapters_for("").count(),
        7,
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
        (
            "tier = \"common\"\nwhen = { tags = [\"nobody_sets\"] }",
            "no earlier chapter's line sets",
        ),
        ("tier = \"common\"\ntext2 = 1", "unknown field"),
        (
            "tier = \"common\"\neffects = [{ stat = { strength = 3 } }]",
            "1 or 2, either way",
        ),
        (
            "tier = \"common\"\neffects = [{ stat = { strength = 2 } }, { stat = { strength = 1 } }]",
            "at most 2 either way a line",
        ),
        (
            "tier = \"rare\"\neffects = [{ stat = { strength = 2 } }, { stat = { dexterity = -1 } }, \
             { skill = { heal = \"1d4\" } }]",
            "never costs anything",
        ),
        (
            "tier = \"rare\"\neffects = [{ skill = { heal = \"1d4\" } }]",
            "a rare line is worth 3 to 5",
        ),
        (
            "tier = \"common\"\neffects = [{ gold = \"10d6+40\" }]",
            "a common line is worth -4 to 2",
        ),
        (
            "tier = \"uncommon\"\nonly = true",
            "an `only` line is common",
        ),
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
            "tier = \"common\"\nwhen = { band = {} }",
            "names a `min` band",
        ),
        (
            "tier = \"common\"\neffects = [{ talent = \"no_such\" }]",
            "unknown talent `no_such`",
        ),
        (
            "tier = \"common\"\neffects = [{ talent = \"blaze\" }]",
            "`blaze` is given, a flaw, or a class ability",
        ),
        (
            "tier = \"common\"\neffects = [{ talent = \"bulwark\" }]",
            "`bulwark` is given, a flaw, or a class ability",
        ),
        (
            "tier = \"common\"\neffects = [{ talent = \"slight\" }]",
            "`slight` is given, a flaw, or a class ability",
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
            "tier = \"common\"\neffects = [{ flaw = \"hardy\" }]",
            "`hardy` is no birth sign's flaw",
        ),
        (
            "tier = \"rare\"\neffects = [{ talent = \"hardy\" }, { talent = \"lean\" }, \
             { flaw = \"slight\" }]",
            "never costs anything",
        ),
        (
            "tier = \"common\"\neffects = [{ gold = 5, alignment = 5 }]",
            "names one kind",
        ),
        (
            "tier = \"common\"\neffects = [{ gold = \"1d6-3\" }]",
            "neither always given",
        ),
        (
            "tier = \"common\"\neffects = [{ item = \"spork\" }]",
            "unknown object `spork`",
        ),
        (
            "tier = \"common\"\neffects = [{ item = { wear = \"dagger\" } }]",
            "wielded, not worn",
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
    let heavy = "[[line]]\nid = \"x\"\ntier = \"common\"\nweight = 21\ntext = \"A.\"\n";
    assert!(refused(heavy).contains("`weight` 21 is not 1 to 20"));
}

#[test]
fn chapters_are_checked_together() {
    // No plain line: a chapter always tells something.
    let plainless = "[chapter]\nid = \"t\"\nname = \"T\"\nparagraph = 4\n\n[[line]]\nid = \"x\"\n\
                     tier = \"common\"\nweight = 1\ntext = \"A.\"\neffects = [{ alignment = 5 }]\n";
    let err = with(&[("data/background/99-t.toml", plainless)]).expect_err("plainless");
    assert!(err.contains("has no plain line"), "{err}");
    // A paragraph before an earlier chapter's, or past the fourth.
    let early = chapter("early", "").replace("paragraph = 4", "paragraph = 1");
    let err = with(&[("data/background/99-early.toml", &early)]).expect_err("early");
    assert!(
        err.contains("paragraph 1 after a chapter of paragraph 4"),
        "{err}"
    );
    let fifth = chapter("fifth", "").replace("paragraph = 4", "paragraph = 5");
    let err = with(&[("data/background/99-fifth.toml", &fifth)]).expect_err("fifth");
    assert!(err.contains("not 1 to 4"), "{err}");
    // A shared chapter's id, in a campaign or twice.
    let path = "campaigns/ascmearc/data/background/09-again.toml";
    let err = with(&[(path, &chapter("youth", ""))]).expect_err("shared id");
    assert!(err.contains("a shared chapter's id"), "{err}");
    let err = with(&[("data/background/99-again.toml", &chapter("youth", ""))]).expect_err("twice");
    assert!(err.contains("used twice"), "{err}");
    // Two campaigns may each have their own `road`; a tag set only in another campaign's
    // chapter is unknown.
    let roads = (CONTENT.background().expect("stories").chapters.iter())
        .filter(|c| c.id == "road")
        .count();
    assert_eq!(roads, 2);
    let sets =
        "[[line]]\nid = \"y\"\ntier = \"common\"\nweight = 1\ntext = \"B.\"\ntags = [\"far\"]\n";
    let asks = "[[line]]\nid = \"x\"\ntier = \"common\"\nweight = 1\ntext = \"A.\"\n\
                when = { tags = [\"far\"] }\n";
    let err = with(&[
        (
            "campaigns/showcase/data/background/09-after.toml",
            &chapter("after", sets),
        ),
        (
            "campaigns/ascmearc/data/background/09-later.toml",
            &chapter("later", asks),
        ),
    ])
    .expect_err("another campaign's tag");
    assert!(
        err.contains("which no earlier chapter's line sets"),
        "{err}"
    );
    // A slot with no default yet.
    let mut files = ContentFiles::builtin();
    files.edit_table("background", |t| {
        t.replace("defaults = { guardian = \"your family\", ", "defaults = { ")
    });
    let err = Content::load(&files).expect_err("no default").to_string();
    assert!(err.contains("`{guardian}`, which has no default"), "{err}");
    // Without the settings file, no story and nothing checked.
    let mut files = ContentFiles::builtin();
    files.remove(SETTINGS);
    files.insert("data/background/99-t.toml", plainless);
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
    assert_eq!(b.tiers.of(0, 100), Tier::Fateful);
    assert_eq!(b.tiers.of(4, 100), Tier::Fateful);
    assert_eq!(b.tiers.of(5, 100), Tier::Rare);
    assert_eq!(b.tiers.of(35, 100), Tier::Uncommon);
    assert_eq!(b.tiers.of(155, 100), Tier::Common);
    assert_eq!(
        b.tiers.of(9, 200),
        Tier::Fateful,
        "the luckier tiers doubled"
    );
}
