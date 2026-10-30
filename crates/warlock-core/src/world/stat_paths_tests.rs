//! The training paths (spec §8 "Stats"; the design `docs/design/stat-training.md` §5): each
//! hook that reports a use of the player's stats, one test each (or a few where one call
//! carries several), on content where every chance is 100 so a mark comes without a draw.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::combat::{Blow, Strike};
use crate::content::{ContentFiles, HAUL_GRAMS, StatUse};
use crate::object::Stack;
use crate::skills::Use;
use crate::stats::{PerStat, Stat};
use crate::temperature::Temperature;
use std::sync::LazyLock;

/// The built-in `data/stats.toml` with every chance at 100.
fn sure_stats() -> String {
    let text = include_str!("../../../../assets/data/stats.toml");
    (text.lines())
        .map(|l| match l.starts_with("chance = ") {
            true => "chance = 100\n".to_string(),
            false => format!("{l}\n"),
        })
        .collect()
}

/// A yard: the player at `@`, water at `~`, a forgotten altar at `A`, a kobold at `k`.
const YARD: &str = "\
#######
#@.k..#
#.~~..#
#A....#
#######
";

static SURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let stats = sure_stats();
    let level = format!(
        r##"name = "Yard"
ambient = [255, 255, 255, 0]
map = '''
{YARD}'''
[legend]
"@" = {{ terrain = "floor", spawn = "player" }}
"~" = {{ terrain = "water" }}
"A" = {{ terrain = "forgotten_altar" }}
"k" = {{ terrain = "floor", creature = "kobold" }}
"##
    );
    let mut files = ContentFiles::builtin();
    files.insert("data/stats.toml", &stats);
    files.insert("levels/sp_yard.toml", &level);
    files.insert(
        "scenarios/sp_yard.toml",
        "name = \"Yard\"\nstart_level = \"sp_yard\"\nhp = 1000\n",
    );
    Arc::new(Content::load(&files).expect("content"))
});

/// The yard, nobody but the player acting.
fn yard() -> World {
    let mut w = World::new(SURE.clone(), "sp_yard", Some(7)).expect("world");
    let player = w.player_id();
    for actor in w.state.actors.values_mut() {
        if actor.id != player {
            actor.energy = -1_000_000_000;
        }
    }
    w
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn marks(world: &World) -> PerStat<i32> {
    world.state.player().stat_marks
}

/// The marks `f` earned, stat by stat.
fn earned(w: &mut World, f: impl FnOnce(&mut World)) -> PerStat<i32> {
    let before = marks(w);
    f(w);
    let after = marks(w);
    let mut out = PerStat::splat(0);
    for stat in Stat::ALL {
        out[stat] = after[stat] - before[stat];
    }
    out
}

/// Only `stats` earned, `n` each.
fn only(stats: &[Stat], n: i32) -> PerStat<i32> {
    let mut out = PerStat::splat(0);
    for &stat in stats {
        out[stat] = n;
    }
    out
}

#[test]
fn every_new_use_has_its_rules_by_stat_in_the_data() {
    let rules = SURE.stat_training();
    let stats = |used| {
        (rules.rules(used))
            .filter_map(|r| r.stat)
            .collect::<Vec<_>>()
    };
    use Stat::*;
    for (used, want) in [
        (StatUse::Forged, vec![Strength]),
        (StatUse::Struggled, vec![Strength, Willpower]),
        (StatUse::Waded, vec![Strength]),
        (StatUse::Hauled, vec![Strength]),
        (StatUse::Picked, vec![Dexterity]),
        (StatUse::Disarmed, vec![Dexterity, Intelligence]),
        (StatUse::Stole, vec![Dexterity]),
        (StatUse::KeptWits, vec![Dexterity]),
        (StatUse::Aimed, vec![Dexterity]),
        (StatUse::Marched, vec![Toughness]),
        (StatUse::HungryStep, vec![Toughness]),
        (StatUse::ShookOff, vec![Toughness]),
        (StatUse::Exerted, vec![Toughness]),
        (StatUse::Chilled, vec![Toughness]),
        (StatUse::Spell, vec![Intelligence, Willpower, Presence]),
        (StatUse::Explored, vec![Intelligence]),
        (StatUse::Appraised, vec![Intelligence]),
        (StatUse::Searched, vec![Intelligence]),
        (StatUse::Foraged, vec![Intelligence]),
        (StatUse::Prayed, vec![Willpower]),
        (StatUse::HeldOut, vec![Willpower]),
        (StatUse::Steadied, vec![Willpower]),
        (StatUse::Inspired, vec![Presence]),
        (StatUse::Promoted, vec![Presence]),
        (StatUse::Bargained, vec![Presence]),
        (StatUse::Greeted, vec![Presence]),
    ] {
        assert_eq!(stats(used), want, "{used:?}");
        assert!(!used.describe().is_empty());
    }
    // Being hurt is a little less sure than it was (decision 13): 15 %.
    let builtin = Content::builtin().expect("content");
    let hurt: Vec<_> = (builtin.stat_training().rules(StatUse::Hurt))
        .map(|r| r.chance)
        .collect();
    assert_eq!(hurt, [15]);
}

#[test]
fn the_skills_uses_that_are_stat_uses_too_train_both_from_one_report() {
    use Stat::*;
    for (used, stats) in [
        (Use::Temper, vec![Strength]),
        (Use::Pick, vec![Dexterity]),
        (Use::Disarm, vec![Dexterity, Intelligence]),
        (Use::Steal, vec![Dexterity]),
        (Use::Travel, vec![Toughness]),
        (Use::Appraise, vec![Intelligence]),
        (Use::Forage, vec![Intelligence]),
        (Use::Kick, vec![Strength]),
    ] {
        let mut w = yard();
        assert!(w.trains(used), "{used:?}");
        assert_eq!(
            earned(&mut w, |w| w.train(used)),
            only(&stats, 1),
            "{used:?}"
        );
    }
    // A search is no stat's use by itself: the search turn says what it found.
    let mut w = yard();
    assert_eq!(earned(&mut w, |w| w.train(Use::Search)), only(&[], 0));
}

#[test]
fn a_spell_cast_trains_the_stats_its_ladders_name() {
    let content = SURE.clone();
    let spell = |id: &str| content.spell(content.spell_id(id).expect(id));
    let trained = |id: &str| -> Vec<Stat> {
        let t = spell(id).trained_stats();
        Stat::ALL.into_iter().filter(|&s| t[s]).collect()
    };
    // Magic missile: damage by Intelligence, a reach by Willpower.
    let missile = spell("magic_missile");
    assert!(missile.grows.damage.is_some());
    assert_eq!(
        trained("magic_missile"),
        [Stat::Intelligence, Stat::Willpower]
    );
    // A spell whose roll has no ladder trains Intelligence; every ladder's stat is trained.
    for def in content.spells().iter() {
        let t = def.trained_stats();
        assert!(
            def.grows.named().iter().all(|(_, l)| t[l.stat]),
            "{}",
            def.id
        );
        if def.grows.roll().is_none() {
            assert!(t[Stat::Intelligence], "{}", def.id);
        }
    }
    let hex = (content.spells().iter())
        .find(|d| d.kind == crate::content::SpellPurpose::Hex)
        .expect("a hex");
    assert!(hex.trained_stats()[Stat::Presence], "{}", hex.id);
    // Cast: only the rules of the stats it names give marks.
    let mut w = yard();
    let got = earned(&mut w, |w| w.train_spell(missile));
    assert_eq!(got, only(&[Stat::Intelligence, Stat::Willpower], 1));
    let mut w = yard();
    let got = earned(&mut w, |w| w.train_spell(hex));
    let t = hex.trained_stats();
    for stat in Stat::ALL {
        let ruled = [Stat::Intelligence, Stat::Willpower, Stat::Presence].contains(&stat);
        assert_eq!(got[stat], i32::from(t[stat] && ruled), "{stat:?}");
    }
}

#[test]
fn an_attack_spell_that_hits_trains_dexterity() {
    let mut w = yard();
    let (player, other) = (w.state.player, ActorId(999));
    let hit = Blow {
        strike: Strike::Hit { critical: false },
        damage: 3,
        kind: None,
    };
    let got = earned(&mut w, |w| {
        w.train_fight((player, other), AttackKind::Spell, &hit)
    });
    assert_eq!(got, only(&[Stat::Dexterity], 1));
}

#[test]
fn wading_deep_water_trains_strength_but_not_walking_its_ice() {
    let mut w = yard();
    let water = SURE.terrain_id("water").expect("water");
    let floor = SURE.terrain_id("floor").expect("floor");
    let got = earned(&mut w, |w| {
        w.train_wade(water, super::ice::Footing::Plain);
        w.train_wade(water, super::ice::Footing::Held { climb: false });
        w.train_wade(floor, super::ice::Footing::Plain);
    });
    assert_eq!(got, only(&[Stat::Strength], 1));
    // A step into the yard's pool, in the game.
    let mut w = yard();
    assert_eq!(earned(&mut w, |w| run(w, "move:SE"))[Stat::Strength], 1);
}

#[test]
fn hauling_a_heavy_stack_trains_strength_once() {
    let mut w = yard();
    let rock = |n| Stack::new(SURE.object_id("dagger").expect("a dagger"), n);
    let light = rock(1);
    assert!(light.weight(&SURE) < HAUL_GRAMS);
    let many = (HAUL_GRAMS / light.weight(&SURE).max(1) + 1) as u32;
    let got = earned(&mut w, |w| {
        w.train_haul(&[rock(1)]);
        w.train_haul(&[rock(many), rock(many)]);
    });
    assert_eq!(got, only(&[Stat::Strength], 1));
}

#[test]
fn acting_while_hungry_trains_toughness_and_waiting_or_weakness_does_not() {
    let mut w = yard();
    let feed = |w: &mut World, n| {
        w.player_mut().satiation = Some(crate::hunger::Satiation::new(n));
    };
    feed(&mut w, 1000);
    assert_eq!(w.player_hunger(), Some(crate::hunger::Hunger::Hungry));
    assert_eq!(earned(&mut w, |w| run(w, "wait")), only(&[], 0));
    assert_eq!(earned(&mut w, |w| run(w, "move:S"))[Stat::Toughness], 1);
    feed(&mut w, 100);
    assert!(w.player_hunger() < Some(crate::hunger::Hunger::Hungry));
    assert_eq!(earned(&mut w, |w| run(w, "move:N"))[Stat::Toughness], 0);
}

#[test]
fn a_harmful_status_wearing_off_trains_toughness() {
    let mut w = yard();
    run(&mut w, "afflict:bleeding");
    let bleeding = SURE.status_id("bleeding").expect("bleeding");
    let got = earned(&mut w, |w| {
        for _ in 0..200 {
            if !w.state.player().effects.has(bleeding) {
                break;
            }
            run(w, "wait");
        }
    });
    assert!(!w.state.player().effects.has(bleeding));
    assert_eq!(got[Stat::Toughness], 1);
    // One that does no harm as it lasts (Hasted) trains nothing as it ends.
    let mut w = yard();
    let hasted = SURE.status_id("hasted").expect("hasted");
    run(&mut w, "afflict:hasted");
    let ended = [*w.state.player().effects.get(hasted).expect("hasted")];
    let player = w.state.player;
    assert_eq!(
        earned(&mut w, |w| w.train_shaking_off(player, &ended)),
        only(&[], 0)
    );
}

#[test]
fn a_weapon_ability_paid_in_stamina_trains_toughness() {
    let mut w = yard();
    let content = w.content().clone();
    let spear = content.object_id("spear").expect("a spear");
    let me = w.player_mut();
    me.inventory.push(Stack::new(spear, 1));
    let item = me.inventory.len() - 1;
    let slot = crate::equipment::free_slot(&content, &me.equipment, spear).expect("a free slot");
    me.equipment
        .put(&crate::equipment::slots_taken(&content, spear, slot), item);
    let table = content.weapon_skills();
    let spears = table.id("spears").expect("spears");
    w.player_mut().weapon_skills =
        crate::weapon_skills::WeaponSkills::from_ranks(table, [(spears, 9)]);
    let stamina = content.resource_id("stamina").expect("stamina");
    let player = w.player_id();
    w.grant_resource(player, stamina);
    run(&mut w, "move:E");
    let got = earned(&mut w, |w| run(w, "use:lunge:E"));
    assert_eq!(got[Stat::Toughness], 1, "{:#?}", w.messages());
}

#[test]
fn a_dawn_below_freezing_trains_toughness_at_that_check() {
    for (degrees, want) in [(-5, 1), (5, 0)] {
        let mut w = yard();
        w.state.current_mut().temperature = Some(Temperature::degrees(degrees));
        let (player, now) = (w.state.player, w.time());
        w.state.next_stat_check = Some(now);
        let got = earned(&mut w, |w| w.daily_stat_check(player, now));
        assert_eq!(got[Stat::Toughness], want, "{degrees} °C");
    }
}

#[test]
fn a_search_that_finds_nothing_trains_intelligence() {
    let mut w = yard();
    assert_eq!(
        earned(&mut w, |w| run(w, "search")),
        only(&[Stat::Intelligence], 1)
    );
}

#[test]
fn a_first_arrival_that_pays_trains_intelligence() {
    let mut w = yard();
    let got = earned(&mut w, |w| w.discover_level("depths:3"));
    assert_eq!(got, only(&[Stat::Intelligence], 1), "{:#?}", w.news);
}

#[test]
fn a_prayer_answered_trains_willpower() {
    let mut w = yard();
    run(&mut w, "move:S");
    let got = earned(&mut w, |w| run(w, "pray"));
    assert!(got[Stat::Willpower] >= 1, "{:#?}", w.messages());
}

#[test]
fn a_struggle_against_a_hold_trains_strength_and_willpower() {
    let mut w = yard();
    run(&mut w, "afflict:held");
    let got = earned(&mut w, |w| run(w, "move:E"));
    assert_eq!((got[Stat::Strength], got[Stat::Willpower]), (1, 1));
}

#[test]
fn a_drain_held_out_trains_willpower_and_a_creatures_trains_nothing_of_the_players() {
    let mut w = yard();
    let drain = crate::content::Drain {
        stat: Some(Stat::Strength),
        points: 1,
        check: Stat::Toughness,
        difficulty: -1000,
    };
    let player = w.state.player;
    let got = earned(&mut w, |w| w.drain(player, drain, None, &mut Vec::new()));
    // The check made: Toughness's resisting mark, and Willpower's for holding out.
    assert_eq!(got, only(&[Stat::Toughness, Stat::Willpower], 1));
    let kobold = (w.state.actors.keys().copied())
        .find(|&id| id != player)
        .expect("the kobold");
    let got = earned(&mut w, |w| w.drain(kobold, drain, None, &mut Vec::new()));
    assert_eq!(got, only(&[], 0));
}

#[test]
fn keeping_ones_wits_and_recovering_them_train_dexterity_and_willpower() {
    let mut w = yard();
    let player = w.state.player;
    let surprised = SURE.surprise().expect("surprise rules").status;
    let (mut made, mut dex) = (0, 0);
    for _ in 0..40 {
        let got = earned(&mut w, |w| {
            w.startle(player, None, &mut Vec::new());
        });
        let caught = w.state.player().effects.has(surprised);
        made += i32::from(!caught);
        dex += got[Stat::Dexterity];
        if caught {
            // An action ends it (not one it was given during): the wits recovered.
            let got = earned(&mut w, |w| run(w, "wait wait"));
            assert_eq!(got[Stat::Willpower], 1);
            assert!(!w.state.player().effects.has(surprised));
        }
    }
    assert!(made > 0 && made < 40, "{made}");
    assert_eq!(dex, made);
}

#[test]
fn throwing_ones_voice_trains_presence() {
    let mut w = yard();
    w.player_mut().talents.take("throw_voice", None);
    let got = earned(&mut w, |w| run(w, "throw_voice:5,3"));
    assert_eq!(got[Stat::Presence], 1, "{:#?}", w.messages());
}

#[test]
fn a_deal_trains_presence_more_when_negotiation_bettered_it() {
    let mut w = yard();
    assert_eq!(
        earned(&mut w, |w| w.train_deal()),
        only(&[Stat::Presence], 1)
    );
    let negotiation = SURE.skill_id("negotiation").expect("negotiation");
    w.player_mut().skills.add(negotiation, 40);
    assert!(
        w.state()
            .player()
            .skill_effect(&SURE, crate::skills::Effect::Bargain)
            > 0
    );
    assert_eq!(
        earned(&mut w, |w| w.train_deal()),
        only(&[Stat::Presence], 2)
    );
}

#[test]
fn the_first_chat_with_someone_trains_presence_twice_and_later_ones_once() {
    let mut w = World::new(SURE.clone(), "ceaptun", Some(7)).expect("world");
    let player = w.state.player;
    // Someone who will talk, beside the player.
    let npc = (w.state.actors.values())
        .find(|a| a.id != player && a.npc.is_some() && a.level == w.state.player().level)
        .map(|a| a.id)
        .expect("someone to talk to");
    let at = w.state.actors[&player].pos.step(Dir::E);
    w.state.actors.get_mut(&npc).expect("npc").pos = at;
    let first = earned(&mut w, |w| run(w, "chat:E"));
    let again = earned(&mut w, |w| run(w, "chat:E"));
    assert_eq!(first[Stat::Presence], 2, "{:#?}", w.messages());
    assert_eq!(again[Stat::Presence], 1);
}

#[test]
fn inspiring_song_trains_presence() {
    let mut w = yard();
    w.player_mut().talents.take("inspire", None);
    w.player_mut().mana = 100;
    let got = earned(&mut w, |w| run(w, "use:inspire"));
    assert_eq!(got[Stat::Presence], 1, "{:#?}", w.messages());
}

#[test]
fn a_guild_rank_won_trains_presence() {
    let mut w = World::new(SURE.clone(), "ceaptun", None).expect("a run in Cēaptūn");
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").pos = Pos::new(81, 27).step(Dir::N);
    w.perceive(&mut Vec::new());
    // Asked for the bounty, the player is made a Tin (asking itself trains nothing).
    let got = earned(&mut w, |w| run(w, "ask:S:quest-guild_quarry"));
    assert!(
        w.messages()
            .iter()
            .any(|m| m.text.contains("names you a Tin"))
    );
    assert_eq!(got, only(&[Stat::Presence], 1));
}
