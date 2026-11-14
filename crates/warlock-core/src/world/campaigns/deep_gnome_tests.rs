//! Dēopgang and Gimmwīc (#1434): the branch's eleventh floor leads down to the deep gnomes' town,
//! a hand-made level (`bottom`), whose `<` leads back up onto that floor's `>`; the town's people
//! are peaceful, and its keepers and unique characters are deep gnomes.

use super::*;
use crate::test_support::{builtin, run as step};

#[test]
fn the_deepest_floor_leads_down_to_the_town_and_back() {
    let mut w = World::new(builtin(), "deopgang", None).expect("a run");
    assert_eq!(w.perception().level_id(), "deopgang:1");
    w.travel("deopgang:11", None).expect("the eleventh floor");
    let down = (w.state.levels["deopgang:11"].stairs.iter())
        .find(|s| s.dir == StairsDir::Down)
        .cloned()
        .expect("a way down");
    assert_eq!(down.to.as_deref(), Some("gimmwic"));
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("the player").pos = down.pos;
    step(&mut w, "descend");
    assert_eq!(w.perception().level_id(), "gimmwic");
    let up = (w.state.levels["gimmwic"].stairs.iter())
        .find(|s| s.dir == StairsDir::Up)
        .cloned()
        .expect("a way up");
    assert_eq!(up.to.as_deref(), Some("deopgang:11"));
    assert_eq!(w.player_pos(), up.pos, "arrived on the stairs up");
    step(&mut w, "ascend");
    assert_eq!(w.perception().level_id(), "deopgang:11");
    assert_eq!(w.player_pos(), down.pos, "back on the stairs down");
}

#[test]
fn the_town_is_peaceful_and_its_people_are_deep_gnomes() {
    let w = World::new(builtin(), "gimmwic", None).expect("a run");
    assert_eq!(w.perception().level_id(), "gimmwic");
    let content = w.content();
    let townsfolk: Vec<&Actor> = (w.state.actors.values())
        .filter(|a| a.level == "gimmwic" && a.id != w.state.player)
        .collect();
    assert!(townsfolk.len() >= 6, "{}", townsfolk.len());
    for a in &townsfolk {
        let def = content.creature(a.creature);
        assert!(def.tags.iter().any(|t| t == "deep-gnome"), "{}", def.id);
        assert_ne!(
            stance_between(&w.state, content, w.state.player(), a),
            Stance::Hostile,
            "{}",
            def.id
        );
    }
    let npcs: Vec<&str> = townsfolk.iter().filter_map(|a| a.npc.as_deref()).collect();
    for npc in ["kerensa", "morwenna", "peran", "senara", "tregear"] {
        assert!(npcs.contains(&npc), "{npc}: {npcs:?}");
    }
}
