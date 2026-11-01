//! The calendar and moon flavour of the gifts (spec §8 "Birth sign"; `[flavour]`): the
//! birthday heals faster and is noted at its dawn, the birth month halves the sign actives'
//! cooldowns, and a full moon lengthens the gifts' reaches; content without the table has none
//! of it.

use super::flavour::BIRTHDAY;
use super::*;
use crate::ability::Ability;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Sign, SignActive};
use crate::time::{Duration, GameTime};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// The built-in content files with the horoscope's `[flavour]` table replaced by `table`.
fn with_flavour(table: &str) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.edit_table("horoscope", |text| {
        let start = text.find("\n[flavour]\n").expect("the table") + 1;
        let end = start + text[start..].find("\n\n").expect("its end");
        let mut cut = text.to_owned();
        cut.replace_range(start..end, table);
        cut
    });
    files
}

/// The built-in content without the horoscope's `[flavour]` table.
static PLAIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&with_flavour("")).expect("content")));

/// 11 Darkening 1444 (the default start) at `hour`:`minute`, plus `days`.
fn at(days: u64, hour: u32, minute: u32) -> GameTime {
    GameTime::from_calendar(1444, 11, 11, hour, minute).expect("a date")
        + Duration::hours(24 * days)
}

/// A human warrior in the showcase born under the `element` `animal` on `month`/`day`, the
/// clock at `time`; restored, so the gifts are granted as a load grants them.
fn born(content: &Arc<Content>, sign: (&str, &str), month: u32, day: u32, time: GameTime) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let w =
        World::with_character(content.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let mut state = w.state().clone();
    let character = state.character.as_mut().expect("a character");
    character.sign = Some(Sign {
        animal: sign.1.into(),
        element: sign.0.into(),
    });
    (character.born.month, character.born.day) = (month, day);
    state.time = time;
    World::from_state(content.clone(), state).expect("restores")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn said(w: &World, text: &str) -> usize {
    (w.messages().iter())
        .filter(|m| m.display() == text)
        .count()
}

/// Hit points (and the banked part) actor `id` gets back over `seconds` from `start`, lived
/// through in `parts` equal pieces.
fn healed(w: &World, start: GameTime, seconds: u64, parts: u64) -> (i32, u32) {
    let mut w = w.clone();
    let id = w.state.player;
    w.player_mut().hp = 1;
    w.player_mut().regen = 0;
    let mut events = Vec::new();
    for i in 0..parts {
        let from = start + Duration::seconds(i * seconds / parts);
        w.pass_time(std::iter::once(id), from, seconds / parts, &mut events);
    }
    let me = w.state.player();
    (me.hp, me.regen)
}

#[test]
fn the_birthday_heals_a_quarter_faster_from_midnight_to_midnight() {
    // Born on the 12th of Darkening; today is the 11th.
    let w = born(&BUILTIN, ("wood", "salamander"), 11, 12, at(0, 12, 0));
    let id = w.state.player;
    let today = w.healing_rate_at(id, at(0, 23, 59));
    assert_eq!(w.healing_rate_at(id, at(1, 0, 0)), today * 125 / 100);
    assert_eq!(w.healing_rate_at(id, at(1, 23, 59)), today * 125 / 100);
    assert_eq!(
        w.healing_rate_at(id, at(2, 0, 0)),
        today,
        "over at midnight"
    );
    // Exact however the time is cut: across both midnights, in one piece or in many.
    let start = at(0, 20, 0);
    let whole = healed(&w, start, 2 * 24 * 3600, 1);
    assert_eq!(whole, healed(&w, start, 2 * 24 * 3600, 7));
    assert_eq!(whole, healed(&w, start, 2 * 24 * 3600, 48));
    // Without the table, a birthday is a day like any other.
    let plain = born(&PLAIN, ("wood", "salamander"), 11, 12, at(0, 12, 0));
    assert_eq!(plain.healing_rate_at(id, at(1, 12, 0)), today);
}

#[test]
fn the_birthday_is_noted_at_its_dawn_and_when_a_run_begins_on_it() {
    let mut w = born(&BUILTIN, ("wood", "salamander"), 11, 12, at(1, 5, 50));
    w.perceived_at = w.state.time;
    run(&mut w, "rest:5m");
    assert_eq!(said(&w, BIRTHDAY), 0, "not before dawn");
    run(&mut w, "rest:10m rest:1h");
    assert_eq!(said(&w, BIRTHDAY), 1, "at dawn, once");
    // Another day's dawn says nothing.
    let mut other = born(&BUILTIN, ("wood", "salamander"), 11, 13, at(1, 5, 50));
    run(&mut other, "rest:1h");
    assert_eq!(said(&other, BIRTHDAY), 0);
    // A run that begins on the birthday says so; without the table, nothing.
    let w = born(&BUILTIN, ("wood", "salamander"), 11, 11, at(0, 12, 0));
    assert_eq!(w.birthday_welcome(), Some(BIRTHDAY));
    let w = born(&BUILTIN, ("wood", "salamander"), 11, 12, at(0, 12, 0));
    assert_eq!(w.birthday_welcome(), None);
    let plain = born(&PLAIN, ("wood", "salamander"), 11, 11, at(0, 12, 0));
    assert_eq!(plain.birthday_welcome(), None);
}

#[test]
fn in_the_birth_month_a_sign_actives_cooldown_lasts_half_as_long() {
    let graceless = |w: &World| {
        let status = w.content().status_id("graceless").expect("graceless");
        w.state.player().effects.get(status).copied()
    };
    let used = |content: &Arc<Content>, month: u32| {
        let mut w = born(content, ("wood", "unicorn"), month, 2, at(0, 12, 0));
        run(&mut w, "afflict:bleeding");
        assert!(
            w.apply(&Action::Gift(SignActive::HornsGrace, None))
                .took_turn
        );
        w
    };
    // Three degrees of eight hours: a day.
    let other = used(&BUILTIN, 3);
    let e = graceless(&other).expect("graceless");
    // (The use's own seconds have worn it a little since.)
    let worn = e.wear;
    assert_eq!(e.severity, 3);
    // In Darkening, the birth month: twelve hours, two degrees with the second's first four
    // hours worn.
    let mut w = used(&BUILTIN, 11);
    let e = graceless(&w).expect("graceless");
    assert_eq!((e.severity, e.wear), (2, 4 * 3600 + worn));
    run(&mut w, "rest:11h");
    assert!(graceless(&w).is_some(), "eleven hours on, still");
    run(&mut w, "rest:1h");
    assert!(graceless(&w).is_none(), "twelve, gone");
    // Without the table, a day in any month.
    let plain = used(&PLAIN, 11);
    assert_eq!(graceless(&plain).map(|e| e.severity), Some(3));
}

#[test]
fn a_full_moon_lengthens_the_gifts_reaches() {
    // The built-in moon is full at the start of the 16th; at midnight it is high.
    let full = at(5, 0, 0);
    let content = &**BUILTIN;
    assert_eq!(content.full_moon_reach(full), 2);
    assert_eq!(content.full_moon_reach(at(5, 12, 0)), 0, "by day");
    assert_eq!(content.full_moon_reach(at(1, 0, 0)), 0, "a waxing moon");
    assert_eq!(content.full_moon_reach(at(6, 0, 0)), 0, "a day past full");
    assert_eq!(PLAIN.full_moon_reach(full), 0, "no table, no flavour");
    let stare = Ability::Sign(SignActive::TransfixingStare);
    assert_eq!(stare.reach_at(content, full), Some(STARE_RANGE + 2));
    assert_eq!(stare.reach_at(content, at(5, 12, 0)), Some(STARE_RANGE));
    // Only the gifts' reaches: Hound's nose's scent and Eyes of the heights' night sight are
    // marked, Blood scent and Moon-eyed are not.
    let hound = born(&BUILTIN, ("wood", "barghest"), 3, 2, full);
    let mods = hound.state.player().talent_mods(content);
    let bleeding = content.status_id("bleeding").expect("bleeding");
    assert_eq!(mods.gift_reach.scent, [bleeding]);
    let roc = born(&BUILTIN, ("wood", "roc"), 3, 2, full);
    assert!(
        roc.state
            .player()
            .talent_mods(content)
            .gift_reach
            .night_sight
    );
    let mut plain = crate::talent::Talents::default();
    plain.take("blood_scent", None);
    assert_eq!(plain.mods(content).gift_reach, Default::default());
    // Inner light burns as its full-moon preset.
    let flavour = content.flavour().expect("the table");
    let ember = content.light_id("ember_full");
    assert_eq!(flavour.inner_light_at(full, content.moons()), ember);
    assert_eq!(flavour.inner_light_at(at(5, 12, 0), content.moons()), None);
}

#[test]
fn the_flavour_table_is_checked() {
    let bad = |table: &str, why: &str| {
        let e = Content::load(&with_flavour(table)).expect_err("refused");
        assert!(e.to_string().contains(why), "{why}: {e}");
    };
    bad(
        "[flavour]\nbirthday_healing = 0",
        "`birthday_healing` is 1 to 400 percent",
    );
    bad(
        "[flavour]\nbirth_month_cooldown = 401",
        "`birth_month_cooldown` is 1 to 400",
    );
    bad(
        "[flavour]\nfull_moon_reach = 11",
        "`full_moon_reach` is 0 to 10 cells",
    );
    bad(
        "[flavour]\nfull_moon_light = \"nope\"",
        "an unknown light `nope`",
    );
    bad("[flavour]\nbirthday = 1", "unknown field");
    // The built-in table, and none.
    let flavour = BUILTIN.flavour().expect("the table");
    assert_eq!(
        (
            flavour.birthday_healing,
            flavour.birth_month_cooldown,
            flavour.full_moon_reach
        ),
        (125, 50, 2)
    );
    assert!(PLAIN.flavour().is_none());
}
