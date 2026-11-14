//! Knowledge (spec §8 "Skills": Knowledge; #667): know your enemy. It adds to the player's
//! check against a creature's drain, disease, or hold only above the creature's level, a little
//! and capped; it earns marks from discovery experience and from every such check; creatures
//! get nothing of it.

use crate::check::check;
use crate::content::{Affliction, Drain};
use crate::skills::SkillId;
use crate::stats::{PerStat, Stat, Stats};
use crate::test_support::CONTENT as BUILTIN;
use crate::world::*;

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
    for (margin, expected) in [
        (-5, 0),
        (0, 0),
        (2, 0),
        (3, 1),
        (4, 1),
        (5, 2),
        (40, 6),
        (300, 6),
    ] {
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
    // Each seed's outcome matches the check rolled with the bonus Knowledge gives (+6 at 40
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
            let base = talents.vigour.check(Stat::Willpower) + talents.vigour.drain_checks;
            let bonus = if by_creature { 6 } else { 0 };
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
        assert_eq!(w.resisting(player, Some(creature)), 6);
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

/// A room of a skeleton (level 3; immune to poison, resistant to piercing, vulnerable to
/// bludgeoning), a kobold (no resistance), and the player, with no Knowledge.
fn lore_room() -> (World, Pos, Pos) {
    let mut files = crate::content::ContentFiles::builtin();
    files.insert(
        "levels/know_room.toml",
        "name = \"Room\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
         #####\n#s.k#\n#@..#\n#####\n'''\n[legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n\
         \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
         \"s\" = { terrain = \"floor\", creature = \"skeleton\" }\n\
         \"k\" = { terrain = \"floor\", creature = \"kobold\" }\n",
    );
    files.insert(
        "scenarios/know_room.toml",
        "name = \"R\"\nstart_level = \"know_room\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = World::new(content, "know_room", None).expect("world");
    w.player_mut().skills = Default::default();
    (w, Pos::new(1, 1), Pos::new(3, 1))
}

/// What the player's view says of whoever is seen at `at`, in the words' ids.
fn known_at(w: &World, at: Pos) -> Vec<(String, &'static str)> {
    let view = w.perception();
    let actor = view.actor_at(at).expect("seen");
    (view.known_resistances(actor).into_iter())
        .map(|(kind, word)| (w.content().damage_type(kind).id.clone(), word.word()))
        .collect()
}

fn train_knowledge(w: &mut World, value: i32) {
    let id = w.content().skill_id("knowledge").expect("knowledge");
    w.player_mut().skills.add(id, value);
}

fn resist_bonus_against(w: &World, at: Pos) -> i32 {
    let view = w.perception();
    let creature = view.actor_at(at).expect("seen").creature;
    let level = u16::from(w.content().creature(creature).level);
    super::knowledge::resist_bonus(w.content(), w.state.player(), level)
}

#[test]
fn knowledge_above_a_creatures_level_tells_what_it_resists_and_at_it_nothing() {
    let (mut w, skeleton, _) = lore_room();
    assert!(known_at(&w, skeleton).is_empty(), "untrained");
    train_knowledge(&mut w, 3);
    assert!(known_at(&w, skeleton).is_empty(), "at its level");
    // One above is enough, though the check's curve gives nothing yet.
    train_knowledge(&mut w, 1);
    assert_eq!(resist_bonus_against(&w, skeleton), 0);
    let known = known_at(&w, skeleton);
    let told: Vec<(&str, &str)> = (known.iter())
        .map(|(kind, word)| (kind.as_str(), *word))
        .collect();
    assert!(told.contains(&("piercing", "resistant")), "{told:?}");
    assert!(told.contains(&("bludgeoning", "vulnerable")), "{told:?}");
    assert!(told.contains(&("poison", "immune")), "{told:?}");
    assert!(told.contains(&("necrotic", "immune")), "{told:?}");
    assert_eq!(told.len(), 4, "{told:?}");
}

#[test]
fn knowledge_tells_nothing_of_a_creature_with_no_resistance_nor_of_the_player() {
    let (mut w, _, kobold) = lore_room();
    train_knowledge(&mut w, 200);
    assert!(known_at(&w, kobold).is_empty());
    assert!(known_at(&w, w.player_pos()).is_empty(), "the stats' say");
}

#[test]
fn knowledge_tells_the_kind_never_what_this_one_has_on_it() {
    let (mut w, skeleton, _) = lore_room();
    train_knowledge(&mut w, 200);
    let before = known_at(&w, skeleton);
    let id = w.perception().actor_at(skeleton).expect("seen").id;
    let warmed = w.content().status_id("warmed").expect("warmed");
    w.afflict(id, warmed, 1, None, &mut Vec::new());
    let fire = w.content().damage_type_id("fire").expect("fire");
    assert_eq!(
        w.resistances_of(id).get(Some(fire)),
        50,
        "it does resist now"
    );
    assert_eq!(known_at(&w, skeleton), before);
}

#[test]
fn knowing_is_not_practising_and_reading_it_changes_no_state() {
    let (mut w, skeleton, _) = lore_room();
    train_knowledge(&mut w, 200);
    let hash = w.state_hash();
    assert!(!known_at(&w, skeleton).is_empty());
    assert_eq!(w.state_hash(), hash);
}
