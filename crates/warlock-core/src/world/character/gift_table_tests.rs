//! The rest of the gift table (docs/design/profane-gifts.md §6.1–§6.3; P6, #1629): a fixture run
//! that eats all ten keepers and the pools after each meal, Twice-born's rising beside the
//! Phoenix's, and the new relations at work.

use crate::action::Meal;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, GiftTier, Sign};
use crate::dump::{self, Layer};
use crate::stats::{PerStat, Stat, Stats};
use crate::test_support::{CONTENT as BUILTIN, said_text as said};
use crate::world::combat::fight::Harm;
use crate::world::*;

/// Food underfoot, nothing else about.
const PANTRY: &str = r##"
name = "Pantry"
ambient = [200, 200, 200, 0]
map = '''
#######
#@....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"##;

/// The ten keepers of Grundlēas, from the shallowest.
const KEEPERS: [&str; 10] = [
    "gnaegan",
    "attorcoppe",
    "hrimwyrm",
    "wealaf",
    "grundwyrgen",
    "ligetdry",
    "staneoten",
    "nihtscinna",
    "waelcyrge",
    "nithdraca",
];

/// A human warrior in the pantry (seed 7): born under the Phoenix, as it happens.
fn eater() -> World {
    let mut files = ContentFiles::builtin();
    files.insert("levels/pantry.toml", PANTRY);
    files.insert(
        "scenarios/pantry.toml",
        "name = \"pantry\"\nstart_level = \"pantry\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    World::start_run(content, "pantry", Some(7), Some(&spec), None, true).expect("world")
}

/// Lays a fresh corpse of `creature` on the player's cell and eats it, hungry.
fn eat(w: &mut World, creature: &str) {
    let corpse = (w.content().object_id(&format!("{creature}_corpse"))).expect("a corpse");
    let here = w.player_pos();
    let now = w.time();
    w.state
        .current_mut()
        .set_pile(here, vec![Stack::corpse(corpse, now)]);
    let player = w.state.player;
    w.state.actors.get_mut(&player).unwrap().satiation = Some(crate::hunger::Satiation::new(1000));
    w.apply(&Action::Eat(Meal::Floor(0)));
}

fn holds(w: &World, id: &str) -> bool {
    let me = w.state.player();
    me.talents.has(id) && me.boons.iter().any(|b| b.id == id)
}

fn give(w: &mut World, id: &str) {
    w.apply(&Action::Profane(id.into()));
}

/// The `progress` dump's profane row.
fn profane_row(w: &World) -> String {
    (dump::layer_rows(w, Layer::Progress).into_iter())
        .find(|r| r.starts_with("profane "))
        .expect("a profane row")
}

/// The acceptance's fixture run: a meal of each keeper in turn, the progress dump's pools after
/// each, the gifts held at the end. Every pool counts what may still be drawn of its tier, so
/// each meal's draws and what they exclude show in the counts.
#[test]
fn the_pools_after_each_of_ten_meals() {
    let mut w = eater();
    assert_eq!(
        profane_row(&w),
        "profane 0 held; pool lesser 14/14, greater 12/12, dire 14/14"
    );
    let mut rows = Vec::new();
    for keeper in KEEPERS {
        eat(&mut w, keeper);
        rows.push(format!("{keeper}: {}", profane_row(&w)));
    }
    let book = w.content.talents();
    let held: Vec<String> = (w.state.player().boons.iter())
        .filter(|b| (book.id(&b.id)).is_some_and(|t| w.content.profane().gift(t).is_some()))
        .map(|b| b.id.clone())
        .collect();
    rows.push(format!("held: {}", held.join(", ")));
    // Every signature is held, and between ten and twenty draws took (an undoer takes none).
    assert!(KEEPERS.len() <= held.len() && held.len() <= 30, "{held:?}");
    insta::with_settings!({ snapshot_path => "../../snapshots" }, {
        insta::assert_snapshot!("profane_ten_meals", rows.join("\n"));
    });
}

/// A human warrior in the showcase (seed 7) born under the fire `animal`, every score 14, whole.
fn born(animal: &str) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let mut state = w.state().clone();
    let character = state.character.as_mut().expect("a character");
    character.sign = Some(Sign {
        animal: animal.into(),
        element: "fire".into(),
    });
    character.born.day = 2;
    let mut w = World::from_state(BUILTIN.clone(), state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w
}

fn talent_rows(w: &World) -> Vec<String> {
    dump::layer_rows(w, Layer::Talents)
}

#[test]
fn a_phoenix_who_is_twice_born_rises_twice() {
    let mut w = born("phoenix");
    give(&mut w, "twice_born");
    assert!(holds(&w, "twice_born"));
    let player = w.state.player;
    let mut events = Vec::new();
    assert!(
        !w.hurt(player, 999, Harm::Barbs, &mut events),
        "rises by the sign"
    );
    assert!(w.state.progress.happened(RISEN));
    assert!(!w.state.progress.happened(PROFANE_RISEN));
    let rows = talent_rows(&w);
    assert!(
        rows.contains(&"Rise from the ashes (sign, spent)".into()),
        "{rows:?}"
    );
    assert!(rows.contains(&"Twice-born (profane)".into()), "{rows:?}");
    assert!(
        !w.hurt(player, 999, Harm::Barbs, &mut events),
        "and by the gift"
    );
    assert!(w.state.progress.happened(PROFANE_RISEN));
    let rows = talent_rows(&w);
    assert!(
        rows.contains(&"Twice-born (profane, spent)".into()),
        "{rows:?}"
    );
    assert!(
        w.hurt(player, 999, Harm::Barbs, &mut events),
        "and then dies"
    );
    assert!(w.state.death.is_some());
}

#[test]
fn twice_born_alone_rises_once() {
    let mut w = born("griffin");
    give(&mut w, "twice_born");
    let player = w.state.player;
    let mut events = Vec::new();
    assert!(!w.hurt(player, 999, Harm::Barbs, &mut events), "rises");
    assert!(said(&w, "You burst into flame and rise from your ashes!"));
    assert!(w.state.progress.happened(PROFANE_RISEN));
    assert!(
        !w.state.progress.happened(RISEN),
        "the sign's once is not spent"
    );
    assert!(
        w.hurt(player, 999, Harm::Barbs, &mut events),
        "and then dies"
    );
}

#[test]
fn wyrm_blooded_replaces_ember_veined_and_its_glow() {
    let mut w = born("griffin");
    let fire = w.content.damage_type_id("fire").expect("fire");
    let total = |w: &World| w.resistances_of(w.state.player).total(Some(fire));
    let base = total(&w);
    give(&mut w, "ember_veined");
    let glow = w.content.light_id("ember_glow").expect("the preset");
    let mods = w.state.player().talent_mods(&w.content).into_owned();
    assert_eq!(mods.senses.inner_light, Some(glow));
    assert_eq!(total(&w), base + 50);
    give(&mut w, "wyrm_blooded");
    assert!(said(&w, "The glow under your skin goes out."));
    assert!(!holds(&w, "ember_veined") && holds(&w, "wyrm_blooded"));
    let mods = w.state.player().talent_mods(&w.content).into_owned();
    assert_eq!(mods.senses.inner_light, None);
    assert_eq!(total(&w), base + 50);
}

#[test]
fn palsied_undoes_adder_quick_and_nothing_more() {
    let mut w = born("griffin");
    let dex = |w: &World| w.state.player().stats.score(Stat::Dexterity);
    let before = dex(&w);
    give(&mut w, "adder_quick");
    assert_eq!(dex(&w), before + 2);
    give(&mut w, "palsied");
    assert!(said(
        &w,
        "Your hands slow, and their quickness drains away."
    ));
    assert!(said(
        &w,
        "The palsy takes your quick hands, and nothing more."
    ));
    assert!(!holds(&w, "adder_quick") && !holds(&w, "palsied"));
    assert_eq!(dex(&w), before);
    // Without the boon it is a bane in its own right.
    give(&mut w, "palsied");
    assert!(holds(&w, "palsied"));
    assert_eq!(dex(&w), before - 3);
}

#[test]
fn scaled_excludes_the_brittle_and_the_hollow() {
    let mut w = born("griffin");
    give(&mut w, "scaled");
    let content = w.content.clone();
    let book = content.talents();
    for other in ["paper_skinned", "glass_boned", "hollowed"] {
        let gift = (content.profane())
            .gift(book.id(other).expect(other))
            .expect("a gift");
        assert!(!w.may_take_gift(gift), "{other}");
    }
    // The pools: the one held, and the lesser and the two dire it excludes.
    assert_eq!(w.gift_pool(GiftTier::Lesser).0, 13);
    assert_eq!(w.gift_pool(GiftTier::Greater).0, 11);
    assert_eq!(w.gift_pool(GiftTier::Dire).0, 12);
}

#[test]
fn wing_shouldered_bars_the_cloak() {
    let mut w = born("griffin");
    give(&mut w, "wing_shouldered");
    let mods = w.state.player().talent_mods(&w.content).into_owned();
    let cloak = w.content.slot_id("cloak").expect("a cloak slot");
    assert!(mods.slots.barred(cloak).is_some());
}
