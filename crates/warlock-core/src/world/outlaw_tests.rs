//! The outlaws (docs/design/adventurers-guild.md §8, #424): the `outlaws` faction, the footpads,
//! and the four wanted criminals. Each criminal spawns with its unique item wielded or worn (and
//! its power works for it as for the player); the outlaws and the things of their lairs are not
//! hostile to each other, by type and in a fight left to run; named monsters are called by their
//! proper names; Brother Osrick's file keeps its note about spells.

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, Stance};
use crate::gear;
use crate::rng::Rng;
use std::sync::LazyLock;

/// The four criminals and the unique item each carries.
const CRIMINALS: [(&str, &str); 4] = [
    ("sallow", "widowmaker"),
    ("wystan", "high_toby"),
    ("osrick", "unhalwed_bedes"),
    ("herrick", "turncoat"),
];

/// Each lair's population (design §8.2), besides its criminal.
const LAIRS: [(&str, &[&str]); 4] = [
    (
        "sallow",
        &[
            "giant_leech",
            "giant_frog",
            "crocodile",
            "will_o_wisp",
            "mud_golem",
            "marsh_troll",
            "giant_centipede",
            "footpad",
        ],
    ),
    (
        "wystan",
        &[
            "footpad",
            "footpad_archer",
            "bat",
            "giant_centipede",
            "barrow_spider",
            "goblin_archer",
        ],
    ),
    (
        "osrick",
        &[
            "skeleton",
            "zombie",
            "barrow_ghoul",
            "ghost",
            "drowned",
            "grave_worm",
            "barrow_spider",
        ],
    ),
    (
        "herrick",
        &[
            "goblin",
            "goblin_archer",
            "goblin_chief",
            "footpad",
            "footpad_archer",
            "ogre",
            "bat",
        ],
    ),
];

static CONTENT: LazyLock<Content> = LazyLock::new(|| Content::builtin().expect("content"));

fn stance(a: &str, b: &str) -> Stance {
    let id = |n: &str| CONTENT.creature_id(n).expect(n);
    CONTENT.stance(id(a), id(b))
}

#[test]
fn the_outlaws_are_the_ascmearcs_own_and_hunt_only_the_player() {
    for id in [
        "footpad",
        "footpad_archer",
        "sallow",
        "wystan",
        "osrick",
        "herrick",
    ] {
        let def = CONTENT.creature(CONTENT.creature_id(id).expect(id));
        assert_eq!(def.campaign.as_deref(), Some("ascmearc"), "{id}");
        assert_eq!(
            def.xp,
            5 * u32::from(def.level) + 10,
            "{id}: the XP guideline"
        );
        assert!(
            CONTENT.has_tag(CONTENT.creature_id(id).expect(id), "outlaw"),
            "{id}"
        );
        assert_eq!(stance(id, "player"), Stance::Hostile, "{id}");
        assert_eq!(stance("player", id), Stance::Hostile, "{id}");
    }
    let outlaws = CONTENT.factions().id("outlaws").expect("the faction");
    // No faction lists the outlaws as hostile: they are hostile to the player alone.
    for (i, def) in CONTENT.creature_defs().iter().enumerate() {
        if def.id == "player" || def.faction == Some(outlaws) {
            continue;
        }
        let other = CONTENT.creature(crate::content::CreatureId(i as u16));
        assert_eq!(
            CONTENT.stance(
                CONTENT.creature_id("footpad").expect("footpad"),
                CONTENT.creature_id(&other.id).expect("id")
            ),
            Stance::Neutral,
            "the footpad and the {}",
            other.id
        );
    }
    // The criminals are uniques in no population: only their lairs' vaults place them.
    for (id, _) in CRIMINALS {
        let creature = CONTENT.creature_id(id).expect(id);
        assert!(CONTENT.creature(creature).unique, "{id}");
        assert_eq!(CONTENT.creature(creature).level, 12, "{id}");
    }
}

#[test]
fn each_criminal_spawns_with_its_unique_wielded_or_worn() {
    for (id, unique) in CRIMINALS {
        let creature = CONTENT.creature_id(id).expect(id);
        for seed in 0..20 {
            let pack = gear::roll(&CONTENT, &mut Rng::gear(seed, "outlaws"), creature, 12);
            let at = (pack.items.iter())
                .position(|s| s.magic().and_then(|m| m.unique.as_deref()) == Some(unique))
                .unwrap_or_else(|| panic!("{id} {seed}: no {unique}"));
            assert!(pack.equipment.holds(at), "{id} {seed}: {unique} not held");
            // Every piece that goes in a slot is wielded or worn, the quiver too.
            for (i, stack) in pack.items.iter().enumerate() {
                if CONTENT.object(stack.object).slot.is_some() {
                    let name = &CONTENT.object(stack.object).id;
                    assert!(pack.equipment.holds(i), "{id} {seed}: {name} not held");
                }
            }
        }
    }
    let wystan = CONTENT.creature_id("wystan").expect("wystan");
    let pack = gear::roll(&CONTENT, &mut Rng::gear(1, "outlaws"), wystan, 12);
    let arrows = (pack.items.iter())
        .find(|s| CONTENT.object(s.object).id == "arrow")
        .map(|s| s.count);
    assert_eq!(arrows, Some(40), "Black Wystan's quiver");
}

#[test]
fn the_outlaws_and_the_things_of_their_lairs_leave_each_other_be() {
    let outlaws = [
        "footpad",
        "footpad_archer",
        "sallow",
        "wystan",
        "osrick",
        "herrick",
    ];
    for (criminal, population) in LAIRS {
        for &other in population.iter().chain([&criminal]) {
            for outlaw in outlaws {
                assert_eq!(
                    stance(outlaw, other),
                    Stance::Neutral,
                    "{outlaw} and {other}"
                );
                assert_eq!(
                    stance(other, outlaw),
                    Stance::Neutral,
                    "{other} and {outlaw}"
                );
            }
        }
    }
}

// Two lit rooms of the Ascmearc (its campaign's types may stand only on its levels), the player
// shut away in a closet of their own: in the west the outlaws among the fen's and the barrows'
// things, in the east footpads among Sergeant Herrick's goblins and an ogre.
const LEVEL: &str = r##"
name = "Outlaws' den"
ambient = [200, 200, 200, 0]
map = '''
#########################
#S.W.O.H.f.a#.f.a.g.c.G.#
#...........#...........#
#.l.r.k.m.T.#.b.A.o.....#
#...........#...........#
#.z.s.u.h.d.#.........###
#.n.w.e.b...#.........#@#
#########################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"S" = { terrain = "floor", creature = "sallow" }
"W" = { terrain = "floor", creature = "wystan" }
"O" = { terrain = "floor", creature = "osrick" }
"H" = { terrain = "floor", creature = "herrick" }
"f" = { terrain = "floor", creature = "footpad" }
"a" = { terrain = "floor", creature = "footpad_archer" }
"l" = { terrain = "floor", creature = "giant_leech" }
"r" = { terrain = "floor", creature = "giant_frog" }
"k" = { terrain = "floor", creature = "crocodile" }
"m" = { terrain = "floor", creature = "mud_golem" }
"T" = { terrain = "floor", creature = "marsh_troll" }
"z" = { terrain = "floor", creature = "zombie" }
"s" = { terrain = "floor", creature = "skeleton" }
"u" = { terrain = "floor", creature = "barrow_ghoul" }
"h" = { terrain = "floor", creature = "ghost" }
"d" = { terrain = "floor", creature = "drowned" }
"n" = { terrain = "floor", creature = "grave_worm" }
"w" = { terrain = "floor", creature = "will_o_wisp" }
"e" = { terrain = "floor", creature = "giant_centipede" }
"b" = { terrain = "floor", creature = "bat" }
"g" = { terrain = "floor", creature = "goblin" }
"A" = { terrain = "floor", creature = "goblin_archer" }
"c" = { terrain = "floor", creature = "goblin_chief" }
"G" = { terrain = "floor", creature = "goblin" }
"o" = { terrain = "floor", creature = "ogre" }
"##;

fn den() -> World {
    let mut files = ContentFiles::builtin();
    files.insert("campaigns/ascmearc/levels/outlaw_den.toml", LEVEL);
    files.insert(
        "campaigns/ascmearc/scenarios/outlaw_den.toml",
        "name = \"Outlaws' den\"\nstart_level = \"outlaw_den\"\nvision = \"normal\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("the den loads"));
    World::new(content, "outlaw_den", Some(1)).expect("a run in the den")
}

fn of_type(w: &World, id: &str) -> ActorId {
    let creature = w.content().creature_id(id).expect(id);
    (w.state.actors.values())
        .find(|a| a.creature == creature)
        .map(|a| a.id)
        .unwrap_or_else(|| panic!("a {id}"))
}

#[test]
fn in_the_den_nobody_fights() {
    let mut w = den();
    let player = w.player_id();
    let before: Vec<(ActorId, i32)> = (w.state.actors.values())
        .filter(|a| a.id != player)
        .map(|a| (a.id, a.hp))
        .collect();
    assert!(before.len() >= 25, "{} creatures", before.len());
    for action in parse_actions("wait*40").expect("actions") {
        w.apply(&action);
    }
    for (id, hp) in before {
        let actor = w.state.actors.get(&id).expect("still alive");
        assert!(actor.hp >= hp, "{:?} was hurt", actor.creature);
    }
}

#[test]
fn the_criminals_items_work_for_them() {
    let w = den();
    let content = w.content().clone();
    // A hand-placed criminal without a kit rolls its gear: its unique comes with it.
    for (id, unique) in CRIMINALS {
        let actor = &w.state.actors[&of_type(&w, id)];
        let at = (actor.inventory.iter())
            .position(|s| s.magic().and_then(|m| m.unique.as_deref()) == Some(unique))
            .unwrap_or_else(|| panic!("{id} carries no {unique}"));
        assert!(actor.equipment.holds(at), "{id}: {unique}");
    }
    // Brother Osrick sees in the dark through the Bedes.
    let osrick = &w.state.actors[&of_type(&w, "osrick")];
    let def = content.creature(osrick.creature);
    let base = &content.vision_profiles()[def.vision];
    assert_eq!(base.darkvision, 0, "his own eyes are a man's");
    assert!(osrick.eyes(&content, base).darkvision > 0, "the Bedes'");
    // The Turncoat wards Sergeant Herrick off fear.
    let herrick = of_type(&w, "herrick");
    let frightened = content.status_id("frightened").expect("frightened");
    let mut w = w;
    w.afflict(herrick, frightened, 1, None, &mut Vec::new());
    assert!(w.state.actors[&herrick].effects.get(frightened).is_none());
}

#[test]
fn named_monsters_go_by_their_proper_names() {
    use crate::content::{the, the_start};
    for (id, name) in [
        ("sallow", "Mother Sallow"),
        ("wystan", "Black Wystan"),
        ("osrick", "Brother Osrick"),
        ("herrick", "Sergeant Herrick"),
    ] {
        let def = CONTENT.creature(CONTENT.creature_id(id).expect(id));
        assert_eq!(def.name, name);
        assert_eq!(the(&def.name), name);
        assert_eq!(the_start(&def.name), name);
    }
    assert_eq!(the("footpad"), "the footpad");
    assert_eq!(the_start("footpad archer"), "The footpad archer");
    let w = den();
    let wystan = &w.state.actors[&of_type(&w, "wystan")];
    assert_eq!(w.called(wystan), "Black Wystan");
    assert_eq!(w.creature_called(wystan.id), "Black Wystan");
    let footpad = &w.state.actors[&of_type(&w, "footpad")];
    assert_eq!(w.called(footpad), "the footpad");
}

#[test]
fn brother_osricks_file_keeps_the_note_on_his_spells() {
    let file = include_str!("../../../../assets/campaigns/ascmearc/data/creatures/osrick.toml");
    let note = "# TODO(creature spells): Brother Osrick is a necromancer who fights with a mace only because\n\
                # creatures cannot cast yet. Once creature spellcasters exist, give him spells (raising the dead,\n\
                # a curse, cold): docs/design/adventurers-guild.md §15.\n";
    assert!(file.starts_with(note), "the note opens his file");
}
