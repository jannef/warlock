//! Weather through the whole turn pipeline (spec §5 "Weather"): known only under the open sky,
//! clouds dimming the sun, the sky, and the moon, rain shortening overworld sight, messages when
//! it changes, rests woken by rain, and the dev override in replays and saves.

use super::light_tests::check_invariants;
use super::*;
use crate::action::parse_actions;
use crate::content::Climate;
use crate::fov::radius2;
use crate::light::Sun;
use crate::perception::Emitter;
use crate::replay::Replay;
use crate::weather::{Clouds, Weather, weather_at};
use std::sync::LazyLock;

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

/// From the showcase start to the courtyard, up the stairs nook's `<`.
const TO_COURTYARD: &str =
    "move:N*3 open:N move:N*2 move:E*67 open:S move:S*4 move:E*2 move:NE ascend";

fn run(world: &mut World, text: &str) {
    for word in text.split_whitespace() {
        let (action, times) = match word.split_once('*') {
            Some((a, n)) => (a, n.parse().expect("a count")),
            None => (word, 1),
        };
        let action: Action = action.parse().expect("an action");
        for _ in 0..times {
            world.apply(&action);
        }
    }
}

fn world(scenario: &str) -> World {
    World::new(CONTENT.clone(), scenario, None).expect("scenario starts")
}

fn climate() -> &'static Climate {
    CONTENT.climate()
}

fn texts(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

/// The weather messages the player could get, in the order they were logged.
fn weather_messages(world: &World) -> Vec<String> {
    const NEWS: [&str; 8] = [
        "Clouds gather.",
        "The sky clears.",
        "It starts to rain.",
        "It starts to pour.",
        "The rain grows heavy.",
        "The rain eases.",
        "The rain stops.",
        "The rain stops and the sky clears.",
    ];
    texts(world)
        .into_iter()
        .filter(|t| NEWS.contains(&t.as_str()))
        .collect()
}

/// The first moment after `from` (in whole minutes) when the natural weather of `seed` turns
/// from not `is(w)` to `is(w)`.
fn first_change(seed: u64, from: GameTime, is: impl Fn(Weather) -> bool) -> GameTime {
    let at = |m: u64| from + Duration::minutes(m);
    (1..60 * 24 * 60)
        .map(at)
        .find(|t| {
            let before = GameTime::from_seconds(t.seconds() - 60);
            !is(weather_at(seed, before, climate(), None))
                && is(weather_at(seed, *t, climate(), None))
        })
        .map(|t| {
            // The change is within the last minute; find its second.
            (0..60)
                .map(|s| GameTime::from_seconds(t.seconds() - 60 + s))
                .find(|s| is(weather_at(seed, *s, climate(), None)))
                .expect("within the minute")
        })
        .expect("the weather changes within two months")
}

#[test]
fn weather_is_known_only_under_the_open_sky() {
    let mut w = world("showcase");
    assert_eq!(w.perception().weather(), None, "no weather underground");
    assert!(w.perception().sky().is_none());
    assert!(!crate::dump::ascii(&w, &[]).contains("weather"));
    // Setting it underground says nothing about it beyond the dev notice.
    w.apply(&Action::SetWeather(Weather::Rain));
    assert_eq!(w.perception().weather(), None);
    assert!(weather_messages(&w).is_empty());
    // Arriving under the sky, the status bar shows it; it is not news.
    run(&mut w, TO_COURTYARD);
    assert_eq!(w.perception().level_id(), "courtyard");
    assert_eq!(w.perception().weather(), Some(Weather::Rain));
    assert!(weather_messages(&w).is_empty(), "{:?}", texts(&w));
    assert!(crate::dump::ascii(&w, &[]).contains(" weather rain "));
    // The sky's weather is the world's.
    let sky = w.perception().sky().expect("under the sky");
    assert_eq!(sky.conditions.weather, Weather::Rain);
    // Out of the rain and back in: still no news.
    w.apply(&Action::Descend);
    assert_eq!(w.perception().weather(), None);
    w.apply(&Action::SetWeather(Weather::Clear));
    w.apply(&Action::Ascend);
    assert_eq!(w.perception().weather(), Some(Weather::Clear));
    assert!(weather_messages(&w).is_empty(), "{:?}", texts(&w));
}

#[test]
fn a_change_seen_under_the_sky_is_news() {
    let mut w = world("courtyard");
    let start = w.perception().weather().expect("outdoors");
    assert_eq!(
        start,
        Weather::Clear,
        "the courtyard's first morning is clear"
    );
    for (set, news) in [
        (Weather::Overcast, "Clouds gather."),
        (Weather::Rain, "It starts to rain."),
        (Weather::HeavyRain, "The rain grows heavy."),
        (Weather::Rain, "The rain eases."),
        (Weather::Clear, "The rain stops and the sky clears."),
        (Weather::HeavyRain, "It starts to pour."),
        (Weather::Overcast, "The rain stops."),
        (Weather::Clear, "The sky clears."),
    ] {
        w.apply(&Action::SetWeather(set));
        assert_eq!(w.perception().weather(), Some(set));
        assert_eq!(texts(&w).last().map(String::as_str), Some(news));
    }
    // The same weather again is no news.
    let before = weather_messages(&w).len();
    w.apply(&Action::SetWeather(Weather::Clear));
    w.apply(&Action::Wait);
    assert_eq!(weather_messages(&w).len(), before);
}

#[test]
fn clouds_dim_the_sun_the_sky_and_the_moon() {
    // Noon in the courtyard, at the start (open ground in the sun).
    let mut w = world("courtyard");
    w.apply(&Action::SkipTime(Duration::minutes(330)));
    let here = w.player_pos();
    let mut last: Option<(u16, u16)> = None;
    for weather in Weather::ALL {
        w.apply(&Action::SetWeather(weather));
        check_invariants(&w).expect("light invariants");
        let view = w.perception();
        let sky = view.sky().expect("outdoors");
        let clouds = Clouds::of(climate().def(weather));
        let clear = Sun::at(w.time());
        assert_eq!(sky.conditions.clouds, clouds);
        assert_eq!(sky.sun.light, clouds.direct(clear.light));
        assert_eq!(sky.sun.sky, clouds.sky(clear.sky, clear.light));
        assert!(sky.sun.is_up(), "the sun is up behind the clouds");
        let light = view.light(here).expect("own cell").brightness();
        let sun = (view.light_sources().iter())
            .find(|s| matches!(s.emitter, Emitter::Sun(_)))
            .and_then(|s| s.contributions.iter().find(|(p, _)| *p == here))
            .map_or(0, |(_, l)| l.luma());
        let base = view.base_light(here).expect("own cell").luma();
        if let Some((light_before, ratio_before)) = last {
            assert!(
                light < light_before,
                "{weather}: darker ({light} after {light_before})"
            );
            // Softer shadows: the sun gives less of the light, the sky more.
            let ratio = sun * 100 / base.max(1);
            assert!(
                ratio < ratio_before,
                "{weather}: sun/sky {ratio} after {ratio_before}"
            );
        }
        last = Some((light, sun * 100 / base.max(1)));
    }
    let (heavy, _) = last.expect("measured");
    assert!(heavy >= 40, "heavy rain at noon is still daylight: {heavy}");

    // The full moon at midnight: each moon shines less, and so the moonlight.
    let mut w = world("courtyard");
    run(&mut w, "skip:113h skip:30m");
    let mut last = u16::MAX;
    for weather in Weather::ALL {
        w.apply(&Action::SetWeather(weather));
        check_invariants(&w).expect("light invariants");
        let sky = w.perception().sky().expect("outdoors").clone();
        assert!(sky.moons[0].shines(), "{weather}: the moon still shines");
        let moonlight = sky.moonlight().luma();
        assert!(moonlight < last, "{weather}: {moonlight} after {last}");
        last = moonlight;
    }
}

#[test]
fn rain_shortens_sight_on_the_overworld() {
    let mut w = world("world");
    w.apply(&Action::SkipTime(Duration::hours(4)));
    assert_eq!(w.time().hour(), 12);
    let reach = |w: &World| {
        let view = w.perception();
        (0..view.height())
            .flat_map(|y| (0..view.width()).map(move |x| Pos::new(x, y)))
            .filter(|p| view.is_visible(*p))
            .map(|p| p.dist2(view.viewer()))
            .max()
            .unwrap_or(0)
    };
    // Creatures see by the same daylight, moonlight, and rain.
    let creature = |w: &World| crate::sight::sight_radius(&w.state, w.content(), 40);
    for (weather, sight) in [
        (Weather::Clear, 9),
        (Weather::Overcast, 9),
        (
            Weather::Rain,
            9 - i32::from(climate().def(Weather::Rain).sight),
        ),
        (
            Weather::HeavyRain,
            9 - i32::from(climate().def(Weather::HeavyRain).sight),
        ),
    ] {
        w.apply(&Action::SetWeather(weather));
        let far = reach(&w);
        assert!(far <= radius2(sight), "{weather}: {far} beyond {sight}");
        assert!(far > radius2(sight - 1), "{weather}: sees {sight} cells");
        assert_eq!(creature(&w), sight, "{weather}: and so do creatures");
    }
    // At night, rain never takes sight below the darkest night's.
    w.apply(&Action::SkipTime(Duration::hours(12)));
    w.apply(&Action::SetWeather(Weather::HeavyRain));
    assert!(reach(&w) > radius2(1));
}

#[test]
fn each_change_of_weather_is_said_once() {
    // Four days and nights in the courtyard, resting a day at a time (rain wakes the player),
    // with no need to eat (spec §8 "Hunger": four days without food would starve them).
    let mut files = crate::content::ContentFiles::builtin();
    files.remove("data/hunger.toml");
    let content = Arc::new(Content::load(&files).expect("content without hunger"));
    let mut w = World::new(content, "courtyard", None).expect("scenario starts");
    let start = w.time();
    let end = start + Duration::hours(96);
    while w.time() < end {
        w.apply(&Action::Rest(Rest::For(Duration::hours(24))));
    }
    // What the weather did, minute by minute.
    let mut expected = Vec::new();
    let mut last = weather_at(w.seed(), start, climate(), None);
    let mut t = start;
    while t < w.time() {
        t += Duration::minutes(1);
        let now = weather_at(w.seed(), t, climate(), None);
        if now != last {
            expected.push((last, now));
            last = now;
        }
    }
    assert!(expected.len() >= 6, "the weather changes: {expected:?}");
    let said = weather_messages(&w);
    assert_eq!(said.len(), expected.len(), "{said:?} for {expected:?}");
    for (text, (from, to)) in said.iter().zip(&expected) {
        let rains = to.is_rain() && !from.is_rain();
        assert_eq!(
            rains,
            text.starts_with("It starts"),
            "{text}: {from} to {to}"
        );
    }
}

#[test]
fn rain_starting_stops_a_rest_and_rain_stopping_does_not() {
    // Half an hour before the rain first starts in the courtyard, rest an hour.
    let mut w = world("courtyard");
    let seed = w.seed();
    let rain = first_change(seed, w.time(), Weather::is_rain);
    let before = GameTime::from_seconds(rain.seconds() - 30 * 60);
    w.apply(&Action::SkipTime(before.since(w.time())));
    w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    let stopped = w.time();
    assert!(
        rain <= stopped && stopped.since(rain) < Duration::seconds(7),
        "{rain} {stopped}"
    );
    let texts = texts(&w);
    let n = texts.len();
    assert!(texts[n - 2].starts_with("It starts to"), "{texts:?}");
    assert!(
        texts[n - 1].starts_with("You stop resting after 30 minutes"),
        "{texts:?}"
    );

    // When the rain stops, the rest goes on.
    let dry = first_change(seed, rain, |w| !w.is_rain());
    let mut w = world("courtyard");
    let before = GameTime::from_seconds(dry.seconds() - 30 * 60);
    w.apply(&Action::SkipTime(before.since(w.time())));
    w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(w.time(), before + Duration::hours(1));
    let texts = self::texts(&w);
    let n = texts.len();
    assert!(texts[n - 2].starts_with("The rain stops"), "{texts:?}");
    assert_eq!(texts[n - 1], "You rest for 1 hour.");
}

#[test]
fn the_override_lasts_three_hours_and_replays_and_saves() {
    let actions = parse_actions(
        "skip:330m weather:heavy wait move:E rest:1h weather:overcast wait skip:2h wait \
         skip:1h wait move:W",
    )
    .expect("actions parse");
    let mut a = world("courtyard");
    let mut replay = Replay::new(&a);
    for action in &actions {
        a.apply(action);
        replay.push(action, &a);
        if *action == Action::Wait {
            let set = a.state.weather_override;
            let natural = weather_at(a.seed(), a.time(), climate(), None);
            match set {
                Some(o) => assert_eq!(a.perception().weather(), Some(o.weather)),
                None => assert_eq!(a.perception().weather(), Some(natural)),
            }
        }
    }
    assert_eq!(a.state.weather_override, None, "it ran out and was dropped");
    let replayed = replay.run(CONTENT.clone()).expect("the replay verifies");
    assert_eq!(replayed.state_hash(), a.state_hash());
    assert_eq!(replayed.perception, a.perception);

    // A save keeps the override and so the weather.
    let mut b = world("courtyard");
    run(&mut b, "skip:330m weather:rain wait");
    let loaded = crate::save::decode(&crate::save::encode(&b), CONTENT.clone()).expect("loads");
    assert_eq!(loaded.state(), b.state());
    assert_eq!(loaded.perception().weather(), Some(Weather::Rain));
    assert_eq!(loaded.perception, b.perception);
    // A state without an override says nothing of it (saves and hashes from before stay valid).
    let json = serde_json::to_string(a.state()).expect("serializes");
    assert!(!json.contains("weather"));
}

#[test]
fn the_dev_action_takes_no_time() {
    let mut w = world("courtyard");
    let (turn, time) = (w.turn(), w.time());
    let outcome = w.apply(&Action::SetWeather(Weather::Rain));
    assert!(!outcome.took_turn);
    assert_eq!((w.turn(), w.time()), (turn, time));
    let texts = texts(&w);
    assert_eq!(texts[texts.len() - 2], "Weather: rain for 3 hours (dev).");
    assert_eq!(texts[texts.len() - 1], "It starts to rain.");
}
