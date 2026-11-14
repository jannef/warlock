//! Splitting, dying into, and breeding (docs/design/monster-dread.md §5.5, §5.7; #1129): a
//! blade cuts the charnel heap in two and a mace does not, a half's kill pays a quarter and
//! counts for no quest, worms crawl out of a dead wormhost on its side, the wormlord sheds worms
//! up to its cap, and the engulfing statuses hold, hurt, and block as data says.

use super::split::Split;
use super::*;
use crate::content::{ActionKind, Affliction, ContentFiles, StatusId};
use crate::test_support::{run, said};

/// A breeder of its own kind at no cost, for the self-pool.
const BREEDER: &str = r##"
[[creature]]
id = "zz_breeder"
level = 1
xp = 10
name = "breeder"
glyph = "b"
fg = "#8a6a9a"
faction = "undead"
tags = ["undead", "summoner"]
weight = 2000
sight = 10
stray = false
[creature.summon]
creatures = ["zz_breeder"]
count = "1"
cooldown = 6
most = 3
"##;

/// A lit hall: the player at (3, 2), `{c}` beside them east.
fn world_with(creature: &str, wield: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/zz_breeder.toml", BREEDER);
    files.insert(
        "levels/split_hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
             ###########\n#.........#\n#..@C.....#\n#.........#\n#.........#\n###########\n\
             '''\n[legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"C\" = {{ terrain = \"floor\", creature = \"{creature}\" }}\n"
        ),
    );
    files.insert(
        "scenarios/split_hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"split_hall\"\nseed = 5\nhp = 1000\n\
             wield = [\"{wield}\"]\n"
        ),
    );
    let content = Arc::new(Content::load(&files).expect("the fixture content loads"));
    World::new(content, "split_hall", Some(5)).expect("world")
}

fn of_kind<'w>(w: &'w World, kind: &str) -> Vec<&'w Actor> {
    let id = w.content().creature_id(kind).expect("a type");
    (w.state.actors.values())
        .filter(|a| a.creature == id)
        .collect()
}

fn first(w: &World, kind: &str) -> ActorId {
    of_kind(w, kind).first().expect("one of them").id
}

fn kind(w: &World, id: &str) -> Option<crate::content::DamageTypeId> {
    w.content().damage_type_id(id)
}

/// Says the news as an action would.
fn flush(w: &mut World) {
    for line in std::mem::take(&mut w.news) {
        w.say(line);
    }
}

#[test]
fn a_blade_cuts_it_in_two_down_to_twice_least_and_nothing_else_does() {
    let mut w = world_with("charnel_heap", "long_sword");
    let heap = first(&w, "charnel_heap");
    let set = |w: &mut World, hp| w.state.actors.get_mut(&heap).expect("heap").hp = hp;
    // Blunt, fire, acid, and no kind at all: one heap still.
    set(&mut w, 40);
    for k in ["bludgeoning", "fire", "acid"] {
        let k = kind(&w, k);
        w.split(heap, k);
    }
    w.split(heap, None);
    assert_eq!(of_kind(&w, "charnel_heap").len(), 1);
    // Below twice `least` (8): no split.
    set(&mut w, 15);
    w.split(heap, kind(&w, "slashing"));
    assert_eq!(of_kind(&w, "charnel_heap").len(), 1);
    // At 41: halves of 21 and 20, the new one beside it, a half.
    set(&mut w, 41);
    w.split(heap, kind(&w, "piercing"));
    let heaps = of_kind(&w, "charnel_heap");
    assert_eq!(heaps.len(), 2);
    let old = &w.state.actors[&heap];
    let new = heaps.iter().find(|a| a.id != heap).expect("the new half");
    assert_eq!((old.hp, new.hp), (21, 20));
    assert_eq!((old.split_from, new.split_from), (None, Some(Split::Half)));
    assert!(new.pos.chebyshev(old.pos) == 1, "beside it");
    assert_eq!(new.faction(w.content()), old.faction(w.content()));
    assert!(
        new.inventory.is_empty() && new.equipment.is_empty(),
        "no gear"
    );
}

#[test]
fn the_first_split_says_what_to_do_once() {
    let mut w = world_with("charnel_heap", "long_sword");
    run(&mut w, "wait");
    let heap = first(&w, "charnel_heap");
    w.state.actors.get_mut(&heap).expect("heap").hp = 80;
    w.split(heap, kind(&w, "slashing"));
    w.split(heap, kind(&w, "slashing"));
    flush(&mut w);
    let said = said(&w);
    let first = "The charnel heap comes apart, and both halves crawl.";
    let later = "The charnel heap splits in two.";
    assert_eq!(said.iter().filter(|l| *l == first).count(), 1, "{said:?}");
    assert_eq!(said.iter().filter(|l| *l == later).count(), 1, "{said:?}");
    assert!(w.state.progress.happened("split:charnel_heap"));
}

#[test]
fn a_sword_splits_the_heap_and_a_mace_never_does() {
    let mut sword = world_with("charnel_heap", "long_sword");
    for _ in 0..30 {
        run(&mut sword, "move:E");
        if of_kind(&sword, "charnel_heap").len() > 1 {
            break;
        }
    }
    assert!(
        of_kind(&sword, "charnel_heap").len() > 1,
        "the sword split it"
    );
    let mut mace = world_with("charnel_heap", "mace");
    for _ in 0..30 {
        run(&mut mace, "move:E");
        assert!(of_kind(&mace, "charnel_heap").len() <= 1, "the mace never");
    }
}

#[test]
fn a_halfs_kill_pays_a_quarter_and_counts_for_no_quest() {
    let mut w = world_with("charnel_heap", "long_sword");
    let heap = first(&w, "charnel_heap");
    assert_eq!(w.offspring_share(heap, 100), (100, true));
    w.state.actors.get_mut(&heap).expect("heap").hp = 60;
    w.split(heap, kind(&w, "slashing"));
    let half = (of_kind(&w, "charnel_heap").into_iter())
        .find(|a| a.id != heap)
        .expect("a half")
        .id;
    assert_eq!(w.offspring_share(half, 100), (25, false));
    assert_eq!(
        w.offspring_share(half, 50),
        (12, false),
        "an ally's share too"
    );
    // Killed: a quarter of what the whole pays, and the quest log's count unmoved.
    let xp = w.state.experience;
    let player = w.state.player;
    w.hurt(half, 1000, fight::Harm::Actor(player), &mut Vec::new());
    let half_paid = w.state.experience - xp;
    assert_eq!(w.state.quests.killed("charnel_heap"), 0);
    let xp = w.state.experience;
    w.hurt(heap, 1000, fight::Harm::Actor(player), &mut Vec::new());
    let whole_paid = w.state.experience - xp;
    assert_eq!(w.state.quests.killed("charnel_heap"), 1);
    // The half was the first kill, with the bonus; the whole is paid without one.
    assert!(half_paid > 0 && whole_paid > 0);
    let rules = w.content().experience();
    let def = w
        .content()
        .creature(w.content().creature_id("charnel_heap").expect("heap"));
    let gain = rules.kill_xp(def.kill_xp(), def.level, w.player_level());
    assert!(half_paid <= (gain + rules.first_kill_xp(def.kill_xp())) / 4 + 1);
}

#[test]
fn worms_crawl_out_of_a_dead_wormhost_on_its_side_paying_a_tenth() {
    let mut w = world_with("wormhost", "long_sword");
    run(&mut w, "wait");
    let host = first(&w, "wormhost");
    let undead = w.state.actors[&host].faction(w.content());
    let at = w.state.actors[&host].pos;
    let player = w.state.player;
    w.hurt(host, 1000, fight::Harm::Actor(player), &mut Vec::new());
    flush(&mut w);
    let worms = of_kind(&w, "grave_worm");
    assert!((2..=8).contains(&worms.len()), "{} worms", worms.len());
    for worm in &worms {
        assert_eq!(worm.faction(w.content()), undead, "of the wormhost's side");
        assert_eq!(worm.split_from, Some(Split::Corpse));
        assert!(worm.pos.chebyshev(at) <= 1, "within its spread");
    }
    let worm = worms[0].id;
    assert_eq!(w.offspring_share(worm, 100), (10, false));
    let said = said(&w);
    assert!(
        said.iter().any(|l| l.contains("out of the wormhost!")),
        "{said:?}"
    );
}

#[test]
fn dying_into_is_deterministic() {
    let worms = || {
        let mut w = world_with("wormhost", "long_sword");
        let host = first(&w, "wormhost");
        let player = w.state.player;
        w.hurt(host, 1000, fight::Harm::Actor(player), &mut Vec::new());
        w.state_hash()
    };
    assert_eq!(worms(), worms());
}

#[test]
fn the_wormlord_sheds_worms_up_to_its_cap() {
    let mut w = world_with("wormlord", "long_sword");
    let lord = first(&w, "wormlord");
    let mut most = 0;
    for _ in 0..120 {
        run(&mut w, "wait");
        let live = w.live_summons(lord);
        assert!(live <= 6, "{live} alive");
        most = most.max(live);
    }
    assert_eq!(most, 6, "it bred to its cap");
    assert!(
        (of_kind(&w, "grave_worm").iter()).all(|a| a.summoner == Some(lord)),
        "every worm its own"
    );
}

#[test]
fn a_breeder_breeds_its_own_kind_to_its_cap_at_no_cost() {
    let mut w = world_with("zz_breeder", "long_sword");
    let c = w.content();
    let me = c.creature_id("zz_breeder").expect("breeder");
    let summon = c.creature(me).summon.as_ref().expect("a summon");
    assert_eq!((summon.pool.clone(), summon.mana), (vec![me], 0));
    let first = first(&w, "zz_breeder");
    for _ in 0..40 {
        run(&mut w, "wait");
        assert!(w.live_summons(first) <= 3);
    }
    assert_eq!(w.live_summons(first), 3);
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect(id)
}

#[test]
fn engulfed_holds_hurts_blocks_and_ends_with_its_holder() {
    let mut w = world_with("charnel_heap", "long_sword");
    let heap = first(&w, "charnel_heap");
    let engulfed = status(&w, "engulfed");
    let def = w.content().status(engulfed);
    let hold = def.hold.expect("a hold");
    assert_eq!(
        (hold.difficulty, hold.turns.min(), hold.turns.max()),
        (14, 2, 8)
    );
    assert_eq!((def.tick, def.damage), (Some(6), 2));
    for kind in [
        ActionKind::Step,
        ActionKind::Shoot,
        ActionKind::Throw,
        ActionKind::Read,
    ] {
        assert!(def.blocks.contains(&kind), "{kind:?}");
    }
    let player = w.state.player;
    w.afflict(player, engulfed, 1, Some(heap), &mut Vec::new());
    assert!(w.state.player().effects.has(engulfed));
    // A tick inside costs hit points.
    let hp = w.state.player().hp;
    // Pinned by the heap's mass: a wait passes a turn of ticks.
    w.state.actors.get_mut(&heap).expect("heap").energy = i64::MIN / 4;
    run(&mut w, "wait");
    assert!(w.state.player().hp < hp, "it hurt");
    // The holder dies: it lets go.
    w.afflict(player, engulfed, 1, Some(heap), &mut Vec::new());
    w.hurt(heap, 1000, fight::Harm::Actor(player), &mut Vec::new());
    run(&mut w, "wait");
    assert!(!w.state.player().effects.has(engulfed), "free once it died");
}

#[test]
fn the_engulfers_carry_their_statuses() {
    let w = world_with("charnel_heap", "long_sword");
    let c = w.content();
    let rider = |creature: &str, status: &str| {
        let def = c.creature(c.creature_id(creature).expect(creature));
        let s = c.status_id(status).expect(status);
        let attack = def.attack.as_ref().expect("an attack");
        (attack.on_hit.iter()).any(|o| match o.what {
            Affliction::Status { status, .. } | Affliction::Hold { status, .. } => status == s,
            _ => false,
        })
    };
    for creature in [
        "charnel_heap",
        "greater_water_elemental",
        "elder_water_elemental",
        "brimwolf",
    ] {
        assert!(rider(creature, "engulfed"), "{creature}");
        assert!(!rider(creature, "held"), "{creature} no longer holds");
    }
    assert!(rider("nightworm", "swallowed") && !rider("nightworm", "held"));
    assert!(rider("giant_leech", "latched"));
    assert!(rider("canoloth", "latched") && !rider("canoloth", "held"));
    let latched = c.status(c.status_id("latched").expect("latched"));
    assert_eq!((latched.tick, latched.damage), (Some(12), 1));
    let swallowed = c.status(c.status_id("swallowed").expect("swallowed"));
    assert_eq!(swallowed.damage, 4);
    // The bearers of splitting and dying into.
    let heap = c.creature(c.creature_id("charnel_heap").expect("heap"));
    assert_eq!(heap.splits.as_ref().map(|s| s.least), Some(8));
    let worm = c.creature_id("grave_worm").expect("worm");
    for (creature, count) in [("wormhost", (2, 8)), ("wormlord", (3, 18))] {
        let def = c.creature(c.creature_id(creature).expect(creature));
        let into = def.dies_into.as_ref().expect(creature);
        assert_eq!(into.pool, vec![worm]);
        assert_eq!((into.count.min(), into.count.max()), count);
        assert!(def.dread, "{creature} is dread");
    }
    let lord = c.creature(c.creature_id("wormlord").expect("lord"));
    let breeds = lord.summon.as_ref().expect("it breeds");
    assert_eq!(
        (breeds.pool.clone(), breeds.most, breeds.mana),
        (vec![worm], 6, 0)
    );
}

#[test]
fn bad_splits_and_dies_into_are_refused() {
    for (text, says) in [
        ("splits = { on = [\"slashing\"], least = 0 }", "`least`"),
        (
            "splits = { on = [\"sharp\"], least = 8 }",
            "unknown damage type",
        ),
        (
            "dies_into = { creatures = [\"nobody\"], count = \"1\" }",
            "unknown creature",
        ),
        (
            "dies_into = { creatures = [\"player\"], count = \"1\" }",
            "cannot crawl out",
        ),
    ] {
        let mut files = ContentFiles::builtin();
        files.insert(
            "data/creatures/zz_bad.toml",
            &format!(
                "[[creature]]\nid = \"zz_bad\"\nname = \"bad\"\nglyph = \"b\"\nfg = \"#ffffff\"\n\
                 faction = \"vermin\"\nweight = 1000\n{text}\n"
            ),
        );
        let err = Content::load(&files).expect_err(text).to_string();
        assert!(err.contains(says), "{text}: {err}");
    }
}

#[test]
fn halves_and_crawlers_survive_a_save() {
    let mut w = world_with("charnel_heap", "long_sword");
    let heap = first(&w, "charnel_heap");
    w.state.actors.get_mut(&heap).expect("heap").hp = 60;
    w.split(heap, kind(&w, "slashing"));
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert!((loaded.state.actors.values()).any(|a| a.split_from == Some(Split::Half)));
}

#[test]
fn heads_crawl_out_of_a_dead_hydra() {
    // #1138: the hydra's `dies_into`, its heads of its side.
    let mut w = world_with("hydra", "long_sword");
    let hydra = first(&w, "hydra");
    let marsh = w.state.actors[&hydra].faction(w.content());
    let player = w.state.player;
    w.hurt(hydra, 1000, fight::Harm::Actor(player), &mut Vec::new());
    let heads = of_kind(&w, "hydra_head");
    assert!((1..=4).contains(&heads.len()), "{} heads", heads.len());
    for head in &heads {
        assert_eq!(head.faction(w.content()), marsh, "of the hydra's side");
    }
}
