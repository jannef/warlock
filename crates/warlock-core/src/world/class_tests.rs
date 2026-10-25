//! Class abilities (spec §8 "Class abilities"; #246): the level tables of the everyday folk
//! (aristocrat, commoner, craftsperson, merchant) give their talents as the levels come, at
//! creation, and again on a restore; bonus picks come from the class's trees; and each ability
//! changes the one rule it names.

use super::*;
use crate::ability::Ability;
use crate::action::{Positions, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::{Category, Defence, Target};
use crate::content::{Factor, MAX_FAVOUR};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::rng::Rng;
use crate::stats::{PerStat, Stat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human of `class` in the showcase.
fn world(class: &str) -> World {
    let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
    World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world")
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn last(world: &World) -> String {
    said(world).last().cloned().unwrap_or_default()
}

/// Grants the XP that takes the player to `level`, spending every choice due on the way
/// (`levelup:auto`), so nothing is due at the end.
fn reach(world: &mut World, level: u8) {
    let spec = world.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(world.content(), spec);
    let need = world.content().experience().threshold_for(level, factor);
    let more = need - world.player_xp();
    run(world, &format!("xp:{more}"));
    assert_eq!(world.player_level(), level, "reached level {level}");
    while world.due().is_some() {
        run(world, "levelup:auto");
    }
}

/// [`reach`] the level before `level`, then grant the XP for `level` and spend its skill rolls
/// only, so its talent picks are left due.
fn arrive(world: &mut World, level: u8) {
    reach(world, level - 1);
    let spec = world.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(world.content(), spec);
    let more = world.content().experience().threshold_for(level, factor) - world.player_xp();
    run(world, &format!("xp:{more}"));
    world.advance_auto();
}

fn holds(world: &World, id: &str) -> bool {
    world.state.player().talents.rank_of(id, None) > 0
}

/// The player holds talent `id` (one rank more), without a level to take it at.
fn give(world: &mut World, id: &str) {
    world.player_mut().talents.take(id, None);
}

fn mods(world: &World) -> crate::talent::TalentMods {
    world
        .state
        .player()
        .talent_mods(world.content())
        .into_owned()
}

#[test]
fn level_one_abilities_come_with_the_character_and_others_with_their_levels() {
    let mut w = world("aristocrat");
    assert!(holds(&w, "well_dressed"), "level 1's");
    assert!(!holds(&w, "fencing"));
    // (A birth sign's gifts come first and are no business of the class.)
    let rows = dump::layer_rows(&w, Layer::Talents);
    assert_eq!(
        rows.last().map(String::as_str),
        Some("Well dressed (class)")
    );
    assert!(rows.iter().filter(|r| r.contains("(class)")).count() == 1);
    // Level 6 brings Fencing, as news of the level.
    reach(&mut w, 6);
    assert!(holds(&w, "fencing"));
    assert!(
        said(&w).contains(&"You gain Fencing.".to_string()),
        "{:?}",
        said(&w)
    );
    // An ordinary talent the table hands out (Riposte at 18) is held like a picked one, so it
    // cannot be picked again.
    reach(&mut w, 18);
    assert!(holds(&w, "riposte") && holds(&w, "noblesse"));
    assert!(w.state.player().talent_mods(w.content()).free_blows.riposte);
    let json = dump::json(&w, &[], false);
    let riposte = (json["talents"].as_array().expect("talents").iter())
        .find(|t| t["id"] == "riposte")
        .cloned()
        .expect("held");
    assert_eq!(riposte["class"], true);
}

#[test]
fn a_talent_already_picked_is_not_given_again() {
    let mut w = world("commoner");
    give(&mut w, "second_wind");
    reach(&mut w, 32);
    assert_eq!(w.state.player().talents.rank_of("second_wind", None), 1);
    assert!(!said(&w).contains(&"You gain Second wind.".to_string()));
    assert!(holds(&w, "thick_skull"), "the others still come");
}

#[test]
fn stat_gifts_raise_score_and_potential_once() {
    let mut w = world("aristocrat");
    let before = w.state.player().stats;
    reach(&mut w, 32);
    let after = w.state.player().stats;
    // Iron will: Presence +1, score and potential (stat training may add more of its own).
    assert!(after.score(Stat::Presence) > before.score(Stat::Presence));
    assert!(after.potential(Stat::Presence) > before.potential(Stat::Presence));
    // A restore gives nothing twice.
    let state = w.state.as_ref().clone();
    let back = World::from_state(BUILTIN.clone(), state).expect("restore");
    assert_eq!(back.state.player().stats, after);
    assert_eq!(back.state_hash(), w.state_hash());
}

#[test]
fn unlikely_hero_raises_every_stat_and_toughness_the_hit_points() {
    let mut w = world("commoner");
    reach(&mut w, 59);
    let (stats, hp, max) = {
        let p = w.state.player();
        (p.stats, p.hp, p.max_hp(w.content()))
    };
    w.grant_class(60, true);
    let p = w.state.player();
    for stat in Stat::ALL {
        assert_eq!(p.stats.score(stat), stats.score(stat) + 1, "{stat:?}");
        assert_eq!(
            p.stats.potential(stat),
            stats.potential(stat) + 1,
            "{stat:?}"
        );
    }
    assert_eq!(
        p.max_hp(w.content()) - max,
        p.hp - hp,
        "hit points rise with the maximum"
    );
}

#[test]
fn a_restore_gives_what_a_grown_table_gives_and_nothing_else_changes() {
    let mut w = world("merchant");
    reach(&mut w, 12);
    let hash = w.state_hash();
    // A save from before the table: the abilities are missing.
    let mut state = w.state.as_ref().clone();
    let player = state.player;
    let picked: Vec<_> = (state.actors[&player].talents.iter())
        .filter(|t| !["haggler", "deep_pockets", "eye_for_quality"].contains(&t.id.as_str()))
        .cloned()
        .collect();
    let actor = state.actors.get_mut(&player).expect("player");
    actor.talents = crate::talent::Talents::default();
    for t in picked {
        for _ in 0..t.rank {
            actor.talents.take(&t.id, t.choice.as_deref());
        }
    }
    let back = World::from_state(BUILTIN.clone(), state).expect("restore");
    for id in ["haggler", "deep_pockets", "eye_for_quality"] {
        assert!(holds(&back, id), "{id}");
    }
    assert!(!holds(&back, "trade_routes"), "not before its level");
    let _ = hash;
}

#[test]
fn bonus_picks_come_from_the_class_trees_before_the_ordinary_ones() {
    let mut w = world("aristocrat");
    arrive(&mut w, 10);
    let due = w.due().expect("level 10 is due");
    assert_eq!((due.level, due.talents, due.bonus), (10, 0, 1));
    // Vigour is no tree of the aristocrat's.
    run(&mut w, "talent:healthy");
    assert_eq!(
        last(&w),
        "You must choose a talent of Weaponcraft or Delver."
    );
    assert!(w.due().is_some());
    run(&mut w, "talent:far_shot");
    assert_eq!(last(&w), "You learn Far shot.");
    assert_eq!(w.due(), None);
    let rows = dump::layer_rows(&w, Layer::Progress);
    assert!(!rows.iter().any(|r| r.starts_with("due")), "{rows:?}");
}

#[test]
fn a_level_with_both_picks_spends_the_bonus_on_a_talent_of_its_trees() {
    let mut w = world("aristocrat");
    // Scores and ranks high enough that picks never run out on the way.
    w.player_mut().stats = Stats::new(PerStat::splat(18), PerStat::splat(40));
    let table = BUILTIN.weapon_skills();
    for skill in ["long_blades", "duelist", "bows"] {
        let id = table.id(skill).expect(skill);
        w.player_mut().weapon_skills.raise(table, id, 9);
    }
    arrive(&mut w, 30);
    // Two ordinary picks: a human's Versatile adds one at level 30.
    let due = w.due().expect("level 30 is due");
    assert_eq!((due.talents, due.bonus), (2, 1));
    let rows = dump::layer_rows(&w, Layer::Progress);
    assert!(
        rows.contains(&"due level 30: 2 talents, 1 bonus talent".to_string()),
        "{rows:?}"
    );
    // A talent of no bonus tree fills the ordinary pick; the bonus waits for its trees.
    let trees = w.bonus_trees().expect("the aristocrat's trees");
    let book = BUILTIN.talents();
    for _ in 0..2 {
        let (other, _) = (crate::talent::offers(w.content(), &w.candidate()).into_iter())
            .find(|(t, ways)| !trees.contains(&book.get(*t).tree) && ways == &[None])
            .expect("a talent of another tree");
        let other = book.get(other).id.clone();
        run(&mut w, &format!("talent:{other}"));
    }
    let due = w.due().expect("the bonus is left");
    assert_eq!((due.talents, due.bonus), (0, 1), "{:?}", said(&w));
    let (fits, ways) = (crate::talent::offers(w.content(), &w.candidate()).into_iter())
        .find(|(t, _)| trees.contains(&book.get(*t).tree))
        .expect("a talent of the aristocrat's trees");
    let fits = book.get(fits).id.clone();
    match ways.into_iter().next().flatten() {
        Some(choice) => run(&mut w, &format!("talent:{fits}:{choice}")),
        None => run(&mut w, &format!("talent:{fits}")),
    }
    assert_eq!(w.due(), None, "{:?}", said(&w));
}

#[test]
fn the_commoners_bonus_picks_come_from_any_tree() {
    let mut w = world("commoner");
    arrive(&mut w, 10);
    assert_eq!(w.due().map(|d| d.bonus), Some(1));
    let pick = if holds(&w, "healthy") {
        "lean"
    } else {
        "healthy"
    };
    run(&mut w, &format!("talent:{pick}"));
    assert_eq!(w.due(), None, "{:?}", said(&w));
}

#[test]
fn class_levels_replay_exactly() {
    let mut w = world("craftsperson");
    let actions = parse_actions("xp:200000 levelup:auto*12 wait").expect("actions");
    let mut replay = Replay::new(&w);
    for action in &actions {
        w.apply(action);
        replay.push(action, &w);
    }
    assert!(holds(&w, "keen_appraisal") && holds(&w, "whetstone"));
    let back = replay.run(BUILTIN.clone()).expect("replays");
    assert_eq!(back.state_hash(), w.state_hash());
}

// The hooks, one each.

#[test]
fn shop_favour_and_its_cap_follow_the_talents() {
    let mut w = world("warrior");
    let shops = BUILTIN.shops().expect("shops");
    let base = crate::world::shop::favour(&BUILTIN, shops, w.state.player());
    give(&mut w, "haggler");
    let haggler = crate::world::shop::favour(&BUILTIN, shops, w.state.player());
    assert_eq!(haggler, (base + 15).clamp(-MAX_FAVOUR, MAX_FAVOUR));
    // Presence of 40 is far past the cap: Magnate lifts it by 10, Golden tongue by 10 more.
    w.player_mut().stats.gift(Stat::Presence, 30);
    let capped = crate::world::shop::favour(&BUILTIN, shops, w.state.player());
    assert_eq!(capped, MAX_FAVOUR);
    give(&mut w, "magnate");
    let magnate = crate::world::shop::favour(&BUILTIN, shops, w.state.player());
    assert_eq!(magnate, MAX_FAVOUR + 10);
    give(&mut w, "golden_tongue");
    let tongue = crate::world::shop::favour(&BUILTIN, shops, w.state.player());
    assert_eq!(tongue, MAX_FAVOUR + 20);
}

#[test]
fn noblesse_makes_anyone_not_hostile_budge() {
    let w = World::new(BUILTIN.clone(), "courtyard", None).expect("courtyard");
    // Born under no sign: the scenario's Basilisk is trusted by none (spec §8 "Birth sign").
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(BUILTIN.clone(), state).expect("restores");
    let keeper = w.state.current().shops[0].keeper;
    assert!(w.refusal(keeper).is_some(), "a keeper at its post keeps it");
    give(&mut w, "noblesse");
    assert_eq!(w.refusal(keeper), None);
}

#[test]
fn checks_of_a_stat_take_the_talents_bonus() {
    let mut w = world("warrior");
    let passive = w.search_modifier(super::search::Look::Passive);
    give(&mut w, "iron_will");
    assert_eq!(w.search_modifier(super::search::Look::Passive), passive + 2);
    assert_eq!(mods(&w).check(Stat::Willpower), 2);
    give(&mut w, "stubborn");
    assert_eq!(mods(&w).check(Stat::Toughness), 1);
    let poisoned = w.content().status_id("poisoned").expect("poisoned");
    assert!(mods(&w).no_penalty.contains(&poisoned));
}

#[test]
fn peerless_criticals_confirm_more_often() {
    let mut w = world("warrior");
    let plain = w.state.player().melee(w.content());
    give(&mut w, "peerless");
    let sure = w.state.player().melee(w.content());
    assert_eq!((plain.confirm, sure.confirm), (0, 2));
    let target = Target::bare(Defence::default().plus(Category::Dodge, 10));
    let criticals = |profile: &crate::combat::Profile| {
        let mut rng = Rng::stream(1, "peerless");
        (0..4000)
            .filter(|_| {
                matches!(
                    crate::combat::strike(profile, &target, &mut rng).strike,
                    crate::combat::Strike::Hit { critical: true }
                )
            })
            .count()
    };
    assert!(criticals(&sure) > criticals(&plain));
}

#[test]
fn weapons_and_styles_read_the_talents_benefits() {
    let mut w = world("aristocrat");
    let duelist = w.content().weapon_skills().id("duelist").expect("duelist");
    let maces = w.content().weapon_skills().id("maces").expect("maces");
    let long = w
        .content()
        .weapon_skills()
        .id("long_blades")
        .expect("long blades");
    let before_maces = w.state.player().benefit(w.content(), Some(maces));
    let before_long = w.state.player().benefit(w.content(), Some(long));
    give(&mut w, "fencing");
    let duelist = &w.content().weapon_skills().skill(duelist).clone();
    let (_, ranks) = mods(&w).weapon_benefit(&duelist.id, duelist.role, 0);
    assert_eq!(ranks, 1, "Duelist reads a rank higher");
    give(&mut w, "big_hands");
    let maced = w.state.player().benefit(w.content(), Some(maces));
    assert_eq!(maced.damage, before_maces.damage + 1);
    give(&mut w, "whetstone");
    give(&mut w, "masterwork");
    let honed = w.state.player().benefit(w.content(), Some(long));
    assert_eq!(
        (honed.damage, honed.threat),
        (before_long.damage + 1, before_long.threat + 1)
    );
}

#[test]
fn armour_talents_add_protection() {
    let mut w = world("warrior");
    // Leather armour and a cap: two pieces with Protection of their own.
    let jerkin = w.content().object_id("leather_jerkin").expect("jerkin");
    let cap = w.content().object_id("leather_cap").expect("cap");
    {
        let p = w.player_mut();
        p.inventory.push(Stack::one(jerkin));
        p.inventory.push(Stack::one(cap));
    }
    let n = w.state.player().inventory.len();
    run(
        &mut w,
        &format!("equip:body:{} equip:head:{}", n - 2, n - 1),
    );
    let p = w.state.player();
    let pieces = (p.equipment.items().into_iter())
        .filter(|&(_, i)| p.inventory[i].protection(w.content()) > 0)
        .count() as i32;
    assert!(pieces > 0);
    let worn = w.state.player().protection(w.content(), None).base;
    give(&mut w, "thick_skull");
    give(&mut w, "sturdy_work");
    give(&mut w, "artisan");
    let more = w.state.player().protection(w.content(), None).base;
    assert_eq!(
        more,
        worn + 1 + 1 + pieces,
        "Thick skull, Sturdy work, Artisan's pieces"
    );
}

#[test]
fn burden_and_coins_follow_the_talents() {
    let mut w = world("merchant");
    let gold = w.content().object_id("gold_coin").expect("gold");
    w.player_mut().inventory.push(Stack::new(gold, 1000));
    let id = w.state.player;
    let before = w.load_of(id);
    give(&mut w, "deep_pockets");
    let after = w.load_of(id);
    assert!(
        after.limit.abs_diff(before.limit + 10_000) < 3,
        "kept a multiple of 3 g"
    );
    assert_eq!(
        before.carried - after.carried,
        10_000,
        "a thousand coins at 10 g"
    );
    // Hard work: Burdened's slowdown nearly gone.
    let mut w = world("warrior");
    let stone = w.content().object_id("gold_coin").expect("gold");
    let limit = w.load_of(id).limit;
    let coins = u32::try_from(limit / 3 / 10 + 10).expect("coins");
    w.player_mut().inventory.push(Stack::new(stone, coins));
    assert_eq!(w.load_of(id).penalty(), crate::burden::Burden::Burdened);
    let slowed = w.price_of(id, crate::content::ActionKind::Wait);
    give(&mut w, "hard_work");
    let eased = w.price_of(id, crate::content::ActionKind::Wait);
    assert!(eased < slowed, "{slowed} → {eased}");
}

#[test]
fn tinker_lets_the_untrained_disarm() {
    let mut w = world("warrior");
    assert!(!Ability::Disarm.had_by(w.state.player(), w.content()));
    give(&mut w, "tinker");
    assert!(Ability::Disarm.had_by(w.state.player(), w.content()));
}

#[test]
fn jeweller_cuts_loose_gems_finer_and_nothing_else() {
    let content = &BUILTIN;
    let gems = content.gems().expect("gems");
    let ruby = content.material_id("ruby").expect("ruby");
    let cutting = crate::content::Cutting {
        size: crate::content::GemSizeId(0),
        cut: crate::content::CutId(0),
    };
    let part = crate::item_parts::ItemPart::gem(content, None, ruby, cutting);
    let mut stack = Stack::one(gems.object).with_parts(vec![part]);
    crate::gem::cut_finer(content, &mut stack, 1);
    assert_eq!(stack.gem(content).map(|(_, c)| c.cut.0), Some(1));
    crate::gem::cut_finer(content, &mut stack, 200);
    let finest = gems.cuts().len() - 1;
    assert_eq!(
        stack.gem(content).map(|(_, c)| usize::from(c.cut.0)),
        Some(finest)
    );
    let mut sword = Stack::one(content.object_id("long_sword").expect("sword"));
    let before = sword.clone();
    crate::gem::cut_finer(content, &mut sword, 1);
    assert_eq!(sword, before);
}

#[test]
fn connoisseur_identifies_a_magic_item_carried_a_day() {
    let mut w = world("warrior");
    w.player_mut().inventory.push(magic_sword(&BUILTIN));
    let item = w.state.player().inventory.len() - 1;
    // Without the ability a day of carrying teaches nothing.
    run(&mut w, "rest:24h");
    assert!(!w.state.player().inventory[item].is_identified());
    give(&mut w, "connoisseur");
    // An hour of rest counts (the time passing for the player), then the rest of the day.
    run(&mut w, "rest:1h");
    w.appraise_carried(22 * crate::time::HOUR);
    assert!(
        !w.state.player().inventory[item].is_identified(),
        "23 hours"
    );
    w.appraise_carried(crate::time::HOUR);
    assert!(w.state.player().inventory[item].is_identified());
    assert!(
        said(&w).iter().any(|m| m.starts_with("This is ")),
        "{:?}",
        said(&w)
    );
}

#[test]
fn worth_shows_only_to_an_eye_for_it() {
    let mut w = world("warrior");
    assert_eq!(w.perception().appraisal(), Default::default());
    give(&mut w, "appraiser");
    let appraisal = w.perception().appraisal();
    assert!(appraisal.worth && appraisal.gems);
    let dagger = Stack::new(w.content().object_id("dagger").expect("dagger"), 3);
    let lines = crate::magic::describe::appraised(&dagger, w.content(), appraisal);
    let worth = dagger.value(w.content());
    let text = format!("Worth {} gold each, {worth} gold in all", worth / 3);
    assert!(lines.iter().any(|l| l.text == text), "{lines:?}");
    let plain = crate::magic::describe::lines(&dagger, w.content());
    assert!(!plain.iter().any(|l| l.text.starts_with("Worth")));
}

#[test]
fn quest_and_trade_talents_sum_into_the_mods() {
    let mut w = world("warrior");
    for id in [
        "well_dressed",
        "folk_hero",
        "insured",
        "credit",
        "commanding",
        "fletching",
    ] {
        give(&mut w, id);
    }
    let m = mods(&w);
    assert_eq!((m.quest_gold_percent, m.quest_xp_percent), (75, 50));
    assert_eq!(
        (m.sell_percent, m.credit),
        (50, 100),
        "the most credit counts"
    );
    assert_eq!(m.missile_breaks, -50);
    let intimidate = w.content().skill_id("intimidate").expect("intimidate");
    assert_eq!(m.skills, [(intimidate, 20)]);
}

#[test]
fn credit_lets_the_player_leave_owing_a_little() {
    // The courtyard store (as `shop_tests` walks in): in, pick up the dagger, out of the door.
    let into = "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 move:S*5 \
                move:E*5";
    let mut w = World::new(BUILTIN.clone(), "courtyard", None).expect("courtyard");
    run(&mut w, into);
    let shop = w.state.current().shops[0].clone();
    assert!(shop.contains(w.player_pos()), "in the shop");
    let keeper = shop.keeper;
    give(&mut w, "credit");
    run(&mut w, "move:E move:NE pickup move:SW move:W move:W");
    assert!(
        said(&w)
            .iter()
            .any(|m| m.ends_with("nods: you can pay next time.")),
        "{:?}",
        said(&w)
    );
    assert!(!w.state.current().shops[0].contains(w.player_pos()), "out");
    assert_eq!(w.stance(w.player_id(), keeper), Stance::Neutral, "no theft");
    assert!(!w.state.current().shops[0].unpaid.is_empty(), "still owed");
}

/// A magic long sword, unidentified.
fn magic_sword(content: &Content) -> Stack {
    let sword = content.object_id("long_sword").expect("sword");
    let mut rng = Rng::stream(3, "connoisseur");
    (0..200)
        .map(|_| {
            crate::magic::loot::make(
                content,
                &mut rng,
                sword,
                1,
                5,
                crate::magic::LootGrade::Magic,
            )
        })
        .find(|s| !s.is_identified())
        .expect("a magic sword")
}

#[test]
fn a_merchant_knows_what_they_sell_and_is_paid_more_for_it() {
    let into = "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 move:S*5 \
                move:E*5";
    let sale = |talents: &[&str]| {
        let mut w = World::new(BUILTIN.clone(), "courtyard", None).expect("courtyard");
        run(&mut w, into);
        for id in talents {
            give(&mut w, id);
        }
        w.player_mut().inventory.push(magic_sword(&BUILTIN));
        let item = w.state.player().inventory.len() - 1;
        let offer = w.perception().sale(&Positions::of([item])).map(|s| s.offer);
        run(&mut w, &format!("sell:{item}"));
        (w, offer.expect("the store buys swords"))
    };
    let (plain, offer) = sale(&[]);
    assert!(!said(&plain).iter().any(|m| m.starts_with("That was")));
    let (w, more) = sale(&["eye_for_quality", "insured"]);
    assert!(more > offer, "{offer} → {more}");
    assert!(
        said(&w).iter().any(|m| m.starts_with("That was ")),
        "{:?}",
        said(&w)
    );
    let sold = said(&w)
        .iter()
        .position(|m| m.starts_with("You sell"))
        .expect("sold");
    assert!(
        said(&w)[sold].contains("(magic)"),
        "the sale names it as it was known"
    );
}
