//! The caster's ladder (spec §8 "Weapon skills", "Fighting styles"; #571): the spell attack
//! skill and the caster styles a spell that rolls to hit draws on and trains, what their ranks
//! do to the roll, the critical held to the cap, the caster styles' abilities, Battle caster's
//! armour, and the classes' starting ranks.

use super::caster_tests::{kobold, reach, run, said, world_with};
use super::*;
use crate::action::Event;
use crate::combat::Strike;
use crate::content::{ContentFiles, WeaponSkillId};
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
        ("wizard", [("spell_attack", 2), ("sniper", 1)]),
        ("witch", [("spell_attack", 1), ("toucher", 1)]),
        ("priest", [("spell_attack", 1), ("toucher", 1)]),
        ("druid", [("spell_attack", 1), ("sniper", 1)]),
        ("paladin", [("spell_attack", 1), ("battle_caster", 1)]),
        ("bard", [("spell_attack", 1), ("sniper", 0)]),
        ("warrior", [("spell_attack", 0), ("battle_caster", 0)]),
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

#[test]
fn a_spell_that_rolls_to_hit_earns_marks_and_one_that_does_not_earns_none() {
    let mut w = ray_wizard();
    ranks(&mut w, &[("spell_attack", 1), ("sniper", 1)]);
    let target = kobold(&mut w, 3, 10_000);
    let (spell, sniper) = (skill(&w, "spell_attack"), skill(&w, "sniper"));
    let table = w.content().weapon_skills().clone();
    let marks = |w: &World, id| (w.state.player().weapon_skills.progress(&table, id)).map(|p| p.0);
    let (strikes, _) = casts(&mut w, target, (3, 0), 1);
    assert_eq!(strikes.len(), 1);
    // `casts` puts the ranks back before each cast, not after the last.
    assert!(marks(&w, spell) > Some(0), "the spell attack skill trained");
    assert!(marks(&w, sniper) > Some(0), "the style trained");
    for other in ["toucher", "battle_caster", "staves", "duelist"] {
        let id = skill(&w, other);
        assert_eq!(w.state.player().weapon_skills.rank(id), 0, "{other}");
        assert_eq!(marks(&w, id), Some(0), "{other}");
    }
    // Magic missile never misses: no roll, no marks.
    ranks(&mut w, &[("spell_attack", 1), ("sniper", 1)]);
    let missile = w.content().spell_id("magic_missile").expect("a spell");
    let here = w.player_pos();
    w.player_mut().mana = 40;
    let out = w.apply(&Action::Cast(
        w.content().spell(missile).id.clone(),
        Some(Pos::new(here.x + 3, here.y)),
    ));
    assert!(
        (out.events.iter()).any(|e| matches!(e, Event::Attacked { target: t, .. } if *t == target))
    );
    assert_eq!(marks(&w, spell), Some(0));
    assert_eq!(marks(&w, sniper), Some(0));
}

#[test]
fn ranks_in_the_skill_raise_the_hit_chance_and_the_critical_range() {
    let mut w = ray_wizard();
    let target = kobold(&mut w, 3, 10_000);
    // A hard target, so the to-hit shows.
    w.state.actors.get_mut(&target).unwrap().stats = crate::stats::Stats::new(
        crate::stats::PerStat::splat(10),
        crate::stats::PerStat::splat(10),
    );
    ranks(&mut w, &[]);
    let (untrained, _) = casts(&mut w, target, (3, 0), 400);
    ranks(&mut w, &[("spell_attack", 10)]);
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
    ranks(&mut w, &[("spell_attack", 10), ("toucher", 9)]);
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
        ranks(w, &[("spell_attack", 10), ("toucher", rank)]);
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
    let free = me.spell_skills(&content, false);
    assert_eq!(free.style, Some(skill(&w, "sniper")));
    assert_eq!(
        me.spell_skills(&content, true).style,
        Some(skill(&w, "toucher"))
    );
    assert_eq!(free.skill, Some(skill(&w, "spell_attack")));
    let plate = content.object_id("plate_armour").expect("plate");
    let me = w.player_mut();
    me.inventory.push(crate::object::Stack::new(plate, 1));
    let item = me.inventory.len() - 1;
    run(&mut w, &format!("equip:body:{item}"));
    let me = w.state.player();
    assert_eq!(
        me.spell_skills(&content, false).style,
        Some(skill(&w, "battle_caster"))
    );
    assert_eq!(me.armour_spell_cost(&content), 40);
    ranks(&mut w, &[("battle_caster", 10)]);
    assert_eq!(w.state.player().worn_spell_cost(&content), 40);
    assert_eq!(w.state.player().armour_spell_cost(&content), 20, "half off");
    // A spell that never rolls to hit, cast in armour, trains Battle caster alone.
    let table = content.weapon_skills();
    let battle = skill(&w, "battle_caster");
    ranks(&mut w, &[("battle_caster", 1), ("spell_attack", 1)]);
    let here = w.player_pos();
    w.player_mut().mana = 40;
    w.apply(&Action::Cast(
        "magic_missile".into(),
        Some(Pos::new(here.x + 3, here.y)),
    ));
    let progress = |w: &World, id| w.state.player().weapon_skills.progress(table, id);
    assert!(progress(&w, battle).is_some_and(|p| p.0 > 0));
    assert_eq!(
        progress(&w, skill(&w, "spell_attack")).map(|p| p.0),
        Some(0)
    );
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
    ranks(&mut w, &[("spell_attack", 10), ("battle_caster", 4)]);
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
    assert!(
        w.state
            .player()
            .weapon_skills
            .rank(skill(&w, "spell_attack"))
            >= 1
    );
}
