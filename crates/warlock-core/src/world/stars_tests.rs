//! The birth sign's gifts (spec §8 "Birth sign"): a sign grants its animal's and its element's
//! talents, a lucky day one more and a title; they are computed, never stored as a choice, so a
//! restored run gets them afresh; and each of the first gifts changes the one rule it names.

use super::search::{Hidden, Look};
use super::*;
use crate::burden::Burden;
use crate::character::CharacterSpec;
use crate::combat::{self, Target};
use crate::content::{ContentFiles, Sign};
use crate::dump::{self, Layer};
use crate::rng::Rng;
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human warrior in the showcase, born under the `element` `animal` on day `day` of the
/// month (the run restored with that birth, so the gifts are granted as a load grants them).
fn born_under(animal: &str, element: &str, day: u32) -> World {
    born_with(BUILTIN.clone(), animal, element, day)
}

fn born_with(content: Arc<Content>, animal: &str, element: &str, day: u32) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let w =
        World::with_character(content.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let mut state = w.state().clone();
    let character = state.character.as_mut().expect("a character");
    character.sign = Some(Sign {
        animal: animal.into(),
        element: element.into(),
    });
    character.born.day = day;
    World::from_state(content, state).expect("restores")
}

/// The talents held except a class's abilities (a warrior's come with the character).
fn held(w: &World) -> Vec<String> {
    let book = w.content().talents();
    (w.state.player().talents.iter())
        .filter(|t| {
            book.id(&t.id)
                .is_none_or(|id| !book.tree(book.get(id).tree).class)
        })
        .map(|t| t.id.clone())
        .collect()
}

fn player(w: &World) -> &Actor {
    w.state.player()
}

fn mods(w: &World) -> crate::talent::TalentMods {
    player(w).talent_mods(w.content())
}

#[test]
fn a_sign_grants_its_animals_then_its_elements_gifts_and_a_lucky_day_one_more() {
    let w = born_under("salamander", "wood", 2);
    assert_eq!(
        held(&w),
        ["fleet_footed", "slip_away", "slight", "supple", "evergreen"]
    );
    let c = w.character().expect("a character");
    assert_eq!(c.title(w.content()), None, "no lucky day on the 2nd");
    let lucky = born_under("salamander", "wood", 1);
    assert_eq!(
        held(&lucky),
        [
            "fleet_footed",
            "slip_away",
            "slight",
            "supple",
            "evergreen",
            "winners_luck"
        ]
    );
    let c = lucky.character().expect("a character");
    assert_eq!(c.title(lucky.content()), Some("the Racer"));
    assert_eq!(
        c.sign_title(lucky.content()).as_deref(),
        Some("Wood Salamander, the Racer")
    );
    // A sign whose gifts are still to come grants nothing yet.
    assert!(held(&born_under("phoenix", "fire", 5)).is_empty());
}

#[test]
fn the_five_lucky_days_are_pinned() {
    let h = BUILTIN.horoscope().expect("a horoscope");
    let days: Vec<(&str, &str, u32, &str)> = (h.lucky.iter())
        .map(|l| {
            (
                l.element.as_str(),
                l.animal.as_str(),
                l.day,
                l.title.as_str(),
            )
        })
        .collect();
    assert_eq!(
        days,
        [
            ("wood", "salamander", 1, "the Racer"),
            ("metal", "manticore", 3, "the Iron Scourge"),
            ("fire", "phoenix", 10, "the Firebird"),
            ("water", "kraken", 11, "the Deep One"),
            ("earth", "barghest", 30, "the Gate Warden"),
        ]
    );
    // Born in the year but on another day: no lucky sign.
    let iron = Sign {
        animal: "manticore".into(),
        element: "metal".into(),
    };
    assert!(h.lucky(&iron, 3).is_some());
    assert!(h.lucky(&iron, 4).is_none());
}

#[test]
fn every_scenario_character_keeps_the_birth_and_sign_it_had() {
    // The gifts read the birth already rolled: the stream, the date, and the sign are untouched
    // (a few births pinned from before the gifts existed).
    let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
    let start = BUILTIN.scenario("showcase").expect("showcase").start;
    let s = spec.start(&BUILTIN, 1, start).expect("starts");
    let again = spec.start(&BUILTIN, 1, start).expect("starts");
    assert_eq!(s.character.born, again.character.born);
    assert_eq!(
        s.character.sign,
        Some(
            BUILTIN
                .horoscope()
                .expect("h")
                .sign_of_year(s.character.born.year)
        )
    );
}

#[test]
fn gifts_are_granted_afresh_on_restore_and_picks_are_kept() {
    let w = born_under("roc", "earth", 9);
    assert_eq!(
        held(&w),
        ["rocs_strength", "heedless", "stone_hide", "stonewise"]
    );
    // A save that lost them (or predates them) gets them back; a picked talent stays after.
    let mut state = w.state().clone();
    let actor = state.actors.get_mut(&state.player).expect("player");
    actor.talents = Default::default();
    actor.talents.take("weapon_drill", None); // the warrior's own ability, held before the pick
    actor.talents.take("hardy", None);
    let restored = World::from_state(w.content().clone(), state).expect("restores");
    assert_eq!(
        held(&restored),
        [
            "rocs_strength",
            "heedless",
            "stone_hide",
            "stonewise",
            "hardy"
        ]
    );
    // And the hash is the same as the run's that never lost them... once the pick is the same.
    let mut picked = w.state().clone();
    let actor = picked.actors.get_mut(&picked.player).expect("player");
    actor.talents.take("hardy", None);
    let picked = World::from_state(w.content().clone(), picked).expect("restores");
    assert_eq!(picked.state_hash(), restored.state_hash());
}

#[test]
fn a_run_without_a_sign_holds_no_gifts() {
    let w = World::new(BUILTIN.clone(), "showcase", Some(3)).expect("world");
    let c = w.character().expect("the showcase's character");
    let granted = c.grants(w.content());
    let held = held(&w);
    assert_eq!(held.len(), granted.len(), "exactly the sign's gifts");
    let mut files = ContentFiles::builtin();
    files.remove("data/horoscope.toml");
    let fixture = Content::load(&files).expect("content without a horoscope");
    let w = World::new(Arc::new(fixture), "showcase", Some(3)).expect("world");
    assert!(self::held(&w).is_empty());
}

#[test]
fn fleet_footed_and_winners_luck_are_quicker_and_ponderous_slower() {
    let normal = born_under("hydra", "fire", 5);
    let base = player(&normal).speed(normal.content());
    assert_eq!(
        player(&born_under("salamander", "fire", 5)).speed(&BUILTIN),
        base * 105 / 100
    );
    assert_eq!(
        player(&born_under("sphinx", "fire", 5)).speed(&BUILTIN),
        base * 95 / 100
    );
    let racer = born_under("salamander", "wood", 1);
    assert_eq!(player(&racer).speed(&BUILTIN), base * 110 / 100);
}

#[test]
fn slight_and_rocs_strength_move_the_carrying_limit() {
    let limit = |w: &World| w.actor_load(w.player_id()).expect("load").limit;
    let base = limit(&born_under("hydra", "fire", 5));
    let slight = limit(&born_under("salamander", "fire", 5));
    let roc = limit(&born_under("roc", "fire", 5));
    assert!(slight < base && base < roc, "{slight} < {base} < {roc}");
    assert_eq!(mods(&born_under("roc", "fire", 5)).door_kick, 4);
}

#[test]
fn regrowth_heals_faster_and_eases_bleeding_twice_as_fast() {
    let mut plain = born_under("manticore", "fire", 5);
    let mut hydra = born_under("hydra", "fire", 5);
    let id = plain.player_id();
    assert_eq!(hydra.healing_rate(id) * 2, plain.healing_rate(id) * 3);
    let bleeding = BUILTIN.status_id("bleeding").expect("bleeding");
    let mut events = Vec::new();
    for w in [&mut plain, &mut hydra] {
        w.afflict(id, bleeding, 2, None, &mut events);
        // A minute: plain bleeding eases a degree, the hydra's two.
        let start = w.state.time;
        w.pass_time(std::iter::once(id), start, 60, &mut events);
    }
    let severity = |w: &World| player(w).effects.get(bleeding).map_or(0, |e| e.severity);
    assert_eq!(severity(&plain), 1);
    assert_eq!(severity(&hydra), 0);
}

#[test]
fn strong_swimmers_wade_as_on_dry_land() {
    let plain = born_under("manticore", "fire", 5);
    let hydra = born_under("hydra", "fire", 5);
    let water = BUILTIN.terrain(BUILTIN.terrain_id("water").expect("water"));
    let marsh = BUILTIN.terrain(BUILTIN.terrain_id("marsh").expect("marsh"));
    let floor = BUILTIN.terrain(BUILTIN.terrain_id("floor").expect("floor"));
    let id = plain.player_id();
    let dry = plain.step_price(id, floor, false);
    assert!(plain.step_price(id, water, false) > dry);
    assert_eq!(hydra.step_price(id, water, false), dry);
    assert_eq!(hydra.step_price(id, marsh, false), dry);
    assert!(super::stars::wading(&BUILTIN, marsh));
    assert!(!super::stars::wading(&BUILTIN, floor));
    assert!(mods(&hydra).swimmer);
}

#[test]
fn pure_blood_spares_the_scores_from_poison() {
    let poisoned = BUILTIN.status_id("poisoned").expect("poisoned");
    let mut events = Vec::new();
    let mut plain = born_under("manticore", "fire", 5);
    let mut unicorn = born_under("unicorn", "fire", 5);
    let id = plain.player_id();
    let before = player(&plain).effective_stats(&BUILTIN).scores();
    plain.afflict(id, poisoned, 6, None, &mut events);
    unicorn.afflict(id, poisoned, 6, None, &mut events);
    let after = player(&plain).effective_stats(&BUILTIN).scores();
    assert_ne!(before, after, "strong poison weakens");
    assert_eq!(
        player(&unicorn).effective_stats(&BUILTIN).scores(),
        player(&unicorn).raised_stats(&BUILTIN).scores(),
        "but not the pure of blood"
    );
    assert_eq!(mods(&unicorn).disease_checks, 4);
}

#[test]
fn heedless_and_always_a_way_move_every_search_check() {
    let plain = born_under("manticore", "fire", 5);
    let roc = born_under("roc", "fire", 5);
    let water = born_under("manticore", "water", 5);
    for look in [Look::Active, Look::Passive] {
        let base = plain.search_modifier(look);
        assert_eq!(roc.search_modifier(look), base - 4);
        assert_eq!(water.search_modifier(look), base + 3);
    }
    assert_eq!(mods(&water).search_reach, 1);
}

#[test]
fn the_unloved_get_worse_prices() {
    let shops = BUILTIN.shops().expect("shops");
    let plain = born_under("hydra", "fire", 5);
    let base = favour(&BUILTIN, shops, player(&plain));
    for animal in ["manticore", "barghest"] {
        let w = born_under(animal, "fire", 5);
        assert_eq!(favour(&BUILTIN, shops, player(&w)), base - 10, "{animal}");
    }
}

#[test]
fn supple_dodges_and_stone_hide_soaks() {
    let plain = born_under("hydra", "fire", 5);
    let wood = born_under("hydra", "wood", 5);
    let earth = born_under("hydra", "earth", 5);
    let defence = |w: &World, burden| player(w).defence(&BUILTIN, burden, None).total();
    assert_eq!(
        defence(&wood, Burden::Unburdened),
        defence(&plain, Burden::Unburdened) + 1
    );
    let soak = |w: &World| player(w).protection(&BUILTIN, None).against(None);
    assert_eq!(soak(&earth), soak(&plain) + 1);
    assert_eq!(mods(&wood).burden_dex, 1);
    assert_eq!(mods(&wood).hunger_percent, -15);
    assert_eq!(mods(&wood).rot_harm_percent, -50);
}

#[test]
fn stonewise_notices_on_stone_and_digs_faster() {
    let earth = born_under("hydra", "earth", 5);
    let m = mods(&earth);
    assert!(m.stone_sense);
    assert_eq!(m.dig_percent, 25);
    // On stone it notices in passing, as a dwarf does; the human it is would not.
    let plain = born_under("hydra", "fire", 5);
    let level = earth.state.current();
    let stone = (level.terrain.iter())
        .find(|(_, t)| BUILTIN.terrain(**t).stone)
        .map(|(p, _)| p)
        .expect("stone somewhere");
    assert!(earth.notices(stone, Hidden::Door(0)));
    assert!(!plain.notices(stone, Hidden::Door(0)));
}

#[test]
fn keen_edge_threatens_lower_and_riddlers_mind_learns_more() {
    let plain = born_under("hydra", "fire", 5);
    let metal = born_under("hydra", "metal", 5);
    let threat = |w: &World| player(w).melee(&BUILTIN).threat;
    assert_eq!(threat(&metal), threat(&plain) - 1);
    assert_eq!(mods(&born_under("sphinx", "fire", 5)).xp_percent, 10);
}

#[test]
fn winners_luck_rolls_a_fumble_again_drawing_only_then() {
    // A seed whose first attack roll is a fumble and whose second hits.
    let profile = player(&born_under("hydra", "fire", 5)).melee(&BUILTIN);
    let target = Target::bare(crate::combat::Defence::from_bonuses([]));
    let seed = (0u64..)
        .find(|&s| {
            let rng = &mut Rng::stream(s, "fumble");
            combat::roll_natural(rng) == combat::FUMBLE && combat::roll_natural(rng) >= 12
        })
        .expect("a fumble");
    let plain = combat::strike(&profile, &target, &mut Rng::stream(seed, "fumble"));
    let lucky = combat::strike_with(&profile, &target, &mut Rng::stream(seed, "fumble"), true);
    assert_eq!(plain.strike, combat::Strike::Missed { by: None });
    assert!(
        matches!(lucky.strike, combat::Strike::Hit { .. }),
        "the second roll decides: {lucky:?}"
    );
    // Without a fumble the luck draws nothing more.
    let other = (seed + 1..)
        .find(|&s| combat::roll_natural(&mut Rng::stream(s, "fumble")) != combat::FUMBLE)
        .expect("no fumble");
    let mut a = Rng::stream(other, "fumble");
    let mut b = Rng::stream(other, "fumble");
    assert_eq!(
        combat::strike(&profile, &target, &mut a),
        combat::strike_with(&profile, &target, &mut b, true)
    );
    assert_eq!(a.below(1000), b.below(1000));
}

#[test]
fn sign_prerequisites_are_read_off_the_birth_and_said_in_words() {
    let w = born_under("roc", "fire", 10);
    let who = w.candidate();
    let content = w.content();
    use crate::content::Need;
    assert!(who.meets(content, &Need::Animal("roc".into()), None, None));
    assert!(!who.meets(content, &Need::Animal("hydra".into()), None, None));
    assert!(who.meets(content, &Need::Element("fire".into()), None, None));
    assert!(!who.meets(content, &Need::Lucky, None, None));
    assert!(born_under("phoenix", "fire", 10).candidate().lucky);
    assert_eq!(
        Need::Animal("roc".into()).words(content),
        "to be born in the year of the Roc"
    );
    assert_eq!(
        Need::Element("fire".into()).words(content),
        "to be Fire-born"
    );
    assert_eq!(Need::Lucky.words(content), "to be born on a lucky day");
}

#[test]
fn gifts_cannot_be_picked_and_dumps_mark_them() {
    let mut w = born_under("salamander", "wood", 1);
    w.player_mut().stats = Stats::new(PerStat::splat(18), PerStat::splat(18));
    let rows = dump::ascii(&w, &[Layer::Stats, Layer::Talents]);
    assert!(rows.contains("Sign Wood Salamander, the Racer"), "{rows}");
    assert!(rows.contains("Fleet-footed (sign)"), "{rows}");
    assert!(rows.contains("Slight (sign, flaw)"), "{rows}");
    let json = dump::json(&w, &[], false);
    assert_eq!(json["character"]["lucky"], "the Racer");
    assert_eq!(json["talents"][2]["flaw"], true);
    let outcome = w.apply(&crate::action::Action::Talent {
        id: "fleet_footed".into(),
        choice: None,
    });
    assert!(!outcome.took_turn);
    assert_eq!(held(&w).iter().filter(|t| *t == "fleet_footed").count(), 1);
}

/// The built-in content with `horoscope.toml` replaced by `text`.
fn load_horoscope(edit: impl Fn(&str) -> String) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    let text = edit(include_str!("../../../../assets/data/horoscope.toml"));
    files.insert("data/horoscope.toml", &text);
    Content::load(&files).map_err(|e| e.to_string())
}

#[test]
fn the_horoscope_and_the_talents_are_checked_against_each_other() {
    let err =
        load_horoscope(|t| t.replacen("\"fleet_footed\"", "\"fleet_footeder\"", 1)).unwrap_err();
    assert!(err.contains("unknown talent `fleet_footeder`"), "{err}");
    let err = load_horoscope(|t| t.replacen("\"fleet_footed\", ", "\"hardy\", ", 1)).unwrap_err();
    assert!(
        err.contains("not a granted talent of the `stars` tree"),
        "{err}"
    );
    let err = load_horoscope(|t| t.replacen("grants = [\"supple\"", "grants = [\"slight\"", 1))
        .unwrap_err();
    assert!(err.contains("a flaw: only an animal has a flaw"), "{err}");
    let err = load_horoscope(|t| t.replacen("day = 30", "day = 31", 1)).unwrap_err();
    assert!(err.contains("`day` is 1 to 30"), "{err}");
    let err = load_horoscope(|t| t.replacen("\"slight\"]", "\"fleet_footed\"]", 1)).unwrap_err();
    assert!(err.contains("grants `fleet_footed` twice"), "{err}");
}

/// The first seed from 1 on which `spec` is born on a lucky day at the showcase's start.
fn lucky_seed(spec: &str) -> u64 {
    let spec: CharacterSpec = spec.parse().expect("a character");
    let start = BUILTIN.scenario("showcase").expect("showcase").start;
    (1u64..100_000)
        .find(|&seed| {
            let s = spec.start(&BUILTIN, seed, start).expect("starts");
            s.character.lucky(&BUILTIN).is_some()
        })
        .expect("a lucky seed")
}

#[test]
fn the_lucky_captures_seed_is_born_on_a_lucky_day() {
    // A seed that gives the showcase's human the Water Kraken on the 11th, the Deep One.
    assert_eq!(lucky_seed("human,female,thief"), 440);
    let w = World::new(BUILTIN.clone(), "showcase", Some(440)).expect("world");
    let c = w.character().expect("a character");
    assert_eq!(c.title(&BUILTIN), Some("the Deep One"));
}
