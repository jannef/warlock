//! The gates (#1295, docs/dev/creatures/gates.md; design `docs/design/miscasts-review.md` §6):
//! a gate row of the miscast table puts its gate beside the caster, and the gate, rooted and of
//! the `gates` faction (#1316: friendly to every monster, hostile to the player's side), lets its
//! fiends through on its cooldown at no cost, `loosed`, to its `most`; no creature but the
//! player's side picks it for a target, and a blast hurts it as anything.

use super::caster_tests::{kobold, world_with};
use super::*;
use crate::content::{ContentFiles, Stance};
use crate::test_support::builtin;

/// The built-in content where every cast miscasts and the table is the `only` row alone,
/// covering every total.
fn content(only: &str) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/miscast.toml",
        "stat = \"intelligence\"\nbase = 1000\nper_level = 1\narmour_percent = 10\n\
         rank_ease = 1\ndie = \"2d12\"\nper_margin = 5\nsafe_below = 71\n",
    );
    let builtin = builtin();
    for m in builtin.miscasts() {
        files.remove(&format!("data/miscasts/{}.toml", m.id));
    }
    let entry = (builtin.miscasts().iter())
        .find(|m| m.id == only)
        .expect("a built-in entry");
    let text = std::fs::read_to_string(format!(
        "{}/../../assets/data/miscasts/{only}.toml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("its file");
    let text = text
        .replace(&format!("from = {}\n", entry.from), "from = 1\n")
        .lines()
        .filter(|l| !l.starts_with("to = "))
        .collect::<Vec<_>>()
        .join("\n");
    files.insert(&format!("data/miscasts/{only}.toml"), &text);
    Arc::new(Content::load(&files).expect("the content loads"))
}

/// A wizard who cannot die of what follows casts Spark at a kobold two cells east, and the
/// table's one row opens `gate`; the world, the kobold, and the gate.
fn opened(gate: &str) -> (World, ActorId, ActorId) {
    let mut w = world_with(content(gate), "wizard");
    let spark = w.content.spell_id("spark").expect("spark");
    w.player_mut().spells.insert(spark, 1);
    w.player_mut().hp_base = 100_000;
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w.player_mut().mana = 40;
    let k = kobold(&mut w, 2, 100_000);
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let here = w.player_pos();
    w.apply(&Action::Cast(
        "spark".into(),
        Some(Pos::new(here.x + 2, here.y)),
    ));
    let kind = w.content.creature_id(gate).expect("the gate's type");
    let g = (w.state.actors.values())
        .find(|a| a.creature == kind)
        .expect("the gate opened")
        .id;
    (w, k, g)
}

#[test]
fn a_gate_row_opens_its_gate_beside_the_caster_hostile_to_the_caster_alone() {
    let (w, k, g) = opened("rift");
    let gate = &w.state.actors[&g];
    assert!(gate.pos.chebyshev(w.player_pos()) <= 1, "beside the caster");
    let content = w.content();
    assert!(content.creature(gate.creature).rooted);
    let gates = content.factions().id("gates").expect("the gates");
    assert_eq!(gate.faction(content), Some(gates));
    let player = w.state.player();
    let kobold = &w.state.actors[&k];
    assert_eq!(
        stance_between(&w.state, content, player, gate),
        Stance::Hostile
    );
    assert_eq!(
        stance_between(&w.state, content, kobold, gate),
        Stance::Neutral
    );
}

#[test]
fn a_gate_lets_its_fiends_through_loosed_to_its_most_and_never_moves() {
    let (mut w, k, g) = opened("rift");
    let at = w.state.actors[&g].pos;
    let mana = w.state.actors[&g].mana;
    let mut most_seen = 0;
    for _ in 0..600 {
        w.apply(&Action::Wait);
        let live = w.live_summons(g);
        assert!(live <= 4, "never more than its `most`: {live}");
        most_seen = most_seen.max(live);
    }
    assert!(most_seen >= 2, "fiends came through: {most_seen}");
    assert_eq!(w.state.actors[&g].pos, at, "the gate never moved");
    assert_eq!(w.state.actors[&g].mana, mana, "it costs the gate nothing");
    let content = w.content().clone();
    let loosed = content.factions().id("loosed").expect("the loosed");
    let pool = ["dretch", "quasit"].map(|c| content.creature_id(c).expect("a fiend"));
    let called: Vec<_> = (w.state.actors.values())
        .filter(|a| a.summoner == Some(g))
        .collect();
    for fiend in &called {
        assert!(pool.contains(&fiend.creature), "a rift calls its pool");
        assert_eq!(
            fiend.faction(&content),
            Some(loosed),
            "loosed, by the summoning's faction"
        );
        assert_eq!(
            stance_between(&w.state, &content, fiend, &w.state.actors[&g]),
            Stance::Neutral,
            "at peace with the gate"
        );
        if let Some(kobold) = w.state.actors.get(&k) {
            assert_eq!(
                stance_between(&w.state, &content, fiend, kobold),
                Stance::Hostile,
                "and hostile to the kobold"
            );
        }
    }
}

/// A free cell beside actor `of`.
fn beside(w: &World, of: ActorId) -> Pos {
    let content = w.content();
    let at = w.state.actors[&of].pos;
    (Dir::ALL.iter())
        .map(|&d| at.step(d))
        .find(|&p| {
            w.state.current().is_walkable(content, p)
                && w.state.actor_at(&w.state.current_level, p).is_none()
        })
        .expect("a free cell beside it")
}

#[test]
fn a_monster_or_a_loosed_fiend_beside_a_gate_never_attacks_it_and_the_player_may() {
    let (mut w, k, g) = opened("rift");
    let content = w.content().clone();
    // Everyone else (a fiend already through, the showcase's folk) sent far off.
    let player = w.state.player;
    for a in w.state.actors.values_mut() {
        if ![player, k, g].contains(&a.id) {
            a.pos = Pos::new(-100, -100);
        }
    }
    // The kobold, of its own faction, beside the gate; a dretch let through, loosed, beside it too.
    let free = beside(&w, g);
    w.state.actors.get_mut(&k).expect("the kobold").pos = free;
    let spawn = crate::content::CreatureSpawn {
        creature: content.creature_id("dretch").expect("a dretch"),
        faction: content.factions().id("loosed"),
        alignment: None,
        npc: None,
        pos: beside(&w, g),
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    let level = w.state.current_level.clone();
    let fiend = travel::make_creature(&mut w.state, &content, &level, &spawn);
    w.state.actors.get_mut(&fiend).expect("the fiend").hp = 100_000;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    for other in [k, fiend] {
        let (them, gate) = (&w.state.actors[&other], &w.state.actors[&g]);
        assert_eq!(
            stance_between(&w.state, &content, them, gate),
            Stance::Neutral
        );
        assert_ne!(
            w.nearest_hostile_in_sight(other).map(|(id, _)| id),
            Some(g),
            "it never picks the gate"
        );
    }
    let hp = w.state.actors[&g].hp;
    for _ in 0..20 {
        w.apply(&Action::Wait);
    }
    assert_eq!(w.state.actors[&g].hp, hp, "no creature struck the gate");
    // The player may: their blows land on it.
    let to = Dir::ALL
        .into_iter()
        .find(|&d| w.player_pos().step(d) == w.state.actors[&g].pos)
        .expect("the gate beside the player");
    for _ in 0..40 {
        w.apply(&Action::Move(to));
        if w.state.actors.get(&g).is_none_or(|a| a.hp < hp) {
            return;
        }
    }
    panic!("the player never struck the gate");
}

#[test]
fn a_monsters_blast_hurts_a_gate_in_it() {
    let (mut w, k, g) = opened("rift");
    let content = w.content().clone();
    // The rift shrugs off fire (resist 100), so the blast is acid: a kobold's Acid splash at the
    // player, the gate beside them.
    let splash = content.spell_id("acid_splash").expect("acid splash");
    w.state.actors.get_mut(&g).expect("the gate").hp = 100_000;
    let player = w.state.player;
    for _ in 0..30 {
        let kobold = w.state.actors.get_mut(&k).expect("the kobold");
        kobold.mana = 1000;
        let mut events = Vec::new();
        w.creature_casts(k, (splash, player), &mut events);
        if w.state.actors[&g].hp < 100_000 {
            return;
        }
    }
    panic!("the blast never hurt the gate");
}
