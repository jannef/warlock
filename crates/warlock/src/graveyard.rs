//! The graveyard (spec §7 "The graveyard"; #1137, design `docs/design/monster-dread.md` §9.5):
//! `graveyard.toml` beside `settings.toml` in the config folder (portable mode's `warlock-data`
//! too; the web build's `localStorage` key `warlock/graveyard.toml`), one `[[fallen]]` table a
//! character who has died.
//!
//! - **Written** once a death, in `ui::screens::on_death` where the character's save is deleted
//!   ([`GraveyardStore::remember`]): an entry **appended**, the file never rewritten. Never by
//!   the core, never read into the rules.
//! - **Its format outlives every version** (the owner's word: the one deliberate exception to
//!   the saves' pinning, `docs/dev/saves.md`): a reader ignores the fields it does not know and
//!   defaults those it misses, and an entry it cannot read is skipped, not the file; a field once
//!   written is never renamed or given another meaning. No format number, no migration. The
//!   test `a_fixture_written_by_0_11_reads` pins it.
//! - **Read** for the main menu's Graveyard row and screen ([`by_experience`]), the Characters
//!   screen's count, and, as a run begins, for the run's dead ([`GraveyardStore::dead`], the
//!   most recent first, the petrified, `cause = "stone"`, marked), which the core keeps with
//!   the run and names statues (the petrified) and gravestones (all) from
//!   (`warlock_core::world::Fallen`).

use crate::cli::Args;
use crate::storage::{self, Backend, Folder};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use warlock_core::World;
use warlock_core::content::Content;

/// The file's name, beside `settings.toml`.
pub const FILE_NAME: &str = "graveyard.toml";

/// The `cause` of a character turned to stone.
pub const STONE: &str = "stone";

/// One character who has died. Every field defaults, so an entry another version wrote, with
/// fields this one lacks or misses, reads (module docs). Never rename a field or change what it
/// means; add new ones with a default.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fallen {
    /// The character's name.
    pub name: String,
    /// The race, by id (`dwarf`).
    pub race: String,
    /// The gender, by id (`female`).
    pub gender: String,
    /// The class, by id (`warrior`).
    pub class: String,
    /// The prestige class they were called into, by id; empty for none.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub prestige: String,
    /// The experience level reached.
    pub level: u32,
    /// The experience total at the moment of death (#1252); 0 in entries written before it.
    pub xp: u32,
    /// What killed them, the death line's id: `stone` (turned to stone), a status's id
    /// (`bleeding`), `starved`, `fell`, `choked`, `altar`, `own_spell`, a trap's id, a scroll's
    /// object id, a creature's id (`kobold`), or `unseen`.
    pub cause: String,
    /// The creature that killed them, in words (`kobold`); empty for none.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub killer: String,
    /// Where: the level's name, then the campaign's (`Fyrdelf 3, The Ascmearc`).
    pub place: String,
    /// The turn they died in.
    pub turn: u64,
    /// Their age in years.
    pub age: u32,
    /// When, in wall-clock seconds since 1970 (UTC).
    pub when: u64,
    /// The game version that wrote the entry, for curiosity only.
    pub version: String,
}

impl Fallen {
    /// The entry for the run in `world`, which has ended in death, at wall-clock `when`; `None`
    /// for a run still going or one without a character.
    pub fn of_death(world: &World, when: u64) -> Option<Self> {
        let death = world.death()?;
        let character = world.character()?;
        let content = world.content();
        let (cause, killer) = match (death.of, death.cause, death.by) {
            (Some(status), _, _) => {
                let def = content.status(status);
                let id = if def.statue { STONE } else { def.id.as_str() };
                (id.to_string(), String::new())
            }
            (None, Some(cause), _) => (cause_id(world, cause), String::new()),
            (None, None, Some(creature)) => {
                let def = content.creature(creature);
                (def.id.clone(), def.name.clone())
            }
            (None, None, None) => ("unseen".to_string(), String::new()),
        };
        let progress = world.perception().own_experience();
        let campaign = (content.campaign(world.campaign())).map(|c| c.name.clone());
        let place = match campaign {
            Some(campaign) => format!("{}, {campaign}", death.place),
            None => death.place.clone(),
        };
        Some(Self {
            name: character.name.clone(),
            race: character.spec.race.clone(),
            gender: character.spec.gender.id().to_string(),
            class: character.spec.class.clone(),
            prestige: character.prestige.clone().unwrap_or_default(),
            level: u32::from(progress.level),
            xp: progress.xp,
            cause,
            killer,
            place,
            turn: death.turn,
            age: character.age_on(death.time),
            when,
            version: warlock_core::GAME_VERSION.to_string(),
        })
    }

    /// Whether they were turned to stone.
    pub fn petrified(&self) -> bool {
        self.cause == STONE
    }

    /// The entry as the file's text: one `[[fallen]]` table.
    pub fn to_text(&self) -> String {
        #[derive(Serialize)]
        struct One<'a> {
            fallen: [&'a Fallen; 1],
        }
        // Plain strings and numbers: serializing cannot fail.
        toml::to_string(&One { fallen: [self] }).unwrap_or_default()
    }
}

/// A non-creature, non-status death's id (the `cause` field).
fn cause_id(world: &World, cause: warlock_core::world::Cause) -> String {
    use warlock_core::world::Cause;
    let content = world.content();
    match cause {
        Cause::Starvation => "starved".into(),
        Cause::Fall => "fell".into(),
        Cause::Choked(_) => "choked".into(),
        Cause::Trap(t) => content.trap(t).id.clone(),
        Cause::Scroll(o) | Cause::Flask(o) => content.object(o).id.clone(),
        Cause::Spell(_) => "own_spell".into(),
        Cause::Miscast(_) => "miscast".into(),
        Cause::Altar => "altar".into(),
    }
}

/// Reads the file's text: every entry it can, in the order written (the oldest first). Each
/// `[[fallen]]` table is read alone, so an entry that cannot be read, even one an interrupted
/// append left half written (#1573), is skipped, not the file; text with no entry reads as none.
pub fn parse(text: &str) -> Vec<Fallen> {
    let mut chunks: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("[[fallen]]") {
            chunks.push(String::new());
        }
        // Text before the first entry (none is written) is no entry.
        if let Some(chunk) = chunks.last_mut() {
            chunk.push_str(line);
            chunk.push('\n');
        }
    }
    (chunks.iter())
        .filter_map(|chunk| chunk.parse::<toml::Table>().ok())
        .filter_map(|mut table| match table.remove("fallen") {
            Some(toml::Value::Array(entries)) => Some(entries),
            _ => None,
        })
        .flatten()
        .filter_map(|entry| entry.try_into::<Fallen>().ok())
        .collect()
}

/// `all` (as [`parse`] reads them, the oldest first) as the Graveyard screen lists them: the
/// most experience at death first, then (entries written before `xp` was, which read 0) the
/// highest level, then the most recent; ties keep the most recently written first.
pub fn by_experience(mut all: Vec<Fallen>) -> Vec<Fallen> {
    all.reverse();
    all.sort_by_key(|f| std::cmp::Reverse((f.xp, f.level, f.when)));
    all
}

/// Where the graveyard lives; `None` keeps none (captures and replays, no config folder).
#[derive(Resource, Debug, Clone, Default)]
pub struct GraveyardStore {
    backend: Option<Backend>,
}

impl GraveyardStore {
    /// The store for this run's command line: beside the settings (`settings::SettingsStore`).
    /// Captures and replays are dev tools, which never write the player's graveyard and read it
    /// only from `--config-dir`.
    pub fn from_args(args: &Args) -> Self {
        let dev_tool = args.capture.is_some() || args.replay.is_some();
        let backend = match &args.config_dir {
            Some(dir) => Some(Backend::Dir(dir.clone())),
            None if dev_tool => None,
            None => storage::default_backend(Folder::Config, "no graveyard will be kept"),
        };
        Self { backend }
    }

    #[cfg(test)]
    pub fn at(dir: std::path::PathBuf) -> Self {
        Self {
            backend: Some(Backend::Dir(dir)),
        }
    }

    /// Every entry, the oldest first; none where there is no graveyard or it cannot be read.
    pub fn read(&self) -> Vec<Fallen> {
        let Some(backend) = &self.backend else {
            return Vec::new();
        };
        match backend.read(FILE_NAME) {
            Ok(bytes) => parse(&String::from_utf8_lossy(&bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => {
                warn!("cannot read {}: {e}", backend.describe(FILE_NAME));
                Vec::new()
            }
        }
    }

    /// The dead, the most recent first, as the core names statues (the petrified) and
    /// gravestones (all of them) from them (`World::set_graveyard`), each with how they died in
    /// the Graveyard screen's words.
    pub fn dead(&self, content: &Content) -> Vec<warlock_core::world::Fallen> {
        let mut all = self.read();
        // The file is in the order written; an entry's own time breaks no tie it makes.
        all.reverse();
        all.sort_by_key(|f| std::cmp::Reverse(f.when));
        (all.into_iter())
            .map(|f| warlock_core::world::Fallen {
                petrified: f.petrified(),
                death: crate::ui::graveyard_screen::death_words(&f, content),
                name: f.name,
                race: f.race,
            })
            .collect()
    }

    /// Appends `fallen` to the file (module docs). `Ok(false)` where there is no graveyard.
    pub fn remember(&self, fallen: &Fallen) -> Result<bool, String> {
        let Some(backend) = &self.backend else {
            return Ok(false);
        };
        // Each entry starts on a line of its own, after a blank one.
        let text = format!("\n{}", fallen.to_text());
        backend
            .append(FILE_NAME, text.as_bytes())
            .map(|()| true)
            .map_err(|e| format!("cannot write {}: {e}", backend.describe(FILE_NAME)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::Action;

    /// An entry as version 0.11 wrote it (#1137), with a field from a later version and one of
    /// its own missing: it must read under every version after.
    const FIXTURE_0_11: &str = r#"
[[fallen]]
name = "Brakka"
race = "dwarf"
gender = "female"
class = "warrior"
level = 7
cause = "stone"
place = "Fyrdelf 3, The Ascmearc"
turn = 4120
age = 31
when = 1793318400
version = "0.11.0"

[[fallen]]
name = "Tamsin"
race = "elf"
gender = "female"
class = "bard"
prestige = "hunter"
level = 31
cause = "kobold"
killer = "kobold"
place = "the Hub, The Showcase"
turn = 99
age = 140
when = 1793404800
version = "0.11.0"
epitaph = "a field a later version added"

[[fallen]]
name = "Wynn"
cause = "starved"

[[fallen]]
name = 12
"#;

    #[test]
    fn a_fixture_written_by_0_11_reads() {
        let all = parse(FIXTURE_0_11);
        // The broken fourth entry is skipped, not the file.
        let names: Vec<&str> = all.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["Brakka", "Tamsin", "Wynn"]);
        assert_eq!(all[0].level, 7);
        assert!(all[0].petrified());
        assert_eq!(all[1].prestige, "hunter");
        assert_eq!(all[1].killer, "kobold");
        // Missing fields default.
        assert_eq!(
            (all[2].level, all[2].when, all[2].race.as_str()),
            (0, 0, "")
        );
        assert!(parse("this is [not toml").is_empty());
    }

    /// An append cut short (a crash, a full disk) leaves half a table; the entries before it
    /// and those appended after it still read (#1573).
    #[test]
    fn a_torn_append_skips_only_its_own_entry() {
        let torn = "\n[[fallen]]\nname = \"Brakka\"\nlevel = 7\n\n[[fallen]]\nname = \"Ta";
        let later = Fallen {
            name: "Hild".into(),
            level: 3,
            ..Fallen::default()
        };
        let text = format!("{torn}\n{}", later.to_text());
        let names: Vec<String> = parse(&text).into_iter().map(|f| f.name).collect();
        assert_eq!(names, ["Brakka", "Hild"]);
    }

    /// An entry as version 0.14 writes it, with its experience (#1252), after the 0.11 ones.
    const FIXTURE_0_14: &str = r#"
[[fallen]]
name = "Hild"
race = "human"
gender = "female"
class = "priest"
level = 5
xp = 1450
cause = "bleeding"
place = "Cēaptūn, The Ascmearc"
turn = 2048
age = 22
when = 1794000000
version = "0.14.0"
"#;

    #[test]
    fn the_experience_field_reads_and_defaults() {
        let all = parse(&format!("{FIXTURE_0_11}{FIXTURE_0_14}"));
        let xp: Vec<(&str, u32)> = all.iter().map(|f| (f.name.as_str(), f.xp)).collect();
        // The broken entry between them is skipped; entries from before the field read 0.
        assert_eq!(
            xp,
            [("Brakka", 0), ("Tamsin", 0), ("Wynn", 0), ("Hild", 1450)]
        );
    }

    #[test]
    fn the_most_experienced_come_first_then_the_highest_level_then_the_most_recent() {
        let all = parse(&format!("{FIXTURE_0_11}{FIXTURE_0_14}"));
        let names = |all: Vec<Fallen>| -> Vec<String> {
            by_experience(all).into_iter().map(|f| f.name).collect()
        };
        // Hild has experience; the rest read 0 and go by level: Tamsin 31, Brakka 7, Wynn 0.
        assert_eq!(names(all.clone()), ["Hild", "Tamsin", "Brakka", "Wynn"]);
        // Equal level: the most recent first; equal in all, the last written first.
        let at = |name: &str, when| Fallen {
            name: name.into(),
            level: 3,
            when,
            ..Fallen::default()
        };
        let all = vec![at("Old", 10), at("New", 20), at("Twin", 10)];
        assert_eq!(names(all), ["New", "Twin", "Old"]);
    }

    #[test]
    fn deaths_are_appended_and_the_dead_come_most_recent_first_the_petrified_marked() {
        let dir = std::env::temp_dir().join(format!("warlock-graveyard-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = GraveyardStore::at(dir.clone());
        assert_eq!(store.read().len(), 0);
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(dir.join(FILE_NAME), FIXTURE_0_11).expect("fixture");
        let stone = |name: &str, when| Fallen {
            name: name.into(),
            race: "human".into(),
            cause: STONE.into(),
            when,
            ..Fallen::default()
        };
        assert_eq!(store.remember(&stone("Osric", 1_800_000_000)), Ok(true));
        assert_eq!(store.remember(&stone("Eadgyth", 1_700_000_000)), Ok(true));
        // Append-only: what was there is there still, byte for byte.
        let text = std::fs::read_to_string(dir.join(FILE_NAME)).expect("read");
        assert!(text.starts_with(FIXTURE_0_11), "{text}");
        assert_eq!(store.read().len(), 5);
        // All the dead go to the run, the petrified marked, each with how they died in words.
        let content = warlock_assets::content().expect("content");
        let dead = store.dead(&content);
        assert_eq!(dead.len(), 5);
        let petrified: Vec<&str> = (dead.iter())
            .filter(|f| f.petrified)
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(petrified, ["Osric", "Brakka", "Eadgyth"]);
        assert_eq!(dead[0].death, "Turned to stone");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_death_is_written_with_its_cause() {
        let content = Arc::new(warlock_assets::content().expect("content"));
        let spec = "dwarf,female,warrior".parse().expect("spec");
        let mut w = World::start_run(content, "showcase", None, Some(&spec), Some("Brakka"), true)
            .expect("world");
        assert!(Fallen::of_death(&w, 0).is_none(), "not dead yet");
        w.apply(&Action::GrantXp(50));
        for _ in 0..4 {
            w.apply(&Action::Afflict("stiffening".into()));
        }
        let fallen = Fallen::of_death(&w, 1_793_318_400).expect("dead");
        assert_eq!(fallen.name, "Brakka");
        assert_eq!(fallen.xp, 50);
        assert_eq!(
            (fallen.race.as_str(), fallen.class.as_str()),
            ("dwarf", "warrior")
        );
        assert_eq!(fallen.cause, STONE, "{fallen:?}");
        assert_eq!(parse(&fallen.to_text()), [fallen]);
    }
}
