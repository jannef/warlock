//! Spells aimed at a corpse (`world/corpse_spell.rs`; the spell types design §7 entries 2, 29, 30):
//! Charnel burst, Grave feast, Rouse the dead and its berserk, on the shipped spells.

use super::caster_tests::{kobold_at, sp};
use super::spell_type_tests::{HALL_CONTENT, cell, files, start};
use super::*;
use crate::ability::{Ability, Aim};
use crate::action::{Action, ReadTarget, parse_actions};
use crate::content::Stance;
use crate::dump::{self, Layer};
use crate::object::Stack;
use crate::time::Duration;

/// The spell types' lit hall, the player at (2, 3) knowing the three spells, with mana.
fn world(seed: u64) -> World {
    let mut w = start(HALL_CONTENT.clone(), seed);
    for spell in ["charnel_burst", "grave_feast", "rouse_the_dead"] {
        w.player_mut().spells.insert(sp(spell), 1);
    }
    w.player_mut().mana_base = 400;
    w.player_mut().mana = 400;
    w.player_mut().weapon_skills = Default::default();
    w
}

/// A corpse of `creature` lies at `at`, died now.
fn lay(w: &mut World, creature: &str, at: Pos) {
    let content = w.content().clone();
    let kind = content.creature_id(creature).expect("a creature");
    let corpse = content.corpse_of(kind).expect("it leaves a corpse");
    let stack = Stack::corpse(corpse, w.state.time);
    let level = w.state.current_mut();
    let mut pile = level.objects_at(at).to_vec();
    pile.insert(0, stack);
    level.set_pile(at, pile);
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
}

fn cast(w: &mut World, spell: &str, at: Pos) {
    w.apply(&Action::Cast(spell.into(), Some(at)));
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn last_said(w: &World) -> String {
    let line = (w.messages().iter().last()).map_or_else(String::new, |m| m.display());
    line.split(" (×").next().unwrap_or_default().to_string()
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.display() == text)
}

fn corpses_at(w: &World, at: Pos) -> usize {
    (w.state.current().objects_at(at).iter())
        .filter(|s| w.content().object(s.object).corpse.is_some())
        .count()
}

/// The actors of type `creature` on the level but those the hall's map placed in its closet.
fn of_type(w: &World, creature: &str) -> Vec<ActorId> {
    let kind = w.content().creature_id(creature).expect("a creature");
    (w.state.actors.values())
        .filter(|a| a.creature == kind && a.pos != cell(20, 1))
        .map(|a| a.id)
        .collect()
}

/// A world of a seed whose next rise of a corpse at `at` comes up berserk (`want`) or serving.
fn rising(want: bool, creature: &str, at: Pos) -> (World, ActorId) {
    for seed in 0..500 {
        let mut w = world(seed);
        lay(&mut w, creature, at);
        cast(&mut w, "rouse_the_dead", at);
        let risen = of_type(&w, creature);
        let id = *risen.first().expect("it stood");
        if w.state.actors[&id].risen.is_some() == want {
            return (w, id);
        }
    }
    panic!("no seed drew it");
}

#[test]
fn they_are_refused_with_no_corpse_in_sight() {
    let mut w = world(1);
    // Nothing on the cell, and a corpse behind a wall the player cannot see.
    lay(&mut w, "kobold", cell(20, 1));
    let (turn, mana) = (w.state.turn, w.state.player().mana);
    cast(&mut w, "charnel_burst", cell(4, 3));
    assert_eq!(last_said(&w), "There is nothing there to burst.");
    cast(&mut w, "charnel_burst", cell(9, 1));
    assert_eq!(last_said(&w), "There is nothing there to burst.");
    cast(&mut w, "rouse_the_dead", cell(20, 1));
    assert_eq!(last_said(&w), "There is no corpse there.");
    cast(&mut w, "grave_feast", cell(3, 3));
    assert_eq!(last_said(&w), "There is no corpse there.");
    assert_eq!((w.state.turn, w.state.player().mana), (turn, mana));
    assert_eq!(corpses_at(&w, cell(20, 1)), 1);
    // A cast with no cell asks for one.
    w.apply(&Action::Cast("rouse_the_dead".into(), None));
    assert!(last_said(&w).starts_with("Cast it where?"));
}

#[test]
fn the_perception_shows_the_corpses_in_sight_nearest_first() {
    let mut w = world(1);
    lay(&mut w, "kobold", cell(9, 1));
    lay(&mut w, "kobold", cell(4, 3));
    lay(&mut w, "kobold", cell(20, 1));
    let view = w.perception();
    assert_eq!(view.corpses_in_sight(), vec![cell(4, 3), cell(9, 1)]);
    assert!(view.corpse_at(cell(4, 3)));
    assert!(!view.corpse_at(cell(20, 1)), "behind a wall");
}

#[test]
fn the_aim_is_named_a_corpse() {
    let w = world(1);
    for spell in ["charnel_burst", "grave_feast", "rouse_the_dead"] {
        assert_eq!(Ability::Spell(sp(spell)).aim(w.content()), Aim::Corpse);
    }
    let text = dump::ascii(&w, &[Layer::Abilities]);
    assert!(
        text.contains("Rouse the dead (spell): a corpse you see"),
        "{text}"
    );
    assert!(
        text.contains("Grave feast (spell): a corpse you see"),
        "{text}"
    );
    // The spells layer has words for a rise, which rolls nothing.
    let spells = dump::ascii(&w, &[Layer::Spells]);
    assert!(spells.contains("Rouse the dead (level 24"), "{spells}");
    assert!(spells.contains("Charnel burst (level 9"), "{spells}");
}

#[test]
fn the_burst_uses_the_corpse_up_where_it_lies() {
    let mut w = world(2);
    let at = cell(9, 3);
    lay(&mut w, "kobold", at);
    // Standing in the way, it does not stop the burst: it flies nowhere.
    let between = kobold_at(&mut w, (3, 0), 500);
    let beside = kobold_at(&mut w, (8, 0), 500);
    let far = kobold_at(&mut w, (11, 0), 500);
    let (mana, turn) = (w.state.player().mana, w.state.turn);
    cast(&mut w, "charnel_burst", at);
    assert!(w.state.turn > turn);
    assert!(w.state.player().mana < mana);
    assert_eq!(corpses_at(&w, at), 0, "used up");
    let hp = |w: &World, id: ActorId| w.state.actors[&id].hp;
    assert!(hp(&w, beside) < 500, "beside it");
    assert_eq!(hp(&w, between), 500, "not on the way");
    assert_eq!(hp(&w, far), 500, "past its radius");
    // Its blood is spattered as a kill's.
    assert!(!w.state.current().blood.is_empty());
    // No second burst from it.
    let again = w.state.turn;
    cast(&mut w, "charnel_burst", at);
    assert_eq!(w.state.turn, again);
}

#[test]
fn a_burst_on_a_rotten_corpse_and_on_ones_own_cell_goes_off() {
    let mut w = world(3);
    let here = w.player_pos();
    lay(&mut w, "kobold", here);
    // Rotten: its stage is not read.
    w.state
        .current_mut()
        .objects
        .get_mut(&here)
        .expect("a pile")[0]
        .died = Some(crate::time::GameTime::default());
    let hp = w.state.player().hp;
    cast(&mut w, "charnel_burst", here);
    assert_eq!(corpses_at(&w, here), 0);
    assert!(w.state.player().hp < hp, "no one is spared it");
}

#[test]
fn a_burst_out_of_range_is_refused() {
    let mut w = world(3);
    lay(&mut w, "kobold", cell(17, 3));
    let turn = w.state.turn;
    cast(&mut w, "charnel_burst", cell(17, 3));
    assert_eq!(last_said(&w), "That is out of range.");
    assert_eq!(w.state.turn, turn);
    assert_eq!(corpses_at(&w, cell(17, 3)), 1);
}

#[test]
fn a_kill_of_the_burst_leaves_a_corpse_for_the_next() {
    let mut w = world(4);
    let at = cell(7, 1);
    lay(&mut w, "kobold", at);
    let victim = kobold_at(&mut w, (6, -2), 1);
    let next = w.state.actors[&victim].pos;
    cast(&mut w, "charnel_burst", at);
    assert!(!w.state.actors.contains_key(&victim), "slain");
    assert_eq!(corpses_at(&w, at), 0);
    assert_eq!(corpses_at(&w, next), 1, "the next link of the chain");
    let turn = w.state.turn;
    cast(&mut w, "charnel_burst", next);
    assert!(w.state.turn > turn);
    assert_eq!(corpses_at(&w, next), 0);
}

#[test]
fn the_top_most_corpse_of_a_pile_is_the_one_used() {
    let mut w = world(4);
    let at = cell(5, 1);
    lay(&mut w, "kobold", at);
    lay(&mut w, "ogre", at);
    cast(&mut w, "rouse_the_dead", at);
    assert_eq!(corpses_at(&w, at), 1);
    let ogre = w.content().creature_id("ogre").expect("ogres");
    let kobold = w
        .content()
        .corpse_of(w.content().creature_id("kobold").expect("kobolds"));
    assert!(
        w.state
            .current()
            .objects_at(at)
            .iter()
            .any(|s| Some(s.object) == kobold)
    );
    assert!(w.state.actors.values().any(|a| a.creature == ogre));
}

#[test]
fn the_feast_heals_and_gives_mana_within_their_maxima() {
    let mut w = world(5);
    let beside = cell(3, 3);
    lay(&mut w, "kobold", beside);
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = 1;
    let turn = w.state.turn;
    w.player_mut().mana = 100;
    cast(&mut w, "grave_feast", beside);
    let me = w.state.player();
    assert!(w.state.turn > turn);
    assert!(me.hp > 1 && me.hp <= 1 + 12 + 6, "{}", me.hp);
    assert!(me.hp <= max);
    // Its cost out, 1 to 6 in.
    assert!(me.mana < 100 && me.mana > 80, "{}", me.mana);
    assert_eq!(corpses_at(&w, beside), 0);
    assert!(said(
        &w,
        "You draw what life is left out of the kobold corpse."
    ));

    // Never past the maxima: hurt by a point, with the mana only just short.
    let max_hp = w.state.player().max_hp(w.content());
    let max_mana = w.state.player().max_mana(w.content());
    lay(&mut w, "kobold", beside);
    w.player_mut().hp = max_hp - 1;
    w.player_mut().mana = max_mana;
    cast(&mut w, "grave_feast", beside);
    let me = w.state.player();
    assert_eq!(me.hp, max_hp);
    assert!(me.mana <= max_mana);
}

#[test]
fn the_feast_reaches_only_a_touch_and_wants_something_to_give() {
    let mut w = world(5);
    lay(&mut w, "kobold", cell(5, 3));
    let turn = w.state.turn;
    w.player_mut().hp = 1;
    cast(&mut w, "grave_feast", cell(5, 3));
    assert_eq!(last_said(&w), "You can only touch a corpse next to you.");
    // On the player's own cell it is within a touch.
    let here = w.player_pos();
    lay(&mut w, "kobold", here);
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    let full = w.state.player().max_mana(w.content());
    w.player_mut().mana = full;
    assert_eq!(w.state.turn, turn);
    cast(&mut w, "grave_feast", here);
    assert_eq!(last_said(&w), "You are not hurt.");
    assert_eq!(w.state.turn, turn);
    assert_eq!(corpses_at(&w, here), 1);
    // Whole but short of mana it goes off.
    w.player_mut().mana = full - 50;
    cast(&mut w, "grave_feast", here);
    assert!(w.state.turn > turn);
    assert_eq!(corpses_at(&w, here), 0);
}

#[test]
fn an_ogres_corpse_rises_an_ogre_at_half_its_hit_points_serving_and_departing() {
    let at = cell(6, 1);
    let (mut w, ogre) = rising(false, "ogre", at);
    let content = w.content().clone();
    let a = &w.state.actors[&ogre];
    assert!(
        a.pos.chebyshev(at) <= 1,
        "its turn has passed since it stood"
    );
    assert_eq!(a.hp, (a.max_hp(&content) + 1) / 2, "half, rounded up");
    assert!(a.inventory.is_empty(), "no gear");
    assert_eq!(a.npc, None);
    assert!(a.risen.is_none());
    let ally = a.ally.as_ref().expect("serving");
    assert_eq!(
        ally.source,
        AllySource::Summoned {
            spell: Some(sp("rouse_the_dead"))
        }
    );
    assert!(ally.until.is_some());
    assert_eq!(corpses_at(&w, at), 0, "used up");
    assert!(said(&w, "The dead thing stirs, and stands."));
    assert_eq!(w.stance(ogre, w.state.player), Stance::Neutral);
    let dump = dump::ascii(&w, &[Layer::Actors]);
    // Ten minutes, fifteen at the hall's Presence 14.
    assert!(dump.contains("ally of #0 (summoned, 15 min"), "{dump}");
    run(&mut w, "skip:1h wait");
    assert!(!w.state.actors.contains_key(&ogre), "gone with its time");
    assert_eq!(corpses_at(&w, at), 0, "nothing left behind");
}

#[test]
fn it_rises_on_its_own_cell_when_free() {
    let mut w = world(1);
    let at = cell(6, 1);
    assert_eq!(w.rising_cell(at), Some(at));
    kobold_at(&mut w, (4, -2), 500);
    assert_eq!(w.rising_cell(at).map(|p| p.chebyshev(at)), Some(1));
}

#[test]
fn it_rises_on_the_nearest_free_cell_when_its_own_is_taken() {
    for seed in 0..50 {
        let mut w = world(seed);
        let at = cell(6, 1);
        lay(&mut w, "ogre", at);
        kobold_at(&mut w, (4, -2), 500);
        cast(&mut w, "rouse_the_dead", at);
        let ogre = of_type(&w, "ogre")[0];
        let p = w.state.actors[&ogre].pos;
        assert_ne!(p, at);
        assert_eq!(p.chebyshev(at), 1);
    }
}

#[test]
fn a_second_death_leaves_nothing() {
    for want in [false, true] {
        let at = cell(6, 1);
        let (mut w, ogre) = rising(want, "ogre", at);
        let was = w.state.actors[&ogre].pos;
        w.state.actors.get_mut(&ogre).expect("there").hp = 1;
        let now = w.state.time;
        w.dies(ogre, None, None, now, &mut Vec::new());
        assert!(!w.state.actors.contains_key(&ogre));
        assert!(
            w.state.current().objects_at(was).is_empty(),
            "berserk {want}"
        );
    }
}

#[test]
fn a_uniques_corpse_will_not_rise() {
    let mut w = world(6);
    let at = cell(6, 1);
    lay(&mut w, "drow_matron", at);
    let (turn, mana) = (w.state.turn, w.state.player().mana);
    cast(&mut w, "rouse_the_dead", at);
    assert_eq!(last_said(&w), "It will not rise for you.");
    assert_eq!((w.state.turn, w.state.player().mana), (turn, mana));
    assert_eq!(corpses_at(&w, at), 1);
}

#[test]
fn a_mimic_rises_undisguised() {
    let at = cell(6, 1);
    let (w, mimic) = rising(false, "mimic", at);
    let a = &w.state.actors[&mimic];
    assert!(!a.effects.hides(w.content()));
    assert!(
        w.perception().actor_at(a.pos).is_some(),
        "seen for what it is"
    );
}

#[test]
fn the_third_sends_the_oldest_back_and_the_cap_refuses() {
    // A seed with three serving rises in a row.
    'seeds: for seed in 0..500 {
        let mut w = world(seed);
        let mut risen = Vec::new();
        for x in [5, 7, 9] {
            let at = cell(x, 1);
            lay(&mut w, "kobold", at);
            let before: Vec<ActorId> = of_type(&w, "kobold");
            cast(&mut w, "rouse_the_dead", at);
            let id = of_type(&w, "kobold")
                .into_iter()
                .find(|id| !before.contains(id));
            let Some(id) = id else {
                continue 'seeds;
            };
            if w.state.actors[&id].risen.is_some() {
                continue 'seeds;
            }
            risen.push(id);
        }
        assert!(
            !w.state.actors.contains_key(&risen[0]),
            "the oldest departed"
        );
        assert!(w.state.actors.contains_key(&risen[1]));
        assert!(w.state.actors.contains_key(&risen[2]));

        return;
    }
    panic!("no seed had three serving");
}

#[test]
fn the_allies_cap_refuses_before_anything_is_spent() {
    let mut w = world(10);
    let most = usize::from(w.content().allies().expect("allies").most);
    while w.allies_here(w.state.player).len() < most {
        run(&mut w, "ally:rat");
    }
    let spare = cell(11, 1);
    lay(&mut w, "kobold", spare);
    let (turn, mana) = (w.state.turn, w.state.player().mana);
    cast(&mut w, "rouse_the_dead", spare);
    assert_eq!(last_said(&w), "You cannot lead more than this.");
    assert_eq!((w.state.turn, w.state.player().mana), (turn, mana));
    assert_eq!(corpses_at(&w, spare), 1);
}

#[test]
fn it_rises_berserk_about_one_time_in_ten_by_chance_not_by_the_corpse() {
    let mut berserk = 0;
    for seed in 0..300 {
        let mut w = world(seed);
        let at = cell(6, 1);
        lay(&mut w, "kobold", at);
        cast(&mut w, "rouse_the_dead", at);
        let id = of_type(&w, "kobold")[0];
        if w.state.actors[&id].risen.is_some() {
            berserk += 1;
        }
    }
    assert!((12..=50).contains(&berserk), "{berserk} of 300");
}

#[test]
fn a_berserk_one_is_friendly_to_nothing() {
    let at = cell(6, 1);
    let (mut w, ogre) = rising(true, "ogre", at);
    let content = w.content().clone();
    let player = w.state.player;
    assert!(said(&w, "The dead thing stands, and it knows no one."));
    assert!(!said(&w, "The dead thing stirs, and stands."));
    let a = &w.state.actors[&ogre];
    assert!(a.ally.is_none(), "it serves no one");
    assert_eq!(
        content.factions().def(a.faction(&content).expect("one")).id,
        "berserk"
    );
    assert_eq!(a.hp, (a.max_hp(&content) + 1) / 2);
    assert_eq!(w.stance(ogre, player), Stance::Hostile);
    assert_eq!(w.stance(ogre, ogre), Stance::Neutral, "never itself");
    assert_eq!(w.allies_here(player).len(), 0, "no ally counted");
    let dump = dump::ascii(&w, &[Layer::Actors]);
    assert!(dump.contains("risen, faction berserk, 15 min"), "{dump}");
    // An ally of the player's, and any creature at all, are its enemies too.
    run(&mut w, "ally:wolf");
    let wolf = of_type(&w, "wolf")[0];
    assert_eq!(w.stance(ogre, wolf), Stance::Hostile);
    let kobold = kobold_at(&mut w, (6, 0), 500);
    assert_eq!(w.stance(ogre, kobold), Stance::Hostile);
    // Two of them are enemies to each other.
    lay(&mut w, "kobold", cell(10, 1));
    w.state.actors.get_mut(&kobold).expect("there").risen = w.state.actors[&ogre].risen;
    let faction = w.state.actors[&ogre].faction_override;
    w.state
        .actors
        .get_mut(&kobold)
        .expect("there")
        .faction_override = faction;
    assert_eq!(w.stance(ogre, kobold), Stance::Hostile);
    // Neither counts as the level's population.
    let spawned = w.hostiles_here();
    w.state.actors.get_mut(&kobold).expect("there").risen = None;
    assert_eq!(w.hostiles_here(), spawned + 1);
}

#[test]
fn a_berserk_one_attacks_the_caster_and_another_berserk() {
    // It comes for the caster.
    let at = cell(3, 1);
    let (mut w, _) = rising(true, "ogre", at);
    w.player_mut().hp = 10_000;
    let hp = w.state.player().hp;
    for _ in 0..30 {
        w.apply(&Action::Wait);
        if w.state.player().hp < hp || w.state.death.is_some() {
            break;
        }
    }
    assert!(w.state.player().hp < hp, "the caster struck");

    // Two berserk beside each other fight when the player is out of the way of neither.
    let (mut w, ogre) = rising(true, "ogre", cell(14, 5));
    let other = kobold_at(&mut w, (13, 2), 10_000);
    let faction = w.state.actors[&ogre].faction_override;
    let until = w.state.actors[&ogre].risen;
    let foe = w.state.actors.get_mut(&other).expect("there");
    foe.faction_override = faction;
    foe.risen = until;
    let start = w.state.actors[&other].hp;
    for _ in 0..30 {
        w.apply(&Action::Wait);
        let now = w.state.actors.get(&other).map_or(0, |a| a.hp);
        if now < start {
            return;
        }
    }
    panic!("they never fought");
}

#[test]
fn a_berserk_ones_clock_ends_it_and_its_kill_pays() {
    let at = cell(16, 5);
    let (mut w, ogre) = rising(true, "ogre", at);
    let until = w.state.actors[&ogre].risen.expect("its clock");
    assert!(until > w.state.time + Duration::minutes(14));
    assert!(until <= w.state.time + Duration::minutes(15));
    // Far off and held still, nothing fights until the time is up.
    run(&mut w, "skip:1h wait");
    assert!(!w.state.actors.contains_key(&ogre), "gone with its time");
    assert!(w.state.current().objects_at(at).is_empty());

    let (mut w, ogre) = rising(true, "ogre", at);
    let before = w.perception().own_experience().xp;
    let (player, now) = (w.state.player, w.state.time);
    w.dies(ogre, Some(player), None, now, &mut Vec::new());
    assert!(w.perception().own_experience().xp > before, "pays");
}

#[test]
fn risen_is_left_out_of_a_save_while_empty() {
    let w = world(7);
    let text = serde_json::to_string(w.state.player()).expect("json");
    assert!(!text.contains("risen"), "{text}");
    let (w, ogre) = rising(true, "ogre", cell(6, 1));
    let state = w.state().clone();
    let back = World::from_state(w.content().clone(), state).expect("restores");
    assert_eq!(back.state.actors[&ogre].risen, w.state.actors[&ogre].risen);
}

#[test]
fn the_scrolls_are_read_at_a_corpse() {
    let mut w = world(8);
    let content = w.content().clone();
    for (name, spell) in [
        ("scroll_of_charnel_burst", "charnel_burst"),
        ("scroll_of_grave_feast", "grave_feast"),
        ("scroll_of_rouse_the_dead", "rouse_the_dead"),
    ] {
        let object = content.object_id(name).expect("a scroll");
        let read = content.object(object).read.as_ref().expect("reads");
        assert_eq!(read.scroll().expect("a spell").spell, sp(spell));
        assert!(read.aimed(), "{name}");
    }
    let object = content
        .object_id("scroll_of_rouse_the_dead")
        .expect("a scroll");
    w.player_mut().inventory.clear();
    w.player_mut().inventory.push(Stack::new(object, 2));
    let mana = w.state.player().mana;
    // No corpse there: refused, the scroll kept, no time.
    let turn = w.state.turn;
    w.apply(&Action::Read {
        item: 0,
        target: Some(ReadTarget::Cell(cell(6, 1))),
    });
    assert_eq!(w.state.turn, turn);
    assert_eq!(w.state.player().inventory[0].count, 2);
    lay(&mut w, "kobold", cell(6, 1));
    w.apply(&Action::Read {
        item: 0,
        target: Some(ReadTarget::Cell(cell(6, 1))),
    });
    assert!(w.state.turn > turn);
    assert_eq!(w.state.player().inventory[0].count, 1);
    assert_eq!(w.state.player().mana, mana);
    assert_eq!(of_type(&w, "kobold").len(), 1);
}

#[test]
fn the_shipped_rise_names_a_faction_hostile_to_all() {
    let w = world(9);
    let rise = (w.content().spell(sp("rouse_the_dead")).effect.rise.as_ref()).expect("a rise");
    let faction = w
        .content()
        .factions()
        .id(&rise.berserk_faction)
        .expect("known");
    assert!(w.content().factions().def(faction).hostile_to_all);
}

#[test]
fn the_rise_and_corpse_fields_are_checked_at_load() {
    let spell = |effect: &str, kind: &str, shape: &str| {
        let mut files = files();
        let text = format!(
            "[[spell]]\nid = \"zz\"\nname = \"Z\"\ndescription = \"x\"\nlevel = 1\n\
             school = \"necromancy\"\nkind = \"{kind}\"\nshape = \"{shape}\"\n\
             resolution = \"{}\"\neffect = {effect}\n",
            if shape == "self" { "auto" } else { "touch" },
        );
        files.insert("data/spells/zz.toml", &text);
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    let rise = "rise = { most = 2, minutes = 10, hit_points = 50, berserk = 10, \
                berserk_faction = \"berserk\", berserk_message = \"x\" }";
    // A rise needs a corpse.
    let err = spell(&format!("{{ {rise} }}"), "summon", "self").unwrap_err();
    assert!(err.contains("needs `corpse`"), "{err}");
    let ok = spell(
        &format!("{{ {rise}, corpse = \"kept\" }}"),
        "summon",
        "self",
    );
    assert!(ok.is_ok(), "{ok:?}");
    // What may ask for a corpse, and mana on a heal only.
    let err = spell("{ heal = \"1d4\", corpse = \"gone\" }", "heal", "touch").unwrap_err();
    assert!(err.contains("`consumed` or `kept`"), "{err}");
    let err = spell("{ heal = \"1d4\", corpse = \"kept\" }", "heal", "self").unwrap_err();
    assert!(err.contains("`corpse` is a burst's"), "{err}");
    let err = spell(&format!("{{ {rise} }}"), "heal", "self").unwrap_err();
    assert!(err.contains("has no `rise`"), "{err}");
}
