//! Spell circles' runes (`assets/data/runes.toml`, spec §4.8 "Spell circles"): the rune and the
//! school of every buff's and hex's status, and each school's colour.
//!
//! **Presentation only**, like spell effects: the core loads and checks the file so all content
//! lives in one place, but no rule reads it, and it is not in the content fingerprint. The
//! renderer draws a circle about every actor the player sees with a runed status
//! (`crates/warlock/src/ui/circles.rs`). Without the file there are no runes and no circles.
//!
//! Load-time checks: every row's status exists and has `seen` (a circle shows only what look mode
//! may say); its rune is exactly one character, used by no other row; its school is one of the
//! seven and has a colour; the status is a buff or hex spell's (which says whether its circle is
//! a buff's or a hex's), and every buff or hex spell's status has a row.

use super::{Content, ContentError, School, SpellPurpose, StatusId};
use crate::color::Rgb;
use serde::Deserialize;
use std::collections::BTreeMap;

/// One runed status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuneDef {
    pub status: StatusId,
    /// The character drawn at the circle's corners.
    pub rune: char,
    /// The school whose colour it is drawn in.
    pub school: School,
    /// A hex's (a broken circle, outside a buff's), else a buff's.
    pub hex: bool,
}

/// Every runed status and the schools' colours.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Runes {
    /// In file order.
    rows: Vec<RuneDef>,
    by_status: BTreeMap<StatusId, usize>,
    /// By [`School::ALL`] position; set for every school once there are rows.
    colors: [Rgb; 7],
}

impl Runes {
    /// The rune of `status`, if it has one.
    pub fn of(&self, status: StatusId) -> Option<&RuneDef> {
        self.by_status.get(&status).map(|&i| &self.rows[i])
    }

    /// The position of `status`'s rune in [`Runes::rows`], if it has one.
    pub fn index(&self, status: StatusId) -> Option<usize> {
        self.by_status.get(&status).copied()
    }

    /// Every runed status, in file order.
    pub fn rows(&self) -> &[RuneDef] {
        &self.rows
    }

    /// The colour of `school`.
    pub fn color(&self, school: School) -> Rgb {
        self.colors[school_index(school)]
    }
}

/// `school`'s position in [`School::ALL`].
pub fn school_index(school: School) -> usize {
    (School::ALL.iter().position(|&s| s == school)).expect("School::ALL has every school")
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    school: Vec<SchoolRow>,
    #[serde(default)]
    rune: Vec<RuneRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SchoolRow {
    id: School,
    color: Rgb,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuneRow {
    status: String,
    rune: String,
    school: School,
}

/// Parses and checks `data/runes.toml` against the loaded statuses and spells.
pub(super) fn parse(path: &str, text: &str, content: &Content) -> Result<Runes, ContentError> {
    let err = |m: String| ContentError::new(path, m);
    let file: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let mut colors: [Option<Rgb>; 7] = [None; 7];
    for row in &file.school {
        let slot = &mut colors[school_index(row.id)];
        if slot.is_some() {
            return Err(err(format!("school `{}` is listed twice", row.id.id())));
        }
        *slot = Some(row.color);
    }
    // Which statuses buff and hex spells lay, and of which kind.
    let mut spell_kind: BTreeMap<StatusId, (bool, &str)> = BTreeMap::new();
    for spell in content.spells() {
        let hex = match spell.kind {
            SpellPurpose::Buff => false,
            SpellPurpose::Hex => true,
            _ => continue,
        };
        if let Some((status, _)) = spell.effect.status {
            spell_kind.entry(status).or_insert((hex, &spell.id));
        }
    }
    let mut runes = Runes::default();
    let mut used: BTreeMap<char, &str> = BTreeMap::new();
    for row in &file.rune {
        let name = &row.status;
        let status =
            (content.status_id(name)).ok_or_else(|| err(format!("unknown status `{name}`")))?;
        if content.status(status).seen.is_none() {
            return Err(err(format!(
                "status `{name}` has a rune but no `seen`: a circle shows only what others see \
                 (data/status.toml)"
            )));
        }
        let mut chars = row.rune.chars();
        let (Some(rune), None) = (chars.next(), chars.next()) else {
            return Err(err(format!(
                "status `{name}`: rune `{}` is not exactly one character",
                row.rune
            )));
        };
        if let Some(first) = used.insert(rune, name) {
            return Err(err(format!(
                "status `{name}`: rune `{rune}` is already `{first}`'s"
            )));
        }
        if colors[school_index(row.school)].is_none() {
            return Err(err(format!(
                "status `{name}`: school `{}` has no colour (`[[school]]`)",
                row.school.id()
            )));
        }
        let &(hex, _) = spell_kind.get(&status).ok_or_else(|| {
            err(format!(
                "status `{name}` has a rune but no buff or hex spell lays it: a circle is a \
                 spell's mark"
            ))
        })?;
        if runes.by_status.insert(status, runes.rows.len()).is_some() {
            return Err(err(format!("status `{name}` has two runes")));
        }
        runes.rows.push(RuneDef {
            status,
            rune,
            school: row.school,
            hex,
        });
    }
    if let Some((status, (_, spell))) =
        (spell_kind.iter()).find(|(s, _)| !runes.by_status.contains_key(s))
    {
        let name = &content.status(*status).id;
        return Err(err(format!(
            "spell `{spell}` lays status `{name}`, which has no rune (`[[rune]]`)"
        )));
    }
    for (i, color) in colors.iter().enumerate() {
        runes.colors[i] = color.unwrap_or(Rgb::WHITE);
    }
    Ok(runes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;

    const PATH: &str = "data/runes.toml";

    fn load_with(text: &str) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        files.insert(PATH, text);
        Content::load(&files)
    }

    fn shipped() -> String {
        ContentFiles::builtin().get(PATH).unwrap().to_string()
    }

    #[test]
    fn the_shipped_runes_cover_every_buff_and_hex() {
        let content = Content::builtin().unwrap();
        let runes = content.runes();
        let rune = |s: &str| {
            runes
                .of(content.status_id(s).unwrap())
                .map(|r| (r.rune, r.hex))
        };
        assert_eq!(rune("lit"), Some(('ヒ', false)));
        assert_eq!(rune("blessed"), Some(('シ', false)));
        assert_eq!(rune("webbed"), Some(('井', true)));
        assert_eq!(rune("ill_wished"), Some(('目', true)));
        assert_eq!(rune("fortified"), None, "a potion's boost has no rune");
        assert_eq!(runes.rows().len(), 9);
        let blessed = runes.of(content.status_id("blessed").unwrap()).unwrap();
        assert_eq!(blessed.school, School::Abjuration);
        assert_eq!(runes.color(School::Abjuration).to_string(), "#80b4e0");
    }

    #[test]
    fn bad_files_are_refused() {
        let shipped = shipped();
        let blessed = "[[rune]]\nstatus = \"blessed\"\nrune = \"シ\"\nschool = \"abjuration\"\n";
        assert!(shipped.contains(blessed));
        let cases = [
            (shipped.replace(blessed, ""), "lays status `blessed`, which has no rune"),
            (
                shipped.replace("status = \"blessed\"", "status = \"blest\""),
                "unknown status `blest`",
            ),
            (
                shipped.replace("rune = \"シ\"", "rune = \"シシ\""),
                "not exactly one character",
            ),
            (shipped.replace("rune = \"シ\"", "rune = \"\""), "not exactly one character"),
            (shipped.replace("rune = \"シ\"", "rune = \"ヒ\""), "already `lit`'s"),
            (
                shipped.replace("school = \"abjuration\"", "school = \"alchemy\""),
                "unknown variant `alchemy`",
            ),
            (
                shipped.replace("id = \"abjuration\"\ncolor = \"#80b4e0\"\n", "id = \"abjuration\"\ncolor = \"#80b4e0\"\n\n[[school]]\nid = \"abjuration\"\ncolor = \"#000000\"\n"),
                "listed twice",
            ),
            (
                shipped.replace("[[school]]\nid = \"abjuration\"\ncolor = \"#80b4e0\"\n", ""),
                "school `abjuration` has no colour",
            ),
            (
                format!("{shipped}\n[[rune]]\nstatus = \"blessed\"\nrune = \"祝\"\nschool = \"abjuration\"\n"),
                "two runes",
            ),
            (
                format!("{shipped}\n[[rune]]\nstatus = \"radiant\"\nrune = \"光\"\nschool = \"restoration\"\n"),
                "no buff or hex spell lays it",
            ),
        ];
        for (text, why) in cases {
            let err = load_with(&text).unwrap_err();
            assert!(err.message.contains(why), "{why}: {}", err.message);
        }
    }

    #[test]
    fn a_runed_status_must_be_seen() {
        let mut files = ContentFiles::builtin();
        let status = files.get("data/status.toml").unwrap().to_string();
        let seen = "seen = \"bark-skinned\"\n";
        assert!(status.contains(seen));
        files.insert("data/status.toml", &status.replace(seen, ""));
        let err = Content::load(&files).unwrap_err();
        assert!(
            err.message.contains("`barkskin` has a rune but no `seen`"),
            "{}",
            err.message
        );
    }

    #[test]
    fn every_buff_status_is_seen() {
        // docs/dev/status-effects.md: a buff status carries `seen`, whatever its source. A buff
        // lends something: a stat, a combat bonus, a quicker pace, eyes, a resistance, or light.
        let content = Content::builtin().unwrap();
        let not_buffs = ["disguised", "lurking"];
        for (_, def) in content.status_ids() {
            let c = &def.combat;
            let lends = [c.to_hit, c.damage, c.dodge, c.parry, c.shield]
                .iter()
                .any(|&b| b > 0)
                || def.cost < 100
                || def.vision.is_some()
                || def.light.is_some()
                || def.resist.iter().any(|&(_, r)| r > 0)
                || (def.penalty.as_ref()).is_some_and(|p| p.by.iter().any(|(_, b)| b > 0));
            if lends && !not_buffs.contains(&def.id.as_str()) {
                assert!(def.seen.is_some(), "buff status `{}` has no `seen`", def.id);
            }
        }
    }
}
