//! The Weaponcraft and Guard talents in the world (spec §8 "Talents"; #238): each changes the
//! one rule it names — a weapon skill's or a role's benefits, ranges, worn armour's cap,
//! Deflection, Protection while heavy armour or a shield is worn, a stance's to-hit penalty and
//! fleeing steps, hits after the soak, a chosen element's resistance — and their prerequisites
//! are checked and said in words.

use super::fight::Harm;
use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::combat::Category;
use crate::content::{Benefit, CombatStance, Factor, MAX_RANK, WeaponSkillId};
use crate::equipment::{free_slot, slots_taken};
use crate::object::Stack;
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human warrior in the showcase, every score 14.
fn world() -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let mut w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(14));
    w
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn last(world: &World) -> String {
    (world.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

/// Grants the XP that takes the player to `level`.
fn reach(world: &mut World, level: u8) {
    let spec = world.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(world.content(), spec);
    let need = world.content().experience().threshold_for(level, factor);
    let more = need - world.player_xp();
    run(world, &format!("xp:{more}"));
}

/// Gives the player talent `id` (one rank more), for `choice`.
fn give(w: &mut World, id: &str, choice: Option<&str>) {
    w.player_mut().talents.take(id, choice);
}

fn skill(w: &World, name: &str) -> WeaponSkillId {
    w.content().weapon_skills().id(name).expect(name)
}

/// Sets the player's rank in weapon skill or style `name`.
fn set_rank(w: &mut World, name: &str, rank: u8) {
    let id = skill(w, name);
    let table = w.content().weapon_skills().clone();
    w.player_mut().weapon_skills.raise(&table, id, rank);
}

/// What the player's ranks (and talents) in `name` give.
fn benefit(w: &World, name: &str) -> Benefit {
    w.state.player().benefit(w.content(), Some(skill(w, name)))
}

/// The player carries `objects` and wears or wields each in the first free slot it fits.
fn arm(w: &mut World, objects: &[&str]) {
    let content = w.content().clone();
    let me = w.player_mut();
    for name in objects {
        let object = content.object_id(name).expect(name);
        me.inventory.push(Stack::new(object, 1));
        let item = me.inventory.len() - 1;
        let slot = free_slot(&content, &me.equipment, object).expect("a free slot");
        me.equipment.put(&slots_taken(&content, object, slot), item);
    }
}

fn defence(w: &World) -> crate::combat::Defence {
    w.state.player().defence(w.content(), Burden::Unburdened)
}

fn protection(w: &World) -> i32 {
    w.state.player().protection(w.content()).against(None)
}

#[test]
fn weapon_focus_and_its_kind_sharpen_only_the_chosen_skill() {
    let mut w = world();
    arm(&mut w, &["long_sword"]);
    let content = w.content().clone();
    let melee = |w: &World| w.state.player().melee(&content);
    let plain = melee(&w);
    give(&mut w, "weapon_focus", Some("axes"));
    assert_eq!(melee(&w).to_hit, plain.to_hit, "another skill's focus");
    give(&mut w, "weapon_focus", Some("long_blades"));
    give(&mut w, "weapon_specialization", Some("long_blades"));
    give(&mut w, "improved_critical", Some("long_blades"));
    let sharp = melee(&w);
    assert_eq!(sharp.to_hit, plain.to_hit + 1);
    assert_eq!(sharp.bonus, plain.bonus + 2);
    assert_eq!(
        sharp.threat,
        plain.threat - 1,
        "threatens from one roll lower"
    );
}

#[test]
fn stylist_reads_a_style_one_rank_higher_but_never_past_the_top() {
    let mut w = world();
    let table = w.content().weapon_skills().clone();
    // An untrained style stays untrained.
    let duelist = skill(&w, "duelist");
    give(&mut w, "stylist", Some("duelist"));
    assert_eq!(benefit(&w, "duelist"), table.benefit(duelist, 0));
    let style = skill(&w, "sword_and_board");
    set_rank(&mut w, "sword_and_board", 3);
    give(&mut w, "stylist", Some("sword_and_board"));
    assert_eq!(benefit(&w, "sword_and_board"), table.benefit(style, 4));
    set_rank(&mut w, "sword_and_board", MAX_RANK);
    assert_eq!(
        benefit(&w, "sword_and_board"),
        table.benefit(style, MAX_RANK)
    );
}

#[test]
fn role_talents_add_to_every_skill_of_their_role() {
    let mut w = world();
    let before = |w: &World, name: &str| benefit(w, name);
    let (bows, crossbows, thrown, block, twin) = (
        before(&w, "bows"),
        before(&w, "crossbows"),
        before(&w, "thrown"),
        before(&w, "shield_block"),
        before(&w, "twin_blades"),
    );
    for talent in [
        "deadly_aim",
        "rapid_reload",
        "thrower",
        "shield_expert",
        "two_weapon_training",
    ] {
        give(&mut w, talent, None);
    }
    assert_eq!(benefit(&w, "bows").damage, bows.damage + 2);
    assert_eq!(benefit(&w, "crossbows").damage, crossbows.damage + 2);
    assert_eq!(benefit(&w, "crossbows").reload, crossbows.reload + 25);
    assert_eq!(benefit(&w, "thrown").damage, thrown.damage + 1);
    assert_eq!(benefit(&w, "thrown").speed, thrown.speed + 10);
    assert_eq!(benefit(&w, "shield_block").shield, block.shield + 1);
    assert_eq!(benefit(&w, "twin_blades").ease_main, twin.ease_main + 2);
    assert_eq!(benefit(&w, "twin_blades").ease_off, twin.ease_off + 2);
    let long_blades = w
        .content()
        .weapon_skills()
        .benefit(skill(&w, "long_blades"), 2);
    assert_eq!(benefit(&w, "long_blades"), long_blades, "no launcher");
    // Shield expert's free bash: more likely only where there is one.
    set_rank(&mut w, "sword_and_board", 1);
    assert_eq!(benefit(&w, "sword_and_board").bash, 0);
    set_rank(&mut w, "sword_and_board", 3);
    let table = w.content().weapon_skills().clone();
    let base = table.benefit(skill(&w, "sword_and_board"), 3).bash;
    assert!(base > 0);
    assert_eq!(benefit(&w, "sword_and_board").bash, base + 10);
    // And the shield turns more blows: Shield +1 in Defence while one is held.
    let without = defence(&w).bonus(Category::Shield);
    arm(&mut w, &["small_shield"]);
    let shield = defence(&w).bonus(Category::Shield);
    w.player_mut().talents = Default::default();
    assert_eq!(defence(&w).bonus(Category::Shield), shield - 1);
    assert_eq!(without, 0);
}

#[test]
fn far_shot_reaches_farther() {
    let mut w = world();
    arm(&mut w, &["short_bow"]);
    let content = w.content().clone();
    let me = |w: &World| w.state.player().clone();
    let dagger = content.object_id("dagger").expect("dagger");
    w.player_mut().inventory.push(Stack::new(dagger, 1));
    let item = me(&w).inventory.len() - 1;
    let (shot, throw) = (
        me(&w).launcher_range(&content).expect("a bow"),
        me(&w).throw_range(item, &content).expect("a dagger"),
    );
    give(&mut w, "far_shot", None);
    assert_eq!(me(&w).launcher_range(&content), Some(shot + 2));
    assert_eq!(me(&w).throw_range(item, &content), Some(throw + 1));
}

#[test]
fn guard_talents_change_defence_and_protection() {
    let mut w = world();
    w.player_mut().stats = Stats::new(PerStat::splat(20), PerStat::splat(20));
    // Plate caps Dexterity at 1: Armour training raises the cap one a rank.
    arm(&mut w, &["plate_armour"]);
    let dex = defence(&w).bonus(Category::Dexterity);
    assert_eq!(dex, 1);
    give(&mut w, "armour_training", None);
    give(&mut w, "armour_training", None);
    assert_eq!(defence(&w).bonus(Category::Dexterity), 3);

    // Deflection +1.
    let deflection = defence(&w).bonus(Category::Deflection);
    give(&mut w, "deflection", None);
    assert_eq!(defence(&w).bonus(Category::Deflection), deflection + 1);

    // Heavy armour mastery: Protection +1 in plate (6), Shield wall +1 while a shield is held.
    let worn = protection(&w);
    give(&mut w, "heavy_armour_mastery", None);
    give(&mut w, "shield_wall", None);
    assert_eq!(protection(&w), worn + 1, "no shield yet");
    arm(&mut w, &["small_shield"]);
    assert_eq!(protection(&w), worn + 2);
}

#[test]
fn heavy_armour_mastery_asks_for_heavy_armour() {
    let mut w = world();
    give(&mut w, "heavy_armour_mastery", None);
    assert_eq!(protection(&w), 0, "nothing worn");
    arm(&mut w, &["leather_jerkin"]);
    assert_eq!(protection(&w), 1, "a jerkin is light");
    let mut w = world();
    give(&mut w, "heavy_armour_mastery", None);
    arm(&mut w, &["chain_mail"]);
    assert_eq!(protection(&w), 4, "chain mail (3) is heavy");
}

#[test]
fn stalwart_and_unflinching_soften_their_stances() {
    let mut w = world();
    let content = w.content().clone();
    let to_hit = |w: &World| w.state.player().melee(&content).to_hit;
    let neutral = to_hit(&w);
    w.player_mut().stance = CombatStance::Defensive;
    let defensive = to_hit(&w);
    assert!(defensive < neutral);
    give(&mut w, "stalwart", None);
    assert_eq!(to_hit(&w), neutral, "no to-hit lost");
    w.player_mut().stance = CombatStance::Coward;
    let coward = to_hit(&w);
    give(&mut w, "unflinching", None);
    assert_eq!(
        to_hit(&w),
        neutral - (neutral - coward) / 2,
        "half the penalty"
    );

    // Fleeing steps 10 % cheaper again.
    let floor = content.terrain(content.terrain_id("floor").expect("floor"));
    let me = w.state.player;
    let fleeing = w.step_price(me, floor, true);
    w.player_mut().talents = Default::default();
    assert_eq!(fleeing, w.step_price(me, floor, true) * 90 / 100);
}

#[test]
fn immune_to_pain_eases_every_hit_but_never_to_nothing() {
    let mut w = world();
    let me = w.state.player;
    give(&mut w, "immune_to_pain", None);
    let hp = |w: &World| w.state.player().hp;
    let mut events = Vec::new();
    let start = hp(&w);
    w.hurt(me, 5, Harm::Actor(ActorId(99)), &mut events);
    assert_eq!(hp(&w), start - 4);
    w.hurt(me, 1, Harm::Actor(ActorId(99)), &mut events);
    assert_eq!(hp(&w), start - 5, "never below 1");
    w.hurt(me, 0, Harm::Actor(ActorId(99)), &mut events);
    assert_eq!(hp(&w), start - 5, "a hit soaked to nothing stays nothing");
    let when = w.state.time;
    w.hurt(
        me,
        3,
        Harm::Status {
            of: None,
            by: None,
            when,
        },
        &mut events,
    );
    assert_eq!(hp(&w), start - 8, "a status is no hit");
}

#[test]
fn elemental_ward_resists_the_chosen_element_by_rank() {
    let mut w = world();
    let content = w.content().clone();
    let fire = content.damage_type_id("fire");
    let cold = content.damage_type_id("cold");
    let resists = |w: &World, kind| w.state.player().resistances(&content).get(kind);
    give(&mut w, "elemental_ward", Some("fire"));
    assert_eq!(resists(&w, fire), 25);
    assert_eq!(resists(&w, cold), 0);
    give(&mut w, "elemental_ward", Some("fire"));
    assert_eq!(resists(&w, fire), 50);
}

#[test]
fn prerequisites_of_the_trees_are_said_in_words() {
    let mut w = world();
    w.player_mut().stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    reach(&mut w, 3);
    run(&mut w, "talent:deadly_aim");
    assert_eq!(
        last(&w),
        "You cannot take Deadly aim: you need Dexterity 13 and a missile weapon skill 3."
    );
    run(&mut w, "talent:weapon_specialization:long_blades");
    assert!(
        last(&w).starts_with(
            "You cannot take Weapon specialization (long blades): you need Weapon focus"
        ),
        "{}",
        last(&w)
    );
    // A launcher at 3 meets "a missile weapon skill 3".
    w.player_mut().stats = Stats::new(PerStat::splat(13), PerStat::splat(13));
    set_rank(&mut w, "slings", 3);
    run(&mut w, "talent:deadly_aim");
    assert_eq!(last(&w), "You learn Deadly aim.");
    // The level's skill rolls come first, or the next level cannot be reached.
    run(&mut w, "levelup:auto");
    // The free ones of both trees ask nothing.
    reach(&mut w, 6);
    run(&mut w, "talent:far_shot");
    assert_eq!(last(&w), "You learn Far shot.");
    // The level's skill rolls come first, or the next level cannot be reached.
    run(&mut w, "levelup:auto");
    reach(&mut w, 9);
    run(&mut w, "talent:armour_training");
    assert_eq!(last(&w), "You learn Armour training (1).");
    // The level's skill rolls come first, or the next level cannot be reached.
    run(&mut w, "levelup:auto");
    reach(&mut w, 12);
    run(&mut w, "talent:elemental_ward:cold");
    assert_eq!(last(&w), "You learn Elemental ward (cold) (1).");
}
