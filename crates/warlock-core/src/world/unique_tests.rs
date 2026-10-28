//! Unique items (spec §8 "Magic items": "Unique items"): made from their table, rare with a set
//! name and affixes; hidden until identified, power and lore too; placed once in all content;
//! and their powers, each read in its one place: Inflict (`World::after_blow`, through the
//! attack's profile), Vision (the eyes), and Ward (`World::afflict`).

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::combat::{Blow, Strike};
use crate::content::{Affliction, ContentFiles, OnHit, Power, StatusId};
use crate::magic::describe::{self, LineKind};
use crate::magic::{Affix, Rarity};

// A dark room: the player, and a giant rat three cells off (no light of its own).
const LEVEL: &str = r##"
name = "Den"
ambient = [0, 0, 0, 0]
map = '''
#######
#@..r.#
#.....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/den.toml", LEVEL);
    files.insert(
        "scenarios/den.toml",
        "name = \"Den\"\nstart_level = \"den\"\nvision = \"normal\"\n\
         stats = { strength = 14, dexterity = 14, toughness = 12 }\n",
    );
    files
}

fn world() -> World {
    let content = Arc::new(Content::load(&files()).expect("test content is valid"));
    World::new(content, "den", Some(1)).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions parse") {
        w.apply(&action);
    }
}

fn unique(w: &World, id: &str) -> Stack {
    w.content().unique(id).expect(id).stack.clone()
}

fn foe(w: &World) -> ActorId {
    let rat = w.content().creature_id("rat").expect("rat");
    (w.state.actors.values())
        .find(|a| a.creature == rat)
        .map(|a| a.id)
        .expect("a giant rat")
}

/// Actor `id` carries unique item `unique` and wears or wields it in `slot`; returns where it lies.
fn wear(w: &mut World, id: ActorId, slot: &str, unique_id: &str) -> usize {
    let stack = unique(w, unique_id);
    wear_stack(w, id, slot, stack)
}

/// Actor `id` carries `stack` and wears or wields it in `slot`; returns where it lies.
fn wear_stack(w: &mut World, id: ActorId, slot: &str, stack: Stack) -> usize {
    let content = w.content().clone();
    let actor = w.state.actors.get_mut(&id).expect("actor");
    actor.inventory.push(stack);
    let item = actor.inventory.len() - 1;
    let slot = content.slot_id(slot).expect("slot");
    let taken = crate::equipment::slots_taken(&content, actor.inventory[item].object, slot);
    actor.equipment.put(&taken, item);
    item
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect(id)
}

fn severity(w: &World, actor: ActorId, id: &str) -> Option<u16> {
    let status = status(w, id);
    w.state.actors[&actor]
        .effects
        .get(status)
        .map(|e| e.severity)
}

fn last_message(w: &World) -> String {
    (w.messages().iter().last())
        .map(|m| m.display())
        .unwrap_or_default()
}

const HIT: Blow = Blow {
    strike: Strike::Hit { critical: false },
    damage: 1,
    kind: None,
};

#[test]
fn a_unique_is_made_from_its_table_rare_with_its_name_and_affixes() {
    let w = world();
    let content = w.content().clone();
    let def = content.unique("widowmaker").expect("Widowmaker");
    assert_eq!(def.name, "Widowmaker");
    assert_eq!(content.object(def.object).id, "stiletto");
    let magic = def.stack.magic().expect("magic");
    assert_eq!(magic.rarity, Rarity::Rare);
    assert_eq!(magic.name.as_deref(), Some("Widowmaker"));
    assert_eq!(magic.unique.as_deref(), Some("widowmaker"));
    assert_eq!(magic.level, 12);
    assert!(!magic.identified, "found unidentified");
    // Each affix at its tier, its value the top of the tier's range.
    let affixes = |id: &str, tier: u8| {
        let a = content.affixes().get(id).expect(id);
        Affix {
            id: id.into(),
            tier,
            value: a.tier(tier).expect("tier").max,
        }
    };
    assert_eq!(
        magic.affixes,
        [
            affixes("venomous", 4),
            affixes("of_celerity", 11),
            affixes("of_the_lynx", 2)
        ]
    );
    assert_eq!(def.stack.count, 1);
    // The four, each with its power.
    let powers: Vec<(&str, &str)> = (content.uniques())
        .filter(|u| u.stack.magic().is_some_and(|m| m.level == 12))
        .map(|u| {
            let power = match u.power.as_ref().expect("a power") {
                Power::Inflict(_) => "inflict",
                Power::Vision(_) => "vision",
                Power::Ward { .. } => "ward",
            };
            (u.id.as_str(), power)
        })
        .collect();
    assert_eq!(
        powers,
        [
            ("high_toby", "inflict"),
            ("turncoat", "ward"),
            ("unhalwed_bedes", "vision"),
            ("widowmaker", "inflict"),
        ]
    );
}

#[test]
fn unidentified_it_hides_its_name_affixes_power_and_lore() {
    let mut w = world();
    let player = w.player_id();
    let content = w.content().clone();
    let item = wear(&mut w, player, "main_hand", "widowmaker");
    let stack = |w: &World| w.state.actors[&player].inventory[item].clone();
    let hidden = stack(&w);
    assert!(hidden.known_magic().is_none());
    assert_eq!(hidden.title(&content), "stiletto");
    assert!(!hidden.describe(&content).contains("Widowmaker"));
    let lines = describe::lines(&hidden, &content);
    let shown = |lines: &[describe::Line], kind: LineKind| {
        (lines.iter())
            .filter(|l| l.kind == kind)
            .map(|l| l.text.clone())
            .collect::<Vec<_>>()
    };
    assert!(shown(&lines, LineKind::Power).is_empty());
    assert!(shown(&lines, LineKind::Lore).is_empty());
    assert!(!lines.iter().any(|l| matches!(l.kind, LineKind::Affix(_))));
    assert!(lines.iter().all(|l| !l.text.contains("Widowmaker")));
    // Identified: its name, affixes, power, and lore, the lore last.
    run(&mut w, "identify");
    let known = stack(&w);
    assert_eq!(known.title(&content), "Widowmaker");
    assert_eq!(known.describe(&content), "Widowmaker, a stiletto");
    let lines = describe::lines(&known, &content);
    assert_eq!(shown(&lines, LineKind::Power), ["Its every cut poisons."]);
    assert_eq!(
        lines.last().map(|l| (l.kind, l.text.as_str())),
        Some((LineKind::Lore, "Thin as a hatpin, and never quite clean."))
    );
    let affixes = (lines.iter()).filter(|l| matches!(l.kind, LineKind::Affix(_)));
    assert_eq!(affixes.count(), 3);
}

#[test]
fn a_unique_has_one_placement_in_all_content() {
    let level = |objects: &str| {
        format!(
            "name = \"Twice\"\nmap = '''\n####\n#@.#\n####\n'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             [[pile]]\nat = [2, 1]\nobjects = {objects}\n"
        )
    };
    // A unique of the test's own: the shipped ones are placed already (the criminals' gear).
    let files = || {
        let mut files = files();
        files.insert(
            "data/uniques/test_blade.toml",
            "[[unique]]\nid = \"test_blade\"\nname = \"Test Blade\"\nobject = \"stiletto\"\n\
             affixes = [\"of_celerity:1\"]\n",
        );
        files
    };
    let load = |objects: &str| {
        let mut files = files();
        files.insert("levels/twice.toml", &level(objects));
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    let twice = load(r#"["unique:test_blade", "unique:test_blade"]"#).expect_err("twice");
    assert!(twice.contains("placed twice"), "{twice}");
    let two = load(r#"["unique:test_blade*2"]"#).expect_err("two at once");
    assert!(two.contains("one at a time"), "{two}");
    // A shipped unique is placed once already: Mother Sallow wields Widowmaker.
    let shipped = load(r#"["unique:widowmaker"]"#).expect_err("placed by her gear");
    assert!(shipped.contains("placed twice"), "{shipped}");
    let unknown = load(r#"["unique:nobody"]"#).expect_err("unknown");
    assert!(
        unknown.contains("unknown unique item `nobody`"),
        "{unknown}"
    );
    // Once is fine, and a legend wields it by the same name.
    let mut files = files();
    files.insert(
        "levels/once.toml",
        "name = \"Once\"\nmap = '''\n#####\n#@.k#\n#####\n'''\n[legend]\n\
         \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
         \"k\" = { terrain = \"floor\", creature = \"kobold\", carry = [\"unique:test_blade\"], \
         wield = \"unique:test_blade\" }\n",
    );
    let content = Content::load(&files).expect("one placement");
    let spawn = &content.level("once").expect("level").creatures[0];
    assert_eq!(spawn.carry.len(), 1);
    assert!(spawn.equipment.holds(0), "wielded");
}

#[test]
fn widowmaker_poisons_on_every_hit() {
    let mut w = world();
    let (player, foe) = (w.player_id(), foe(&w));
    wear(&mut w, player, "main_hand", "widowmaker");
    let content = w.content().clone();
    let profile = w.state.actors[&player].melee(&content);
    let poisoned = status(&w, "poisoned");
    let poisons = OnHit {
        what: Affliction::Status {
            status: poisoned,
            severity: 1,
        },
        chance: 100,
    };
    assert_eq!(
        profile.on_hit,
        [poisons],
        "the stiletto's power rides its blows"
    );
    // Its blows: every hit adds a degree, and nothing is drawn for it.
    w.state.actors.get_mut(&foe).expect("rat").hp = 10_000;
    let mut hits = 0;
    for _ in 0..12 {
        let blow = w.attack_with(
            player,
            foe,
            AttackKind::Melee,
            Some(profile.clone()),
            &mut Vec::new(),
        );
        hits += u16::from(blow.is_some_and(|b| b.strike.is_hit() && b.damage > 0));
        assert_eq!(
            severity(&w, foe, "poisoned"),
            (hits > 0).then_some(hits.min(10))
        );
    }
    assert!(hits >= 3, "{hits} hits");
    // A kick is no cut, and the off hand's other weapon carries nothing of it.
    assert!(w.state.actors[&player].kick(&content).on_hit.is_empty());
}

#[test]
fn high_toby_frightens_at_its_chance_from_the_fixed_stream() {
    let mut w = world();
    let (player, foe) = (w.player_id(), foe(&w));
    wear(&mut w, player, "missile_weapon", "high_toby");
    let arrows = Stack::new(w.content().object_id("arrow").expect("arrow"), 20);
    wear_stack(&mut w, player, "missiles", arrows);
    let content = w.content().clone();
    let profile = w.state.actors[&player].ranged(&content).expect("a shot");
    let frightened = status(&w, "frightened");
    let frightens = OnHit {
        what: Affliction::Status {
            status: frightened,
            severity: 1,
        },
        chance: 25,
    };
    assert_eq!(profile.on_hit, [frightens]);
    let landed: Vec<bool> = (0..40)
        .map(|_| {
            let mut expected = w.state.rng.clone();
            let lands = expected.chance(25);
            let actor = w.state.actors.get_mut(&foe).expect("rat");
            actor.effects.remove(frightened);
            let made = profile.on_hit.clone();
            w.after_blow(
                player,
                foe,
                AttackKind::Ranged,
                (&HIT, &made),
                &mut Vec::new(),
            );
            assert_eq!(w.state.rng, expected, "one draw of the gameplay stream");
            assert_eq!(severity(&w, foe, "frightened").is_some(), lands);
            lands
        })
        .collect();
    assert!(
        landed.contains(&true) && landed.contains(&false),
        "{landed:?}"
    );
}

#[test]
fn the_bedes_give_darkvision_while_worn_and_not_after() {
    let mut w = world();
    let (player, foe) = (w.player_id(), foe(&w));
    assert_eq!(w.eyes().darkvision, 0, "normal eyes");
    let item = wear(&mut w, player, "neck", "unhalwed_bedes");
    assert_eq!(w.eyes().darkvision, 5, "darkvision while worn");
    assert_eq!(w.eyes().id, "normal", "the base profile's name stays");
    // In the dark, the rat three cells off is seen only with it.
    let seen = |w: &mut World| {
        w.perceive(&mut Vec::new());
        (w.perception().actors().iter()).any(|a| a.id == foe)
    };
    assert!(seen(&mut w));
    w.state
        .actors
        .get_mut(&player)
        .expect("player")
        .equipment
        .take_off(item);
    assert_eq!(w.eyes().darkvision, 0, "and not after");
    assert!(!seen(&mut w));
    // The dev profile shows itself as it is.
    let neck = w.content().slot_id("neck").expect("neck");
    let me = w.state.actors.get_mut(&player).expect("player");
    me.equipment.put(&[neck], item);
    assert_eq!(w.eyes().darkvision, 5, "on again");
    run(&mut w, "vision:normal");
    assert_eq!(w.eyes().darkvision, 0, "the dev override wins");
    // A creature wearing it sees in the dark too.
    let content = w.content().clone();
    let base = &content.vision_profiles()[content.creature(w.state.actors[&foe].creature).vision];
    let before = w.state.actors[&foe].eyes(&content, base).darkvision;
    wear(&mut w, foe, "neck", "unhalwed_bedes");
    let after = w.state.actors[&foe].eyes(&content, base).darkvision;
    assert_eq!(after, before.max(5));
}

#[test]
fn the_turncoat_wards_off_fear() {
    let mut w = world();
    let (player, foe) = (w.player_id(), foe(&w));
    let frightened = status(&w, "frightened");
    w.afflict(player, frightened, 1, None, &mut Vec::new());
    assert_eq!(
        severity(&w, player, "frightened"),
        Some(1),
        "fear takes the unarmoured"
    );
    w.state
        .actors
        .get_mut(&player)
        .expect("player")
        .effects
        .remove(frightened);
    wear(&mut w, player, "body", "turncoat");
    w.afflict(player, frightened, 1, Some(foe), &mut Vec::new());
    assert_eq!(severity(&w, player, "frightened"), None);
    assert_eq!(last_message(&w), "Fear finds no hold on you.");
    // Only fear: poison takes as ever.
    let poisoned = status(&w, "poisoned");
    w.afflict(player, poisoned, 1, None, &mut Vec::new());
    assert_eq!(severity(&w, player, "poisoned"), Some(1));
    // A creature wearing it is not frightened either, and nothing is said.
    let said = w.messages().len();
    wear(&mut w, foe, "body", "turncoat");
    w.afflict(foe, frightened, 1, Some(player), &mut Vec::new());
    assert_eq!(severity(&w, foe, "frightened"), None);
    assert_eq!(w.messages().len(), said);
}

#[test]
fn a_unique_survives_a_save() {
    let mut w = world();
    let player = w.player_id();
    wear(&mut w, player, "main_hand", "widowmaker");
    let json = serde_json::to_string(&w.state).expect("save");
    assert!(json.contains("\"unique\":\"widowmaker\""));
    let back: WorldState = serde_json::from_str(&json).expect("load");
    assert_eq!(back.hash(), w.state.hash());
    assert_eq!(
        back.actors[&player]
            .inventory
            .last()
            .and_then(|s| s.magic())
            .and_then(|m| m.unique.clone()),
        Some("widowmaker".into())
    );
}
