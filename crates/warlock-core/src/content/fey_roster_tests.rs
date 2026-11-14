//! The fey roster (#452): the courts, Seelie and Unseelie, stand as the plan has them, and a
//! type's `factions` are checked when content loads.

use super::Content;
use crate::test_support::builtin;

#[test]
fn the_courts_stand_as_planned() {
    use super::Stance::{Hostile, Neutral};
    let content = builtin();
    let f = content.factions();
    let id = |n: &str| f.id(n).unwrap_or_else(|| panic!("faction {n}"));
    let (player, seelie, unseelie, wrathful) = (
        id("player"),
        id("seelie"),
        id("unseelie"),
        id("wrathful_fey"),
    );
    assert_eq!(f.stance(seelie, player), Neutral);
    assert_eq!(f.stance(unseelie, player), Hostile);
    assert_eq!(f.stance(seelie, unseelie), Hostile);
    assert_eq!(f.stance(wrathful, player), Hostile);
    assert_eq!(f.stance(wrathful, unseelie), Hostile);
    assert_eq!(f.stance(wrathful, seelie), Neutral);
    assert_eq!(f.def(seelie).provoked, Some(wrathful));
    assert_eq!(f.def(unseelie).provoked, None);
    // The courts keep to their own quarrel: every other faction is neutral to them, but the
    // berserk and the loosed, hostile to all.
    for (i, other) in f.defs().iter().enumerate() {
        let other_id = super::FactionId(i as u16);
        if other.hates_everyone()
            || ["player", "seelie", "unseelie", "wrathful_fey"].contains(&other.id.as_str())
        {
            continue;
        }
        for court in [seelie, unseelie, wrathful] {
            assert_eq!(
                f.stance(court, other_id),
                Neutral,
                "{} and a court",
                other.id
            );
        }
    }
    // A type counts as its heaviest court: the even ones as the first by id, the Seelie.
    let court_of = |t: &str| content.creature(content.creature_id(t).expect(t)).faction;
    assert_eq!(court_of("redcap"), Some(unseelie));
    assert_eq!(court_of("dryad"), Some(seelie));
    assert_eq!(court_of("pixie"), Some(seelie));
}

#[test]
fn factions_are_checked_when_content_loads() {
    let cases = [
        (
            "factions = { seelie = 50, elves = 50 }",
            "unknown faction `elves`",
        ),
        ("factions = { seelie = 0, unseelie = 50 }", "weighs 0"),
        (
            "faction = \"seelie\"\nfactions = { seelie = 50, unseelie = 50 }",
            "replaces `faction`",
        ),
    ];
    for (line, expected) in cases {
        let mut files = super::ContentFiles::builtin();
        files.insert(
            "data/creatures/zz_test_fey.toml",
            &format!(
                "[[creature]]\nid = \"zz_test_fey\"\nname = \"test fey\"\nglyph = \"f\"\n\
                 fg = \"#ffffff\"\nweight = 1000\n{line}\n"
            ),
        );
        let err = Content::load(&files).expect_err(line).to_string();
        assert!(err.contains(expected), "{line}: {err}");
    }
}
