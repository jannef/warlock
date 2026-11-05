//! The three buff statuses with fields of their own (`data/status.toml`, the spell types design
//! `docs/design/spell-types-and-new-spells.md` §7 entries 24 to 26): Fire cloak's `retaliates`,
//! Water walking's `wades_free` and `ice_holds`, Brand's `blows`; and their spells' lights,
//! class rows, and scrolls.

use super::caster_tests::{kobold_at, sp};
use super::*;
use crate::action::{Action, AttackKind, Event, parse_actions};
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, StatusId};
use crate::dice::Dice;
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

const POND: &str = r#"
name = "Pond"
temperature = -5
ambient = [255, 255, 255, 0]
map = '''
#####
#@t~#
#####
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"~" = { terrain = "water" }
"t" = { terrain = "water", ice = 20 }
"#;

const THIN: Pos = Pos::new(2, 1);

static PONDED: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert("levels/pond.toml", POND);
    files.insert(
        "scenarios/pond.toml",
        "name = \"pond\"\nstart_level = \"pond\"\nhp = 100\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
});

// Without background stories: the kit's weapons alone.
static BUILTIN: LazyLock<Arc<Content>> = LazyLock::new(|| Arc::new(crate::background::storyless()));

fn showcase(seed: u64) -> World {
    let mut w = World::new(BUILTIN.clone(), "showcase", Some(seed)).expect("world");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    w
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect("a status")
}

fn on(w: &mut World, id: ActorId, name: &str) {
    let s = status(w, name);
    w.afflict(id, s, 1, None, &mut Vec::new());
}

fn look(w: &mut World) {
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.text == text)
}

/// A kobold beside the player (east), of `hp` hit points, and the player cloaked in fire.
fn cloaked(seed: u64, hp: i32) -> (World, ActorId, ActorId) {
    let mut w = showcase(seed);
    let player = w.state.player;
    let kobold = kobold_at(&mut w, (1, 0), hp);
    on(&mut w, player, "fire_cloaked");
    look(&mut w);
    (w, player, kobold)
}

#[test]
fn the_cloak_burns_whoever_lands_a_blow_in_melee_and_only_them() {
    let (mut w, player, kobold) = cloaked(1, 1000);
    let mut events = Vec::new();
    w.barbed(kobold, player, AttackKind::Melee, &mut events);
    let lost = 1000 - w.state.actors[&kobold].hp;
    assert!((1..=6).contains(&lost), "1d6, not {lost}");
    assert!(said(&w, "The fire burns the kobold."));
    assert!(
        (events.iter()).any(|e| matches!(
            e,
            Event::Attacked { attacker, target, kind: AttackKind::Spell, .. }
                if *attacker == player && *target == kobold
        )),
        "{events:?}"
    );
    // A shot is no blow up close, and one's own blow is not struck back at.
    let hp = w.state.actors[&kobold].hp;
    w.barbed(kobold, player, AttackKind::Ranged, &mut Vec::new());
    w.barbed(kobold, player, AttackKind::Spell, &mut Vec::new());
    assert_eq!(w.state.actors[&kobold].hp, hp);
    let mine = w.state.player().hp;
    w.barbed(player, player, AttackKind::Melee, &mut Vec::new());
    assert_eq!(w.state.player().hp, mine);
    // Without the status, nothing and no draw.
    let mut plain = showcase(1);
    let (player, kobold) = (plain.state.player, kobold_at(&mut plain, (1, 0), 50));
    let rng = plain.state.rng.clone();
    plain.barbed(kobold, player, AttackKind::Melee, &mut Vec::new());
    assert_eq!(plain.state.actors[&kobold].hp, 50);
    assert_eq!(plain.state.rng, rng);
}

/// How many of 400 strikers burn, by `seed`.
fn burns(seed: u64) -> u32 {
    let (mut w, player, kobold) = cloaked(seed, 1000);
    let burning = status(&w, "burning");
    let mut count = 0;
    for _ in 0..400 {
        let actor = w.state.actors.get_mut(&kobold).expect("there");
        actor.hp = 1000;
        actor.effects = Default::default();
        w.barbed(kobold, player, AttackKind::Melee, &mut Vec::new());
        count += u32::from(w.state.actors[&kobold].effects.has(burning));
    }
    count
}

#[test]
fn the_fire_types_burn_falls_on_the_striker_one_time_in_four_by_the_draw() {
    let count = burns(3);
    assert!((70..=130).contains(&count), "about 100 of 400, not {count}");
    assert_eq!(count, burns(3), "the same seed, the same burns");
}

#[test]
fn one_immune_to_fire_is_neither_hurt_nor_burnt_by_the_cloak() {
    let (mut w, player, kobold) = cloaked(3, 1000);
    let id = w.content().creature_id("cinder_elemental").expect("one");
    w.state.actors.get_mut(&kobold).expect("there").creature = id;
    let burning = status(&w, "burning");
    for _ in 0..50 {
        w.barbed(kobold, player, AttackKind::Melee, &mut Vec::new());
        let actor = &w.state.actors[&kobold];
        assert_eq!(actor.hp, 1000);
        assert!(!actor.effects.has(burning));
    }
}

#[test]
fn a_kill_of_the_cloak_is_the_wearers_unlike_the_barbs() {
    let (mut w, player, kobold) = cloaked(3, 1);
    let xp = w.player_xp();
    w.barbed(kobold, player, AttackKind::Melee, &mut Vec::new());
    assert!(!w.state.actors.contains_key(&kobold), "burnt to death");
    assert!(w.player_xp() > xp, "it earned its experience");
    assert!(said(&w, "The kobold dies."), "{:?}", w.messages());
}

#[test]
fn a_player_who_strikes_a_cloaked_foe_burns_and_may_die_of_it() {
    let mut w = showcase(5);
    let kobold = kobold_at(&mut w, (1, 0), 1000);
    on(&mut w, kobold, "fire_cloaked");
    w.player_mut().hp = 1;
    for _ in 0..200 {
        look(&mut w);
        w.apply(&Action::Move(Dir::E));
        let me = w.state.player();
        if me.hp < 1 || w.state.death.is_some() {
            break;
        }
        // Whatever else it did, a landed blow burnt the player: keep them on one hit point
        // until one kills.
        let pos = w.player_pos();
        let actor = w.state.actors.get_mut(&kobold).expect("there");
        actor.hp = 1000;
        actor.pos = Pos::new(pos.x + 1, pos.y);
    }
    assert!(w.state.death.is_some(), "the fire killed them");
    assert!(said(&w, "The fire burns you."), "{:?}", w.messages());
}

/// A real fight: a kobold that bites a cloaked player until the fire has answered a blow; the
/// state's hash at the end, for the replay's sake.
fn bitten(seed: u64) -> u64 {
    let (mut w, player, kobold) = cloaked(seed, 1000);
    let s = status(&w, "fire_cloaked");
    for _ in 0..500 {
        let max = w.state.player().max_hp(w.content());
        w.player_mut().hp = max;
        w.player_mut().effects.remove(s);
        w.afflict(player, s, 1, None, &mut Vec::new());
        look(&mut w);
        w.apply(&Action::Wait);
        if w.state.actors[&kobold].hp < 1000 {
            break;
        }
    }
    assert!(
        w.state.actors[&kobold].hp < 1000,
        "a bite landed and burnt it"
    );
    assert!(said(&w, "The fire burns the kobold."));
    w.state_hash()
}

#[test]
fn a_creatures_bite_is_answered_in_a_real_turn_and_the_same_every_time() {
    assert_eq!(bitten(11), bitten(11));
}

fn pond(seed: u64) -> World {
    World::new(PONDED.clone(), "pond", Some(seed)).expect("world")
}

fn price(w: &World, terrain: &str) -> i64 {
    let id = w.content().terrain_id(terrain).expect("terrain");
    w.step_price(w.player_id(), w.content().terrain(id), false)
}

#[test]
fn the_water_borne_wade_free_and_after_it_pay_again() {
    let mut w = pond(1);
    let player = w.player_id();
    let (dry, wet) = (price(&w, "floor"), price(&w, "water"));
    assert!(wet > dry);
    assert!(!w.state.player().wades_free(w.content()));
    on(&mut w, player, "water_borne");
    assert!(w.state.player().wades_free(w.content()));
    assert_eq!(price(&w, "water"), dry);
    assert_eq!(price(&w, "marsh"), dry);
    assert_eq!(price(&w, "bush"), price(&pond(1), "bush"), "water's alone");
    let s = status(&w, "water_borne");
    w.player_mut().effects.remove(s);
    assert_eq!(price(&w, "water"), wet);
    // The talent asks through the same question.
    w.player_mut().talents.take("undertow", None);
    assert!(w.state.player().wades_free(w.content()));
    assert_eq!(price(&w, "water"), dry);
}

/// The player steps east onto the thin ice: whether they plunged in.
fn step_onto_thin(w: &mut World) -> bool {
    let mut fell = false;
    for action in parse_actions("move:E").expect("step") {
        let outcome = w.apply(&action);
        fell |= (outcome.events.iter()).any(|e| matches!(e, Event::FellThroughIce { .. }));
    }
    assert_eq!(w.player_pos(), THIN);
    fell
}

#[test]
fn under_the_water_borne_thin_ice_holds() {
    let mut w = pond(3);
    let player = w.player_id();
    assert!(!w.state.player().ice_holds(w.content()));
    on(&mut w, player, "water_borne");
    assert!(w.state.player().ice_holds(w.content()));
    let hp = w.state.player().hp;
    assert!(!step_onto_thin(&mut w), "no plunge");
    assert!(w.state.current().ice_at(THIN).is_some(), "the ice is whole");
    assert_eq!(w.state.player().hp, hp);
    assert!(
        !w.state.current().is_open_water(w.content(), THIN),
        "not wet"
    );
    // Without it the same step plunges through.
    let mut plain = pond(3);
    let hp = plain.state.player().hp;
    assert!(step_onto_thin(&mut plain));
    assert!(plain.state.current().is_open_water(plain.content(), THIN));
    assert!(plain.state.player().hp < hp);
}

#[test]
fn a_burst_of_heat_leaves_the_ice_under_the_water_borne() {
    let mut w = pond(3);
    let player = w.player_id();
    on(&mut w, player, "water_borne");
    step_onto_thin(&mut w);
    let hp = w.state.player().hp;
    let mut events = Vec::new();
    let broke = w.shock(THIN, 1, crate::ice::Shock::Heat, &mut events);
    assert_eq!(broke, 0, "the open water beside it has no ice to melt");
    assert!(!(events.iter()).any(|e| matches!(e, Event::FellThroughIce { .. })));
    assert_eq!(w.state.player().hp, hp);
    assert!(w.state.current().ice_at(THIN).is_some_and(|i| i.um > 0));
    // When it wears off, the next burst melts it under them.
    let s = status(&w, "water_borne");
    w.player_mut().effects.remove(s);
    let mut events = Vec::new();
    assert_eq!(w.shock(THIN, 0, crate::ice::Shock::Heat, &mut events), 1);
    assert!((events.iter()).any(|e| matches!(e, Event::FellThroughIce { .. })));
}

/// A ranger with a bow, a sword, and a dagger in the off hand.
fn ranger() -> World {
    let spec: CharacterSpec = "human,male,ranger".parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    for action in parse_actions("move:SW*2 pickup equip:off_hand:k").expect("actions") {
        w.apply(&action);
    }
    w
}

#[test]
fn brands_fire_rides_both_hands_blows_and_the_attack_line_and_no_other_attack() {
    let mut w = ranger();
    let player = w.player_id();
    let content = w.content().clone();
    let fire = content.damage_type_id("fire").expect("fire");
    let rode = |p: &crate::combat::Profile| {
        (p.riders.iter()).any(|r| r.kind == fire && r.dice == Dice::of(1, 4))
    };
    let me = w.state.player();
    assert!(!rode(&me.melee(&content)));
    let (kick, shot) = (me.kick(&content), me.ranged(&content).expect("a bow"));
    on(&mut w, player, "branded_blade");
    look(&mut w);
    let me = w.state.player();
    assert!(rode(&me.melee(&content)));
    assert!(rode(&me.off_hand_blow(&content).expect("an off hand").1));
    assert_eq!(me.kick(&content), kick, "a kick is no held weapon's");
    assert_eq!(me.ranged(&content).expect("a bow"), shot, "melee's alone");
    let attacks = w.perception().own_attacks();
    let lines: Vec<String> = attacks.iter().map(|a| a.damage_text(&content)).collect();
    assert!(lines[0].ends_with("+1d4 fire"), "{lines:?}");
    assert!(lines[1].ends_with("+1d4 fire"), "{lines:?}");
    assert!(!lines[2].contains("fire"), "{lines:?}");
}

#[test]
fn brands_fire_hurts_and_never_burns_for_type_rules_never_reach_a_blow() {
    let mut w = showcase(9);
    let player = w.state.player;
    let kobold = kobold_at(&mut w, (1, 0), 100_000);
    on(&mut w, player, "branded_blade");
    let burning = status(&w, "burning");
    let mut hits = 0;
    for _ in 0..200 {
        look(&mut w);
        let max = w.state.player().max_hp(w.content());
        w.player_mut().hp = max;
        let pos = w.player_pos();
        let hp = w.state.actors[&kobold].hp;
        let actor = w.state.actors.get_mut(&kobold).expect("there");
        actor.pos = Pos::new(pos.x + 1, pos.y);
        w.apply(&Action::Move(Dir::E));
        hits += u32::from(w.state.actors[&kobold].hp < hp);
        assert!(!w.state.actors[&kobold].effects.has(burning));
    }
    assert!(hits > 20, "it struck home, {hits} times");
}

#[test]
fn each_status_sheds_its_light_while_it_lasts_and_shows_on_the_wearer() {
    for (name, light) in [
        ("fire_cloaked", "ember"),
        ("branded_blade", "ember"),
        ("water_borne", "circle_transmutation"),
    ] {
        let mut w = showcase(1);
        let kobold = kobold_at(&mut w, (2, 0), 50);
        let light = w.content().light_id(light).expect("a light");
        let at = w.state.actors[&kobold].pos;
        look(&mut w);
        let shed = |w: &World| {
            (w.perception().light_sources().iter())
                .any(|s| s.pos() == Some(at) && s.preset() == Some(light))
        };
        assert!(!shed(&w), "{name}: none before");
        on(&mut w, kobold, name);
        look(&mut w);
        assert!(shed(&w), "{name}: its light");
        let s = status(&w, name);
        assert!(w.perception().effects_of(kobold).any(|e| e == s), "{name}");
    }
}

#[test]
fn the_spells_cast_on_oneself_and_the_presence_lengthens_them() {
    for (spell, name) in [
        ("fire_cloak", "fire_cloaked"),
        ("water_walking", "water_borne"),
        ("brand", "branded_blade"),
    ] {
        let mut w = showcase(1);
        w.player_mut().spells.insert(sp(spell));
        w.player_mut().mana_base = 100;
        w.player_mut().mana = 100;
        look(&mut w);
        w.apply(&Action::Cast(spell.into(), None));
        let s = status(&w, name);
        let got = w.state.player().effects.get(s).expect("cast");
        assert_eq!(got.severity, 2, "{spell}: Presence 14's");
    }
}

#[test]
fn the_class_rows_and_the_scrolls_are_the_designs() {
    let content = Content::builtin().expect("content");
    let row = |class: &str, talent: &str| {
        let class = (content.classes().iter())
            .find(|c| c.id == class)
            .expect("a class");
        (class.abilities.iter())
            .find(|(_, t)| t == talent)
            .map(|(l, _)| *l)
    };
    assert_eq!(row("druid", "water_walking"), Some(6));
    assert_eq!(row("ranger", "water_walking"), Some(12));
    assert_eq!(row("paladin", "brand"), Some(14));
    let at = content.spell_id("fire_cloak").expect("the spell");
    assert!(content.class_spell(at).is_none(), "scroll only");
    for (scroll, level, frequency, value) in [
        ("scroll_of_fire_cloak", 6, 20, 200),
        ("scroll_of_water_walking", 3, 35, 100),
        ("scroll_of_brand", 4, 30, 130),
    ] {
        let id = content.object_id(scroll).expect("a scroll");
        let def = content.object(id);
        assert_eq!(
            (def.level, def.frequency, def.value),
            (level, frequency, value)
        );
    }
}
