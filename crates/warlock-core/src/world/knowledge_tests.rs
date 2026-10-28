//! Knowledge (spec §8 "Skills": Knowledge; #667): know your enemy. It adds to the player's
//! check against a creature's drain, disease, or hold only above the creature's level, a little
//! and capped; it earns marks from discovery experience and from every such check; creatures
//! get nothing of it.

use super::*;
use crate::check::check;
use crate::content::{Affliction, Drain};
use crate::skills::SkillId;
use crate::stats::{PerStat, Stat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

fn knowledge() -> SkillId {
    BUILTIN.skill_id("knowledge").expect("knowledge")
}

/// The showcase with average stats and nothing trained, and a creature on its level.
fn world(seed: u64) -> (World, ActorId) {
    let mut w = World::new(BUILTIN.clone(), "showcase", Some(seed)).expect("world");
    let player = w.player_id();
    let me = w.player_mut();
    me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    me.skills = Default::default();
    let creature = (w.state.actors.keys().copied())
        .find(|&id| id != player)
        .expect("a creature on the showcase's level");
    (w, creature)
}

/// The level of actor `id`'s type.
fn level(w: &World, id: ActorId) -> u16 {
    u16::from(BUILTIN.creature(w.state.actors[&id].creature).level)
}

/// The pale drinker's drain (a Willpower check).
fn drain() -> Drain {
    let creature = BUILTIN.creature_id("pale_drinker").expect("pale drinker");
    let attack = BUILTIN.creature(creature).attack.as_ref().expect("attack");
    (attack.on_hit.iter())
        .find_map(|o| match o.what {
            Affliction::Drain(d) => Some(d),
            _ => None,
        })
        .expect("a drain")
}

#[test]
fn knowledge_adds_only_above_the_creatures_level_a_little() {
    let (mut w, creature) = world(1);
    let at = level(&w, creature);
    let bonus = |w: &World, margin: Option<i32>| {
        let mut me = w.state.player().clone();
        if let Some(margin) = margin {
            me.skills.add(knowledge(), i32::from(at) + margin);
        }
        super::knowledge::resist_bonus(&BUILTIN, &me, at)
    };
    assert_eq!(bonus(&w, None), 0, "untrained gives nothing");
    for (margin, expected) in [(-5, 0), (0, 0), (4, 0), (5, 1), (40, 4), (300, 4)] {
        assert_eq!(bonus(&w, Some(margin)), expected, "{margin} above");
    }
    // A rank of 0 stands above no creature of level 1 or more.
    w.player_mut().skills.add(knowledge(), 0);
    assert!(at >= 1);
    assert_eq!(
        super::knowledge::resist_bonus(&BUILTIN, w.state.player(), at),
        0
    );
}

#[test]
fn knowledge_raises_the_check_against_a_creatures_drain_only() {
    let drain = drain();
    // Each seed's outcome matches the check rolled with the bonus Knowledge gives (+4 at 40
    // above), and against no creature (a scroll's backlash) without it.
    let holds = |by_creature: bool| -> usize {
        let mut held = 0;
        for seed in 1..=60 {
            let (mut w, creature) = world(seed);
            let above = i32::from(level(&w, creature)) + 40;
            let player = w.player_id();
            w.player_mut().skills.add(knowledge(), above);
            // What the rules add besides (talents, a sign's).
            let talents = w.state.player().talent_mods(&BUILTIN);
            let base = talents.check(Stat::Willpower) + talents.drain_checks;
            let bonus = if by_creature { 4 } else { 0 };
            let mut expected = w.state.rng.clone();
            let resisted = check(&mut expected, base + bonus, drain.difficulty);
            let by = by_creature.then_some(creature);
            w.drain(player, drain, by, &mut Vec::new());
            let kept = w.state.player().stats.score(Stat::Strength) == 10;
            assert_eq!(kept, resisted, "seed {seed}");
            assert_eq!(w.state.rng, expected, "the check's two draws only");
            held += usize::from(resisted);
        }
        held
    };
    let (known, unknown) = (holds(true), holds(false));
    assert!(known > unknown, "{known} {unknown}");
}

#[test]
fn resisting_marks_knowledge_and_creatures_get_nothing() {
    let mut marked = 0;
    for seed in 1..=40 {
        let (mut w, creature) = world(seed);
        let player = w.player_id();
        // Untrained: nothing drawn, nothing given.
        let training = w.state.training.clone();
        assert_eq!(w.resisting(player, Some(creature)), 0);
        assert_eq!(w.state.training, training);
        w.player_mut().skills.add(knowledge(), 200);
        // A creature resisting gets nothing and draws nothing; nor does a check no creature
        // asked for.
        let training = w.state.training.clone();
        assert_eq!(w.resisting(creature, Some(player)), 0);
        assert_eq!(w.resisting(player, None), 0);
        assert_eq!(w.state.training, training);
        let rng = w.state.rng.clone();
        assert_eq!(w.resisting(player, Some(creature)), 4);
        assert_eq!(w.state.rng, rng, "never the gameplay stream");
        marked += w
            .state
            .player()
            .skills
            .get(knowledge())
            .expect("trained")
            .marks;
    }
    // A mark at 25 %: some seeds, not all.
    assert!(marked > 0 && marked < 40, "{marked}");
}

#[test]
fn discovery_experience_marks_knowledge() {
    let (mut w, creature) = world(1);
    w.player_mut().skills.add(knowledge(), 10);
    let kind = w.state.actors[&creature].creature;
    let marks = |w: &World| {
        w.state
            .player()
            .skills
            .get(knowledge())
            .expect("trained")
            .marks
    };
    w.earn_kill(kind);
    assert_eq!(marks(&w), 1, "a first kill of a kind");
    w.earn_kill(kind);
    assert_eq!(marks(&w), 1, "the second pays no discovery");
}
