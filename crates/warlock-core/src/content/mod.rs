//! Game content: definitions, levels, and scenarios loaded from TOML (spec §7).
//!
//! Content is read from a [`ContentFiles`] set: the built-in set compiled into the binary (so every
//! build, including the web build, is self-contained) or a directory on disk for development.

mod anim;
mod creature;
mod level;
mod scenario;
mod terrain;

pub use anim::GlyphAnim;
pub use creature::{CreatureDef, CreatureId};
pub use level::{CreatureSpawn, LevelTemplate, StairsLink, parse_route};
pub use scenario::ScenarioDef;
pub use terrain::{StairsDir, TerrainDef, TerrainId};

use level::{LegendEntry, legend_char};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

/// Content files that failed to load, with the offending file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentError {
    pub file: String,
    pub message: String,
}

impl ContentError {
    fn new(file: &str, message: impl Into<String>) -> Self {
        Self {
            file: file.to_string(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.file, self.message)
    }
}

impl std::error::Error for ContentError {}

mod builtin {
    include!(concat!(env!("OUT_DIR"), "/builtin_assets.rs"));
}

/// Raw content files keyed by path relative to the assets root, e.g. `levels/hub.toml`.
#[derive(Clone, Debug, Default)]
pub struct ContentFiles {
    files: BTreeMap<String, String>,
}

impl ContentFiles {
    /// The content compiled into the binary from `assets/`.
    pub fn builtin() -> Self {
        let mut files = Self::default();
        for (path, text) in builtin::FILES {
            files.insert(path, text);
        }
        files
    }

    /// Reads `data/`, `levels/`, and `scenarios/` TOML files under `root`.
    pub fn from_dir(root: &Path) -> std::io::Result<Self> {
        let mut files = Self::default();
        for dir in ["data", "levels", "scenarios"] {
            let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
                continue;
            };
            for entry in entries {
                let path = entry?.path();
                if path.extension().is_some_and(|e| e == "toml") {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    files.insert(&format!("{dir}/{name}"), &std::fs::read_to_string(&path)?);
                }
            }
        }
        Ok(files)
    }

    pub fn insert(&mut self, path: &str, text: &str) {
        self.files.insert(path.to_string(), text.to_string());
    }

    fn get(&self, path: &str) -> Result<&str, ContentError> {
        self.files
            .get(path)
            .map(String::as_str)
            .ok_or_else(|| ContentError::new(path, "file is missing"))
    }

    /// Files directly under `dir/` with their stem (`levels/hub.toml` → `hub`).
    fn in_dir<'a>(&'a self, dir: &'a str) -> impl Iterator<Item = (&'a str, &'a str, &'a str)> {
        self.files.iter().filter_map(move |(path, text)| {
            let rest = path.strip_prefix(dir)?.strip_prefix('/')?;
            let stem = rest.strip_suffix(".toml")?;
            (!stem.contains('/')).then_some((path.as_str(), stem, text.as_str()))
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TerrainFile {
    terrain: Vec<terrain::TerrainSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatureFile {
    creature: Vec<CreatureDef>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegendFile {
    legend: BTreeMap<String, LegendEntry>,
}

/// All loaded and cross-validated content.
#[derive(Debug)]
pub struct Content {
    terrain: Vec<TerrainDef>,
    terrain_ids: BTreeMap<String, TerrainId>,
    creatures: Vec<CreatureDef>,
    creature_ids: BTreeMap<String, CreatureId>,
    levels: BTreeMap<String, LevelTemplate>,
    scenarios: BTreeMap<String, ScenarioDef>,
}

impl Content {
    /// Loads the built-in content.
    pub fn builtin() -> Result<Self, ContentError> {
        Self::load(&ContentFiles::builtin())
    }

    pub fn load(files: &ContentFiles) -> Result<Self, ContentError> {
        let mut content = Content {
            terrain: Vec::new(),
            terrain_ids: BTreeMap::new(),
            creatures: Vec::new(),
            creature_ids: BTreeMap::new(),
            levels: BTreeMap::new(),
            scenarios: BTreeMap::new(),
        };
        content.load_terrain(files)?;
        content.load_creatures(files)?;

        let path = "data/legend.toml";
        let legend_file: LegendFile =
            toml::from_str(files.get(path)?).map_err(|e| ContentError::new(path, e.to_string()))?;
        let mut default_legend = BTreeMap::new();
        for (key, entry) in legend_file.legend {
            default_legend.insert(legend_char(path, &key)?, entry);
        }

        for (path, id, text) in files.in_dir("levels") {
            let level = level::parse_level(path, id, text, &content, &default_legend)?;
            content.levels.insert(id.to_string(), level);
        }
        for (path, id, text) in files.in_dir("scenarios") {
            let scenario = ScenarioDef::parse(path, id, text)?;
            content.scenarios.insert(id.to_string(), scenario);
        }
        content.validate()?;
        Ok(content)
    }

    fn load_terrain(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/terrain.toml";
        let err = |m: String| ContentError::new(path, m);
        let parsed: TerrainFile =
            toml::from_str(files.get(path)?).map_err(|e| err(e.to_string()))?;
        for (i, spec) in parsed.terrain.iter().enumerate() {
            let id = TerrainId(u16::try_from(i).map_err(|_| err("too many terrain types".into()))?);
            if self.terrain_ids.insert(spec.id.clone(), id).is_some() {
                return Err(err(format!("duplicate terrain id `{}`", spec.id)));
            }
        }
        let resolve = |name: &Option<String>| -> Result<Option<TerrainId>, ContentError> {
            name.as_ref()
                .map(|n| {
                    self.terrain_ids
                        .get(n)
                        .copied()
                        .ok_or_else(|| err(format!("unknown terrain `{n}`")))
                })
                .transpose()
        };
        let mut defs = Vec::with_capacity(parsed.terrain.len());
        for spec in &parsed.terrain {
            defs.push(TerrainDef {
                id: spec.id.clone(),
                name: spec.name.clone(),
                glyph: spec.glyph,
                fg: spec.fg,
                bg: spec.bg,
                walkable: spec.walkable,
                transparent: spec.transparent,
                opens_to: resolve(&spec.opens_to)?,
                closes_to: resolve(&spec.closes_to)?,
                stairs: spec.stairs,
                anim: spec.anim.clone(),
            });
        }
        self.terrain = defs;
        Ok(())
    }

    fn load_creatures(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/creatures.toml";
        let err = |m: String| ContentError::new(path, m);
        let parsed: CreatureFile =
            toml::from_str(files.get(path)?).map_err(|e| err(e.to_string()))?;
        for (i, def) in parsed.creature.into_iter().enumerate() {
            let id = CreatureId(u16::try_from(i).map_err(|_| err("too many creatures".into()))?);
            if self.creature_ids.insert(def.id.clone(), id).is_some() {
                return Err(err(format!("duplicate creature id `{}`", def.id)));
            }
            self.creatures.push(def);
        }
        if !self.creature_ids.contains_key("player") {
            return Err(err("a creature with id `player` is required".into()));
        }
        Ok(())
    }

    /// Cross-file checks: stair targets and scenario start levels exist.
    fn validate(&self) -> Result<(), ContentError> {
        for level in self.levels.values() {
            for link in &level.stairs {
                if let Some(to) = &link.to
                    && !self.levels.contains_key(to)
                {
                    return Err(ContentError::new(
                        &format!("levels/{}.toml", level.id),
                        format!("stairs at {} lead to unknown level `{to}`", link.pos),
                    ));
                }
            }
        }
        for scenario in self.scenarios.values() {
            let file = format!("scenarios/{}.toml", scenario.id);
            let level = self.levels.get(&scenario.start_level).ok_or_else(|| {
                ContentError::new(
                    &file,
                    format!("unknown start level `{}`", scenario.start_level),
                )
            })?;
            if level.player_start.is_none() {
                return Err(ContentError::new(
                    &file,
                    format!("start level `{}` has no player start", level.id),
                ));
            }
        }
        Ok(())
    }

    pub fn terrain(&self, id: TerrainId) -> &TerrainDef {
        &self.terrain[usize::from(id.0)]
    }

    pub fn terrain_id(&self, name: &str) -> Option<TerrainId> {
        self.terrain_ids.get(name).copied()
    }

    pub fn terrain_defs(&self) -> &[TerrainDef] {
        &self.terrain
    }

    pub fn creature(&self, id: CreatureId) -> &CreatureDef {
        &self.creatures[usize::from(id.0)]
    }

    pub fn creature_id(&self, name: &str) -> Option<CreatureId> {
        self.creature_ids.get(name).copied()
    }

    pub fn creature_defs(&self) -> &[CreatureDef] {
        &self.creatures
    }

    pub fn level(&self, id: &str) -> Option<&LevelTemplate> {
        self.levels.get(id)
    }

    pub fn levels(&self) -> impl Iterator<Item = &LevelTemplate> {
        self.levels.values()
    }

    pub fn scenario(&self, id: &str) -> Option<&ScenarioDef> {
        self.scenarios.get(id)
    }

    pub fn scenarios(&self) -> impl Iterator<Item = &ScenarioDef> {
        self.scenarios.values()
    }

    /// Every glyph content may display (for building the glyph atlas).
    pub fn glyphs(&self) -> BTreeSet<char> {
        let terrain = self.terrain.iter().flat_map(|t| {
            std::iter::once(t.glyph).chain(t.anim.iter().flat_map(|a| a.extra_glyphs()))
        });
        let creatures = self.creatures.iter().flat_map(|c| {
            std::iter::once(c.glyph).chain(c.anim.iter().flat_map(|a| a.extra_glyphs()))
        });
        terrain.chain(creatures).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Pos;

    #[test]
    fn builtin_content_loads() {
        let content = Content::builtin().expect("built-in content must be valid");
        assert!(content.scenario("showcase").is_some());
        assert!(content.level("hub").is_some());
        assert!(content.terrain_id("wall").is_some());
        assert!(content.glyphs().contains(&'#'));
    }

    fn minimal_files(level_map: &str, extra_legend: &str) -> ContentFiles {
        let mut files = ContentFiles::default();
        files.insert(
            "data/terrain.toml",
            r##"
            [[terrain]]
            id = "void"
            name = "rock"
            glyph = " "
            fg = "#000000"
            walkable = false
            transparent = false
            [[terrain]]
            id = "floor"
            name = "floor"
            glyph = "."
            fg = "#808080"
            [[terrain]]
            id = "wall"
            name = "wall"
            glyph = "#"
            fg = "#a0a0a0"
            walkable = false
            transparent = false
            "##,
        );
        files.insert(
            "data/creatures.toml",
            r##"
            [[creature]]
            id = "player"
            name = "you"
            glyph = "@"
            fg = "#ffffff"
            [[creature]]
            id = "rat"
            name = "rat"
            glyph = "r"
            fg = "#806040"
            "##,
        );
        files.insert(
            "data/legend.toml",
            "[legend]\n\" \" = \"void\"\n\".\" = \"floor\"\n\"#\" = \"wall\"\n",
        );
        files.insert(
            "levels/test.toml",
            &format!(
                "name = \"Test\"\nmap = '''\n{level_map}'''\n[legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n{extra_legend}"
            ),
        );
        files.insert(
            "scenarios/test.toml",
            "name = \"T\"\nstart_level = \"test\"\n",
        );
        files
    }

    #[test]
    fn short_rows_are_padded_with_void() {
        let content = Content::load(&minimal_files("####\n#@.\n####\n", "")).unwrap();
        let level = content.level("test").unwrap();
        assert_eq!((level.terrain.width(), level.terrain.height()), (4, 3));
        assert_eq!(
            level.terrain[Pos::new(3, 1)],
            content.terrain_id("void").unwrap()
        );
        assert_eq!(level.player_start, Some(Pos::new(1, 1)));
    }

    #[test]
    fn unknown_legend_character_is_reported_with_position() {
        let err = Content::load(&minimal_files("#@X#\n", "")).unwrap_err();
        assert_eq!(err.file, "levels/test.toml");
        assert!(err.message.contains("`X` at (2, 0)"), "{}", err.message);
    }

    #[test]
    fn patrol_routes_must_loop_and_stay_walkable() {
        let map = "#####\n#@.r#\n#...#\n#####\n";
        let good = "\"r\" = { terrain = \"floor\", creature = \"rat\", route = \"S W S\" }";
        let err = Content::load(&minimal_files(map, good)).unwrap_err();
        assert!(
            err.message.contains("leaves walkable terrain"),
            "{}",
            err.message
        );

        let not_loop = "\"r\" = { terrain = \"floor\", creature = \"rat\", route = \"S W\" }";
        let err = Content::load(&minimal_files(map, not_loop)).unwrap_err();
        assert!(err.message.contains("loop back"), "{}", err.message);

        let ok = "\"r\" = { terrain = \"floor\", creature = \"rat\", route = \"S W N E\" }";
        let content = Content::load(&minimal_files(map, ok)).unwrap();
        assert_eq!(content.level("test").unwrap().creatures[0].route.len(), 4);
    }
}
