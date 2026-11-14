//! The caster's ladder (spec §8 "Weapon skills", "Fighting styles"; #571, #1251): the caster
//! style a spell is cast in (Sniper for a ray or a bolt that rolls to hit, Toucher for a touch
//! that does, Battle caster in hindering armour, none for the rest) and the marks every spell
//! paid in mana earns in it, Sniper's cells and Toucher's bonus, a touch's fixed reach, what the
//! ranks do to a rolling spell's roll, the critical held by nothing, Battle caster's armour and
//! Spell guard, the style marked in use, and the classes' starting ranks.

use super::caster_tests::{kobold, reach, run, said, sp, world_with};
use super::*;
use crate::action::Event;
use crate::combat::Strike;
use crate::content::{Role, WeaponSkillId};
use crate::weapon_skills::WeaponSkills;

/// The built-in content with the wizard's Arcane bolt (level 6) as `body` says: its shape,
/// resolution, and dice (as `cast_tests` tries the spells the casters do not have).
fn content_with_bolt(body: &str) -> Arc<Content> {
    let mut files = super::caster_tests::files();
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
    w.player_mut().weapon_skills = WeaponSkills::from_ranks(ids);
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
    let content = Arc::new(super::caster_tests::quiet());
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
        ("wizard", [("sniper", 1)]),
        ("witch", [("toucher", 1)]),
        ("priest", [("toucher", 1)]),
        ("druid", [("sniper", 1)]),
        ("paladin", [("battle_caster", 1)]),
        ("bard", [("sniper", 0)]),
        ("ranger", [("sniper", 0)]),
        ("warrior", [("battle_caster", 0)]),
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
    talents
        .offence
        .marks(Role::Style, rules.earned(success, level, rank))
}

fn level_of(w: &World, id: ActorId) -> u32 {
    let creature = w.state.actors[&id].creature;
    u32::from(w.content().creature(creature).level)
}

/// The player casts `name` (learnt now if need be) at the cell `(dx, dy)` from them (`None`: on
/// themselves), mana topped up; what the cast's events were.
fn cast(w: &mut World, name: &str, at: Option<(i32, i32)>) -> Vec<Event> {
    let spell = w.content().spell_id(name).expect(name);
    w.player_mut().spells.insert(spell, 1);
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
fn a_ray_trains_sniper_wherever_it_is_aimed_and_an_area_no_style() {
    let mut w = ray_wizard();
    let target = kobold(&mut w, 5, 10_000);
    // A burst rolls no attack: no style trains.
    ranks(&mut w, &[("sniper", 1)]);
    cast(&mut w, "fireball", Some((5, 0)));
    assert!(trained_styles(&w).is_empty());
    // Nor does a bolt that never misses.
    cast(&mut w, "magic_missile", Some((5, 0)));
    assert!(trained_styles(&w).is_empty());
    // The ray at the kobold beside the caster: a Sniper's all the same, landed or not.
    ranks(&mut w, &[("sniper", 1)]);
    let (strikes, _) = casts(&mut w, target, (1, 0), 1);
    assert_eq!(strikes.len(), 1);
    assert_eq!(trained_styles(&w), ["sniper"]);
    let level = level_of(&w, target);
    let landed = strikes[0].is_hit();
    assert_eq!(marks(&w, "sniper"), style_earned(&w, landed, level, 1));
    // At an empty cell it is an attempt against one's own level.
    w.state.actors.remove(&target);
    ranks(&mut w, &[("sniper", 1)]);
    cast(&mut w, "arcane_bolt", Some((3, 0)));
    let own = u32::from(w.player_level());
    assert_eq!(trained_styles(&w), ["sniper"]);
    assert_eq!(marks(&w, "sniper"), style_earned(&w, false, own, 1));
}

#[test]
fn a_spell_on_oneself_out_of_armour_trains_no_style() {
    let mut w = ray_wizard();
    ranks(&mut w, &[("sniper", 1), ("toucher", 1)]);
    cast(&mut w, "blessing", None);
    assert!(trained_styles(&w).is_empty());
    // A heal laid on oneself is on oneself, a touch or not.
    w.player_mut().hp = 1;
    cast(&mut w, "mend", Some((0, 0)));
    assert!(trained_styles(&w).is_empty());
}

#[test]
fn a_touch_that_rolls_to_hit_trains_toucher_and_a_touch_heal_no_style() {
    let mut w = touch_wizard();
    let target = kobold(&mut w, 1, 10_000);
    // A heal laid on the creature beside: a touch, but no attack roll.
    w.state.actors.get_mut(&target).expect("there").hp = 1;
    ranks(&mut w, &[]);
    cast(&mut w, "cure_wounds", Some((1, 0)));
    assert!(trained_styles(&w).is_empty());
    // The touch that rolls to hit: a Toucher's, landed or not.
    ranks(&mut w, &[]);
    let (strikes, _) = casts(&mut w, target, (1, 0), 1);
    assert_eq!(strikes.len(), 1);
    assert_eq!(trained_styles(&w), ["toucher"]);
    let level = level_of(&w, target);
    let landed = strikes[0].is_hit();
    assert_eq!(marks(&w, "toucher"), style_earned(&w, landed, level, 0));
    let content = w.content().clone();
    let me = w.state.player();
    let touch = me.spell_skills(&content, content.spell(sp("arcane_bolt")));
    assert_eq!(touch.style, Some(skill(&w, "toucher")));
    assert_eq!(touch.skill, None, "no school's skill");
}

#[test]
fn in_hindering_armour_every_spell_trains_battle_caster_with_full_marks() {
    let mut w = ray_wizard();
    in_plate(&mut w);
    let own = u32::from(w.player_level());
    // A buff on oneself: a success against one's own level.
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
    let ray = content.spell(sp("arcane_bolt"));
    let missile = content.spell(sp("magic_missile"));
    let range = |w: &mut World, spell, rank: u8| {
        ranks(w, &[("sniper", rank)]);
        (w.state.player().spell_reach(&content, spell).range).expect("a range")
    };
    let base = range(&mut w, ray, 0);
    assert_eq!(range(&mut w, ray, 4), base);
    assert_eq!(range(&mut w, ray, 5), base + 1);
    assert_eq!(range(&mut w, ray, 9), base + 2);
    // A bolt that never misses is no Sniper's: no cell.
    let bare = range(&mut w, missile, 0);
    assert_eq!(range(&mut w, missile, 10), bare);
    // The cast reaches it: a cell one past the base range is refused at rank 4 and goes off at
    // rank 5 (an open row of floor east of the showcase's start).
    let at = Some(Pos::new(w.player_pos().x + base + 1, w.player_pos().y));
    w.player_mut().spells.insert(sp("arcane_bolt"), 1);
    w.player_mut().mana = 60;
    ranks(&mut w, &[("sniper", 4)]);
    assert!(!w.apply(&Action::Cast("arcane_bolt".into(), at)).took_turn);
    ranks(&mut w, &[("sniper", 5)]);
    assert!(
        w.apply(&Action::Cast("arcane_bolt".into(), at)).took_turn,
        "{:?}",
        said(&w).last()
    );
    // In armour that hinders casting a spell is a Battle caster's: no Sniper's cell.
    in_plate(&mut w);
    assert_eq!(range(&mut w, ray, 10), base);
}

#[test]
fn a_touch_reaches_the_cell_beside_and_no_farther() {
    let mut w = touch_wizard();
    let content = w.content().clone();
    for school in ["evocation", "necromancy", "restoration"] {
        w.player_mut().talents.take("school_focus", Some(school));
    }
    ranks(&mut w, &[("toucher", 10), ("sniper", 10)]);
    let me = w.state.player();
    for id in [
        "arcane_bolt",
        "cause_wounds",
        "grave_feast",
        "cure_wounds",
        "mend",
    ] {
        let reach = me.spell_reach(&content, content.spell(sp(id)));
        assert_eq!((reach.range, reach.radius), (Some(1), None), "{id}");
    }
}

#[test]
fn touchers_bonus_reaches_a_touch_that_rolls_to_hit_from_rank_four() {
    let mut w = touch_wizard();
    let content = w.content().clone();
    // The roll of `id` as the menu shows it, with Toucher at `rank`.
    let shown = |w: &mut World, id: &str, rank: u8| {
        ranks(w, &[("toucher", rank)]);
        let spell = content.spell(sp(id));
        let roll = w.state.player().spell_roll(&content, spell, CastBy::Own);
        roll.expect("a roll").to_string()
    };
    let bare = shown(&mut w, "arcane_bolt", 0);
    assert_eq!(shown(&mut w, "arcane_bolt", 3), bare);
    assert_ne!(shown(&mut w, "arcane_bolt", 4), bare);
    // A touch heal rolls no attack: no Toucher's point.
    assert_eq!(
        shown(&mut w, "cure_wounds", 10),
        shown(&mut w, "cure_wounds", 0)
    );
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
fn toucher_ranks_raise_the_hit_chance_and_the_critical_range() {
    let mut w = touch_wizard();
    let target = kobold(&mut w, 1, 10_000);
    // A hard target, so the to-hit shows.
    w.state.actors.get_mut(&target).unwrap().stats = crate::stats::Stats::new(
        crate::stats::PerStat::splat(10),
        crate::stats::PerStat::splat(10),
    );
    ranks(&mut w, &[]);
    let (untrained, _) = casts(&mut w, target, (1, 0), 400);
    ranks(&mut w, &[("toucher", 10)]);
    let (trained, _) = casts(&mut w, target, (1, 0), 400);
    assert_eq!((untrained.len(), trained.len()), (400, 400));
    // +5 to hit: about a fifth of the 2d12 range more of the rolls land.
    assert!(
        hits(&trained) > hits(&untrained) + 40,
        "{} against {}",
        hits(&trained),
        hits(&untrained)
    );
    // A threat range of 20-24 rather than 22-24: many more criticals.
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
fn a_touch_critical_is_said_and_never_passes_the_cap() {
    let mut w = touch_wizard();
    let target = kobold(&mut w, 1, 10_000);
    ranks(&mut w, &[("toucher", 10)]);
    let mut seen_critical = false;
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
        }
    }
    assert!(seen_critical);
    // A critical (x4 at Toucher 10) is held by nothing (decision 16): past the old touch
    // cap of 21.
    assert!(most > 21, "{most}");
}

#[test]
fn sniper_and_toucher_teach_no_ability() {
    let content = super::caster_tests::quiet();
    let table = content.weapon_skills();
    let styles = [table.id("sniper"), table.id("toucher")];
    assert!(styles.iter().all(Option::is_some));
    assert!(
        (content.weapon_abilities().iter())
            .all(|a| !a.skills.iter().any(|&s| styles.contains(&Some(s)))),
        "of the caster styles only Battle caster teaches one (Spell guard)"
    );
}

#[test]
fn in_armour_battle_caster_is_the_style_and_eases_the_armours_cost() {
    let mut w = ray_wizard();
    let content = w.content().clone();
    let me = w.state.player();
    let spell = |id: &str| content.spell(content.spell_id(id).expect(id));
    let (ray, touch) = (spell("ray_of_frost"), spell("cause_wounds"));
    let standing = |me: &Actor, spell| me.spell_skills(&content, spell);
    let free = standing(me, ray);
    assert_eq!(free.style, Some(skill(&w, "sniper")));
    assert_eq!(standing(me, touch).style, Some(skill(&w, "toucher")));
    assert_eq!((free.skill, standing(me, touch).skill), (None, None));
    let blessing = spell("blessing");
    assert_eq!(
        standing(me, blessing).style,
        None,
        "no attack roll: no style"
    );
    for id in ["fireball", "magic_missile", "cure_wounds"] {
        assert_eq!(standing(me, spell(id)).style, None, "{id}");
    }
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
    ranks(&mut w, &[("battle_caster", 4)]);
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
    assert!(marks(&w, "sniper") > 0);
}
