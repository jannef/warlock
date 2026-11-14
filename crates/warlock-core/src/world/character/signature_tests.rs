//! The layer bosses' signature gifts (docs/design/profane-gifts.md §5; P4, #1627): each corpse's
//! own gift held after the meal, its stats moved, its resistance totals; the two wyrms' sums
//! (#1620: uncapped totals, 75 effective); one Fire breath, the larger; the abilities' source.

use crate::ability::{Ability, Source};
use crate::action::Meal;
use crate::content::{ContentFiles, GiftTier, SignActive};
use crate::dump::{self, Layer};
use crate::stats::Stat;
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

/// A human warrior in the pantry.
fn eater() -> World {
    let mut files = ContentFiles::builtin();
    files.insert("levels/pantry.toml", PANTRY);
    files.insert(
        "scenarios/pantry.toml",
        "name = \"pantry\"\nstart_level = \"pantry\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let spec: crate::character::CharacterSpec = "human,male,warrior".parse().expect("a character");
    World::start_run(content, "pantry", Some(7), Some(&spec), None, true).expect("world")
}

/// Lays a fresh corpse of `creature` on the player's cell and eats it.
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

/// Gives the player every drawn gift they may take, the dire first (Withered would undo
/// Oak-hearted), so a meal's draws find nothing and only its signature takes.
fn empty_the_table(w: &mut World) {
    for tier in [GiftTier::Dire, GiftTier::Greater, GiftTier::Lesser] {
        let all: Vec<_> = (w.content.profane().iter())
            .filter(|g| g.tier == tier && g.drawn())
            .map(|g| g.talent)
            .collect();
        for t in all {
            let content = w.content.clone();
            if (content.profane().gift(t)).is_some_and(|g| w.may_take_gift(g)) {
                w.bestow_gift(t);
            }
        }
    }
    for tier in GiftTier::ALL {
        assert_eq!(w.gift_pool(tier).0, 0, "{tier:?} emptied");
    }
}

fn holds(w: &World, id: &str) -> bool {
    let me = w.state.player();
    me.talents.has(id) && me.boons.iter().any(|b| b.id == id)
}

fn kind(w: &World, id: &str) -> crate::content::DamageTypeId {
    w.content.damage_type_id(id).expect(id)
}

/// The player's resistance total to `damage` (uncapped) and what of it counts.
fn resists(w: &World, damage: &str) -> (i32, i32) {
    let all = w.resistances_of(w.state.player);
    let k = Some(kind(w, damage));
    (all.total(k), all.get(k))
}

const RESISTED: [&str; 7] = [
    "poison",
    "cold",
    "fire",
    "necrotic",
    "electricity",
    "slashing",
    "piercing",
];

/// A corpse: its keeper, its signature, the stats it moves, the resistance totals it adds.
type Signature = (
    &'static str,
    &'static str,
    &'static [(Stat, i32)],
    &'static [(&'static str, i32)],
);

/// Each corpse's.
const SIGNATURES: [Signature; 9] = [
    ("attorcoppe", "spinners_blood", &[], &[("poison", 100)]),
    (
        "hrimwyrm",
        "rime_blooded",
        &[(Stat::Strength, 1), (Stat::Toughness, 1)],
        &[("cold", 100), ("fire", -25)],
    ),
    (
        "wealaf",
        "barrow_kings_will",
        &[(Stat::Willpower, 2)],
        &[("necrotic", 50)],
    ),
    (
        "grundwyrgen",
        "mere_wifes_hide",
        &[(Stat::Strength, 2)],
        &[],
    ),
    (
        "ligetdry",
        "storm_wrights_marrow",
        &[(Stat::Intelligence, 2)],
        &[("electricity", 100)],
    ),
    ("staneoten", "hurlers_arm", &[(Stat::Strength, 3)], &[]),
    (
        "nihtscinna",
        "hollow_of_the_night",
        &[(Stat::Presence, -2)],
        &[("slashing", 25), ("piercing", 25)],
    ),
    (
        "waelcyrge",
        "choosers_quickness",
        &[(Stat::Dexterity, 3)],
        &[],
    ),
    (
        "nithdraca",
        "hate_wyrms_heart",
        &[(Stat::Strength, 2), (Stat::Toughness, 2)],
        &[("fire", 100), ("cold", -25)],
    ),
];

#[test]
fn each_corpse_gives_its_signature_its_stats_and_its_resistances() {
    for (boss, gift, stats, resist) in SIGNATURES {
        let mut w = eater();
        empty_the_table(&mut w);
        let before = w.state.player().stats;
        let totals: Vec<i32> = RESISTED.iter().map(|d| resists(&w, d).0).collect();
        eat(&mut w, boss);
        assert!(holds(&w, gift), "{boss}: {gift}");
        let messages: Vec<&str> = w.messages().iter().map(|m| m.text.as_str()).collect();
        assert!(
            messages.contains(&"Nothing more of it will take in you."),
            "{boss}: {messages:?}"
        );
        let after = w.state.player().stats;
        for stat in Stat::ALL {
            let moved = stats
                .iter()
                .find(|(s, _)| *s == stat)
                .map_or(0, |(_, v)| *v);
            let delta = i32::from(after.score(stat)) - i32::from(before.score(stat));
            assert_eq!(delta, moved, "{boss}: {stat:?}");
        }
        for (i, damage) in RESISTED.iter().enumerate() {
            let added = resist
                .iter()
                .find(|(d, _)| d == damage)
                .map_or(0, |(_, v)| *v);
            assert_eq!(resists(&w, damage).0 - totals[i], added, "{boss}: {damage}");
        }
        // It always takes: no vomiting.
        assert!(!messages.contains(&"You vomit!"), "{boss}");
    }
}

#[test]
fn the_storm_wrights_corpse_teaches_decipher_script_beside_its_gift() {
    let mut w = eater();
    let id = w.content().skill_id("decipher_script").expect("the skill");
    assert_eq!(w.state.player().skills.value(id), None);
    eat(&mut w, "ligetdry");
    assert_eq!(w.state.player().skills.value(id), Some(10));
    assert!(holds(&w, "storm_wrights_marrow"));
}

#[test]
fn the_two_wyrms_sum_and_a_hundred_counts_as_seventy_five() {
    let mut w = eater();
    let gift = |w: &World, id: &str| w.content.talents().id(id).expect(id);
    let rime = gift(&w, "rime_blooded");
    w.bestow_gift(rime);
    // A hundred to cold: 75 of it counts (#1620), and no immunity; fire is 25 worse.
    assert_eq!(resists(&w, "cold"), (100, 75));
    assert_eq!(resists(&w, "fire"), (-25, -25));
    // Hate-wyrm's heart's cold −25 is the curse the surplus covers: still 75 to cold.
    let heart = gift(&w, "hate_wyrms_heart");
    w.bestow_gift(heart);
    assert_eq!(resists(&w, "cold"), (75, 75));
    assert_eq!(resists(&w, "fire"), (75, 75));
}

#[test]
fn a_wyverns_breath_and_the_hate_wyrms_heart_are_one_breath_the_larger() {
    let mut w = eater();
    w.player_mut().talents.take("fire_breath", None);
    let small = w
        .state
        .player()
        .talent_mods(&w.content)
        .sign
        .fire_breath
        .clone();
    let small = small.expect("the Wyvern's breath");
    let heart = w
        .content
        .talents()
        .id("hate_wyrms_heart")
        .expect("the gift");
    w.bestow_gift(heart);
    let mods = w.state.player().talent_mods(&w.content);
    let breath = mods.sign.fire_breath.as_ref().expect("a breath");
    assert!(breath.bolt.damage.mean_x2() > small.bolt.damage.mean_x2());
    assert_eq!(breath.bolt.length, 10);
    // One row, and the sign's: a sign talent gives it too.
    let view = w.perception();
    let rows: Vec<_> = (view.abilities().into_iter())
        .filter(|o| o.ability == Ability::Sign(SignActive::FireBreath))
        .collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source, Source::Sign);
    // Taken the other way round, the larger all the same.
    let mut w = eater();
    w.bestow_gift(heart);
    w.player_mut().talents.take("fire_breath", None);
    let mods = w.state.player().talent_mods(&w.content);
    assert_eq!(
        mods.sign.fire_breath.as_ref().map(|b| b.bolt.length),
        Some(10)
    );
}

#[test]
fn a_breath_only_a_gift_gives_is_listed_as_profane() {
    let mut w = eater();
    let heart = w
        .content
        .talents()
        .id("hate_wyrms_heart")
        .expect("the gift");
    w.bestow_gift(heart);
    let rows = dump::layer_rows(&w, Layer::Abilities);
    assert!(
        rows.iter().any(|r| r == "Fire breath (profane): a cell"),
        "{rows:?}"
    );
    // Its own group, after the birth sign's and before the class's.
    let sources: Vec<Source> = w
        .perception()
        .abilities()
        .iter()
        .map(|o| o.source)
        .collect();
    assert!(sources.is_sorted(), "{sources:?}");
}
