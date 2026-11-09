//! The gates (#1295, docs/dev/creatures/gates.md; design `docs/design/miscasts-review.md` §6):
//! a gate row of the miscast table puts its gate beside the caster, and the gate, rooted and
//! `loosed`, lets its fiends through on its cooldown at no cost, as `loosed` as itself, to its
//! `most`, hostile to the caster and to everyone else but its own.

use super::caster_tests::{kobold, world_with};
use super::*;
use crate::content::{ContentFiles, Stance};

/// The built-in content where every cast miscasts and the table is the `only` row alone,
/// covering every total.
fn content(only: &str) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/miscast.toml",
        "stat = \"intelligence\"\nbase = 1000\nper_level = 1\narmour_percent = 10\n\
         rank_ease = 1\ndie = \"2d12\"\nper_margin = 5\nsafe_below = 71\n",
    );
    let builtin = Content::builtin().expect("content");
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
fn a_gate_row_opens_its_gate_beside_the_caster_hostile_to_all_but_its_own() {
    let (w, k, g) = opened("rift");
    let gate = &w.state.actors[&g];
    assert!(gate.pos.chebyshev(w.player_pos()) <= 1, "beside the caster");
    let content = w.content();
    assert!(content.creature(gate.creature).rooted);
    let loosed = content.factions().id("loosed").expect("the loosed");
    assert_eq!(gate.faction(content), Some(loosed));
    let player = w.state.player();
    let kobold = &w.state.actors[&k];
    assert_eq!(
        stance_between(&w.state, content, player, gate),
        Stance::Hostile
    );
    assert_eq!(
        stance_between(&w.state, content, kobold, gate),
        Stance::Hostile
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
            "as loosed as the gate"
        );
        assert_eq!(
            stance_between(&w.state, &content, fiend, &w.state.actors[&g]),
            Stance::Neutral,
            "loyal to the gate"
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
