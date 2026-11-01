//! The caster's ladder (spec §8 "Weapon skills", "Fighting styles"; #571; the spell scaling
//! design §10.2): the caster style of where a spell is cast from (afar, beside, in armour, on
//! oneself) and the marks every spell paid in mana earns in it, Sniper's cells and Toucher's
//! bonus, what the ranks do to a rolling spell's roll, the critical held by nothing, the caster
//! styles' abilities, Battle caster's armour, the style marked in use, and the classes' starting
//! ranks (the schools' own marks: `school_skill_tests.rs`).

use super::caster_tests::{kobold, reach, run, said, sp, world_with};
use super::*;
use crate::action::Event;
use crate::combat::Strike;
use crate::content::{ContentFiles, Role, WeaponSkillId};
use crate::weapon_skills::WeaponSkills;

/// The built-in content with the wizard's Arcane bolt (level 6) as `body` says: its shape,
/// resolution, and dice (as `cast_tests` tries the spells the casters do not have).
fn content_with_bolt(body: &str) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    let text = format!(
        "[[spell]]\nid = \"arcane_bolt\"\nname = \"Arcane bolt\"\ndescription = \"x\"\n\
         level = 6\nschool = \"evocation\"\nkind = \"damage\"\ndamage_type = \"force\"\n{body}\n"
    );
    files.insert("data/spells/arcane_bolt.toml", &text);
    Arc::new(Content::load(&files).expect("the spell loads"))
}

/// A level-18 wizard whose Arcane bolt is a ray (an attack roll, 3d6, up to 8 cells).
fn ray_wizard() -> World {
    let content =
        content_with_bolt("shape = \"ray\"\nresolution = \"attack\"\nrange = 8\ndamage = \"3d6\"");
    let mut w = world_with(content, "wizard");
    reach(&mut w, 18);
    w
}

/// A level-18 wizard whose Arcane bolt is a touch (an attack roll beside the caster, 3d6).
fn touch_wizard() -> World {
    let content = content_with_bolt("shape = \"touch\"\nresolution = \"touch\"\ndamage = \"3d6\"");
    let mut w = world_with(content, "wizard");
    reach(&mut w, 18);
    w
}

fn skill(w: &World, name: &str) -> WeaponSkillId {
    w.content().weapon_skills().id(name).expect(name)
}

/// The player's ranks set to exactly `ranks` (everything else untrained).
fn ranks(w: &mut World, ranks: &[(&str, u8)]) {
    let content = w.content().clone();
    let table = content.weapon_skills();
    let ids: Vec<(WeaponSkillId, u8)> = (ranks.iter())
        .map(|&(name, rank)| (table.id(name).expect(name), rank))
        .collect();
    w.player_mut().weapon_skills = WeaponSkills::from_ranks(table, ids);
}

/// The player casts Arcane bolt at `target`, `(tx, ty)` from them, `n` times, their ranks and
/// mana put back before each cast and the target's hit points high; what each blow on it was,
/// and the most damage one did.
fn casts(w: &mut World, target: ActorId, (tx, ty): (i32, i32), n: usize) -> (Vec<Strike>, i32) {
    let here = w.player_pos();
    let skills = w.state.player().weapon_skills.clone();
    let mut strikes = Vec::new();
    let mut most = 0;
    for _ in 0..n {
        w.player_mut().weapon_skills = skills.clone();
        w.player_mut().mana = 40;
        let max = w.state.player().max_hp(w.content());
        w.player_mut().hp = max;
        let them = w.state.actors.get_mut(&target).expect("the kobold");
        them.pos = Pos::new(here.x + tx, here.y + ty);
        them.hp = 10_000;
        them.effects = Default::default();
        *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
        let at = Pos::new(here.x + tx, here.y + ty);
        let out = w.apply(&Action::Cast("arcane_bolt".into(), Some(at)));
        for e in &out.events {
            if let Event::Attacked {
                target: t,
                strike: Some(strike),
                ..
            } = e
                && *t == target
            {
                strikes.push(*strike);
            }
        }
        most = most.max(10_000 - w.state.actors[&target].hp);
    }
    (strikes, most)
}

fn hits(strikes: &[Strike]) -> usize {
    strikes.iter().filter(|s| s.is_hit()).count()
}

fn criticals(strikes: &[Strike]) -> usize {
    (strikes.iter())
        .filter(|s| matches!(s, Strike::Hit { critical: true }))
        .count()
}

#[test]
fn the_classes_that_cast_start_on_the_ladder_and_warriors_do_not() {
    let content = Arc::new(Content::builtin().expect("content"));
    // A character of `class` with their creation picks drawn at random.
    let start = |class: &str| {
        let spec: crate::character::CharacterSpec = format!("human,female,{class}")
            .parse()
            .expect("a character");
        World::start_run(
            content.clone(),
            "showcase",
            Some(7),
            Some(&spec),
            None,
            true,
        )
        .expect("world")
    };
    for (class, expect) in [
        ("wizard", [("evocation", 2), ("sniper", 1)]),
        ("witch", [("necromancy", 1), ("toucher", 1)]),
        ("priest", [("restoration", 1), ("toucher", 1)]),
        ("druid", [("evocation", 1), ("sniper", 1)]),
        ("paladin", [("abjuration", 1), ("battle_caster", 1)]),
        ("bard", [("restoration", 1), ("sniper", 0)]),
        ("ranger", [("evocation", 0), ("sniper", 0)]),
        ("warrior", [("evocation", 0), ("battle_caster", 0)]),
    ] {
        let w = start(class);
        for (name, rank) in expect {
            let id = skill(&w, name);
            assert_eq!(
                w.state.player().weapon_skills.rank(id),
                rank,
                "{class} {name}"
            );
        }
    }
}

/// The marks the player holds in `name` toward its next rank.
fn marks(w: &World, name: &str) -> u32 {
    let table = w.content().weapon_skills();
    (w.state
        .player()
        .weapon_skills
        .progress(table, skill(w, name)))
    .map_or(0, |(m, _)| m)
}

/// The marks a cast earns in a caster style at `rank`: a `success` or an attempt, against
/// `level`, with the player's talents.
fn style_earned(w: &World, success: bool, level: u32, rank: u8) -> u32 {
    let rules = w.content().weapon_skills().marks().expect("marks");
    let talents = w.state.player().talent_mods(w.content());
    talents.marks(Role::Style, rules.earned(success, level, rank))
}

fn level_of(w: &World, id: ActorId) -> u32 {
    let creature = w.state.actors[&id].creature;
    u32::from(w.content().creature(creature).level)
}

/// The player casts `name` (learnt now if need be) at the cell `(dx, dy)` from them (`None`: on
/// themselves), mana topped up; what the cast's events were.
fn cast(w: &mut World, name: &str, at: Option<(i32, i32)>) -> Vec<Event> {
    let spell = w.content().spell_id(name).expect(name);
    w.player_mut().spells.insert(spell);
    w.player_mut().mana = 60;
    let here = w.player_pos();
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let at = at.map(|(dx, dy)| Pos::new(here.x + dx, here.y + dy));
    let out = w.apply(&Action::Cast(name.into(), at));
    assert!(out.took_turn, "{name} went off: {:?}", said(w).last());
    out.events
}

/// The player in plate (armour that hinders casting).
fn in_plate(w: &mut World) {
    let plate = w.content().object_id("plate_armour").expect("plate");
    let me = w.player_mut();
    me.inventory.push(crate::object::Stack::new(plate, 1));
    let item = me.inventory.len() - 1;
    run(w, &format!("equip:body:{item}"));
    assert!(w.state.player().worn_spell_cost(w.content()) > 0);
}

/// The caster styles that hold marks.
fn trained_styles(w: &World) -> Vec<&'static str> {
    (["sniper", "toucher", "battle_caster"].into_iter())
        .filter(|s| marks(w, s) > 0)
        .collect()
}

#[test]
fn a_fireball_from_afar_trains_sniper_as_its_school_landed() {
    let mut w = ray_wizard();
    let target = kobold(&mut w, 5, 10_000);
    ranks(&mut w, &[("evocation", 1), ("sniper", 1)]);
    cast(&mut w, "fireball", Some((5, 0)));
    assert_eq!(trained_styles(&w), ["sniper"]);
    let level = level_of(&w, target);
    assert_eq!(marks(&w, "sniper"), style_earned(&w, true, level, 1));
    // A bolt at an empty cell beside the caster is from afar too (no creature beside it), and
    // at nothing it is an attempt against one's own level.
    w.state.actors.remove(&target);
    ranks(&mut w, &[("evocation", 1), ("sniper", 1)]);
    cast(&mut w, "magic_missile", Some((1, 0)));
    let own = u32::from(w.player_level());
    assert_eq!(trained_styles(&w), ["sniper"]);
    assert_eq!(marks(&w, "sniper"), style_earned(&w, false, own, 1));
}

#[test]
fn a_spell_on_oneself_out_of_armour_trains_no_style() {
    let mut w = ray_wizard();
    ranks(&mut w, &[("abjuration", 1), ("sniper", 1), ("toucher", 1)]);
    cast(&mut w, "blessing", None);
    assert!(marks(&w, "abjuration") > 0, "its school trains");
    assert!(trained_styles(&w).is_empty());
    // A heal laid on oneself is on oneself, a touch or not.
    w.player_mut().hp = 1;
    cast(&mut w, "mend", Some((0, 0)));
    assert!(marks(&w, "restoration") > 0, "its school trains");
    assert!(trained_styles(&w).is_empty());
}

#[test]
fn a_heal_and_a_ray_at_the_creature_beside_train_toucher() {
    let mut w = ray_wizard();
    let target = kobold(&mut w, 1, 10_000);
    w.state.actors.get_mut(&target).expect("there").hp = 1;
    ranks(&mut w, &[("restoration", 1)]);
    cast(&mut w, "cure_wounds", Some((1, 0)));
    assert_eq!(trained_styles(&w), ["toucher"]);
    let level = level_of(&w, target);
    assert_eq!(marks(&w, "toucher"), style_earned(&w, true, level, 0));
    // The ray (no touch) at the kobold next to the caster: a Toucher's too.
    ranks(&mut w, &[("evocation", 1)]);
    let (strikes, _) = casts(&mut w, target, (1, 0), 1);
    assert_eq!(strikes.len(), 1);
    assert_eq!(trained_styles(&w), ["toucher"]);
    let content = w.content().clone();
    let me = w.state.player();
    let ray = content.spell(sp("ray_of_frost"));
    let beside = me.spell_skills(&content, ray, SpellAim::Beside);
    assert_eq!(beside.style, Some(skill(&w, "toucher")));
    assert_eq!(beside.skill, Some(skill(&w, "evocation")));
}

#[test]
fn in_hindering_armour_every_spell_trains_battle_caster_with_full_marks() {
    let mut w = ray_wizard();
    in_plate(&mut w);
    let own = u32::from(w.player_level());
    // A buff on oneself: a success against one's own level, as its school's.
    ranks(&mut w, &[("battle_caster", 1)]);
    cast(&mut w, "blessing", None);
    assert_eq!(trained_styles(&w), ["battle_caster"]);
    assert_eq!(marks(&w, "battle_caster"), style_earned(&w, true, own, 1));
    // A spell that never rolls to hit, from afar: a success against the one it struck.
    let target = kobold(&mut w, 3, 10_000);
    ranks(&mut w, &[("battle_caster", 1)]);
    cast(&mut w, "magic_missile", Some((3, 0)));
    assert_eq!(trained_styles(&w), ["battle_caster"]);
    let level = level_of(&w, target);
    assert_eq!(marks(&w, "battle_caster"), style_earned(&w, true, level, 1));
    // And at the one beside: Battle caster still, wherever it is aimed.
    ranks(&mut w, &[("battle_caster", 1)]);
    let here = w.player_pos();
    w.state.actors.get_mut(&target).expect("there").pos = Pos::new(here.x + 1, here.y);
    cast(&mut w, "magic_missile", Some((1, 0)));
    assert_eq!(trained_styles(&w), ["battle_caster"]);
}

#[test]
fn snipers_reach_a_cell_farther_from_rank_five() {
    let mut w = ray_wizard();
    let content = w.content().clone();
    let missile = content.spell(sp("magic_missile"));
    let range = |w: &mut World, rank: u8| {
        ranks(w, &[("sniper", rank)]);
        (w.state.player().spell_reach(&content, missile).range).expect("a range")
    };
    let base = range(&mut w, 0);
    assert_eq!(range(&mut w, 4), base);
    assert_eq!(range(&mut w, 5), base + 1);
    assert_eq!(range(&mut w, 9), base + 2);
    // The cast reaches it: a cell one past the base range is refused at rank 4 and goes off at
    // rank 5 (an open row of floor east of the showcase's start).
    let at = Some(Pos::new(w.player_pos().x + base + 1, w.player_pos().y));
    w.player_mut().spells.insert(sp("magic_missile"));
    w.player_mut().mana = 60;
    ranks(&mut w, &[("sniper", 4)]);
    assert!(!w.apply(&Action::Cast("magic_missile".into(), at)).took_turn);
    ranks(&mut w, &[("sniper", 5)]);
    assert!(
        w.apply(&Action::Cast("magic_missile".into(), at)).took_turn,
        "{:?}",
        said(&w).last()
    );
    // In armour that hinders casting a spell is a Battle caster's: no Sniper's cell.
    in_plate(&mut w);
    assert_eq!(range(&mut w, 10), base);
}

#[test]
fn touchers_bonus_reaches_the_roll_from_rank_four_up_close_only() {
    let content =
        content_with_bolt("shape = \"bolt\"\nresolution = \"auto\"\nrange = 8\ndamage = \"+6\"");
    let mut w = world_with(content, "wizard");
    reach(&mut w, 18);
    let target = kobold(&mut w, 1, 10_000);
    // What one cast at the kobold `dx` cells east does with Toucher at `rank`.
    let harm = |w: &mut World, dx: i32, rank: u8| {
        ranks(w, &[("toucher", rank)]);
        casts(w, target, (dx, 0), 1).1
    };
    let bare = harm(&mut w, 1, 3);
    assert!(bare > 0);
    assert_eq!(harm(&mut w, 1, 0), bare);
    assert_eq!(harm(&mut w, 1, 4), bare + 1);
    assert_eq!(harm(&mut w, 1, 10), bare + 3);
    // From afar it is a Sniper's cast: no Toucher's bonus.
    assert_eq!(harm(&mut w, 3, 10), bare);
    // A touch spell's roll as the menu shows it counts it (a touch is cast beside).
    let cure = w.content().clone();
    let cure = cure.spell(sp("cure_wounds"));
    let shown = |w: &mut World, rank: u8| {
        ranks(w, &[("toucher", rank)]);
        let roll = w.state.player().spell_roll(w.content(), cure, CastBy::Own);
        roll.expect("a heal").to_string()
    };
    let (three, four) = (shown(&mut w, 3), shown(&mut w, 4));
    assert_ne!(three, four);
}

#[test]
fn battle_caster_is_marked_in_use_in_hindering_armour_only() {
    let mut w = ray_wizard();
    let battle = skill(&w, "battle_caster");
    let casters = [skill(&w, "sniper"), skill(&w, "toucher")];
    let in_use = |w: &World| w.perception().own_styles_in_use();
    assert!(!in_use(&w).contains(&battle));
    assert!(casters.iter().all(|s| !in_use(&w).contains(s)));
    in_plate(&mut w);
    assert!(in_use(&w).contains(&battle));
    assert!(casters.iter().all(|s| !in_use(&w).contains(s)));
}

#[test]
fn ranks_in_the_school_raise_the_hit_chance_and_the_critical_range() {
    let mut w = ray_wizard();
    let target = kobold(&mut w, 3, 10_000);
    // A hard target, so the to-hit shows.
    w.state.actors.get_mut(&target).unwrap().stats = crate::stats::Stats::new(
        crate::stats::PerStat::splat(10),
        crate::stats::PerStat::splat(10),
    );
    ranks(&mut w, &[]);
    let (untrained, _) = casts(&mut w, target, (3, 0), 400);
    ranks(&mut w, &[("evocation", 10)]);
    let (trained, _) = casts(&mut w, target, (3, 0), 400);
    assert_eq!((untrained.len(), trained.len()), (400, 400));
    // +6 to hit: about a quarter of the 2d12 range more of the rolls land.
    assert!(
        hits(&trained) > hits(&untrained) + 40,
        "{} against {}",
        hits(&trained),
        hits(&untrained)
    );
    // A threat range of 19-24 rather than 22-24: many more criticals.
    assert!(
        criticals(&trained) > criticals(&untrained) * 2,
        "{} against {}",
        criticals(&trained),
        criticals(&untrained)
    );
}

#[test]
fn sniper_aims_better_at_range_five_and_more() {
    let mut w = ray_wizard();
    let target = kobold(&mut w, 2, 10_000);
    ranks(&mut w, &[("sniper", 10)]);
    let (near, _) = casts(&mut w, target, (4, 0), 400);
    let (far, _) = casts(&mut w, target, (6, 0), 400);
    // +3 to hit and a critical range 2 wider from five cells.
    assert!(
        hits(&far) > hits(&near),
        "{} against {}",
        hits(&far),
        hits(&near)
    );
    assert!(criticals(&far) > criticals(&near), "{far:?}");
}

#[test]
fn a_touch_critical_is_said_staggers_and_never_passes_the_cap() {
    let mut w = touch_wizard();
    let target = kobold(&mut w, 1, 10_000);
    ranks(&mut w, &[("evocation", 10), ("toucher", 9)]);
    let staggered = w.content().status_id("staggered").expect("a status");
    let (mut seen_critical, mut seen_staggered) = (false, false);
    let mut most = 0;
    for _ in 0..200 {
        let (strikes, hurt) = casts(&mut w, target, (1, 0), 1);
        most = most.max(hurt);
        if strikes == [Strike::Hit { critical: true }] {
            seen_critical = true;
            let lines: Vec<String> = said(&w).iter().rev().take(6).cloned().collect();
            assert!(
                lines.iter().any(|l| l.contains("the kobold hard!")),
                "a critical says so: {lines:?}"
            );
            // Staggering touch (Toucher 4): the critical leaves the kobold staggered.
            if w.state.actors[&target].effects.has(staggered) {
                seen_staggered = true;
            }
        }
    }
    assert!(seen_critical && seen_staggered);
    // A critical (x5 at these ranks) is held by nothing (decision 16): past the old touch
    // cap of 21.
    assert!(most > 21, "{most}");
}

#[test]
fn withering_touch_weakens_at_toucher_ten() {
    let mut w = touch_wizard();
    let target = kobold(&mut w, 1, 10_000);
    let weakened = w.content().status_id("weakened").expect("a status");
    let weakens = |w: &mut World, rank: u8| {
        ranks(w, &[("evocation", 10), ("toucher", rank)]);
        (0..30).any(|_| {
            let (strikes, _) = casts(w, target, (1, 0), 1);
            hits(&strikes) == 1 && w.state.actors[&target].effects.has(weakened)
        })
    };
    assert!(!weakens(&mut w, 9));
    assert!(weakens(&mut w, 10));
}

#[test]
fn in_armour_battle_caster_is_the_style_and_eases_the_armours_cost() {
    let mut w = ray_wizard();
    let content = w.content().clone();
    let me = w.state.player();
    let spell = |id: &str| content.spell(content.spell_id(id).expect(id));
    let (ray, touch) = (spell("ray_of_frost"), spell("cause_wounds"));
    let standing =
        |me: &Actor, spell| me.spell_skills(&content, spell, me.standing_aim(&content, spell));
    let free = standing(me, ray);
    assert_eq!(free.style, Some(skill(&w, "sniper")));
    assert_eq!(standing(me, touch).style, Some(skill(&w, "toucher")));
    assert_eq!(free.skill, Some(skill(&w, "evocation")));
    assert_eq!(standing(me, touch).skill, Some(skill(&w, "necromancy")));
    let blessing = spell("blessing");
    assert_eq!(standing(me, blessing).style, None, "on oneself: no style");
    let plate = content.object_id("plate_armour").expect("plate");
    let me = w.player_mut();
    me.inventory.push(crate::object::Stack::new(plate, 1));
    let item = me.inventory.len() - 1;
    run(&mut w, &format!("equip:body:{item}"));
    let me = w.state.player();
    assert_eq!(standing(me, ray).style, Some(skill(&w, "battle_caster")));
    assert_eq!(
        standing(me, blessing).style,
        Some(skill(&w, "battle_caster"))
    );
    assert_eq!(me.armour_spell_cost(&content), 40);
    ranks(&mut w, &[("battle_caster", 10)]);
    assert_eq!(w.state.player().worn_spell_cost(&content), 40);
    assert_eq!(w.state.player().armour_spell_cost(&content), 20, "half off");
}

#[test]
fn spell_guard_wards_a_battle_caster_whose_spell_lands() {
    let mut w = ray_wizard();
    let content = w.content().clone();
    let plate = content.object_id("plate_armour").expect("plate");
    let me = w.player_mut();
    me.inventory.push(crate::object::Stack::new(plate, 1));
    let item = me.inventory.len() - 1;
    run(&mut w, &format!("equip:body:{item}"));
    let target = kobold(&mut w, 3, 10_000);
    let ward = content.status_id("spell_ward").expect("a status");
    let defence = |w: &World| w.defence_of(w.state.player).total();
    ranks(&mut w, &[("evocation", 10), ("battle_caster", 4)]);
    let bare = defence(&w);
    let warded = (0..20).any(|_| {
        let (strikes, _) = casts(&mut w, target, (3, 0), 1);
        hits(&strikes) == 1 && w.state.player().effects.has(ward)
    });
    assert!(warded, "a spell that lands wards its caster");
    assert_eq!(defence(&w), bare + 1, "a Defence point");
    // Gone with the caster's next action.
    run(&mut w, "wait");
    assert!(!w.state.player().effects.has(ward));
}

#[test]
fn a_cast_sequence_replays_exactly() {
    let mut w = ray_wizard();
    kobold(&mut w, 3, 10_000);
    let here = w.player_pos();
    let at = format!("{},{}", here.x + 3, here.y);
    let script = format!("use:arcane_bolt:{at} use:arcane_bolt:{at} use:arcane_bolt:{at}");
    let mut again = w.clone();
    run(&mut w, &script);
    run(&mut again, &script);
    assert_eq!(w.state_hash(), again.state_hash());
    assert!(w.state.player().weapon_skills.rank(skill(&w, "evocation")) >= 1);
}
