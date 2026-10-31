//! The schools of magic (spec §8 "Weapon skills": "Schools of magic"; the spell scaling design
//! §8): marks by casting, every kind of cast (a harm landed, a hex taken, a heal, a buff, a cast
//! at nothing, a miss), the strongest struck weighing an area, scrolls and wands training
//! nothing; what a rank does to the roll, the mana, and the cast's time; the defaults; Schooled,
//! Devotee, and Magister; the dumps and the dev rank.

use super::caster_tests::{kobold, kobold_at, reach, run, said, sp, world_with};
use super::*;
use crate::action::{Event, parse_actions};
use crate::combat::Strike;
use crate::content::{ContentFiles, Role, WeaponSkillId};
use crate::dump::{self, Layer};
use crate::stats::{PerStat, Stats};
use crate::weapon_skills::WeaponSkills;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A level-18 wizard of the built-in content.
fn wizard() -> World {
    let mut w = world_with(BUILTIN.clone(), "wizard");
    reach(&mut w, 18);
    w
}

fn skill(w: &World, name: &str) -> WeaponSkillId {
    w.content().weapon_skills().id(name).expect(name)
}

/// The player's ranks set to exactly `ranks` (every other skill untrained, no marks).
fn ranks(w: &mut World, ranks: &[(&str, u8)]) {
    let content = w.content().clone();
    let table = content.weapon_skills();
    let ids: Vec<(WeaponSkillId, u8)> = (ranks.iter())
        .map(|&(name, rank)| (table.id(name).expect(name), rank))
        .collect();
    w.player_mut().weapon_skills = WeaponSkills::from_ranks(table, ids);
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

/// The marks a cast earns in a school at `rank`: a `success` or an attempt, against `level`,
/// with the player's talents.
fn earned(w: &World, success: bool, level: u32, rank: u8) -> u32 {
    let rules = w.content().weapon_skills().marks().expect("marks");
    let talents = w.state.player().talent_mods(w.content());
    talents.marks(Role::School, rules.earned(success, level, rank))
}

fn level_of(w: &World, id: ActorId) -> u32 {
    let creature = w.state.actors[&id].creature;
    u32::from(w.content().creature(creature).level)
}

/// The player casts `name` at the cell `(dx, dy)` from them (`None`: on themselves), mana and
/// hit points topped up; what the cast's events were.
fn cast(w: &mut World, name: &str, at: Option<(i32, i32)>) -> Vec<Event> {
    w.player_mut().spells.insert(sp(name));
    w.player_mut().mana = 60;
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = w.state.player().hp.max(max / 2);
    let here = w.player_pos();
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let at = at.map(|(dx, dy)| Pos::new(here.x + dx, here.y + dy));
    let out = w.apply(&Action::Cast(name.into(), at));
    assert!(out.took_turn, "{name} went off: {:?}", said(w).last());
    out.events
}

#[test]
fn a_harm_landed_earns_a_success_against_the_one_struck() {
    let mut w = wizard();
    ranks(&mut w, &[("evocation", 1)]);
    let target = kobold(&mut w, 3, 10_000);
    cast(&mut w, "magic_missile", Some((3, 0)));
    let want = earned(&w, true, level_of(&w, target), 1);
    assert!(want > 0);
    assert_eq!(marks(&w, "evocation"), want);
    // The other schools learn nothing of it.
    for other in ["necromancy", "transmutation", "divination"] {
        assert_eq!(marks(&w, other), 0, "{other}");
    }
}

#[test]
fn a_cast_at_nothing_earns_an_attempt_against_ones_own_level() {
    let mut w = wizard();
    ranks(&mut w, &[("evocation", 1)]);
    cast(&mut w, "magic_missile", Some((3, 0)));
    let own = u32::from(w.player_level());
    assert_eq!(marks(&w, "evocation"), earned(&w, false, own, 1));
}

#[test]
fn a_miss_is_an_attempt_against_the_one_missed() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/spells/arcane_bolt.toml",
        "[[spell]]\nid = \"arcane_bolt\"\nname = \"Arcane bolt\"\ndescription = \"x\"\n\
         level = 6\nschool = \"evocation\"\nkind = \"damage\"\ndamage_type = \"force\"\n\
         shape = \"ray\"\nresolution = \"attack\"\nrange = 8\ndamage = \"3d6\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("the spell loads"));
    let mut w = world_with(content, "wizard");
    reach(&mut w, 18);
    let target = kobold(&mut w, 3, 10_000);
    // A nimble target, so the ray misses now and then.
    w.state.actors.get_mut(&target).expect("the kobold").stats =
        Stats::new(PerStat::splat(30), PerStat::splat(30));
    let level = level_of(&w, target);
    let (mut missed, mut hit) = (false, false);
    for _ in 0..200 {
        ranks(&mut w, &[("evocation", 1)]);
        let events = cast(&mut w, "arcane_bolt", Some((3, 0)));
        let strike = (events.iter()).find_map(|e| match e {
            Event::Attacked {
                target: t,
                strike: Some(s),
                ..
            } if *t == target => Some(*s),
            _ => None,
        });
        match strike {
            Some(Strike::Missed { .. }) => {
                missed = true;
                assert_eq!(marks(&w, "evocation"), earned(&w, false, level, 1));
            }
            Some(_) => {
                hit = true;
                assert_eq!(marks(&w, "evocation"), earned(&w, true, level, 1));
            }
            None => panic!("the ray struck at the kobold"),
        }
        if missed && hit {
            break;
        }
    }
    assert!(missed && hit);
}

#[test]
fn an_area_is_one_practise_weighed_by_the_strongest_struck() {
    let mut w = wizard();
    let weak = kobold_at(&mut w, (5, 0), 10_000);
    let strong = kobold_at(&mut w, (5, 1), 10_000);
    let kobold_level = level_of(&w, weak);
    let content = w.content().clone();
    let stronger = (content.creature_defs().iter())
        .position(|c| u32::from(c.level) >= kobold_level + 3)
        .map(|i| crate::content::CreatureId(i as u16))
        .expect("a stronger creature");
    w.state.actors.get_mut(&strong).expect("there").creature = stronger;
    let top = level_of(&w, strong);
    ranks(&mut w, &[("evocation", 1)]);
    let events = cast(&mut w, "fireball", Some((5, 0)));
    let struck =
        |id| (events.iter()).any(|e| matches!(e, Event::Attacked { target, .. } if *target == id));
    assert!(struck(weak) && struck(strong));
    assert_eq!(marks(&w, "evocation"), earned(&w, true, top, 1));
}

#[test]
fn a_hex_taken_a_heal_given_and_a_buff_done_are_successes() {
    let mut w = wizard();
    let own = u32::from(w.player_level());
    // A hex that took on the kobold: necromancy, against the kobold.
    let target = kobold(&mut w, 3, 10_000);
    ranks(&mut w, &[("necromancy", 1)]);
    cast(&mut w, "hex_of_weakness", Some((3, 0)));
    let weakened = w.content().status_id("weakened").expect("a status");
    assert!(w.state.actors[&target].effects.has(weakened));
    assert_eq!(
        marks(&w, "necromancy"),
        earned(&w, true, level_of(&w, target), 1)
    );
    // A heal on oneself, hurt: restoration, against one's own level.
    ranks(&mut w, &[("restoration", 1)]);
    w.player_mut().hp = 1;
    cast(&mut w, "mend", Some((0, 0)));
    assert_eq!(marks(&w, "restoration"), earned(&w, true, own, 1));
    // A buff: abjuration.
    ranks(&mut w, &[("abjuration", 1)]);
    cast(&mut w, "blessing", None);
    assert_eq!(marks(&w, "abjuration"), earned(&w, true, own, 1));
}

#[test]
fn a_scroll_read_and_a_wand_used_train_nothing() {
    const HALL: &str =
        "#############\n#...........#\n#..@.r......#\n#...........#\n#############\n";
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{HALL}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\n\
         carry = [\"wand_of_magic_missile\", \"scroll_of_frost\"]\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "hall", Some(3)).expect("world");
    let rat = (w.state.actors.values())
        .find(|a| a.id != w.state.player)
        .map(|a| a.id)
        .expect("the rat");
    w.state.actors.get_mut(&rat).expect("the rat").hp = 1000;
    let at = w.state.actors[&rat].pos;
    let before = w.state.player().weapon_skills.clone();
    let mut took = Vec::new();
    for action in parse_actions(&format!(
        "equip:missile_weapon:a fire:{},{} read:b:{},{}",
        at.x, at.y, at.x, at.y
    ))
    .expect("actions")
    {
        took.push(w.apply(&action).took_turn);
    }
    assert_eq!(took, [true, true, true], "{:?}", said(&w));
    assert!(w.state.actors[&rat].hp < 1000, "the rat was struck");
    assert_eq!(w.state.player().weapon_skills, before);
}

#[test]
fn a_rank_changes_the_roll_the_mana_and_the_casts_time() {
    let mut w = wizard();
    let content = w.content().clone();
    let fireball = content.spell(sp("fireball"));
    let roll = |w: &World| {
        let me = w.state.player();
        let roll = me
            .spell_roll(&content, fireball, CastBy::Own)
            .expect("a roll");
        (roll.to_string(), me.spell_mods(&content, fireball).reroll)
    };
    // Intelligence 14: the ladder's first rung, 3d6.
    ranks(&mut w, &[]);
    assert_eq!(roll(&w), ("3d6".to_string(), 0));
    ranks(&mut w, &[("evocation", 3)]);
    assert_eq!(roll(&w), ("3d6".to_string(), 1), "1s rolled again");
    ranks(&mut w, &[("evocation", 5)]);
    assert_eq!(roll(&w), ("3d7".to_string(), 1), "a die a size bigger");
    ranks(&mut w, &[("evocation", 6)]);
    assert_eq!(roll(&w), ("4d7".to_string(), 1), "a die more");
    // The mana: a twentieth off at rank 4, the floor of 1 holding.
    let cost = |w: &World| w.state.player().spell_cost(&content, fireball);
    ranks(&mut w, &[]);
    let full = cost(&w);
    ranks(&mut w, &[("evocation", 3)]);
    assert_eq!(cost(&w), full);
    ranks(&mut w, &[("evocation", 10)]);
    assert!(cost(&w) < full && cost(&w) >= 1, "{} of {full}", cost(&w));
    // The time: restoration 3 takes a tenth off a cure's two casts.
    let cure = content.spell(sp("cure_wounds"));
    let player = w.state.player;
    ranks(&mut w, &[]);
    let slow = w.cast_price(player, cure);
    ranks(&mut w, &[("restoration", 3)]);
    let quick = w.cast_price(player, cure);
    assert_eq!(quick, slow * 9 / 10, "{quick} of {slow}");
    // A rank in another school does nothing to it.
    ranks(&mut w, &[("evocation", 10)]);
    assert_eq!(w.cast_price(player, cure), slow);
}

#[test]
fn a_rank_defaults_to_its_kin_and_divination_stands_alone() {
    let mut w = wizard();
    let table = w.content().weapon_skills().clone();
    let evocation = skill(&w, "evocation");
    ranks(&mut w, &[]);
    w.player_mut().weapon_skills.raise(&table, evocation, 8);
    let rank = |w: &World, name| w.state.player().weapon_skills.rank(skill(w, name));
    assert_eq!(rank(&w, "transmutation"), 4);
    assert_eq!(rank(&w, "necromancy"), 0);
    let divination = skill(&w, "divination");
    w.player_mut().weapon_skills.raise(&table, divination, 10);
    for other in ["abjuration", "enchantment", "restoration"] {
        assert_eq!(rank(&w, other), 0, "{other}");
    }
}

#[test]
fn schooled_and_magister_read_a_rank_higher_and_devotee_marks_half_again() {
    let mut w = wizard();
    let content = w.content().clone();
    let fireball = content.spell(sp("fireball"));
    let hex = content.spell(sp("hex_of_weakness"));
    let roll = |w: &World| {
        let me = w.state.player();
        (me.spell_roll(&content, fireball, CastBy::Own)
            .expect("a roll"))
        .to_string()
    };
    ranks(&mut w, &[("evocation", 4), ("necromancy", 1)]);
    assert_eq!(roll(&w), "3d6");
    w.player_mut().talents.take("schooled", Some("evocation"));
    assert_eq!(roll(&w), "3d7", "evocation reads as 5");
    let bonus = |w: &World| w.state.player().school_benefit(&content, hex).bonus;
    assert_eq!(bonus(&w), 0, "Schooled is for its chosen school");
    w.player_mut().talents.take("magister", None);
    assert_eq!(roll(&w), "4d7", "evocation reads as 6");
    assert_eq!(bonus(&w), 1, "necromancy reads as 2");
    // An untrained school stays untrained.
    let blessing = content.spell(sp("blessing"));
    let abjuration = w.state.player().school_benefit(&content, blessing);
    assert_eq!(abjuration, crate::content::Benefit::default());
    // Devotee: half again the marks.
    let mut plain = wizard();
    let mut devout = wizard();
    devout.player_mut().talents.take("devotee", None);
    for w in [&mut plain, &mut devout] {
        ranks(w, &[("evocation", 1)]);
        kobold(w, 3, 10_000);
        cast(w, "magic_missile", Some((3, 0)));
    }
    let (a, b) = (marks(&plain, "evocation"), marks(&devout, "evocation"));
    assert!(a > 0);
    assert_eq!(b, a * 3 / 2, "{b} against {a}");
}

#[test]
fn the_schools_layer_and_json_and_the_dev_rank() {
    let mut w = wizard();
    ranks(&mut w, &[("evocation", 2), ("sniper", 1)]);
    run(&mut w, "rank:evocation:5");
    assert_eq!(
        w.state.player().weapon_skills.rank(skill(&w, "evocation")),
        5
    );
    assert!(
        said(&w)
            .iter()
            .any(|m| m.contains("You feel more skilled at evocation.")),
        "the rank says itself"
    );
    let rows = dump::layer_rows(&w, Layer::Schools);
    assert_eq!(rows[0], "Evocation 5 (0%)");
    assert_eq!(
        rows.len(),
        7 + 3,
        "seven schools and three caster styles: {rows:?}"
    );
    assert!(rows.iter().any(|r| r == "Necromancy 0 (0%)"));
    assert!(rows.iter().any(|r| r.starts_with("Sniper 1")));
    // No school among the weapon skills.
    let weapons = dump::layer_rows(&w, Layer::WeaponSkills);
    assert!(
        !weapons.iter().any(|r| r.starts_with("Evocation")),
        "{weapons:?}"
    );
    let json = dump::json(&w, &[], false);
    let schools = json["schools"].as_array().expect("schools");
    assert_eq!(schools.len(), 7);
    assert_eq!(schools[0]["id"], "evocation");
    assert_eq!(schools[0]["rank"], 5);
    assert_eq!(schools[0]["progress"], 0);
    let styles = json["caster_styles"].as_array().expect("caster styles");
    assert_eq!(styles[0]["id"], "sniper");
    assert!(json["weapon_skills"].get("evocation").is_none());
    // The dev progress layer: the marks of the next rank's need.
    kobold(&mut w, 3, 10_000);
    cast(&mut w, "magic_missile", Some((3, 0)));
    let progress = dump::layer_rows(&w, Layer::Progress);
    let need = w
        .content()
        .weapon_skills()
        .need(skill(&w, "evocation"), 5)
        .expect("a need");
    let line = format!("evocation marks {}/{need}", marks(&w, "evocation"));
    assert!(progress.contains(&line), "{line} in {progress:?}");
}

#[test]
fn a_spell_of_a_school_without_its_skill_is_refused() {
    let mut files = ContentFiles::builtin();
    let text = include_str!("../../../../assets/data/weapon_skills.toml");
    let start = text
        .find("[[skill]]\nid = \"divination\"")
        .expect("divination's row");
    let end = start + text[start..].find("\n\n").expect("its end");
    let without = format!("{}{}", &text[..start], &text[end + 2..]);
    files.insert("data/weapon_skills.toml", &without);
    let e = Content::load(&files).expect_err("refused");
    assert!(
        e.message
            .contains("is of school `divination`, which has no skill"),
        "{e:?}"
    );
}
