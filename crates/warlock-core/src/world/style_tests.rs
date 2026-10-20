//! Fighting styles in the world (spec §8 "Fighting styles"): each style's configuration, marks
//! earned only while fighting in it, its benefits on attacks, Defence, and prices, two-weapon
//! fighting (both hands strike, at penalties a style eases; the off hand only takes a light
//! weapon and draws on its own skill), the shield bash that may follow a blow, defaults between
//! styles, starting ranks from classes, and all of it saved and replayed.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::Category;
use crate::content::{ContentFiles, Grip, ObjectId, WeaponSkillId};
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
"##;

// The dummy stands and casts (spells hurt no one): something to hit for as long as it takes.
// The duellist kobold fights with a blade in each hand.
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
id = "kobold"
name = "kobold"
glyph = "k"
fg = "#c08040"
weight = 25000
faction = "vermin"
base_hp = 1000
"##;

const OBJECTS: &str = r##"
[[object]]
id = "sword"
name = "sword"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
material = "steel"
weight = 1000
value = 0
slot = "main_hand"
damage = "1d8"
category = "long_blades"

[[object]]
id = "dagger"
name = "dagger"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
material = "steel"
weight = 450
value = 0
slot = "main_hand"
light_weapon = true
damage = "1d4"
category = "daggers"
range = 5

[[object]]
id = "knife"
name = "knife"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
material = "steel"
weight = 300
value = 0
slot = "main_hand"
light_weapon = true
damage = "1d3"
category = "daggers"

[[object]]
id = "greatsword"
name = "greatsword"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
material = "steel"
weight = 3000
value = 0
slot = "main_hand"
two_handed = true
damage = "2d6"
category = "great_blades"

[[object]]
id = "staff"
name = "staff"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
material = "steel"
weight = 2000
value = 0
slot = "main_hand"
two_handed = true
damage = "1d6"
category = "staves"

[[object]]
id = "buckler"
name = "buckler"
glyph = "["
fg = "#c0c0c0"
class = "shields"
material = "steel"
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
material = "steel"
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
material = "steel"
weight = 50
value = 0
stackable = true
slot = "missiles"
ammo = "bolt"

[[object]]
id = "rock"
name = "rock"
glyph = "*"
fg = "#c0c0c0"
class = "miscellaneous"
material = "steel"
weight = 800
value = 0
stackable = true
slot = "missiles"
damage = "1d3"
category = "thrown"
range = 6
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
"k" = { terrain = "floor", creature = "kobold", carry = ["sword", "dagger"], wield = ["sword", "dagger"] }
"##;

const WEAPON_SKILLS: &str = include_str!("../../../../assets/data/weapon_skills.toml");

/// The test content with a fully lit level on `map` and `tables` after it (`[[pile]]`s); the
/// scenario gives the player a thousand hit points.
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

/// The player beside a dummy (east), with room around.
const YARD: &str = "######\n#....#\n#.@d.#\n#....#\n######\n";

fn yard() -> World {
    World::new(content_with(YARD, ""), "yard", None).expect("world")
}

/// The player carries `objects` and wields or readies each in the first free slot it fits
/// (bolts and rocks come twenty to a stack): a second light weapon goes in the off hand.
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

fn id(w: &World, name: &str) -> WeaponSkillId {
    w.content().weapon_skills().id(name).expect(name)
}

fn training(w: &World, name: &str) -> Training {
    w.state.player().weapon_skills.get(id(w, name))
}

/// Sets the player's rank in skill or style `name` (and what it raises by default).
fn set(w: &mut World, name: &str, rank: u8) {
    let skill = id(w, name);
    let player = w.state.player;
    let table = w.content.weapon_skills().clone();
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.weapon_skills.raise(&table, skill, rank);
}

/// The style the player fights in for a blow, by its id.
fn melee_style(w: &World) -> Option<String> {
    let me = w.state.player();
    (me.style(w.content(), AttackKind::Melee, None))
        .map(|s| w.content().weapon_skills().skill(s).id.clone())
}

/// The player's attacks in `outcome`: what each was made with.
fn players_attacks(w: &World, outcome: &Outcome) -> Vec<Option<ObjectId>> {
    (outcome.events.iter())
        .filter_map(|e| match e {
            Event::Attacked { attacker, with, .. } if *attacker == w.player_id() => Some(*with),
            _ => None,
        })
        .collect()
}

fn object(w: &World, name: &str) -> ObjectId {
    w.content().object_id(name).expect(name)
}

#[test]
fn each_style_has_its_configuration() {
    let cases: [(&[&str], Option<&str>); 9] = [
        (&[], Some("brawler")),
        (&["sword"], Some("duelist")),
        (&["dagger"], Some("duelist")),
        (&["sword", "dagger"], Some("twin_blades")),
        (&["dagger", "knife"], Some("twin_blades")),
        (&["greatsword"], Some("great_weapon")),
        (&["staff"], Some("staff_fighter")),
        (&["sword", "buckler"], Some("sword_and_board")),
        // A shield alone is no configuration of any style.
        (&["buckler"], None),
    ];
    for (held, style) in cases {
        let mut w = yard();
        arm(&mut w, held);
        assert_eq!(melee_style(&w).as_deref(), style, "{held:?}");
    }
    // The grips themselves.
    let mut w = yard();
    arm(&mut w, &["sword", "dagger"]);
    let grip = w.state.player().grip(w.content());
    let (long, daggers) = (id(&w, "long_blades"), id(&w, "daggers"));
    assert_eq!(grip, Some((Grip::TwoWeapons, vec![long, daggers])));
    // A shot is a marksman's, a throw a skirmisher's, whatever the hands hold; and the styles
    // in use are the hands', the readied launcher's, and throwing's when something is ready.
    let mut w = yard();
    arm(&mut w, &["sword", "crossbow", "bolt"]);
    let me = w.state.player();
    let content = w.content();
    let name = |s: Option<WeaponSkillId>| s.map(|s| content.weapon_skills().skill(s).id.clone());
    assert_eq!(
        name(me.style(content, AttackKind::Ranged, None)).as_deref(),
        Some("marksman")
    );
    let rock = object(&w, "rock");
    assert_eq!(
        name(me.style(content, AttackKind::Thrown, Some(rock))).as_deref(),
        Some("skirmisher")
    );
    assert_eq!(me.style(content, AttackKind::Spell, None), None);
    let in_use: Vec<String> = (w.perception().own_styles_in_use().into_iter())
        .map(|s| name(Some(s)).expect("a style"))
        .collect();
    assert_eq!(in_use, ["duelist", "marksman"]);
    let mut w = yard();
    arm(&mut w, &["rock"]);
    let in_use: Vec<String> = (w.perception().own_styles_in_use().into_iter())
        .map(|s| w.content().weapon_skills().skill(s).id.clone())
        .collect();
    assert_eq!(in_use, ["skirmisher", "brawler"]);
}

#[test]
fn styles_train_only_in_their_configuration() {
    // A sword alone: duelist trains, sword and board does not.
    let mut w = yard();
    arm(&mut w, &["sword"]);
    for _ in 0..5 {
        w.apply(&Action::Move(Dir::E));
    }
    assert!(training(&w, "duelist") != Training::default());
    assert!(training(&w, "long_blades") != Training::default());
    assert_eq!(training(&w, "sword_and_board"), Training::default());
    // Its marks are the weapon's: the same rules, the same curve.
    assert_eq!(training(&w, "duelist"), training(&w, "long_blades"));
    // With a buckler beside it: sword and board, not duelist; a bash trains it too.
    let mut w = yard();
    arm(&mut w, &["sword", "buckler"]);
    for action in ["move:E", "move:E", "bash:E", "bash:E"] {
        w.apply(&parse_actions(action).expect("an action")[0]);
    }
    assert!(training(&w, "sword_and_board") != Training::default());
    assert_eq!(training(&w, "duelist"), Training::default());
    // A buckler alone is no style: bashing trains shield bashing only.
    let mut w = yard();
    arm(&mut w, &["buckler"]);
    for _ in 0..4 {
        w.apply(&parse_actions("bash:E").expect("a bash")[0]);
    }
    assert!(training(&w, "shield_bash") != Training::default());
    let styles: Vec<_> = (w.state.player().weapon_skills.trained())
        .filter(|(s, _)| w.content().weapon_skills().skill(*s).is_style())
        .collect();
    assert!(styles.is_empty(), "{styles:?}");
    // Bare fists train the brawler; shots the marksman.
    let mut w = yard();
    for _ in 0..3 {
        w.apply(&Action::Move(Dir::E));
    }
    assert!(training(&w, "brawler") != Training::default());
    let mut w = yard();
    arm(&mut w, &["crossbow", "bolt"]);
    w.apply(&parse_actions("fire:3,2").expect("a shot")[0]);
    assert!(training(&w, "marksman") != Training::default());
    assert_eq!(training(&w, "brawler"), Training::default());
    // A rank gained in a style is said after the blow, like a weapon's.
    let mut w = yard();
    arm(&mut w, &["staff"]);
    let need = w.content().weapon_skills().need(id(&w, "staff_fighter"), 0);
    let player = w.state.player;
    let staff = id(&w, "staff_fighter");
    let table = w.content.weapon_skills().clone();
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.weapon_skills
        .train(&table, staff, need.expect("a need") - 1);
    w.apply(&Action::Move(Dir::E));
    assert_eq!(training(&w, "staff_fighter").rank, 1);
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.contains(&"You feel more skilled at fighting with a staff.".to_string()),
        "{said:#?}"
    );
}

#[test]
fn style_ranks_add_to_attacks_defence_and_prices() {
    let player = |w: &World| w.state.player().clone();
    // Duelist 10: +4 to hit and +5 parry with a sword alone.
    let mut w = yard();
    arm(&mut w, &["sword"]);
    let content = w.content().clone();
    let (blow, dv) = (player(&w).melee(&content), w.perception().own_defence());
    set(&mut w, "duelist", 10);
    let trained = player(&w).melee(&content);
    assert_eq!(trained.to_hit, blow.to_hit + 4);
    assert_eq!(w.perception().own_defence(), dv + 5);
    let parry = w.defence_of(w.player_id()).bonus(Category::Parry);
    assert_eq!(parry, 5);
    // Sword and Board 10: +4 to the buckler's Shield; Duelist's parry is gone with it.
    let mut w = yard();
    arm(&mut w, &["sword", "buckler"]);
    set(&mut w, "duelist", 10);
    set(&mut w, "sword_and_board", 10);
    let dv = w.defence_of(w.player_id());
    assert_eq!(
        (dv.bonus(Category::Shield), dv.bonus(Category::Parry)),
        (6, 0)
    );
    // Great Weapon 10: +6 damage, a critical one more time (×3), and blows 15 % quicker.
    let mut w = yard();
    arm(&mut w, &["greatsword"]);
    let plain = player(&w).melee(&content);
    let me = w.player_id();
    let blow = player(&w).attack_skills(&content, AttackKind::Melee, None);
    let sword = player(&w).weapon_item(&content);
    let price = w.attack_price(me, ActionKind::Melee, sword, blow);
    set(&mut w, "great_weapon", 10);
    let great = player(&w).melee(&content);
    assert_eq!((great.bonus, great.crit), (plain.bonus + 6, plain.crit + 1));
    let quicker = w.attack_price(me, ActionKind::Melee, sword, blow);
    assert_eq!((price, quicker), (6000, 5100));
    // Staff Fighter 10 with the staff: +4 to hit, +4 parry. Great Weapon 10 gives it nothing of
    // its own (no damage, no critical), only Staff Fighter 7 by default: +3 to hit.
    let mut w = yard();
    arm(&mut w, &["staff"]);
    let plain = player(&w).melee(&content);
    set(&mut w, "great_weapon", 10);
    assert_eq!(training(&w, "staff_fighter").rank, 7);
    let great = player(&w).melee(&content);
    assert_eq!(
        (great.to_hit, great.bonus, great.crit),
        (plain.to_hit + 3, plain.bonus, plain.crit),
        "the staff is no great weapon"
    );
    set(&mut w, "staff_fighter", 10);
    assert_eq!(player(&w).melee(&content).to_hit, plain.to_hit + 4);
    assert_eq!(w.defence_of(w.player_id()).bonus(Category::Parry), 4);
    // Brawler 10: +4 to hit, +6 damage with bare hands.
    let mut w = yard();
    let plain = player(&w).melee(&content);
    set(&mut w, "brawler", 10);
    let fists = player(&w).melee(&content);
    assert_eq!(
        (fists.to_hit, fists.bonus),
        (plain.to_hit + 4, plain.bonus + 6)
    );
    // Marksman 10: +4 to hit, shots 15 % and reloading 30 % quicker.
    let mut w = yard();
    arm(&mut w, &["crossbow", "bolt"]);
    let shot = player(&w).ranged(&content).expect("a crossbow");
    set(&mut w, "marksman", 10);
    let aimed = player(&w).ranged(&content).expect("a crossbow");
    assert_eq!(aimed.to_hit, shot.to_hit + 4);
    let shoot = player(&w).attack_skills(&content, AttackKind::Ranged, None);
    let crossbow = player(&w).launcher_item(&content);
    assert_eq!(w.attack_price(me, ActionKind::Shoot, crossbow, shoot), 5100);
    assert_eq!(w.reload_price(me, shoot), 4200);
    // Skirmisher 10: +3 to hit and damage on throws, 20 % quicker.
    let mut w = yard();
    arm(&mut w, &["rock"]);
    let rock = object(&w, "rock");
    let item = (player(&w).inventory.iter())
        .position(|s| s.object == rock)
        .expect("the rocks");
    let throw = player(&w)
        .thrown(item, &content)
        .expect("a throwing weapon");
    let thrown = player(&w).attack_skills(&content, AttackKind::Thrown, Some(rock));
    // Twenty rocks are a burden: a throw takes 7.5 s.
    assert_eq!(w.attack_price(me, ActionKind::Throw, None, thrown), 7500);
    set(&mut w, "skirmisher", 10);
    let skirmish = player(&w)
        .thrown(item, &content)
        .expect("a throwing weapon");
    assert_eq!(
        (skirmish.to_hit, skirmish.bonus),
        (throw.to_hit + 3, throw.bonus + 3)
    );
    assert_eq!(w.attack_price(me, ActionKind::Throw, None, thrown), 6000);
    // A style's rank never helps another configuration: Duelist 10 with nothing in hand.
    let mut w = yard();
    let plain = player(&w).melee(&content);
    set(&mut w, "duelist", 10);
    assert_eq!(player(&w).melee(&content), plain);
}

#[test]
fn two_weapons_strike_twice_at_penalties_twin_blades_eases() {
    let mut w = yard();
    arm(&mut w, &["sword", "dagger"]);
    let content = w.content().clone();
    let dagger = object(&w, "dagger");
    let me = w.state.player().clone();
    // The sword alone, for comparison.
    let mut alone = yard();
    arm(&mut alone, &["sword"]);
    let single = alone.state.player().melee(&content);
    // Main hand −4, off hand −8, each with its own weapon's dice.
    let main = me.melee(&content);
    let (off, off_blow) = me.off_hand_blow(&content).expect("an off hand");
    assert_eq!(off, dagger);
    assert_eq!(main.to_hit, single.to_hit - 4);
    let dagger_alone = {
        let mut w = yard();
        arm(&mut w, &["dagger"]);
        w.state.player().melee(&content)
    };
    assert_eq!(off_blow.to_hit, dagger_alone.to_hit - 8);
    assert_eq!(off_blow.damage, dagger_alone.damage);
    // Twin Blades eases them: rank 4 by 2 and 3, rank 10 to nothing at all.
    set(&mut w, "twin_blades", 4);
    let me = w.state.player().clone();
    assert_eq!(me.melee(&content).to_hit, single.to_hit - 2);
    let (_, eased) = me.off_hand_blow(&content).expect("an off hand");
    assert_eq!(eased.to_hit, dagger_alone.to_hit - 5);
    set(&mut w, "twin_blades", 10);
    let me = w.state.player().clone();
    assert_eq!(me.melee(&content).to_hit, single.to_hit);
    let (_, full) = me.off_hand_blow(&content).expect("an off hand");
    assert_eq!(full.to_hit, dagger_alone.to_hit);
    // Every attack is two blows, the main hand's first; one action's time.
    let outcome = w.apply(&Action::Move(Dir::E));
    assert_eq!(players_attacks(&w, &outcome), [None, Some(dagger)]);
    assert_eq!(outcome.elapsed, Duration::seconds(6));
    // Each blow is said ("You hit the dummy. (×2)" when both hit).
    let blows: u32 = (w.messages().iter())
        .filter(|m| m.display().starts_with("You") && m.display().contains("the dummy"))
        .map(|m| m.count)
        .sum();
    assert_eq!(blows, 2);
    // The off hand trains its own weapon's skill, the main hand the sword's, and both the
    // style.
    let mut w = yard();
    arm(&mut w, &["sword", "dagger"]);
    for _ in 0..4 {
        w.apply(&Action::Move(Dir::E));
    }
    for skill in ["long_blades", "daggers", "twin_blades"] {
        assert!(training(&w, skill) != Training::default(), "{skill}");
    }
    assert_eq!(training(&w, "duelist"), Training::default());
    // The off hand's blow is made with the dagger, and worded so ("parries your dagger").
    let me = w.state.player();
    let words = super::fight::Words::of(me, AttackKind::Melee, Some(dagger), &content);
    assert_eq!(words, super::fight::Words::test("dagger", "hits"));
}

#[test]
fn the_off_hand_takes_only_light_weapons() {
    // The dagger and the sword lie at the player's feet.
    let pile = "[[pile]]\nat = [2, 2]\nobjects = [\"sword\", \"dagger\", \"buckler\"]\n";
    let content = content_with(YARD, pile);
    let mut w = World::new(content.clone(), "yard", None).expect("world");
    for action in parse_actions("pickup equip:main_hand:a").expect("actions") {
        w.apply(&action);
    }
    // The sword is not light: it does not go in the off hand.
    w.apply(&parse_actions("equip:off_hand:a").expect("an action")[0]);
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.iter().any(|m| m.contains("does not go there")),
        "{said:#?}"
    );
    // The dagger does, beside the sword: two weapons.
    w.apply(&parse_actions("equip:off_hand:b").expect("an action")[0]);
    let main = content.slot_id("main_hand").expect("main hand");
    let off = content.slot_id("off_hand").expect("off hand");
    let me = w.state.player();
    assert_eq!(
        (me.equipment.get(main), me.equipment.get(off)),
        (Some(0), Some(1))
    );
    assert_eq!(melee_style(&w).as_deref(), Some("twin_blades"));
    // A buckler in its place: weapon and shield.
    w.apply(&parse_actions("equip:off_hand:c").expect("an action")[0]);
    assert_eq!(melee_style(&w).as_deref(), Some("sword_and_board"));
    // A light weapon fits either hand; the main hand's things only the main hand.
    let (dagger, sword) = (object(&w, "dagger"), object(&w, "sword"));
    assert!(content.fits(dagger, main) && content.fits(dagger, off));
    assert!(content.fits(sword, main) && !content.fits(sword, off));
}

#[test]
fn creatures_fight_with_two_weapons_too() {
    // The kobold wields a sword and a dagger from its spawn; the player stands beside it.
    let mut w = World::new(content_with("#####\n#.@k#\n#####\n", ""), "yard", None).expect("world");
    let content = w.content().clone();
    let kobold = ActorId(1);
    let dagger = object(&w, "dagger");
    assert!(w.state.actors[&kobold].off_hand(&content).is_some());
    let mut offs = 0;
    for _ in 0..5 {
        let outcome = w.apply(&Action::Wait);
        let attacks: Vec<Option<ObjectId>> = (outcome.events.iter())
            .filter_map(|e| match e {
                Event::Attacked { attacker, with, .. } if *attacker == kobold => Some(*with),
                _ => None,
            })
            .collect();
        if !attacks.is_empty() {
            assert_eq!(attacks, [None, Some(dagger)]);
            offs += 1;
        }
    }
    assert!(offs > 0);
    // Creatures do not train.
    assert!(w.state.actors[&kobold].weapon_skills.is_empty());
}

#[test]
fn sword_and_board_follows_a_hit_with_a_shield_bash() {
    let mut w = yard();
    arm(&mut w, &["sword", "buckler"]);
    let buckler = object(&w, "buckler");
    // Untrained: never.
    for _ in 0..20 {
        let outcome = w.apply(&Action::Move(Dir::E));
        assert_eq!(players_attacks(&w, &outcome), [None]);
    }
    // Sword and Board 10: a quarter of the hits bring a bash, never a miss.
    set(&mut w, "sword_and_board", 10);
    let (mut bashes, mut hits) = (0, 0);
    for _ in 0..120 {
        let outcome = w.apply(&Action::Move(Dir::E));
        let attacks = players_attacks(&w, &outcome);
        let first = outcome.events.iter().find_map(|e| match e {
            Event::Attacked {
                attacker,
                strike: Some(s),
                ..
            } if *attacker == w.player_id() => Some(*s),
            _ => None,
        });
        let hit = first.is_some_and(|s| s.is_hit());
        hits += usize::from(hit);
        match attacks.as_slice() {
            [None] => {}
            [None, Some(shield)] => {
                assert!(hit, "a bash follows only a hit");
                assert_eq!(*shield, buckler);
                bashes += 1;
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(bashes > 0 && bashes < hits, "{bashes} of {hits}");
    assert!(training(&w, "shield_bash") != Training::default());
}

#[test]
fn styles_default_to_styles() {
    let w = yard();
    let table = w.content().weapon_skills();
    let duelist = id(&w, "duelist");
    let skills = WeaponSkills::from_ranks(table, [(duelist, 7)]);
    assert_eq!(skills.rank(id(&w, "sword_and_board")), 4);
    assert_eq!(skills.rank(id(&w, "twin_blades")), 4);
    assert_eq!(skills.rank(id(&w, "great_weapon")), 0);
    // Never a weapon skill.
    assert_eq!(skills.rank(id(&w, "long_blades")), 0);
    let skills = WeaponSkills::from_ranks(table, [(id(&w, "long_blades"), 10)]);
    for (style, ..) in table.styles() {
        assert_eq!(skills.rank(style), 0);
    }
}

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

#[test]
fn starting_styles_come_from_the_class_and_move_no_other_roll() {
    let table = BUILTIN.weapon_skills();
    let rank = |character: &str, seed: u64, style: &str| {
        let spec: CharacterSpec = character.parse().expect("a character");
        let start = BUILTIN.scenario("showcase").expect("showcase").start;
        let skills = spec
            .start(&BUILTIN, seed, start)
            .expect("starts")
            .weapon_skills;
        skills.rank(table.id(style).expect(style))
    };
    assert_eq!(rank("human,male,warrior", 7, "sword_and_board"), 1);
    assert_eq!(rank("half_orc,male,barbarian", 3, "great_weapon"), 1);
    assert_eq!(rank("elf,female,ranger", 3, "marksman"), 1);
    assert_eq!(rank("human,male,wizard", 3, "staff_fighter"), 0);
    // A thief is a twin blade or a duelist, by the seed, never both; the same for the same seed.
    let picks: Vec<(u8, u8)> = (0..40)
        .map(|seed| {
            let c = "human,female,thief";
            (rank(c, seed, "twin_blades"), rank(c, seed, "duelist"))
        })
        .collect();
    assert!(
        picks.iter().all(|&p| p == (1, 0) || p == (0, 1)),
        "{picks:?}"
    );
    assert!(picks.contains(&(1, 0)) && picks.contains(&(0, 1)));
    assert_eq!(
        rank("human,female,thief", 5, "duelist"),
        rank("human,female,thief", 5, "duelist")
    );
    // Styles roll from their own stream: without them the stats and the weapon skills are the
    // same.
    let mut files = ContentFiles::builtin();
    let stripped: String = (include_str!("../../../../assets/data/classes.toml").lines())
        .filter(|l| !l.starts_with("style"))
        .map(|l| format!("{l}\n"))
        .collect();
    files.insert("data/classes.toml", &stripped);
    let without = Content::load(&files).expect("valid");
    let start = BUILTIN.scenario("showcase").expect("showcase").start;
    for character in [
        "human,female,thief",
        "human,male,monk",
        "half_orc,male,barbarian",
    ] {
        let spec: CharacterSpec = character.parse().expect("a character");
        for seed in 0..10 {
            let with = spec.start(&BUILTIN, seed, start).expect("starts");
            let plain = spec.start(&without, seed, start).expect("starts");
            assert_eq!(with.stats, plain.stats, "{character} {seed}");
            assert_eq!(with.skills, plain.skills);
            let weapons = |s: &WeaponSkills| -> Vec<(WeaponSkillId, Training)> {
                (s.trained())
                    .filter(|(id, _)| !table.skill(*id).is_style())
                    .collect()
            };
            assert_eq!(weapons(&with.weapon_skills), weapons(&plain.weapon_skills));
            assert!(
                with.weapon_skills
                    .trained()
                    .any(|(id, _)| table.skill(id).is_style())
            );
        }
    }
}

#[test]
fn styles_and_two_weapons_are_saved_hashed_and_replayed() {
    let pile = "[[pile]]\nat = [2, 2]\nobjects = [\"sword\", \"dagger\"]\n";
    let content = content_with(YARD, pile);
    let mut w = World::new(content.clone(), "yard", None).expect("world");
    let mut replay = Replay::new(&w);
    let actions = "pickup equip:main_hand:a equip:off_hand:b move:E*8";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(training(&w, "twin_blades") != Training::default());
    let replayed = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(replayed.state_hash(), w.state_hash());
    // The hash covers a style's rank and marks.
    let mut other = w.clone();
    set(&mut other, "twin_blades", 5);
    assert_ne!(other.state_hash(), w.state_hash());
    // A save keeps them, and the off hand's weapon.
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert!(loaded.state().player().off_hand(&content).is_some());
}
