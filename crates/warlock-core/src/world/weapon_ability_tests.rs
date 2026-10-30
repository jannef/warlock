//! Weapon abilities in the world (the combat progression design §4, §6; #561): each triggered
//! ability on a hand-made fight, for the player and for creatures with the rank alike, the abuse
//! rules (Rend once an attack and never on a bash, Staggered refreshed never stacked, Pinned
//! keeping the longer, Deep cut to bleeding's max, one riposte from the style and a switch
//! together, Light-footed spent by one action), no draw while no rank qualifies, the three
//! talents, the menu's rows, and the news of a rank that teaches one.

use super::weapon_ability::Fired;
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::AttackKind;
use crate::combat::{Category, Strike};
use crate::content::{ContentFiles, Trigger};
use crate::equipment;
use crate::weapon_skills::WeaponSkills;

// Sure blades that do a point or two, a cleaver that takes a third of a post at once, a foil that
// parries nearly everything, a staff that threatens on every roll, a shield to block and bash.
const OBJECTS: &str = r##"
[[object]]
id = "wa_knife"
name = "test knife"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 300
value = 0
slot = "main_hand"
light_weapon = true
damage = "1d2"
to_hit = 20
category = "long_blades"
damage_type = "slashing"

[[object]]
id = "wa_cleaver"
name = "test cleaver"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 1000
value = 0
slot = "main_hand"
damage = "1d2+38"
to_hit = 20
category = "long_blades"
damage_type = "slashing"

[[object]]
id = "wa_axe"
name = "test axe"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 1000
value = 0
slot = "main_hand"
damage = "1d2+38"
to_hit = 20
category = "axes"
damage_type = "slashing"

[[object]]
id = "wa_foil"
name = "test foil"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 500
value = 0
slot = "main_hand"
damage = "1d2"
parry = 20
category = "long_blades"
damage_type = "piercing"

[[object]]
id = "wa_staff"
name = "test staff"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["wood"]
weight = 1500
value = 0
slot = "main_hand"
two_handed = true
damage = "1d2"
to_hit = 20
threat = 2
category = "staves"
damage_type = "bludgeoning"

[[object]]
id = "wa_shield"
name = "test shield"
glyph = "["
fg = "#c0c0c0"
class = "shields"
materials = ["wood"]
weight = 2500
value = 0
slot = "off_hand"
shield = 20
bash = "1d2"
"##;

// A post (the player's target), a spearman, and an archer that only stand (`world` keeps them
// from acting), and a brute that fights.
const CREATURES: &str = r##"
[[creature]]
id = "wa_post"
name = "post"
glyph = "p"
fg = "#806040"
weight = 60000
faction = "vermin"
speed = 100
base_hp = 100

[[creature]]
id = "wa_brute"
name = "brute"
glyph = "b"
fg = "#806040"
weight = 60000
faction = "vermin"
base_hp = 1000
attack = { damage = "1d2", damage_type = "bludgeoning", name = "fist", verb = "hits", to_hit = 2 }

[[creature]]
id = "wa_spearman"
name = "spearman"
glyph = "s"
fg = "#806040"
weight = 60000
faction = "vermin"
speed = 100
base_hp = 1000
weapon_skills = { spears = 5 }

[[creature]]
id = "wa_novice"
name = "novice"
glyph = "n"
fg = "#806040"
weight = 60000
faction = "vermin"
speed = 100
base_hp = 1000
weapon_skills = { spears = 4 }

[[creature]]
id = "wa_archer"
name = "archer"
glyph = "a"
fg = "#806040"
weight = 60000
faction = "vermin"
speed = 100
base_hp = 1000
weapon_skills = { bows = 5 }
"##;

fn level(map: &str) -> String {
    format!(
        r##"name = "Yard"
ambient = [255, 255, 255, 0]
map = '''
{map}'''
[legend]
"@" = {{ terrain = "floor", spawn = "player" }}
"p" = {{ terrain = "floor", creature = "wa_post" }}
"b" = {{ terrain = "floor", creature = "wa_brute" }}
"s" = {{ terrain = "floor", creature = "wa_spearman", carry = ["spear"], wield = "spear" }}
"n" = {{ terrain = "floor", creature = "wa_novice", carry = ["spear"], wield = "spear" }}
"a" = {{ terrain = "floor", creature = "wa_archer", carry = ["short_bow", "arrow*5"], wield = ["short_bow", "arrow"] }}
"A" = {{ terrain = "floor", creature = "wa_archer", carry = ["short_bow"], wield = "short_bow" }}
"##
    )
}

/// The game's content with the test objects, creatures, and `map`; `abilities` false leaves the
/// weapon abilities out (every file emptied).
fn content(map: &str, abilities: bool) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("data/objects/zz_wa_test.toml", OBJECTS);
    files.insert("data/creatures/zz_wa_test.toml", CREATURES);
    files.insert("levels/wa_yard.toml", &level(map));
    files.insert(
        "scenarios/wa_yard.toml",
        "name = \"Yard\"\nstart_level = \"wa_yard\"\nhp = 1000\n",
    );
    if !abilities {
        let paths: Vec<String> = (files.table("weapon_abilities"))
            .map(|(p, _)| p.to_string())
            .collect();
        for path in paths {
            files.insert(&path, "");
        }
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
}

/// A world on `map` where the posts, spearmen, and archers never get to act: they only stand
/// (what they do comes of the triggers alone).
fn world(map: &str) -> World {
    let mut w = World::new(content(map, true), "wa_yard", None).expect("world");
    let content = w.content().clone();
    for actor in w.state.actors.values_mut() {
        if content.creature(actor.creature).speed == 100 {
            actor.energy = -1_000_000_000;
        }
    }
    w
}

/// Actor `id` holds `object` in a free slot it fits.
fn give(w: &mut World, id: ActorId, object: &str) {
    let content = w.content().clone();
    let me = w.state.actors.get_mut(&id).expect("the actor");
    let object = content.object_id(object).expect("an object");
    me.inventory.push(crate::object::Stack::new(object, 1));
    let item = me.inventory.len() - 1;
    let slot = equipment::free_slot(&content, &me.equipment, object).expect("a free slot");
    me.equipment
        .put(&equipment::slots_taken(&content, object, slot), item);
}

/// Actor `id`'s ranks are exactly `ranks` (and their defaults).
fn ranks(w: &mut World, id: ActorId, ranks: &[(&str, u8)]) {
    let content = w.content().clone();
    let table = content.weapon_skills();
    let ranks = ranks.iter().map(|&(s, r)| (table.id(s).expect(s), r));
    w.state
        .actors
        .get_mut(&id)
        .expect("the actor")
        .weapon_skills = WeaponSkills::from_ranks(table, ranks);
}

fn at(w: &World, x: i32, y: i32) -> ActorId {
    let level = &w.state.current().id;
    w.state.actor_at(level, Pos::new(x, y)).expect("someone").id
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn severity(w: &World, id: ActorId, status: &str) -> u16 {
    let status = w.content().status_id(status).expect(status);
    (w.state.actors.get(&id))
        .and_then(|a| a.effects.get(status))
        .map_or(0, |e| e.severity)
}

fn hits(outcome: &Outcome, by: ActorId, kind: AttackKind) -> usize {
    (outcome.events.iter())
        .filter(|e| {
            matches!(e, Event::Attacked { attacker, kind: k, strike: Some(s), .. }
                if *attacker == by && *k == kind && s.is_hit())
        })
        .count()
}

fn attacks(outcome: &Outcome, by: ActorId, kind: AttackKind) -> usize {
    (outcome.events.iter())
        .filter(|e| matches!(e, Event::Attacked { attacker, kind: k, .. } if *attacker == by && *k == kind))
        .count()
}

#[test]
fn rend_fires_once_an_attack_with_both_blades_and_never_on_a_bash() {
    let mut w = world("#####\n#@p.#\n#####\n");
    let (player, post) = (w.player_id(), at(&w, 2, 1));
    give(&mut w, player, "wa_knife");
    give(&mut w, player, "wa_knife");
    ranks(&mut w, player, &[("long_blades", 1), ("twin_blades", 4)]);
    for _ in 0..4 {
        let before = said(&w).len();
        let hp = w.state.actors[&post].hp;
        let outcome = w.apply(&Action::Move(Dir::E));
        let both = hits(&outcome, player, AttackKind::Melee) == 2;
        let rends = (said(&w)[before..].iter())
            .filter(|m| m.contains("Your blades rend the post."))
            .count();
        assert_eq!(rends, usize::from(both), "{:#?}", &said(&w)[before..]);
        // Rend is one roll of the main hand's dice on top of the blows.
        let lost = hp - w.state.actors[&post].hp;
        assert!(!both || lost >= 3, "two blows and the rend: {lost}");
    }
    // Below the rank: no rend.
    let mut w = world("#####\n#@p.#\n#####\n");
    give(&mut w, player, "wa_knife");
    give(&mut w, player, "wa_knife");
    ranks(&mut w, player, &[("long_blades", 1), ("twin_blades", 3)]);
    for _ in 0..3 {
        w.apply(&Action::Move(Dir::E));
    }
    assert!(!said(&w).iter().any(|m| m.contains("rend")));
    // A blade and a shield, even at Twin Blades 10: a bash is no second hand.
    let mut w = world("#####\n#@p.#\n#####\n");
    give(&mut w, player, "wa_knife");
    give(&mut w, player, "wa_shield");
    ranks(
        &mut w,
        player,
        &[
            ("long_blades", 1),
            ("twin_blades", 10),
            ("sword_and_board", 10),
        ],
    );
    for _ in 0..6 {
        w.apply(&Action::Move(Dir::E));
    }
    assert!(!said(&w).iter().any(|m| m.contains("rend")));
}

#[test]
fn second_blades_rend_rolls_the_off_hand_too() {
    let lost = |talent: bool| {
        let mut w = world("#####\n#@p.#\n#####\n");
        let (player, post) = (w.player_id(), at(&w, 2, 1));
        give(&mut w, player, "wa_knife");
        give(&mut w, player, "wa_knife");
        ranks(&mut w, player, &[("long_blades", 1), ("twin_blades", 6)]);
        if talent {
            w.player_mut().talents.take("second_blades_rend", None);
        }
        let hp = w.state.actors[&post].hp;
        let outcome = w.apply(&Action::Move(Dir::E));
        (
            hits(&outcome, player, AttackKind::Melee),
            hp - w.state.actors[&post].hp,
        )
    };
    let (plain, with) = (lost(false), lost(true));
    if plain.0 == 2 && with.0 == 2 {
        assert!(
            with.1 > plain.1,
            "the off hand's dice more: {plain:?} {with:?}"
        );
    }
}

#[test]
fn staggering_block_staggers_the_striker_and_only_refreshes() {
    let mut w = world("#####\n#@b.#\n#####\n");
    let (player, brute) = (w.player_id(), at(&w, 2, 1));
    give(&mut w, player, "wa_knife");
    give(&mut w, player, "wa_shield");
    ranks(
        &mut w,
        player,
        &[("long_blades", 1), ("sword_and_board", 4)],
    );
    for _ in 0..12 {
        w.apply(&Action::Wait);
        assert!(severity(&w, brute, "staggered") <= 1, "one degree at most");
    }
    let blocked = (said(&w).iter())
        .filter(|m| m.starts_with("You block"))
        .count();
    let staggers = (said(&w).iter())
        .filter(|m| *m == "The brute staggers from your shield.")
        .count();
    assert!(blocked > 0 && staggers == blocked, "{:#?}", said(&w));
    // Twice in one action: still one degree, kept for one slowed action.
    let staggered = w.content().status_id("staggered").expect("staggered");
    let mut events = Vec::new();
    w.afflict(brute, staggered, 1, Some(player), &mut events);
    w.afflict(brute, staggered, 1, Some(player), &mut events);
    assert_eq!(severity(&w, brute, "staggered"), 1);
    // Without the style's rank: never.
    let mut w = world("#####\n#@b.#\n#####\n");
    give(&mut w, player, "wa_knife");
    give(&mut w, player, "wa_shield");
    ranks(
        &mut w,
        player,
        &[("long_blades", 1), ("sword_and_board", 3)],
    );
    for _ in 0..12 {
        w.apply(&Action::Wait);
        assert_eq!(severity(&w, brute, "staggered"), 0);
    }
}

#[test]
fn pinned_keeps_the_longer_and_blocks_a_step() {
    let mut w = world("#####\n#@.p#\n#####\n");
    let player = w.player_id();
    let pinned = w.content().status_id("pinned").expect("pinned");
    let mut events = Vec::new();
    w.afflict(player, pinned, 3, None, &mut events);
    w.afflict(player, pinned, 1, None, &mut events);
    assert_eq!(severity(&w, player, "pinned"), 3, "the longer kept");
    w.afflict(player, pinned, 2, None, &mut events);
    assert_eq!(severity(&w, player, "pinned"), 3, "never added");
    let from = w.player_pos();
    w.apply(&Action::Move(Dir::E));
    assert_eq!(w.player_pos(), from, "pinned where they stand");
}

#[test]
fn deep_cut_bleeds_more_on_a_big_hit_up_to_the_max() {
    let bled = |rank: u8, before: u16| {
        let mut w = world("#####\n#@p.#\n#####\n");
        let (player, post) = (w.player_id(), at(&w, 2, 1));
        give(&mut w, player, "wa_cleaver");
        ranks(&mut w, player, &[("long_blades", rank)]);
        if before > 0 {
            let bleeding = w.content().status_id("bleeding").expect("bleeding");
            w.afflict(post, bleeding, before, None, &mut Vec::new());
        }
        let outcome = w.apply(&Action::Move(Dir::E));
        assert_eq!(hits(&outcome, player, AttackKind::Melee), 1);
        severity(&w, post, "bleeding")
    };
    assert_eq!(bled(5, 0), bled(4, 0) + 1, "a degree more");
    assert_eq!(bled(5, 5), 5, "never past bleeding's max");
}

/// Actor `target` is Surprised (a sneak attack's second door: it need not be unaware).
fn surprise(w: &mut World, target: ActorId) {
    let surprised = w.content().status_id("surprised").expect("surprised");
    w.afflict(target, surprised, 1, None, &mut Vec::new());
}

/// The player, with `sneak` in Sneak attack and `ranks`, attacks the post `ahead` by `action`
/// (a blow or a shot), the post Surprised before each attack when `surprised`, until one hits:
/// the post's `status` then.
fn sneak_status(
    (map, ahead): (&str, (i32, i32)),
    (sneak, ranks_of): (u16, &[(&str, u8)]),
    (action, surprised, status): (Action, bool, &str),
) -> u16 {
    let mut w = world(map);
    let (player, post) = (w.player_id(), at(&w, ahead.0, ahead.1));
    if matches!(action, Action::Fire(_)) {
        give(&mut w, player, "short_bow");
        let arrow = w.content().object_id("arrow").expect("arrow");
        let me = w.player_mut();
        me.inventory.push(crate::object::Stack::new(arrow, 40));
        let item = me.inventory.len() - 1;
        let content = w.content().clone();
        let me = w.player_mut();
        let slot = equipment::free_slot(&content, &me.equipment, arrow).expect("the quiver");
        me.equipment
            .put(&equipment::slots_taken(&content, arrow, slot), item);
    } else {
        give(&mut w, player, "dagger");
    }
    ranks(&mut w, player, ranks_of);
    super::vitals_tests::train(&mut w, "sneak_attack", sneak);
    let kind = match action {
        Action::Fire(_) => AttackKind::Ranged,
        _ => AttackKind::Melee,
    };
    for _ in 0..40 {
        if surprised {
            surprise(&mut w, post);
        }
        let outcome = w.apply(&action);
        if hits(&outcome, player, kind) == 1 {
            return severity(&w, post, status);
        }
    }
    panic!("no hit in forty attacks");
}

#[test]
fn knife_in_the_dark_bleeds_a_foe_a_dagger_sneak_attacks() {
    let bled = |rank: u8, sneak: u16, surprised: bool| {
        let fight = ("#####\n#@p.#\n#####\n", (2, 1));
        let ranks = [("daggers", rank)];
        sneak_status(
            fight,
            (sneak, &ranks),
            (Action::Move(Dir::E), surprised, "bleeding"),
        )
    };
    assert_eq!(bled(5, 10, true), bled(4, 10, true) + 1, "the vein found");
    // Only when the dice land: not on a foe that is ready, nor without the dice.
    assert_eq!(bled(5, 10, false), bled(4, 10, false));
    assert_eq!(bled(5, 0, true), bled(4, 0, true));
}

#[test]
fn hunters_shot_pins_a_foe_a_shot_sneak_attacks() {
    let pinned = |rank: u8, sneak: u16, surprised: bool| {
        let fight = ("#######\n#@..p.#\n#######\n", (4, 1));
        let ranks = [("bows", 1), ("marksman", rank)];
        sneak_status(
            fight,
            (sneak, &ranks),
            (Action::Fire(Pos::new(4, 1)), surprised, "pinned"),
        )
    };
    let landed = pinned(6, 10, true);
    assert!((1..=2).contains(&landed), "pinned a turn or two: {landed}");
    assert_eq!(pinned(5, 10, true), 0, "not below the rank");
    assert_eq!(pinned(6, 10, false), 0, "not on a foe that is ready");
    assert_eq!(pinned(6, 0, true), 0, "not without the dice");
}

#[test]
fn the_sneak_attack_trigger_draws_nothing_without_the_rank() {
    // A sneak attack with the dice but no qualifying rank: the same draws with and without
    // the abilities' data.
    let run = |abilities: bool| {
        let content = content("#####\n#@p.#\n#####\n", abilities);
        let mut w = World::new(content, "wa_yard", None).expect("world");
        let (player, post) = (w.player_id(), at(&w, 2, 1));
        give(&mut w, player, "dagger");
        ranks(&mut w, player, &[("daggers", 4)]);
        super::vitals_tests::train(&mut w, "sneak_attack", 10);
        let mut hashes = Vec::new();
        for _ in 0..6 {
            surprise(&mut w, post);
            w.apply(&Action::Move(Dir::E));
            hashes.push(w.state_hash());
        }
        hashes
    };
    assert_eq!(run(true), run(false));
}

#[test]
fn crushing_blow_stunning_bash_stunning_fist_and_trip_land_their_statuses() {
    // Crushing blow: a big hit with an axe.
    let mut w = world("#####\n#@p.#\n#####\n");
    let (player, post) = (w.player_id(), at(&w, 2, 1));
    give(&mut w, player, "wa_axe");
    ranks(&mut w, player, &[("axes", 5)]);
    w.apply(&Action::Move(Dir::E));
    assert_eq!(severity(&w, post, "staggered"), 1);
    // Stunning bash: a bash that hits.
    let mut w = world("#####\n#@p.#\n#####\n");
    give(&mut w, player, "wa_shield");
    ranks(&mut w, player, &[("shield_bash", 5)]);
    let mut landed = false;
    for _ in 0..8 {
        let outcome = w.apply(&Action::Bash(Dir::E));
        if hits(&outcome, player, AttackKind::Melee) > 0 {
            landed = true;
            break;
        }
    }
    assert!(landed && severity(&w, post, "staggered") == 1);
    // Stunning fist and Trip, on a critical: fired as the critical line fires them.
    let fired = |w: &World, skill: &str, style: &str| Fired {
        other: post,
        kind: AttackKind::Melee,
        skills: AttackSkills {
            skill: w.content().weapon_skills().id(skill),
            style: w.content().weapon_skills().id(style),
        },
        blow: None,
    };
    let mut w = world("#####\n#@p.#\n#####\n");
    ranks(&mut w, player, &[("brawler", 4)]);
    let f = fired(&w, "unarmed", "brawler");
    w.weapon_trigger(player, Trigger::Critical, f, &mut Vec::new());
    assert_eq!(severity(&w, post, "staggered"), 1);
    let mut w = world("#####\n#@p.#\n#####\n");
    ranks(&mut w, player, &[("staff_fighter", 4)]);
    let f = fired(&w, "staves", "staff_fighter");
    w.weapon_trigger(player, Trigger::Critical, f, &mut Vec::new());
    assert_eq!(severity(&w, post, "pinned"), 1, "a turn");
    // Trip through a real critical: a staff that threatens on every roll.
    let mut w = world("#####\n#@p.#\n#####\n");
    give(&mut w, player, "wa_staff");
    ranks(&mut w, player, &[("staves", 1), ("staff_fighter", 4)]);
    let mut pinned = false;
    for _ in 0..6 {
        w.apply(&Action::Move(Dir::E));
        pinned |= severity(&w, post, "pinned") > 0;
    }
    assert!(pinned);
    // Pinning shot: 1 to 3 turns, one draw.
    let mut w = world("#####\n#@p.#\n#####\n");
    ranks(&mut w, player, &[("marksman", 4)]);
    let f = fired(&w, "bows", "marksman");
    w.weapon_trigger(player, Trigger::Critical, f, &mut Vec::new());
    assert!((1..=3).contains(&severity(&w, post, "pinned")));
}

#[test]
fn riposte_from_the_style_and_a_switch_together_is_one() {
    let mut w = world("#####\n#@b.#\n#####\n");
    let (player, brute) = (w.player_id(), at(&w, 2, 1));
    give(&mut w, player, "wa_foil");
    ranks(&mut w, player, &[("long_blades", 1), ("duelist", 4)]);
    w.player_mut().free_blows.riposte = true;
    let mut answered = 0;
    for _ in 0..12 {
        let outcome = w.apply(&Action::Wait);
        // Each parried attack of the brute's earns one blow back, never two.
        let parried = (outcome.events.iter())
            .filter(|e| matches!(e, Event::Attacked { attacker, strike: Some(Strike::Missed { by: Some(Category::Parry) }), .. } if *attacker == brute))
            .count();
        let back = attacks(&outcome, player, AttackKind::Melee);
        assert!(back <= parried, "{back} blows back for {parried} parries");
        answered += back;
    }
    assert!(answered > 0, "{:#?}", said(&w));
}

#[test]
fn a_creature_with_the_rank_braces_and_one_without_does_not() {
    let mut w = world("#######\n#@.s..#\n#######\n");
    let spearman = at(&w, 3, 1);
    let outcome = w.apply(&Action::Move(Dir::E));
    assert_eq!(attacks(&outcome, spearman, AttackKind::Melee), 1);
    let mut w = world("#######\n#@.n..#\n#######\n");
    let novice = at(&w, 3, 1);
    let outcome = w.apply(&Action::Move(Dir::E));
    assert_eq!(attacks(&outcome, novice, AttackKind::Melee), 0);
}

#[test]
fn brace_of_steel_staggers_a_braced_foe() {
    // The player braces with a spear: the brute steps up and is struck, and staggers on a hit.
    let mut w = world("#######\n#@..b.#\n#######\n");
    let (player, brute) = (w.player_id(), at(&w, 4, 1));
    give(&mut w, player, "spear");
    ranks(&mut w, player, &[("spears", 5)]);
    w.player_mut().talents.take("brace_of_steel", None);
    let mut staggered = false;
    for _ in 0..4 {
        let outcome = w.apply(&Action::Wait);
        let struck = (outcome.events.iter()).any(|e| {
            matches!(e, Event::Attacked { attacker, target, strike: Some(s), .. }
                if *attacker == player && *target == brute && s.is_hit())
        });
        staggered |= struck && severity(&w, brute, "staggered") == 1;
    }
    assert!(staggered, "{:#?}", said(&w));
}

#[test]
fn point_blank_shoots_only_with_ammunition_ready() {
    let mut w = world("#######\n#@.a..#\n#######\n");
    let archer = at(&w, 3, 1);
    let outcome = w.apply(&Action::Move(Dir::E));
    assert_eq!(attacks(&outcome, archer, AttackKind::Ranged), 1);
    // No arrows: nothing, and nothing drawn.
    let mut w = world("#######\n#@.A..#\n#######\n");
    let archer = at(&w, 3, 1);
    let outcome = w.apply(&Action::Move(Dir::E));
    assert_eq!(attacks(&outcome, archer, AttackKind::Ranged), 0);
}

#[test]
fn hit_and_run_halves_the_next_action_once() {
    let mut w = world("########\n#@...p.#\n########\n");
    let player = w.player_id();
    give(&mut w, player, "javelin");
    ranks(&mut w, player, &[("thrown", 1), ("skirmisher", 4)]);
    let target = Pos::new(5, 1);
    let mut light = false;
    for _ in 0..6 {
        let item = w.state.player().inventory.len() - 1;
        let outcome = w.apply(&Action::Throw { item, at: target });
        if hits(&outcome, player, AttackKind::Thrown) == 1 {
            light = true;
            break;
        }
        // Fetch it back for another throw.
        let javelin = w.content().object_id("javelin").expect("javelin");
        (w.player_mut().inventory).push(crate::object::Stack::new(javelin, 1));
    }
    assert!(light, "{:#?}", said(&w));
    assert_eq!(
        severity(&w, player, "light_footed"),
        1,
        "given, not spent by the throw"
    );
    let quick = w.apply(&Action::Wait).elapsed;
    assert_eq!(
        severity(&w, player, "light_footed"),
        0,
        "spent by one action"
    );
    let slow = w.apply(&Action::Wait).elapsed;
    assert_eq!(quick * 2, slow, "the next action costs half");
}

#[test]
fn measure_parries_shots_and_throws() {
    let parry = |rank: u8| {
        let mut w = world("#####\n#@p.#\n#####\n");
        let player = w.player_id();
        give(&mut w, player, "wa_foil");
        ranks(&mut w, player, &[("long_blades", 1), ("duelist", rank)]);
        (
            w.target_against(player, false)
                .defence
                .bonus(Category::Parry),
            w.target_against(player, true)
                .defence
                .bonus(Category::Parry),
        )
    };
    let (blows, missiles) = parry(9);
    assert!(blows > 0);
    assert_eq!(missiles, 0, "a blade parries no arrow");
    let (blows, missiles) = parry(10);
    assert!(blows > 0);
    assert_eq!(missiles, blows, "Measure: it does");
}

#[test]
fn no_draws_while_no_rank_qualifies() {
    let run = |abilities: bool| {
        let content = content("#######\n#@b...#\n#######\n", abilities);
        let mut w = World::new(content, "wa_yard", None).expect("world");
        let player = w.player_id();
        give(&mut w, player, "wa_knife");
        give(&mut w, player, "wa_shield");
        ranks(
            &mut w,
            player,
            &[("long_blades", 4), ("sword_and_board", 3)],
        );
        let mut hashes = Vec::new();
        for action in [Action::Move(Dir::E), Action::Wait, Action::Bash(Dir::E)] {
            for _ in 0..3 {
                w.apply(&action);
                hashes.push(w.state_hash());
            }
        }
        hashes
    };
    assert_eq!(run(true), run(false));
}

#[test]
fn early_mastery_teaches_a_rank_sooner() {
    let mut w = world("#####\n#@p.#\n#####\n");
    let player = w.player_id();
    ranks(&mut w, player, &[("long_blades", 4)]);
    let content = w.content().clone();
    let deep = content.weapon_ability_id("deep_cut").expect("deep cut");
    let knows =
        |w: &World| (w.state.player().weapon_abilities(&content).iter()).any(|(_, a)| *a == deep);
    assert!(!knows(&w));
    w.player_mut().talents.take("early_mastery", None);
    assert!(knows(&w));
}

#[test]
fn the_menu_lists_what_ranks_teach_greyed_as_firing_by_itself() {
    let mut w = world("#####\n#@p.#\n#####\n");
    let content = w.content().clone();
    w.apply(&Action::SetRank {
        skill: "long_blades".into(),
        rank: 5,
    });
    assert!(
        (said(&w).iter())
            .any(|m| m == "You feel more skilled with long blades. You have learned the deep cut."),
        "{:#?}",
        said(&w)
    );
    let mine = w.perception().abilities();
    let deep = (mine.iter())
        .find(|a| matches!(a.ability, Ability::Weapon(..)))
        .expect("a weapon ability");
    assert_eq!(deep.ability.name(&content), "Long blades: Deep cut");
    assert_eq!(deep.unavailable, Some(Unavailable::FiresByItself));
    assert_eq!(deep.ability.action(&content, Some(Dir::E)), None);
    // Listed once, under the first skill that teaches it.
    let rows = mine
        .iter()
        .filter(|a| a.ability.id(&content) == "deep_cut")
        .count();
    assert_eq!(rows, 1);
}

// Spell reflection (#566): Sword and Board's capstone turns a spell its shield blocked back.

/// The game's content with the test objects and the wizard's Arcane bolt as `bolt` says (its
/// shape, resolution, and dice); `abilities` false leaves the weapon abilities out.
fn spell_content(bolt: &str, abilities: bool) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("data/objects/zz_wa_test.toml", OBJECTS);
    let text = format!(
        "[[spell]]\nid = \"arcane_bolt\"\nname = \"Arcane bolt\"\ndescription = \"x\"\n\
         level = 6\nschool = \"evocation\"\nkind = \"damage\"\ndamage_type = \"force\"\n{bolt}\n"
    );
    files.insert("data/spells/arcane_bolt.toml", &text);
    if !abilities {
        let paths: Vec<String> = (files.table("weapon_abilities"))
            .map(|(p, _)| p.to_string())
            .collect();
        for path in paths {
            files.insert(&path, "");
        }
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
}

/// An attack ray of tier 2 (cap 18), 3d6, its cap lifted by Intelligence and Willpower, aimed
/// with Intelligence.
const RAY: &str = "shape = \"ray\"\nresolution = \"attack\"\nmax_range = 6\n\
                   damage = { dice = \"3d6\" }\n[spell.scales]\n\
                   damage = { intelligence = 10, willpower = 5 }\n\
                   to_hit = { intelligence = 100 }";

/// A level-18 `class` (a wizard: Arcane bolt) with `content`, and a kobold two cells east,
/// holding a knife and a shield, with Sword and Board at `rank`; the kobold's id. No creature
/// acts.
fn reflector(content: Arc<Content>, class: &str, rank: u8) -> (World, ActorId) {
    let mut w = super::caster_tests::world_with(content, class);
    super::caster_tests::reach(&mut w, 18);
    let kobold = super::caster_tests::kobold(&mut w, 2, 10_000);
    give(&mut w, kobold, "wa_knife");
    give(&mut w, kobold, "wa_shield");
    ranks(
        &mut w,
        kobold,
        &[("long_blades", 1), ("sword_and_board", rank)],
    );
    (w, kobold)
}

/// Keeps every creature from acting (those come since too).
fn freeze(w: &mut World) {
    let player = w.player_id();
    for actor in w.state.actors.values_mut() {
        if actor.id != player {
            actor.energy = -1_000_000_000;
        }
    }
}

/// The player, hit points and mana topped up (and plenty of both: no hit is a big one), no
/// status on them, takes `action` aimed at the kobold's cell, no creature acting; the spell
/// blows of it: (attacker, target, strike).
fn cast_at_kobold(w: &mut World, kobold: ActorId, action: &str) -> Vec<(ActorId, ActorId, Strike)> {
    w.player_mut().hp_base = 10_000;
    w.player_mut().hp = 10_000;
    w.player_mut().effects = Default::default();
    w.player_mut().mana = 40;
    w.state.actors.get_mut(&kobold).expect("kobold").hp = 10_000;
    freeze(w);
    let at = w.state.actors[&kobold].pos;
    let action = format!("{action}:{},{}", at.x, at.y);
    let actions = crate::action::parse_actions(&action).expect("an action");
    let out = w.apply(&actions[0]);
    (out.events.iter())
        .filter_map(|e| match *e {
            Event::Attacked {
                attacker,
                target,
                kind: AttackKind::Spell,
                strike: Some(strike),
                ..
            } => Some((attacker, target, strike)),
            _ => None,
        })
        .collect()
}

const BLOCKED: Strike = Strike::Missed {
    by: Some(Category::Shield),
};

#[test]
fn a_blocked_spell_is_turned_back_once_a_turn_of_the_reflectors() {
    let (mut w, kobold) = reflector(spell_content(RAY, true), "wizard", 10);
    let player = w.player_id();
    let (mut blocked, mut turned) = (0, 0);
    for _ in 0..40 {
        let blows = cast_at_kobold(&mut w, kobold, "use:arcane_bolt");
        assert_eq!(blows[0].0, player, "{blows:?}");
        let back = blows.iter().filter(|b| b.0 == kobold).count();
        // The kobold never acts: one reflection, then none until its turn.
        if turned == 0 && blows[0].2 == BLOCKED {
            assert_eq!(back, 1, "turned back: {blows:?}");
            assert_eq!((blows[1].1, blows.len()), (player, 2), "at its caster");
            assert!(
                (said(&w).iter())
                    .any(|m| m == "The kobold's shield turns your arcane bolt back at you."),
                "{:#?}",
                said(&w)
            );
        } else {
            assert_eq!(back, 0, "once a turn: {blows:?}");
        }
        if blows[0].2 == BLOCKED {
            blocked += 1;
        }
        turned += back;
    }
    assert!(blocked > 1 && turned == 1, "{blocked} {turned}");
    // Its action spent, it turns the next blocked one back again.
    w.spend_actions(kobold, &mut Vec::new());
    let mut again = 0;
    for _ in 0..40 {
        let blows = cast_at_kobold(&mut w, kobold, "use:arcane_bolt");
        again += blows.iter().filter(|b| b.0 == kobold).count();
    }
    assert_eq!(again, 1);
    // A rank short: never.
    let (mut w, kobold) = reflector(spell_content(RAY, true), "wizard", 9);
    for _ in 0..20 {
        let blows = cast_at_kobold(&mut w, kobold, "use:arcane_bolt");
        assert!(blows.iter().all(|b| b.0 != kobold));
    }
}

#[test]
fn a_reflected_spell_is_never_reflected_again() {
    let (mut w, kobold) = reflector(spell_content(RAY, true), "wizard", 10);
    let player = w.player_id();
    // The player blocks with a shield and the same rank.
    w.player_mut().equipment = Default::default();
    give(&mut w, player, "wa_knife");
    give(&mut w, player, "wa_shield");
    ranks(
        &mut w,
        player,
        &[("long_blades", 1), ("sword_and_board", 10)],
    );
    let mut blocked_back = 0;
    for _ in 0..60 {
        w.spend_actions(kobold, &mut Vec::new());
        w.spend_actions(player, &mut Vec::new());
        let blows = cast_at_kobold(&mut w, kobold, "use:arcane_bolt");
        assert!(blows.len() <= 2, "no third flight: {blows:?}");
        if let [_, (by, at, strike)] = blows[..] {
            assert_eq!((by, at), (kobold, player));
            if strike == BLOCKED {
                blocked_back += 1;
            }
        }
    }
    assert!(
        blocked_back > 0,
        "the player's shield blocked a reflected one"
    );
}

#[test]
fn bursts_cones_lines_and_hexes_are_never_turned_back() {
    for shape in [
        "shape = \"burst\"\nresolution = \"area\"\nmax_range = 6\nradius = 1\nmax_radius = 1\n\
         damage = { dice = \"1d2\" }",
        "shape = \"cone\"\nresolution = \"area\"\nlength = 4\nmax_length = 4\n\
         damage = { dice = \"1d2\" }",
        "shape = \"line\"\nresolution = \"area\"\nlength = 4\nmax_length = 4\n\
         damage = { dice = \"1d2\" }",
    ] {
        let (mut w, kobold) = reflector(spell_content(shape, true), "wizard", 10);
        for _ in 0..10 {
            let blows = cast_at_kobold(&mut w, kobold, "use:arcane_bolt");
            assert!(!blows.is_empty(), "{shape}: it reached the kobold");
            assert!(blows.iter().all(|b| b.0 != kobold), "{shape}: {blows:?}");
        }
    }
    let (mut w, kobold) = reflector(spell_content(RAY, true), "witch", 10);
    for _ in 0..5 {
        let blows = cast_at_kobold(&mut w, kobold, "use:hex_of_weakness");
        assert!(blows.iter().all(|b| b.0 != kobold));
    }
    assert!(severity(&w, kobold, "weakened") > 0, "hexed");
    assert_eq!(severity(&w, w.player_id(), "weakened"), 0, "nothing back");
}

#[test]
fn a_miss_by_parry_is_not_turned_back() {
    // A touch (Parry counts against it), at a kobold with a foil and a shield.
    let touch = "shape = \"touch\"\nresolution = \"touch\"\ndamage = { dice = \"1d6\" }";
    let (mut w, kobold) = reflector(spell_content(touch, true), "wizard", 10);
    let content = w.content().clone();
    let me = w.state.actors.get_mut(&kobold).expect("kobold");
    me.inventory.clear();
    me.equipment = Default::default();
    give(&mut w, kobold, "wa_foil");
    give(&mut w, kobold, "wa_shield");
    let here = w.player_pos();
    w.state.actors.get_mut(&kobold).expect("kobold").pos = Pos::new(here.x + 1, here.y);
    *w.perception = crate::perception::Perception::compute(&w.state, &content, &mut w.lights);
    let (mut parried, mut blocked) = (0, 0);
    for _ in 0..80 {
        w.spend_actions(kobold, &mut Vec::new());
        let blows = cast_at_kobold(&mut w, kobold, "use:arcane_bolt");
        let back = blows.iter().filter(|b| b.0 == kobold).count();
        match blows[0].2 {
            BLOCKED => {
                blocked += 1;
                assert_eq!(back, 1, "{blows:?}");
            }
            Strike::Missed {
                by: Some(Category::Parry),
            } => {
                parried += 1;
                assert_eq!(back, 0, "{blows:?}");
            }
            _ => assert_eq!(back, 0, "{blows:?}"),
        }
    }
    assert!(parried > 0 && blocked > 0, "{parried} {blocked}");
}

#[test]
fn a_reflection_spends_no_more_mana_and_keeps_the_casters_lifted_dice() {
    let (mut w, kobold) = reflector(spell_content(RAY, true), "wizard", 10);
    let mut all = crate::stats::PerStat::splat(10);
    all[crate::stats::Stat::Intelligence] = 18;
    all[crate::stats::Stat::Willpower] = 14;
    w.player_mut().stats = crate::stats::Stats::new(all, crate::stats::PerStat::splat(40));
    let content = w.content().clone();
    let bolt = content.spell(content.spell_id("arcane_bolt").expect("bolt"));
    let me = w.state.player();
    let cost = me.spell_cost(&content, bolt);
    assert_eq!(me.spell_lift(&content, bolt), 50, "a die more on 3d6");
    let mut most = 0;
    for _ in 0..400 {
        w.spend_actions(kobold, &mut Vec::new());
        let blows = cast_at_kobold(&mut w, kobold, "use:arcane_bolt");
        // Paid once (a turn's regeneration may come back already).
        let mana = w.state.player().mana;
        assert!(
            mana >= 40 - cost && mana < 40,
            "paid once: {mana} {blows:?}"
        );
        if blows.iter().any(|b| b.0 == kobold) {
            let hurt = 10_000 - w.state.player().hp;
            most = most.max(hurt);
        }
    }
    assert!(most > 18, "past the unlifted dice: {most}");
}

#[test]
fn a_wands_bolt_is_turned_back() {
    let (mut w, kobold) = reflector(spell_content(RAY, true), "wizard", 10);
    let player = w.player_id();
    let content = w.content().clone();
    let wand = content.object_id("wand_of_scorching_ray").expect("a wand");
    let mut turned = 0;
    for _ in 0..3 {
        let me = w.player_mut();
        me.inventory.push(crate::object::Stack::new(wand, 1));
        let item = me.inventory.len() - 1;
        let equip = crate::action::parse_actions(&format!("equip:missile_weapon:{item}"));
        w.apply(&equip.expect("equip")[0]);
        for _ in 0..5 {
            w.spend_actions(kobold, &mut Vec::new());
            let blows = cast_at_kobold(&mut w, kobold, "fire");
            assert!(blows.first().is_some_and(|b| b.0 == player), "{blows:?}");
            turned += blows.iter().filter(|b| b.0 == kobold).count();
        }
    }
    assert!(turned > 0);
    assert!(
        (said(&w).iter()).any(|m| m == "The kobold's shield turns your scorching ray back at you."),
        "{:#?}",
        said(&w)
    );
}

#[test]
fn reflection_draws_nothing_without_a_qualifying_rank() {
    let run = |abilities: bool| {
        let (mut w, kobold) = reflector(spell_content(RAY, abilities), "wizard", 9);
        let mut hashes = Vec::new();
        for _ in 0..20 {
            cast_at_kobold(&mut w, kobold, "use:arcane_bolt");
            hashes.push(w.state_hash());
        }
        hashes
    };
    assert_eq!(run(true), run(false));
}

#[test]
fn a_creature_turns_the_players_wand_back_and_the_replay_verifies() {
    // A champion with a knife, a shield, and Sword and Board 10, who fights; the player zaps a
    // wand of scorching ray at it.
    let mut files = ContentFiles::builtin();
    files.insert("data/objects/zz_wa_test.toml", OBJECTS);
    files.insert(
        "data/creatures/zz_wa_champion.toml",
        "[[creature]]\nid = \"wa_champion\"\nname = \"champion\"\nglyph = \"C\"\n\
         fg = \"#806040\"\nweight = 60000\nfaction = \"vermin\"\nspeed = 1000\nbase_hp = 1000\n\
         weapon_skills = { long_blades = 1, sword_and_board = 10 }\n",
    );
    files.insert(
        "levels/wa_range.toml",
        "name = \"Range\"\nambient = [255, 255, 255, 0]\nmap = '''\n#######\n#@...C#\n\
         #######\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
         \"C\" = { terrain = \"floor\", creature = \"wa_champion\", \
         carry = [\"wa_knife\", \"wa_shield\"], wield = [\"wa_knife\", \"wa_shield\"] }\n",
    );
    files.insert(
        "scenarios/wa_range.toml",
        "name = \"Range\"\nstart_level = \"wa_range\"\nhp = 1000\n\
         carry = [\"wand_of_scorching_ray\"]\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content.clone(), "wa_range", Some(5)).expect("world");
    let champion = at(&w, 5, 1);
    let mut replay = crate::replay::Replay::new(&w);
    let mut apply = |w: &mut World, text: &str| {
        for action in crate::action::parse_actions(text).expect("actions") {
            w.apply(&action);
            replay.push(&action, w);
        }
    };
    apply(&mut w, "equip:missile_weapon:a");
    for _ in 0..10 {
        let Some(at) = w.state.actors.get(&champion).map(|a| a.pos) else {
            break;
        };
        apply(&mut w, &format!("fire:{},{}", at.x, at.y));
    }
    assert!(
        (said(&w).iter())
            .any(|m| m == "The champion's shield turns your scorching ray back at you."),
        "{:#?}",
        said(&w)
    );
    let verified = crate::replay::Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content)
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
