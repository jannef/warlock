//! The saved characters (spec §7 "Saves"): each character has a save of its own, named by an id
//! made of a slug of the name and a number (`brakka-1`, `brakka-2`: two Brakkas live side by
//! side). Beside each save (`saves/<id>.sav`) lies its summary (`saves/<id>.toml`): what the
//! list of saved characters shows (name, race, class, level, place and campaign, when last
//! played), so the
//! list reads no save.
//!
//! When a character was last played is wall-clock time, which only the app knows: seconds since
//! 1970 (UTC), shown in UTC.

use crate::settings::Clock;
use serde::{Deserialize, Serialize};
use warlock_core::World;
use warlock_core::character::NAMELESS;

/// The longest slug of a name in an id.
const MAX_SLUG: usize = 20;

/// What the list of saved characters shows of one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub name: String,
    /// The race's name (`Dwarf`); empty for a run without a character.
    #[serde(default)]
    pub race: String,
    /// The class's name (`Warrior`).
    #[serde(default)]
    pub class: String,
    /// The experience level.
    #[serde(default)]
    pub level: u32,
    /// Where the character is: the level's name.
    #[serde(default)]
    pub place: String,
    /// The world the character is in: its campaign's name (`The Ascmearc`); empty in a summary
    /// written before it was kept, or for a run outside every campaign.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub campaign: String,
    /// When the character was last saved: seconds since 1970 (UTC).
    #[serde(default)]
    pub played: u64,
}

impl Summary {
    /// The summary of the run in `world`, saved at `played`.
    pub fn of(world: &World, played: u64) -> Self {
        let view = world.perception();
        let content = world.content();
        let (name, race, class) = match world.character() {
            Some(c) => (
                c.name.clone(),
                (c.spec.race(content)).map_or(c.spec.race.clone(), |r| r.name.clone()),
                (c.spec.class(content)).map_or(c.spec.class.clone(), |k| k.name.clone()),
            ),
            None => (NAMELESS.to_string(), String::new(), String::new()),
        };
        Self {
            name,
            race,
            class,
            level: u32::from(view.own_experience().level),
            place: view.level_name().to_string(),
            campaign: (content.campaign(world.campaign()))
                .map_or_else(String::new, |c| c.name.clone()),
            played,
        }
    }

    /// The summary file's text.
    pub fn to_text(&self) -> String {
        // Plain strings and numbers: serializing cannot fail.
        toml::to_string(self).unwrap_or_default()
    }

    /// Reads a summary file's text; `None` when it is not one.
    pub fn from_text(text: &str) -> Option<Self> {
        toml::from_str(text).ok()
    }

    /// Race and class together: `Dwarf Warrior` (either may be missing).
    pub fn race_and_class(&self) -> String {
        [self.race.as_str(), self.class.as_str()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// A saved character: its save's id and, if it could be read, its summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedCharacter {
    pub id: String,
    pub summary: Option<Summary>,
}

impl SavedCharacter {
    /// The name the list shows: the summary's, else the id.
    pub fn name(&self) -> &str {
        self.summary.as_ref().map_or(&self.id, |s| &s.name)
    }

    /// When it was last played, if known.
    fn played(&self) -> u64 {
        self.summary.as_ref().map_or(0, |s| s.played)
    }
}

/// Sorts saved characters for the list: the one played last first, then by id.
pub fn sort(characters: &mut [SavedCharacter]) {
    characters.sort_by(|a, b| b.played().cmp(&a.played()).then(a.id.cmp(&b.id)));
}

/// `name` as the first part of a save's id: lower-case ASCII letters and digits, anything else
/// a single `-` between them, at most [`MAX_SLUG`] long; `character` when nothing is left.
pub fn slug(name: &str) -> String {
    let mut slug = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.truncate(MAX_SLUG);
    let slug = slug.trim_end_matches('-');
    if slug.is_empty() {
        "character".into()
    } else {
        slug.into()
    }
}

/// A new save id for a character named `name`: its [`slug`] and the lowest number from 1 that
/// `taken` does not refuse (`brakka-1`, then `brakka-2`).
pub fn fresh_id(name: &str, taken: impl Fn(&str) -> bool) -> String {
    let slug = slug(name);
    (1u32..)
        .map(|n| format!("{slug}-{n}"))
        .find(|id| !taken(id))
        .unwrap_or_else(|| format!("{slug}-0"))
}

/// Seconds since 1970 (UTC), now.
pub fn now() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (web_sys::js_sys::Date::now() / 1000.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    }
}

/// `secs` (since 1970, UTC) as the list shows when a character was last played:
/// `2026-10-20 14:32 UTC`, the time of day as `clock` writes it.
pub fn played_text(secs: u64, clock: Clock) -> String {
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    // Civil-from-days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    let time = clock.format_hm((rem / 3600) as u32, (rem % 3600 / 60) as u32);
    format!("{year:04}-{month:02}-{day:02} {time} UTC")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::content::Content;

    #[test]
    fn slugs_keep_letters_and_digits() {
        assert_eq!(slug("Brakka"), "brakka");
        assert_eq!(slug("Anne-Marie O'Hara"), "anne-marie-o-hara");
        assert_eq!(slug("  Ånne  "), "nne");
        assert_eq!(slug("ÅÄÖ"), "character");
        assert_eq!(slug("Brakka the Unbroken of Deep"), "brakka-the-unbroken");
        assert!(slug(&"x".repeat(40)).len() <= MAX_SLUG);
    }

    #[test]
    fn fresh_ids_take_the_lowest_free_number() {
        let taken = ["brakka-1", "brakka-2", "brakka-4"];
        assert_eq!(fresh_id("Brakka", |id| taken.contains(&id)), "brakka-3");
        assert_eq!(fresh_id("Tamsin", |id| taken.contains(&id)), "tamsin-1");
    }

    #[test]
    fn summaries_round_trip_and_describe_the_run() {
        let content = Arc::new(Content::builtin().expect("content"));
        let world = World::new(content, "showcase", None).expect("world");
        let summary = Summary::of(&world, 1_781_654_400);
        assert_eq!(summary.name, "Tamsin");
        assert_eq!(summary.race_and_class(), "Human Thief");
        assert_eq!(summary.level, 1);
        assert_eq!(summary.place, world.perception().level_name());
        assert_eq!(summary.campaign, "The Showcase");
        assert_eq!(Summary::from_text(&summary.to_text()), Some(summary));
        assert_eq!(Summary::from_text("not = [a summary"), None);
    }

    #[test]
    fn the_list_puts_the_character_played_last_first() {
        let saved = |id: &str, played: Option<u64>| SavedCharacter {
            id: id.into(),
            summary: played.map(|played| Summary {
                name: id.into(),
                race: String::new(),
                class: String::new(),
                level: 1,
                place: String::new(),
                campaign: String::new(),
                played,
            }),
        };
        let mut list = vec![
            saved("a-1", Some(5)),
            saved("b-1", None),
            saved("c-1", Some(9)),
            saved("d-1", Some(5)),
        ];
        sort(&mut list);
        let ids: Vec<&str> = list.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["c-1", "a-1", "d-1", "b-1"]);
        assert_eq!(list[3].name(), "b-1");
    }

    #[test]
    fn played_times_are_written_in_utc() {
        assert_eq!(played_text(0, Clock::H24), "1970-01-01 00:00 UTC");
        assert_eq!(
            played_text(1_781_654_400 + 14 * 3600 + 32 * 60, Clock::H24),
            "2026-06-17 14:32 UTC"
        );
        assert_eq!(
            played_text(1_781_654_400 + 14 * 3600 + 32 * 60, Clock::H12),
            "2026-06-17 2:32 PM UTC"
        );
    }
}
