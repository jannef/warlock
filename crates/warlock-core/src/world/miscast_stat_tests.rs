//! Miscasts' stat losses (#1290, the miscasts design §5.2): `stat_damage` below 71, which mends
//! a point a day, and the drain's `times` from 71, each its own draw and check.

use super::caster_tests::{said, world_with};
use super::miscast::Aim;
use super::*;
use crate::action::Rest;
use crate::content::{ContentFiles, Miscast, MishapEffect};
use crate::stats::Stat;
use crate::time::{DAY, Duration};

/// A wizard who knows Spark, with the built-in content.
fn wizard() -> World {
    let c = Arc::new(Content::builtin().expect("content"));
    let mut w = world_with(c, "wizard");
    let spark = w.content.spell_id("spark").expect("spark");
    w.player_mut().spells.insert(spark, 1);
    w
}

/// The player miscasts Spark into `effect`.
fn befalls(w: &mut World, effect: MishapEffect) -> Vec<Event> {
    let c = w.content.clone();
    let player = w.state.player;
    let spell = c.spell_id("spark").expect("spark");
    let me = w.state.player().clone();
    let cast = super::cast::Cast::by_creature(&me, &c, (spell, c.spell(spell)));
    let entry = Miscast {
        id: "test".into(),
        from: 1,
        to: Some(2),
        message: "The weave slips.".into(),
        seen: "{who} slips.".into(),
        effect,
    };
    let mut events = Vec::new();
    w.miscast(player, &cast, Aim::Player(None), &entry, &mut events);
    events
}

fn entry(c: &Content, id: &str) -> Miscast {
    (c.miscasts().iter())
        .find(|m| m.id == id)
        .cloned()
        .expect("a built-in entry")
}

#[test]
fn the_aches_are_stat_damage_and_the_witherings_drain_times_over() {
    let c = Content::builtin().expect("content");
    let damage = |id| match entry(&c, id).effect {
        MishapEffect::StatDamage { stat: None, points } => points,
        other => panic!("{id}: {other:?}"),
    };
    assert_eq!((damage("ache"), damage("racked")), (1, 2));
    let drain = |id| match entry(&c, id).effect {
        MishapEffect::Drain {
            stat: None,
            points,
            difficulty,
            times,
        } => (points, difficulty, times),
        other => panic!("{id}: {other:?}"),
    };
    assert_eq!(drain("withering"), (1, 16, 1));
    assert_eq!(drain("twofold_withering"), (1, 16, 2));
    assert_eq!(drain("deep_withering"), (2, 18, 1));
    assert_eq!(drain("threefold_withering"), (1, 18, 3));
    assert_eq!(drain("ruinous_withering"), (2, 20, 2));
}

#[test]
fn stat_damage_lowers_the_score_says_so_and_mends_a_point_a_day() {
    let mut w = wizard();
    let score = |w: &World| w.perception().own_scores()[Stat::Strength];
    let before = score(&w);
    let effect = MishapEffect::StatDamage {
        stat: Some(Stat::Strength),
        points: 2,
    };
    befalls(&mut w, effect);
    assert_eq!(w.state.player().damage.get(Stat::Strength), 2);
    assert!(said(&w).contains(&"You feel weaker.".to_string()));
    assert_eq!(w.state.player().drained[Stat::Strength], 0, "not for good");
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    assert_eq!(score(&w), before - 2);
    // No disease: it mends all the same, a point a day.
    let start = w.time();
    w.apply(&Action::Rest(Rest::For(Duration::seconds(DAY))));
    assert_eq!(w.state.player().damage.get(Stat::Strength), 1);
    // Hunger may break a rest: rest on to the second day's end.
    let end = start + Duration::seconds(2 * DAY);
    while w.time() < end {
        assert!(w.state.player().damage.is_damaged(), "not before two days");
        let left = end.seconds() - w.time().seconds();
        w.apply(&Action::Rest(Rest::For(Duration::seconds(left))));
    }
    assert!(w.state.player().damage.is_none(), "all mended");
    assert_eq!(score(&w), before);
}

#[test]
fn stat_damage_never_takes_a_score_below_one_nor_the_last_hit_point() {
    let mut w = wizard();
    let toughness = w.state.player().stats.score(Stat::Toughness);
    w.player_mut()
        .stats
        .gift(Stat::Toughness, 3 - i32::from(toughness));
    w.player_mut().hp = 1;
    let effect = MishapEffect::StatDamage {
        stat: Some(Stat::Toughness),
        points: 9,
    };
    befalls(&mut w, effect);
    let me = w.state.player();
    assert_eq!(me.damage.get(Stat::Toughness), 2, "the score left at 1");
    assert_eq!(me.hp, 1);
    assert!(w.state.death.is_none());
    // At 1 already, nothing more is taken and nothing is said.
    let lines = said(&w).len();
    befalls(&mut w, effect);
    assert_eq!(w.state.player().damage.get(Stat::Toughness), 2);
    assert_eq!(said(&w).len(), lines + 1, "the entry's words only");
}

#[test]
fn a_drain_times_over_draws_and_checks_each_time() {
    let mut w = wizard();
    let before = w.state.player().stats.score(Stat::Strength);
    let effect = MishapEffect::Drain {
        stat: Some(Stat::Strength),
        points: 1,
        difficulty: 1000,
        times: 2,
    };
    let events = befalls(&mut w, effect);
    let drained = (events.iter())
        .filter(|e| matches!(e, Event::Drained { .. }))
        .count();
    assert_eq!(drained, 2, "two checks");
    assert_eq!(w.state.player().drained[Stat::Strength], 2);
    assert_eq!(w.state.player().stats.score(Stat::Strength), before - 2);
}

#[test]
fn stat_damage_is_allowed_below_71_and_a_drains_times_is_checked() {
    let load = |effect: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "data/miscasts/glimmer.toml",
            &format!(
                "[[miscast]]\nid = \"glimmer\"\nfrom = 1\nto = 2\nmessage = \"x\"\n\
                 seen = \"{{who}}\"\neffect = {{ {effect} }}\n"
            ),
        );
        Content::load(&files)
    };
    assert!(load("kind = \"stat_damage\", stat = \"random\", points = 2").is_ok());
    assert!(load("kind = \"stat_damage\", stat = \"strength\", points = 1").is_ok());
    assert!(load("kind = \"stat_damage\", stat = \"random\", points = 0").is_err());
    assert!(load("kind = \"stat_damage\", stat = \"random\"").is_err());
    assert!(load("kind = \"stat_damage\", points = 1, times = 2").is_err());
    let drain = "kind = \"drain\", stat = \"random\", points = 1, difficulty = 16";
    assert!(
        load(&format!("{drain}, times = 2")).is_err(),
        "still no drain below 71"
    );
    // A mishap's table knows no stat damage.
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/mishaps/ache_test.toml",
        "[[mishap]]\nid = \"ache_test\"\nband = \"bad\"\nweight = 1\nmessage = \"x\"\n\
         effect = { kind = \"stat_damage\", stat = \"random\", points = 1 }\n",
    );
    assert!(Content::load(&files).is_err());
}
