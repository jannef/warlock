//! Weapon skills in the world (spec §8 "Weapon skills"): attacks earn marks (more for a hit,
//! fewer as the rank rises), a rank gained raises the skills it defaults to and is said, a
//! shield's blocks train shield blocking, a shield bash is a blow of its own, ranks change
//! to-hit, damage, threat, Defence, and cost, starting ranks come from race and class, and all of
//! it is saved and replayed.

/// The test creatures name no level: level 1 (spec §8 "Experience").
const UNLEVELLED: u32 = 1;
use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::{Category, Strike};
use crate::content::{ContentFiles, Role, WeaponSkillId};
use crate::equipment;
use crate::replay::Replay;
use crate::weapon_skills::{Training, WeaponSkills};
use std::sync::LazyLock;

const TERRAIN: &str = r##"
[[terrain]]
id = "floor"
name = "floor"
glyph = "."
fg = "#808080"

[[terrain]]
id = "wall"
name = "wall"
glyph = "#"
fg = "#a0a0a0"
walkable = false
transparent = false
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "vermin"
name = "vermin"
hostile = ["player"]

[[faction]]
id = "wildlife"
name = "wildlife"
"##;

// The dummy stands and casts (spells hurt no one): something to hit for as long as it takes.
// The rat bites, weakly. The deer is nobody's enemy.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "dummy"
name = "dummy"
glyph = "d"
fg = "#806040"
weight = 4000
faction = "vermin"
combat = "caster"
range = 2
base_hp = 1000

[[creature]]
id = "golem"
name = "stone golem"
glyph = "g"
fg = "#808080"
weight = 400000
faction = "vermin"
combat = "caster"
range = 2
base_hp = 1000
level = 6

[[creature]]
id = "rat"
name = "giant rat"
glyph = "r"
fg = "#806040"
weight = 4000
faction = "vermin"
base_hp = 1000
attack = { damage = "1d2", name = "bite", verb = "bites" }
weapon_skills = { unarmed = 3 }

[[creature]]
id = "deer"
name = "deer"
glyph = "d"
fg = "#c89a64"
weight = 80000
faction = "wildlife"
"##;

const OBJECTS: &str = r##"
[[object]]
id = "sword"
name = "sword"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 1000
value = 0
slot = "main_hand"
damage = "1d8"
threat = 20
category = "long_blades"

[[object]]
id = "dagger"
name = "dagger"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 450
value = 0
slot = "main_hand"
damage = "1d4"
category = "daggers"
range = 5

[[object]]
id = "buckler"
name = "buckler"
glyph = "["
fg = "#c0c0c0"
class = "shields"
materials = ["steel"]
weight = 2000
value = 0
slot = "off_hand"
shield = 2
bash = "1d3"

[[object]]
id = "crossbow"
name = "crossbow"
glyph = "}"
fg = "#c0c0c0"
class = "missile_weapons"
materials = ["steel"]
weight = 3000
value = 0
slot = "missile_weapon"
launches = "bolt"
damage = "1d10"
category = "crossbows"
range = 10
reloads = true

[[object]]
id = "bolt"
name = "bolt"
glyph = "/"
fg = "#c0c0c0"
class = "missiles"
materials = ["steel"]
weight = 50
value = 0
stackable = true
slot = "missiles"
ammo = "bolt"
"##;

const SLOTS: &str = r##"
[[slot]]
id = "main_hand"
name = "Main hand"
action = "wield"
mark = "wielded"
hand = true

[[slot]]
id = "off_hand"
name = "Off hand"
action = "wield"
mark = "in off hand"
hand = true

[[slot]]
id = "missile_weapon"
name = "Missile weapon"
action = "ready"
mark = "readied"

[[slot]]
id = "missiles"
name = "Missiles"
action = "ready"
mark = "quivered"
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"d" = { terrain = "floor", creature = "dummy" }
"r" = { terrain = "floor", creature = "rat" }
"D" = { terrain = "floor", creature = "deer" }
"g" = { terrain = "floor", creature = "golem" }
"##;

const WEAPON_SKILLS: &str = include_str!("../../../../assets/data/weapon_skills.toml");

/// The test content with a fully lit level on `map`; the scenario gives the player a thousand
/// hit points.
fn content_on(map: &str) -> Arc<Content> {
    content_with(map, "")
}

/// [`content_on`] with `tables` after the level's map (`[[pile]]`s).
fn content_with(map: &str, tables: &str) -> Arc<Content> {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/objects.toml", OBJECTS),
        ("data/slots.toml", SLOTS),
        ("data/weapon_skills.toml", WEAPON_SKILLS),
        (
            "data/materials.toml",
            "[[material]]\nid = \"steel\"\nname = \"steel\"\n",
        ),
        ("data/legend.toml", LEGEND),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/yard.toml",
        &format!(
            "name = \"The Yard\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n{tables}"
        ),
    );
    files.insert(
        "scenarios/yard.toml",
        "name = \"Yard\"\nstart_level = \"yard\"\nhp = 1000\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

/// The player beside a dummy (east), a deer (south), with room around.
const YARD: &str = "######\n#....#\n#.@d.#\n#.D..#\n######\n";

fn world_on(map: &str) -> World {
    World::new(content_on(map), "yard", None).expect("world")
}

/// The player carries and wields or readies `objects` (bolts come twenty to a stack).
fn arm(w: &mut World, objects: &[&str]) {
    let content = w.content().clone();
    let player = w.state.player;
    let me = w.state.actors.get_mut(&player).expect("the player");
    for name in objects {
        let object = content.object_id(name).expect(name);
        let count = if content.object(object).stackable {
            20
        } else {
            1
        };
        me.inventory.push(crate::object::Stack::new(object, count));
        let item = me.inventory.len() - 1;
        let slot = equipment::free_slot(&content, &me.equipment, object).expect("a free slot");
        let slots = equipment::slots_taken(&content, object, slot);
        me.equipment.put(&slots, item);
    }
}

fn skill(w: &World, name: &str) -> WeaponSkillId {
    w.content().weapon_skills().id(name).expect(name)
}

fn training(w: &World, name: &str) -> Training {
    w.state.player().weapon_skills.get(skill(w, name))
}

/// Sets the player's rank and marks in skill `name`.
fn set(w: &mut World, name: &str, rank: u8, marks: u32) {
    let id = skill(w, name);
    let player = w.state.player;
    let me = w.state.actors.get_mut(&player).expect("the player");
    let mut skills = me.weapon_skills.clone();
    skills.raise(w.content.weapon_skills(), id, rank);
    skills.train(w.content.weapon_skills(), id, marks);
    me.weapon_skills = skills;
}

fn messages(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

/// What came of the player's attack in `outcome`.
fn players_strike(w: &World, outcome: &Outcome) -> Option<Strike> {
    outcome.events.iter().find_map(|e| match e {
        Event::Attacked {
            attacker, strike, ..
        } if *attacker == w.player_id() => *strike,
        _ => None,
    })
}

#[test]
fn blows_earn_marks_more_for_a_hit_and_fewer_as_the_rank_rises() {
    let mut w = world_on(YARD);
    arm(&mut w, &["sword"]);
    let rules = *w.content().weapon_skills().marks().expect("marks");
    let (mut hits, mut misses, mut ranks) = (0, 0, 0);
    for _ in 0..60 {
        let before = training(&w, "long_blades");
        let outcome = w.apply(&Action::Move(Dir::E));
        let strike = players_strike(&w, &outcome).expect("the player struck");
        let after = training(&w, "long_blades");
        let earned = rules.earned(strike.is_hit(), UNLEVELLED, before.rank);
        if strike.is_hit() {
            hits += 1;
        } else {
            misses += 1;
        }
        if after.rank == before.rank {
            assert_eq!(after.marks, before.marks + earned, "{strike:?}");
        } else {
            ranks += 1;
            assert_eq!(after.rank, before.rank + 1);
            let need = w
                .content()
                .weapon_skills()
                .need(skill(&w, "long_blades"), before.rank);
            assert_eq!(Some(before.marks + earned - after.marks), need);
        }
    }
    assert!(
        hits > 0 && misses > 0 && ranks >= 2,
        "{hits} {misses} {ranks}"
    );
    // A miss earns 1 mark, a hit 3 more; at rank 0 against level 1 a fifth more (rounded).
    assert_eq!(rules.earned(false, UNLEVELLED, 0), 1);
    assert_eq!(rules.earned(true, UNLEVELLED, 0), 4);
    assert_eq!(rules.earned(true, UNLEVELLED, 1), 3);
    // Against level-1 opponents a rank 6 sword learns nothing more.
    set(&mut w, "long_blades", 6, 0);
    w.apply(&Action::Move(Dir::E));
    w.apply(&Action::Move(Dir::E));
    assert_eq!(training(&w, "long_blades"), Training { rank: 6, marks: 0 });
    // The dummy casts at the player, and creatures do not train.
    let dummy = &w.state.actors[&ActorId(1)];
    assert!(dummy.weapon_skills.is_empty());
}

#[test]
fn stronger_opponents_teach_more() {
    // A level-6 golem against long blades 1: five levels above, twice the marks.
    let mut w = world_on("#####\n#.@g#\n#####\n");
    arm(&mut w, &["sword"]);
    set(&mut w, "long_blades", 1, 0);
    let rules = *w.content().weapon_skills().marks().expect("marks");
    assert_eq!(rules.percent(6, 1), 200);
    let outcome = w.apply(&Action::Move(Dir::E));
    let strike = players_strike(&w, &outcome).expect("the player struck");
    let earned = rules.earned(strike.is_hit(), 6, 1);
    assert_eq!(earned, if strike.is_hit() { 6 } else { 2 });
    assert_eq!(
        training(&w, "long_blades"),
        Training {
            rank: 1,
            marks: earned
        }
    );
    // A level-1 dummy at rank 1 teaches the plain marks.
    assert_eq!(rules.earned(true, UNLEVELLED, 1), 3);
}

#[test]
fn a_rank_gained_raises_its_defaults_and_is_said() {
    let mut w = world_on(YARD);
    arm(&mut w, &["sword"]);
    // One mark short of long blades 3; short blades has marks of its own.
    set(&mut w, "long_blades", 2, 59);
    set(&mut w, "short_blades", 0, 5);
    w.apply(&Action::Move(Dir::E));
    assert_eq!(training(&w, "long_blades").rank, 3);
    // Long blades 3 gives short and great blades 1 (−2), and short blades' marks start over;
    // daggers (−4) stay untrained.
    assert_eq!(training(&w, "short_blades"), Training { rank: 1, marks: 0 });
    assert_eq!(training(&w, "great_blades"), Training { rank: 1, marks: 0 });
    assert_eq!(training(&w, "daggers"), Training::default());
    let said = messages(&w);
    let at = |text: &str| said.iter().position(|m| m == text);
    let long = at("You feel more skilled with long blades.").expect("said");
    let short = at("You feel more skilled with short blades.").expect("said");
    let great = at("You feel more skilled with great blades.").expect("said");
    // After the blow that earned it, the skill first, then what it raised.
    let blow = (said.iter())
        .position(|m| m.starts_with("You hit") || m.starts_with("You miss"))
        .expect("the blow");
    assert!(blow < long && long < short && short < great, "{said:#?}");
    // No numbers.
    assert!(
        !said
            .iter()
            .any(|m| m.contains("skilled") && m.contains(char::is_numeric))
    );
}

#[test]
fn blocks_by_the_shield_train_shield_blocking() {
    let mut w = world_on("#####\n#.@r#\n#####\n");
    arm(&mut w, &["buckler"]);
    let table = w.content().weapon_skills().clone();
    let rules = *table.marks().expect("marks");
    let block = table.of_role(Role::ShieldBlock).expect("shield blocking");
    let mut expected = WeaponSkills::default();
    let (mut blocks, mut others) = (0, 0);
    for _ in 0..150 {
        let outcome = w.apply(&Action::Wait);
        for e in &outcome.events {
            if let Event::Attacked {
                target,
                strike: Some(strike),
                ..
            } = e
                && *target == w.player_id()
            {
                if *strike
                    == (Strike::Missed {
                        by: Some(Category::Shield),
                    })
                {
                    blocks += 1;
                    let marks = rules.earned(true, UNLEVELLED, expected.rank(block));
                    expected.train(&table, block, marks);
                } else {
                    others += 1;
                }
            }
        }
    }
    assert!(
        blocks >= 5 && others > 0,
        "{blocks} blocks, {others} others"
    );
    assert_eq!(
        w.state.player().weapon_skills.get(block),
        expected.get(block)
    );
    assert!(expected.rank(block) >= 1);
    assert!((messages(&w).iter()).any(|m| m == "You feel more skilled at blocking with a shield."),);
    // Only blocks train it: nothing else rose.
    assert_eq!(
        (w.state.player().weapon_skills.trained())
            .filter(|(id, _)| *id != block)
            .count(),
        expected.trained().filter(|(id, _)| *id != block).count()
    );
}

#[test]
fn a_shield_bash_is_a_blow_with_the_shield_that_trains_shield_bashing() {
    let mut w = world_on(YARD);
    // Without a shield, nothing to bash with: refused, no time; the abilities menu lists it,
    // greyed out with why.
    use crate::ability::{Ability, OwnAbility, Unavailable};
    let bash = |unavailable| OwnAbility {
        ability: Ability::ShieldBash,
        unavailable,
    };
    assert_eq!(
        w.perception().abilities(),
        [bash(Some(Unavailable::NoShield))]
    );
    let outcome = w.apply(&Action::Bash(Dir::E));
    assert!(!outcome.took_turn);
    assert_eq!(
        messages(&w).last().map(String::as_str),
        Some("You have no shield to bash with.")
    );
    arm(&mut w, &["sword", "buckler"]);
    // With one, the abilities menu (#156) lists it available, and it asks for a direction.
    assert_eq!(w.perception().abilities(), [bash(None)]);
    assert_eq!(Ability::ShieldBash.aim(), crate::ability::Aim::Direction);
    assert_eq!(
        Ability::ShieldBash.action(Some(Dir::E)),
        Some(Action::Bash(Dir::E))
    );
    for (dir, said) in [
        (Dir::W, "There is nothing there to bash."),
        (Dir::SW, "There is nothing there to bash."),
        (Dir::S, "You have no quarrel with the deer."),
    ] {
        let outcome = w.apply(&Action::Bash(dir));
        assert!(!outcome.took_turn, "{dir:?}");
        let last = messages(&w).last().cloned().unwrap_or_default();
        assert!(last.starts_with(said), "{last}");
    }
    // A bash: the shield's dice and Strength, a melee blow's time and noise.
    let content = w.content().clone();
    let buckler = content.object_id("buckler").expect("buckler");
    let (shield, profile) = w
        .state
        .player()
        .bash(&content)
        .expect("a shield to bash with");
    assert_eq!(
        (shield, profile.damage.to_string()),
        (buckler, "1d3".into())
    );
    let bash = skill(&w, "shield_bash");
    let (mut hit, mut missed) = (false, false);
    for _ in 0..12 {
        let before = training(&w, "shield_bash");
        let outcome = w.apply(&Action::Bash(Dir::E));
        assert!(outcome.took_turn);
        assert_eq!(outcome.elapsed, Duration::seconds(6));
        let event = outcome.events.iter().find_map(|e| match e {
            Event::Attacked {
                kind, with, strike, ..
            } => Some((*kind, *with, *strike)),
            _ => None,
        });
        let (kind, with, strike) = event.expect("the bash is seen");
        assert_eq!((kind, with), (AttackKind::Melee, Some(buckler)));
        let strike = strike.expect("rolled");
        let said = messages(&w);
        let text = said
            .iter()
            .rev()
            .find(|m| m.contains("dummy") && !m.contains("spell"))
            .expect("said");
        match strike {
            Strike::Hit { critical: false } => {
                hit = true;
                assert_eq!(text, "You bash the dummy.");
            }
            Strike::Hit { critical: true } => assert_eq!(text, "You bash the dummy hard!"),
            Strike::Missed { .. } => {
                missed = true;
                assert_eq!(text, "You miss the dummy.");
            }
        }
        let after = training(&w, "shield_bash");
        assert!(after.marks > before.marks || after.rank > before.rank);
    }
    assert!(hit && missed);
    // Bashing trained shield bashing, and nothing of the sword's.
    assert!(w.state.player().weapon_skills.get(bash) != Training::default());
    assert_eq!(training(&w, "long_blades"), Training::default());
}

#[test]
fn ranks_change_to_hit_damage_threat_defence_and_cost() {
    let mut w = world_on(YARD);
    arm(&mut w, &["sword", "buckler", "crossbow", "bolt"]);
    let content = w.content().clone();
    let me = |w: &World| w.state.player().clone();
    let untrained = me(&w).melee(&content);
    let defence = w.perception().own_defence();
    // Long blades 8: +4 to hit, +3 damage, threat 20 → 19, and +3 parry with the sword.
    set(&mut w, "long_blades", 8, 0);
    let trained = me(&w).melee(&content);
    assert_eq!(trained.to_hit, untrained.to_hit + 4);
    assert_eq!(trained.bonus, untrained.bonus + 3);
    assert_eq!((untrained.threat, trained.threat), (20, 19));
    assert_eq!(
        trained.damage, untrained.damage,
        "the dice are the weapon's"
    );
    assert_eq!(w.perception().own_defence(), defence + 3);
    // Shield blocking 2 adds 1 to the buckler's Shield bonus.
    set(&mut w, "shield_block", 2, 0);
    assert_eq!(w.perception().own_defence(), defence + 4);
    let dv = w.defence_of(w.player_id());
    assert_eq!(
        (dv.bonus(Category::Shield), dv.bonus(Category::Parry)),
        (3, 3)
    );
    // Crossbows 2: shots hit better and reloading takes a tenth less.
    let shot = me(&w).ranged(&content).expect("a crossbow");
    set(&mut w, "crossbows", 2, 0);
    let trained_shot = me(&w).ranged(&content).expect("a crossbow");
    assert_eq!(trained_shot.to_hit, shot.to_hit + 1);
    let player = w.player_id();
    let shot = me(&w).attack_skills(&content, AttackKind::Ranged, None);
    let crossbow = me(&w).launcher_item(&content);
    assert_eq!(
        w.attack_price(player, ActionKind::Shoot, crossbow, shot),
        6000
    );
    assert_eq!(w.reload_price(player, shot), 5400);
    // Daggers 7: a blow takes a tenth less time.
    let mut w = world_on(YARD);
    arm(&mut w, &["dagger"]);
    let blow = me(&w).attack_skills(&content, AttackKind::Melee, None);
    let dagger = me(&w).weapon_item(&content);
    assert_eq!(
        w.attack_price(player, ActionKind::Melee, dagger, blow),
        6000
    );
    set(&mut w, "daggers", 7, 0);
    assert_eq!(
        w.attack_price(player, ActionKind::Melee, dagger, blow),
        5400
    );
    let mut elapsed = Duration::ZERO;
    for _ in 0..5 {
        elapsed = elapsed + w.apply(&Action::Move(Dir::E)).elapsed;
    }
    assert_eq!(elapsed, Duration::seconds(27), "five blows of 5.4 s");
    // Throwing with the dagger draws on throwing, not on daggers.
    let dagger = content.object_id("dagger").expect("dagger");
    let thrown_with = |w: &World| {
        let item = (me(w).inventory.iter())
            .position(|s| s.object == dagger)
            .expect("the dagger");
        me(w).thrown(item, &content).expect("a throwing weapon")
    };
    let thrown = thrown_with(&w);
    let mut fresh = world_on(YARD);
    arm(&mut fresh, &["dagger"]);
    let untrained = thrown_with(&fresh);
    assert_eq!(
        thrown.to_hit,
        untrained.to_hit + 3,
        "daggers 7 gives throwing 5 (−2)"
    );
}

#[test]
fn an_attack_is_priced_as_it_begins_so_a_rank_it_earns_quickens_only_the_next() {
    // The clock keeps whole seconds and leftover energy carries: what is left of the player's
    // energy after an action tells what it cost.
    let energy = |w: &World| w.state.player().energy;
    let after = |energy: i64, price: i64| (energy - price).rem_euclid(1000);
    let player = ActorId(0);
    // Daggers 7 makes a blow quicker; the stone golem (level 6) teaches a rank-6 fighter
    // something with every blow.
    let mut w = world_on("######\n#.@g.#\n######\n");
    arm(&mut w, &["dagger"]);
    let content = w.content().clone();
    let blow_price = |w: &World| {
        let me = w.state.player();
        let skills = me.attack_skills(&content, AttackKind::Melee, None);
        w.attack_price(player, ActionKind::Melee, me.weapon_item(&content), skills)
    };
    let need = content.weapon_skills().skill(skill(&w, "daggers")).need[6];
    set(&mut w, "daggers", 6, need - 1);
    let (was, start) = (blow_price(&w), energy(&w));
    w.apply(&Action::Move(Dir::E));
    assert_eq!(training(&w, "daggers").rank, 7, "the blow made the rank");
    let now = blow_price(&w);
    assert!(now < was, "rank 7 is quicker");
    assert_eq!(
        energy(&w),
        after(start, was),
        "the blow cost what it began at"
    );
    let start = energy(&w);
    w.apply(&Action::Move(Dir::E));
    assert_eq!(energy(&w), after(start, now), "the next blow is quicker");
    // Crossbows 2 makes reloading quicker: the shot that earns it still costs what it began at.
    let mut w = world_on(YARD);
    arm(&mut w, &["crossbow", "bolt"]);
    let shot_price = |w: &World| {
        let me = w.state.player();
        let skills = me.attack_skills(&content, AttackKind::Ranged, None);
        let shoot = w.attack_price(
            player,
            ActionKind::Shoot,
            me.launcher_item(&content),
            skills,
        );
        shoot + w.reload_price(player, skills)
    };
    let need = content.weapon_skills().skill(skill(&w, "crossbows")).need[1];
    set(&mut w, "crossbows", 1, need - 1);
    let dummy = Pos::new(3, 2);
    let (was, start) = (shot_price(&w), energy(&w));
    w.apply(&Action::Fire(dummy));
    assert_eq!(training(&w, "crossbows").rank, 2, "the shot made the rank");
    let now = shot_price(&w);
    assert!(now < was, "rank 2 reloads quicker");
    assert_eq!(
        energy(&w),
        after(start, was),
        "the shot cost what it began at"
    );
    let start = energy(&w);
    w.apply(&Action::Fire(dummy));
    assert_eq!(energy(&w), after(start, now), "the next shot is quicker");
}

#[test]
fn creatures_fight_with_the_ranks_their_type_gives() {
    let w = world_on("#####\n#.@r#\n#####\n");
    let content = w.content().clone();
    let rat = &w.state.actors[&ActorId(1)];
    assert_eq!(rat.weapon_skills.rank(skill(&w, "unarmed")), 3);
    // Unarmed 3: +2 to hit and +1 damage on its bite.
    let bite = content
        .creature(rat.creature)
        .attack
        .as_ref()
        .expect("a bite");
    let plain = crate::combat::Profile::melee(&bite.attack, &rat.stats);
    let profile = rat.melee(&content);
    assert_eq!(
        (profile.to_hit, profile.bonus),
        (plain.to_hit + 2, plain.bonus + 1)
    );
    // The built-in kobolds: spears 1, and the archer bows 1.
    let builtin = Content::builtin().expect("content");
    let kobold = builtin.creature_id("kobold").expect("kobold");
    let spears = builtin.weapon_skills().id("spears").expect("spears");
    assert_eq!(builtin.creature(kobold).weapon_skills.rank(spears), 1);
}

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

fn ranks_of(character: &str, seed: u64) -> WeaponSkills {
    let spec: CharacterSpec = character.parse().expect("a character");
    let showcase = BUILTIN.scenario("showcase").expect("showcase");
    spec.start(&BUILTIN, seed, showcase.start)
        .expect("starts")
        .weapon_skills
}

#[test]
fn starting_ranks_come_from_race_and_class_and_never_move_the_stats() {
    let table = BUILTIN.weapon_skills();
    let id = |name| table.id(name).expect(name);
    // A human warrior: long blades 2 and shield blocking 1 or 2, the same for the same seed.
    let warrior = ranks_of("human,male,warrior", 7);
    assert_eq!(warrior, ranks_of("human,male,warrior", 7));
    assert_eq!(warrior.rank(id("long_blades")), 2);
    assert!((1..=2).contains(&warrior.rank(id("shield_block"))));
    // An elf ranger: bows 1 + 2, long blades 1 from the race, short blades 1 from the class
    // (long blades 1 − 2 adds nothing); defaults from bows 3: crossbows −4, nothing.
    let ranger = ranks_of("elf,female,ranger", 3);
    assert_eq!(ranger.rank(id("bows")), 3);
    assert_eq!(ranger.rank(id("long_blades")), 1);
    assert_eq!(ranger.rank(id("short_blades")), 1);
    assert_eq!(ranger.rank(id("crossbows")), 0);
    // A thief: daggers 2 and thrown 1; daggers 2 gives thrown 0 (−2): thrown stays 1.
    let thief = ranks_of("human,female,thief", 1);
    assert_eq!(thief.rank(id("daggers")), 2);
    assert_eq!(thief.rank(id("thrown")), 1);
    // A barbarian takes axes 2 or great blades 2, by the seed, never both.
    let (axes, great) = (id("axes"), id("great_blades"));
    let picks: Vec<(u8, u8)> = (0..40)
        .map(|seed| {
            let r = ranks_of("half_orc,male,barbarian", seed);
            (r.rank(axes), r.rank(great))
        })
        .collect();
    // Half-orcs have axes 1 of their own: 3 with the choice, else 1 (and great blades 2).
    assert!(
        picks.iter().all(|&p| p == (3, 0) || p == (1, 2)),
        "{picks:?}"
    );
    assert!(picks.contains(&(3, 0)) && picks.contains(&(1, 2)));
    // Starting ranks roll from their own stream: without them the stats are the same.
    let mut files = ContentFiles::builtin();
    for path in ["data/races.toml", "data/classes.toml"] {
        let text = match path {
            "data/races.toml" => include_str!("../../../../assets/data/races.toml"),
            _ => include_str!("../../../../assets/data/classes.toml"),
        };
        let stripped: String = (text.lines())
            .filter(|l| !l.starts_with("weapon_skill") && !l.starts_with("style"))
            .map(|l| format!("{l}\n"))
            .collect();
        files.insert(path, &stripped);
    }
    let without = Content::load(&files).expect("valid");
    let showcase = BUILTIN.scenario("showcase").expect("showcase");
    for character in [
        "human,male,warrior",
        "half_orc,other,barbarian",
        "elf,female,ranger",
        "half_elf,male,bard",
    ] {
        let spec: CharacterSpec = character.parse().expect("a character");
        for seed in 0..10 {
            let with = spec.start(&BUILTIN, seed, showcase.start).expect("starts");
            let plain = spec.start(&without, seed, showcase.start).expect("starts");
            assert_eq!(with.stats, plain.stats, "{character} {seed}");
            assert_eq!(with.character, plain.character);
            assert!(plain.weapon_skills.is_empty());
        }
    }
    // The run starts with them.
    let w = World::with_character(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&"human,male,warrior".parse().expect("a character")),
    )
    .expect("world");
    assert_eq!(w.state.player().weapon_skills, warrior);
    assert_eq!(w.perception().own_weapon_skills(), &warrior);
}

#[test]
fn weapon_skills_are_saved_hashed_and_replayed() {
    // The sword and the buckler lie at the player's feet.
    let pile = "[[pile]]\nat = [2, 2]\nobjects = [\"sword\", \"buckler\"]\n";
    let content = content_with(YARD, pile);
    let mut w = World::new(content.clone(), "yard", None).expect("world");
    let mut replay = Replay::new(&w);
    let actions = "pickup equip:main_hand:a equip:off_hand:b move:E*6 bash:E*4";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(training(&w, "long_blades") != Training::default());
    assert!(training(&w, "shield_bash") != Training::default());
    // The replay verifies, bashes and all.
    let replayed = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(replayed.state_hash(), w.state_hash());
    // The hash covers ranks and marks.
    let mut other = w.clone();
    let player = other.player_id();
    let me = other.state.actors.get_mut(&player).expect("the player");
    me.weapon_skills = WeaponSkills::default();
    assert_ne!(other.state_hash(), w.state_hash());
    // A save keeps them.
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(loaded.state_hash(), w.state_hash());
    // A save with ranks this content cannot have is refused.
    let mut broken = w.state().clone();
    let me = broken.actors.get_mut(&player).expect("the player");
    me.weapon_skills = serde_json::from_str("[{\"rank\":11}]").expect("parses");
    assert!(World::from_state(content.clone(), broken).is_err());
}
