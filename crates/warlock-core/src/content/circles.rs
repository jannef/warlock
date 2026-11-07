//! Spell circles' statuses (`assets/data/circles.toml`, spec §4.8 "Spell circles"): the school of
//! every buff's and hex's status, and each school's colour.
//!
//! The renderer draws a circle about every actor the player sees with a circled status
//! (`crates/warlock/src/ui/circles.rs`). Without the file there are no circles.
//!
//! **One rule** (spec §8 "Status effects", docs/design/buff-circles.md §8): a circled status sheds
//! its school's light, `circle_<school>` in `data/lights.toml`, through the status light
//! (`StatusDef::light`, filled by [`shed_lights`] for every circled status without a light of its
//! own: `lit` keeps the Light spell's). So the school of a row decides a light, and the rows'
//! statuses and schools are in the content fingerprint.
//!
//! Load-time checks: every row's status exists and has `seen` (a circle shows only what look mode
//! may say); its school is one of the seven and has a colour and a `circle_<school>` light; the
//! status is a buff or hex spell's (which says whether its circle is a buff's or a hex's), and
//! every buff or hex spell's status has a row but a sleep's (`sleeps`: never circled).

use super::{Content, ContentError, LightId, School, SpellPurpose, StatusId};
use crate::color::Rgb;
use serde::Deserialize;
use std::collections::BTreeMap;

/// One circled status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CircledStatus {
    pub status: StatusId,
    /// The school whose colour its circle is drawn in.
    pub school: School,
    /// A hex's (a broken circle, outside a buff's), else a buff's.
    pub hex: bool,
    /// The light its school sheds (`circle_<school>`).
    pub light: LightId,
}

/// Every circled status and the schools' colours.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CircleStatuses {
    /// In file order.
    rows: Vec<CircledStatus>,
    by_status: BTreeMap<StatusId, usize>,
    /// By [`School::ALL`] position; set for every school once there are rows.
    colors: [Rgb; 7],
}

impl CircleStatuses {
    /// The row of `status`, if it is circled.
    pub fn of(&self, status: StatusId) -> Option<&CircledStatus> {
        self.by_status.get(&status).map(|&i| &self.rows[i])
    }

    /// The position of `status`'s row in [`CircleStatuses::rows`], if it is circled: a number
    /// of its own, which tells one effect from another on a circle.
    pub fn index(&self, status: StatusId) -> Option<usize> {
        self.by_status.get(&status).copied()
    }

    /// Every circled status, in file order.
    pub fn rows(&self) -> &[CircledStatus] {
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
    status: Vec<StatusRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SchoolRow {
    id: School,
    color: Rgb,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusRow {
    status: String,
    school: School,
}

/// Parses and checks `data/circles.toml` against the loaded statuses and spells.
pub(super) fn parse(
    path: &str,
    text: &str,
    content: &Content,
) -> Result<CircleStatuses, ContentError> {
    let err = |m: String| ContentError::new(path, m);
    let file: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let mut colors: [Option<Rgb>; 7] = [None; 7];
    let mut lights: [Option<LightId>; 7] = [None; 7];
    for row in &file.school {
        let slot = &mut colors[school_index(row.id)];
        if slot.is_some() {
            return Err(err(format!("school `{}` is listed twice", row.id.id())));
        }
        *slot = Some(row.color);
        let light = format!("circle_{}", row.id.id());
        lights[school_index(row.id)] = Some(content.light_id(&light).ok_or_else(|| {
            err(format!(
                "school `{}` has no light `{light}` (data/lights.toml)",
                row.id.id()
            ))
        })?);
    }
    // Which statuses buff and hex spells lay, and of which kind.
    let mut spell_kind: BTreeMap<StatusId, (bool, &str)> = BTreeMap::new();
    for spell in content.spells() {
        let hex = match spell.kind {
            SpellPurpose::Buff => false,
            SpellPurpose::Hex => true,
            _ => continue,
        };
        // Asleep is never circled (docs/design/buff-circles.md §9: a den's sleepers would
        // glow), Slumber's either.
        if let Some((status, _)) = spell.effect.status
            && !content.status(status).sleeps
        {
            spell_kind.entry(status).or_insert((hex, &spell.id));
        }
    }
    let mut circles = CircleStatuses::default();
    for row in &file.status {
        let name = &row.status;
        let status =
            (content.status_id(name)).ok_or_else(|| err(format!("unknown status `{name}`")))?;
        if content.status(status).seen.is_none() {
            return Err(err(format!(
                "status `{name}` is circled but has no `seen`: a circle shows only what others \
                 see (data/status.toml)"
            )));
        }
        let Some(light) = lights[school_index(row.school)] else {
            return Err(err(format!(
                "status `{name}`: school `{}` has no colour (`[[school]]`)",
                row.school.id()
            )));
        };
        let &(hex, _) = spell_kind.get(&status).ok_or_else(|| {
            err(format!(
                "status `{name}` is circled but no buff or hex spell lays it: a circle is a \
                 spell's mark"
            ))
        })?;
        if circles
            .by_status
            .insert(status, circles.rows.len())
            .is_some()
        {
            return Err(err(format!("status `{name}` is listed twice")));
        }
        circles.rows.push(CircledStatus {
            status,
            school: row.school,
            hex,
            light,
        });
    }
    if let Some((status, (_, spell))) =
        (spell_kind.iter()).find(|(s, _)| !circles.by_status.contains_key(s))
    {
        let name = &content.status(*status).id;
        return Err(err(format!(
            "spell `{spell}` lays status `{name}`, which has no row (`[[status]]`)"
        )));
    }
    for (i, color) in colors.iter().enumerate() {
        circles.colors[i] = color.unwrap_or(Rgb::WHITE);
    }
    Ok(circles)
}

/// The one rule of the circles: every circled status without a light of its own sheds its
/// school's (`lit` keeps the Light spell's, brighter in every way). Constant whatever the
/// severity, so a source's shadow mask stays cached while its bearer stands.
pub(super) fn shed_lights(content: &mut Content) {
    for row in content.circles.rows.clone() {
        let def = &mut content.statuses[usize::from(row.status.0)];
        def.light.get_or_insert(row.light);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;

    const PATH: &str = "data/circles.toml";

    fn load_with(text: &str) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        files.insert(PATH, text);
        Content::load(&files)
    }

    fn shipped() -> String {
        ContentFiles::builtin().get(PATH).unwrap().to_string()
    }

    #[test]
    fn the_shipped_rows_cover_every_buff_and_hex() {
        let content = Content::builtin().unwrap();
        let circles = content.circles();
        let row = |s: &str| {
            circles
                .of(content.status_id(s).unwrap())
                .map(|r| (r.school, r.hex))
        };
        assert_eq!(row("lit"), Some((School::Evocation, false)));
        assert_eq!(row("blessed"), Some((School::Abjuration, false)));
        assert_eq!(row("webbed"), Some((School::Transmutation, true)));
        assert_eq!(row("ill_wished"), Some((School::Enchantment, true)));
        assert_eq!(row("fortified"), None, "a potion's boost has no circle");
        assert_eq!(circles.rows().len(), 22);
        assert_eq!(circles.color(School::Abjuration).to_string(), "#80b4e0");
    }

    #[test]
    fn bad_files_are_refused() {
        let shipped = shipped();
        let blessed = "[[status]]\nstatus = \"blessed\"\nschool = \"abjuration\"\n";
        assert!(shipped.contains(blessed));
        let cases = [
            (shipped.replace(blessed, ""), "lays status `blessed`, which has no row"),
            (
                shipped.replace("status = \"blessed\"", "status = \"blest\""),
                "unknown status `blest`",
            ),
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
            (format!("{shipped}\n{blessed}"), "`blessed` is listed twice"),
            (
                format!("{shipped}\n[[status]]\nstatus = \"radiant\"\nschool = \"restoration\"\n"),
                "no buff or hex spell lays it",
            ),
        ];
        for (text, why) in cases {
            let err = load_with(&text).unwrap_err();
            assert!(err.message.contains(why), "{why}: {}", err.message);
        }
    }

    #[test]
    fn a_circled_status_sheds_its_schools_light() {
        let content = Content::builtin().unwrap();
        let light = |s: &str| {
            let id = content.status(content.status_id(s).unwrap()).light;
            id.map(|l| content.light(l).id.as_str())
        };
        assert_eq!(light("blessed"), Some("circle_abjuration"));
        assert_eq!(light("hasted"), Some("circle_transmutation"));
        assert_eq!(light("drowsy"), Some("circle_enchantment"));
        assert_eq!(light("weakened"), Some("circle_necromancy"));
        assert_eq!(light("lit"), Some("spell_light"), "Light keeps its own");
        assert_eq!(light("fortified"), None, "no circle, no light");
        assert_eq!(
            light("fire_cloaked"),
            Some("ember"),
            "flames keep their own"
        );
        for row in content.circles().rows() {
            let def = content.light(content.status(row.status).light.unwrap());
            // A status's own fire flickers (`ember`); a circle's faint glow never does.
            if def.id.starts_with("circle_") {
                assert!(def.stability.is_stable(), "{}: steady", def.id);
            }
        }
        // A school without its light is refused.
        let mut files = ContentFiles::builtin();
        let lights = files.get("data/lights.toml").unwrap().to_string();
        files.insert(
            "data/lights.toml",
            &lights.replace("id = \"circle_necromancy\"", "id = \"circle_bones\""),
        );
        let err = Content::load(&files).unwrap_err();
        assert!(
            err.message
                .contains("school `necromancy` has no light `circle_necromancy`"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_rows_school_is_in_the_fingerprint() {
        // A school decides a light, which decides what creatures see.
        let a = Content::builtin().unwrap().fingerprint_for("showcase");
        let blessed = "status = \"blessed\"\nschool = \"abjuration\"";
        let text = shipped().replace(blessed, &blessed.replace("abjuration", "restoration"));
        assert_ne!(a, load_with(&text).unwrap().fingerprint_for("showcase"));
    }

    #[test]
    fn a_circled_status_must_be_seen() {
        let mut files = ContentFiles::builtin();
        let status = files.get("data/status.toml").unwrap().to_string();
        let seen = "seen = \"bark-skinned\"\n";
        assert!(status.contains(seen));
        files.insert("data/status.toml", &status.replace(seen, ""));
        let err = Content::load(&files).unwrap_err();
        assert!(
            err.message
                .contains("`barkskin` is circled but has no `seen`"),
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
