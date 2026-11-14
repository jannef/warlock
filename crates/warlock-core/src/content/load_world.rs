//! Loading terrain, lights, sight, and dungeon branches.

use super::{
    Breakable, Content, ContentError, ContentFiles, CreatureId, DungeonDef, Landscape, LightId,
    LightPreset, TerrainDef, TerrainId, VisionProfile, campaign, danger, dungeon, duplicate_id,
    terrain, validate::check_effects,
};
use crate::noise::MAX_LOUDNESS;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

impl Content {
    pub(super) fn load_terrain(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
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
            let light = self.resolve_light(path, spec.light.as_deref())?;
            let height = spec.height.unwrap_or(u8::from(!spec.transparent));
            check_terrain_noise(spec).map_err(|m| err(format!("terrain `{}`: {m}", spec.id)))?;
            check_terrain_ice(spec).map_err(|m| err(format!("terrain `{}`: {m}", spec.id)))?;
            check_kicking(spec).map_err(|m| err(format!("terrain `{}`: {m}", spec.id)))?;
            let breaks = match (spec.hardness, &spec.becomes) {
                (None, None) => None,
                (Some(hardness), Some(_)) => Some(Breakable {
                    hardness,
                    becomes: resolve(&spec.becomes)?.expect("given"),
                }),
                _ => {
                    return Err(err(format!(
                        "terrain `{}`: `hardness` and `becomes` go together",
                        spec.id
                    )));
                }
            };
            check_effects(
                path,
                "terrain",
                &spec.id,
                spec.anim.as_ref(),
                spec.glow,
                spec.haze,
                &spec.particles,
            )?;
            defs.push(TerrainDef {
                id: spec.id.clone(),
                name: spec.name.clone(),
                description: spec.description.clone(),
                announce: spec.announce,
                glyph: spec.glyph,
                variants: spec.variants.chars().collect(),
                fg: spec.fg,
                bg: spec.bg,
                walkable: spec.walkable,
                transparent: spec.transparent,
                move_cost: spec.move_cost,
                travel_minutes: spec.travel_minutes,
                local: resolve(&spec.local)?,
                landscape: spec.landscape,
                opens_to: resolve(&spec.opens_to)?,
                closes_to: resolve(&spec.closes_to)?,
                kicked_in: match (resolve(&spec.breaks_to)?, spec.sturdiness) {
                    (Some(to), Some(sturdiness)) => Some((to, sturdiness)),
                    _ => None,
                },
                chest: spec.chest,
                statue: spec.sturdiness.filter(|_| spec.statue),
                grave: spec.grave,
                stairs: spec.stairs,
                light,
                bounce: spec.bounce,
                height,
                roofed: spec.roofed,
                stains: spec.stains,
                stone: spec.stone,
                works_as: resolve(&spec.works_as)?,
                plants: spec.plants,
                step_noise: spec.step_noise,
                toggle_noise: spec.toggle_noise,
                // Solid terrain blocks sound unless it muffles it; everything else passes it.
                muffle: spec
                    .muffle
                    .or((spec.walkable || spec.transparent).then_some(0)),
                swim: spec.swim,
                freeze: match &spec.freeze {
                    None => None,
                    Some(f) => {
                        let kind = match &f.damage_type {
                            None => None,
                            Some(name) => Some(self.damage_type_id(name).ok_or_else(|| {
                                ContentError::new(
                                    path,
                                    format!(
                                        "terrain `{}`: `freeze`: unknown damage type `{name}`",
                                        spec.id
                                    ),
                                )
                            })?),
                        };
                        Some(f.resolve(kind))
                    }
                },
                ice: spec.ice,
                breaks,
                face: resolve(&spec.face)?,
                anim: spec.anim.clone(),
                glow: spec.glow,
                haze: spec.haze,
                particles: spec.particles.clone(),
            });
        }
        check_chests(&defs).map_err(err)?;
        if let Some(def) = (defs.iter()).find(|d| {
            d.works_as
                .is_some_and(|t| defs[usize::from(t.0)].works_as.is_some())
        }) {
            return Err(err(format!(
                "terrain `{}`: `works_as` names a terrain that works as another",
                def.id
            )));
        }
        let statues: Vec<&str> = (defs.iter().filter(|d| d.statue.is_some()))
            .map(|d| d.id.as_str())
            .collect();
        if let [first, second, ..] = statues[..] {
            return Err(err(format!(
                "terrain `{first}` and `{second}` are both a `statue`"
            )));
        }
        if let Some(def) = defs
            .iter()
            .find(|d| d.local.is_some_and(|l| !defs[usize::from(l.0)].walkable))
        {
            return Err(err(format!(
                "terrain `{}`: its local area ground must be walkable",
                def.id
            )));
        }
        if let Some(def) = defs.iter().find(|d| {
            d.local.is_some()
                && !d
                    .landscape
                    .is_some_and(|l| l.is_land() || l == Landscape::Road)
        }) {
            return Err(err(format!(
                "terrain `{}` has a local area but no land or road `landscape`",
                def.id
            )));
        }
        let mut ice: Vec<(u16, TerrainId)> = (defs.iter().enumerate())
            .filter_map(|(i, d)| d.ice.map(|mm| (mm, TerrainId(i as u16))))
            .collect();
        ice.sort();
        if let Some(pair) = ice.windows(2).find(|w| w[0].0 == w[1].0) {
            return Err(err(format!(
                "terrain `{}` and `{}` both look like ice from {} mm",
                defs[usize::from(pair[0].1.0)].id,
                defs[usize::from(pair[1].1.0)].id,
                pair[0].0
            )));
        }
        if let Some(water) = defs.iter().find(|d| d.freeze.is_some())
            && ice.is_empty()
        {
            return Err(err(format!(
                "terrain `{}` freezes, but no terrain looks like ice (`ice`)",
                water.id
            )));
        }
        self.terrain = defs;
        self.ice = ice;
        Ok(())
    }

    pub(super) fn load_lights(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/lights.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let err = |m: String| ContentError::new(path, m);
        let parsed: LightFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for (i, preset) in parsed.light.into_iter().enumerate() {
            let id = LightId(u16::try_from(i).map_err(|_| err("too many lights".into()))?);
            preset
                .stability
                .check()
                .map_err(|m| err(format!("light `{}`: stability: {m}", preset.id)))?;
            if self.light_ids.insert(preset.id.clone(), id).is_some() {
                return Err(err(format!("duplicate light id `{}`", preset.id)));
            }
            self.lights.push(preset);
        }
        // A hood names another light, which has no hood of its own.
        for preset in &self.lights {
            let Some(hooded) = &preset.hooded else {
                continue;
            };
            match self.light_id(hooded).map(|id| self.light(id)) {
                None => {
                    return Err(err(format!(
                        "light `{}`: hooded: unknown light `{hooded}`",
                        preset.id
                    )));
                }
                Some(other) if other.hooded.is_some() => {
                    return Err(err(format!(
                        "light `{}`: hooded: `{hooded}` has a hood of its own",
                        preset.id
                    )));
                }
                Some(_) => {}
            }
        }
        Ok(())
    }

    pub(super) fn load_dungeons(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        for (path, text) in files.table("dungeons") {
            let parsed: dungeon::DungeonFile =
                toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
            if parsed.defaults.is_some() && path != "data/dungeons.toml" {
                return Err(ContentError::new(
                    path,
                    "`[defaults]` belongs in data/dungeons.toml only",
                ));
            }
            let key_id = |name: &str| {
                self.object_id(name)
                    .filter(|o| self.object(*o).key.is_some())
            };
            let resolved = parsed.resolve(
                path,
                |name| (self.creature_id(name)).map(|c| (c, self.creature(c).level)),
                key_id,
                |name| self.trap_id(name),
                &self.room_effects,
                self.shops.as_ref(),
                self.draw_odds,
            )?;
            for mut def in resolved {
                if let Some(first) = seen.insert(def.id.clone(), path) {
                    return Err(duplicate_id("dungeon", &def.id, path, first));
                }
                if self.levels.contains_key(&def.id) {
                    return Err(ContentError::new(
                        path,
                        format!("dungeon `{}` has the same id as a level", def.id),
                    ));
                }
                def.campaign = campaign::of_path(path).map(str::to_string);
                self.pin_vaults(&def)
                    .and_then(|()| self.pin_lairs(&def))
                    .map_err(|m| ContentError::new(path, format!("dungeon `{}`: {m}", def.id)))?;
                self.dungeons.insert(def.id.clone(), def);
            }
        }
        self.check_lairs()
            .map_err(|m| ContentError::new("data/dungeons.toml", m))?;
        if !self.dungeons.is_empty() {
            crate::mapgen::Palette::resolve(self)
                .map_err(|m| ContentError::new("data/dungeons.toml", m))?;
        }
        Ok(())
    }

    /// Fills every area's generic creature list (spec §7 "Danger levels",
    /// [`danger::Spawning::generic`]). The populations' list of a campaign is every type that
    /// lives in some area with a danger (a branch's or a hand-made level's `population`) of that
    /// campaign or of no campaign, uniques and a portal branch's left out. An area's list is that one
    /// ([`danger::StrayPool::Populations`]), or by default ([`danger::StrayPool::All`]) that one
    /// and every type of the campaign or of none that may stray ([`Content::may_stray`]):
    /// hostile to the player, no unique, no animated object (those come by their own
    /// populations), and not marked `stray = false` (people, keepers, guardians, test types).
    pub(super) fn fill_generic(&mut self) {
        let mut lives: BTreeMap<Option<String>, BTreeSet<CreatureId>> = BTreeMap::new();
        // A Pit's folk stay in it (spec §7 "The Pits"): though shared, its population is in no
        // other area's list.
        let branches = (self.dungeons.values())
            .filter(|d| d.portal.is_none())
            .filter_map(|d| Some((d.campaign.clone(), d.spawning.as_ref()?)));
        let levels = (self.levels.iter()).filter_map(|(id, l)| {
            let campaign = self.level_campaigns.get(id).cloned().flatten();
            Some((campaign, &l.spawning.as_ref()?.1))
        });
        for (campaign, s) in branches.chain(levels) {
            let kinds = (s.population.iter()).filter(|&&c| !self.creature(c).unique);
            lives.entry(campaign).or_default().extend(kinds);
        }
        let shared = lives.get(&None).cloned().unwrap_or_default();
        let generic = |campaign: &Option<String>, pool: danger::StrayPool| -> Vec<CreatureId> {
            let own = lives.get(campaign).into_iter().flatten();
            let mut all: BTreeSet<CreatureId> = own.chain(&shared).copied().collect();
            if pool == danger::StrayPool::All {
                let strays = (0..self.creatures.len())
                    .map(|i| CreatureId(i as u16))
                    .filter(|&c| {
                        let def = self.creature(c);
                        def.campaign.is_none() || def.campaign == *campaign
                    })
                    .filter(|&c| self.may_stray(c));
                all.extend(strays);
            }
            all.into_iter().collect()
        };
        let lists: Vec<Vec<CreatureId>> = (self.dungeons.values())
            .map(|d| (d.spawning.as_ref()).map_or_else(Vec::new, |s| generic(&d.campaign, s.pool)))
            .collect();
        let level_lists: Vec<Vec<CreatureId>> = (self.levels.iter())
            .map(|(id, l)| {
                let campaign = self.level_campaigns.get(id).cloned().flatten();
                (l.spawning.as_ref()).map_or_else(Vec::new, |(_, s)| generic(&campaign, s.pool))
            })
            .collect();
        for (d, list) in self.dungeons.values_mut().zip(lists) {
            if let Some(s) = d.spawning.as_mut() {
                s.generic = list;
            }
        }
        for ((_, l), list) in self.levels.iter_mut().zip(level_lists) {
            if let Some((_, s)) = l.spawning.as_mut() {
                s.generic = list;
            }
        }
        self.fill_rosters();
    }

    /// How deep each endless branch's roster reaches: the floor whose danger is the highest level
    /// that may be drawn there (a family's top counts), its population's or its strays'.
    pub(super) fn fill_rosters(&mut self) {
        let none = BTreeSet::new();
        let tops: Vec<Option<u32>> = (self.dungeons.values())
            .map(|d| {
                d.endless?;
                let s = d.spawning.as_ref()?;
                let kinds = s.population.iter().chain(&s.generic).copied();
                let kinds: Vec<CreatureId> = kinds.collect();
                let top = (self.candidates(&kinds, &none).iter())
                    .map(|(_, (_, to), ..)| *to)
                    .max()?;
                Some(d.plan(1)?.danger?.deepest_at(top))
            })
            .collect();
        for (d, top) in self.dungeons.values_mut().zip(tops) {
            if let (Some(e), Some(top)) = (d.endless.as_mut(), top) {
                e.roster_depth = top;
            }
        }
    }

    /// Checks the vaults branch `def`'s levels name (dungeon vaults, shared or of its campaign)
    /// and marks them [`Vault::pinned`].
    pub(super) fn pin_vaults(&mut self, def: &DungeonDef) -> Result<(), String> {
        for depth in 1..=def.depth {
            let names = def.vaults(depth);
            for name in names {
                let vault = (self.vaults.iter_mut())
                    .find(|v| v.id() == name)
                    .ok_or_else(|| format!("unknown vault `{name}`"))?;
                if vault.is_outdoor() {
                    return Err(format!(
                        "vault `{name}` is outdoor: it stamps in no dungeon"
                    ));
                }
                if vault.campaign.is_some() && vault.campaign != def.campaign {
                    let theirs = vault.campaign.as_deref().unwrap_or_default();
                    return Err(format!(
                        "vault `{name}` is {theirs}'s: a branch stamps its own campaign's or shared ones"
                    ));
                }
                if vault.danger.is_given() {
                    return Err(format!(
                        "vault `{name}` has a `danger`: one a level names is stamped there for certain"
                    ));
                }
                vault.pinned = true;
            }
        }
        Ok(())
    }

    pub(super) fn load_vision(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/vision.toml";
        let err = |m: String| ContentError::new(path, m);
        let profiles = match files.get_optional(path) {
            Some(text) => {
                let parsed: VisionFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
                parsed.vision
            }
            None => vec![VisionProfile::needs_no_light()],
        };
        if profiles.is_empty() {
            return Err(err("at least one vision profile is required".into()));
        }
        for (i, profile) in profiles.into_iter().enumerate() {
            if self.vision_ids.insert(profile.id.clone(), i).is_some() {
                return Err(err(format!("duplicate vision id `{}`", profile.id)));
            }
            self.visions.push(profile);
        }
        Ok(())
    }

    /// Resolves an optional light preset name used in `file`.
    pub(super) fn resolve_light(
        &self,
        file: &str,
        name: Option<&str>,
    ) -> Result<Option<LightId>, ContentError> {
        name.map(|n| {
            self.light_id(n)
                .ok_or_else(|| ContentError::new(file, format!("unknown light `{n}`")))
        })
        .transpose()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TerrainFile {
    terrain: Vec<terrain::TerrainSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LightFile {
    light: Vec<LightPreset>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VisionFile {
    vision: Vec<VisionProfile>,
}

/// Checks a terrain type's noise fields (spec §5 "Hearing"): loudness within
/// [`MAX_LOUDNESS`] (it bounds how far a sound is carried), and a `toggle_noise` only on what
/// opens and shuts.
fn check_terrain_noise(spec: &terrain::TerrainSpec) -> Result<(), String> {
    if spec.toggle_noise > MAX_LOUDNESS || spec.muffle.is_some_and(|m| m > MAX_LOUDNESS) {
        return Err(format!(
            "`toggle_noise` and `muffle` must be at most {MAX_LOUDNESS}"
        ));
    }
    let shut_chest = spec.chest == Some(crate::chest::ChestState::Shut);
    if spec.toggle_noise > 0 && spec.opens_to.is_none() && spec.closes_to.is_none() && !shut_chest {
        return Err("only doors and shut chests have a `toggle_noise`".into());
    }
    Ok(())
}

/// Checks a terrain type's ice fields (spec §5 "Ice"): water that freezes or ice, not both; both
/// are walkable, transparent, give no light, and are neither doors nor stairs (ice and water
/// trade places as it freezes and thaws, under whoever stands there, so nothing else may change
/// with them).
fn check_terrain_ice(spec: &terrain::TerrainSpec) -> Result<(), String> {
    if spec.freeze.is_none() && spec.ice.is_none() {
        return Ok(());
    }
    if spec.freeze.is_some() && spec.ice.is_some() {
        return Err("terrain is water that freezes (`freeze`) or ice (`ice`), not both".into());
    }
    let plain = spec.walkable
        && spec.transparent
        && spec.light.is_none()
        && spec.stairs.is_none()
        && spec.opens_to.is_none()
        && spec.closes_to.is_none()
        && spec.travel_minutes.is_none();
    if !plain {
        return Err(
            "water that freezes and ice must be walkable, transparent, without a light, \
                    and neither a door nor stairs nor overworld terrain"
                .into(),
        );
    }
    Ok(())
}

/// Checks a terrain type's kicking fields (spec §8 "Doors"): only a closed door breaks, and
/// `breaks_to` and `sturdiness` go together; a statue (spec §8 "Statues") has a `sturdiness`
/// of its own, and no `breaks_to` or `chest` the kick or the chest code would read instead.
fn check_kicking(spec: &terrain::TerrainSpec) -> Result<(), String> {
    if spec.statue {
        return if spec.sturdiness.is_some() && spec.breaks_to.is_none() && spec.chest.is_none() {
            Ok(())
        } else {
            Err("a `statue` has a `sturdiness`, and no `breaks_to` or `chest`".into())
        };
    }
    match (&spec.breaks_to, spec.sturdiness) {
        (None, None) => Ok(()),
        (Some(_), Some(_)) if spec.opens_to.is_none() => {
            Err("only closed doors (`opens_to`) break when kicked".into())
        }
        (Some(_), Some(_)) => Ok(()),
        _ => Err("`breaks_to` and `sturdiness` go together".into()),
    }
}

/// The chest terrains (spec §8 "Chests"): at most one each of shut, open, and smashed (the code
/// finds each by its state); a shut chest is not walkable (it opens as one walks into it), open
/// and smashed ones are; none is a door, stairs, or breakable. Deep water (`swim`) is not
/// walkable: swimming is the way onto it.
fn check_chests(defs: &[TerrainDef]) -> Result<(), String> {
    for state in crate::chest::ChestState::ALL {
        let of: Vec<&TerrainDef> = defs.iter().filter(|d| d.chest == Some(*state)).collect();
        if of.len() > 1 {
            return Err(format!(
                "terrain `{}` and `{}` are both a {} chest",
                of[0].id,
                of[1].id,
                state.id()
            ));
        }
    }
    if let Some(def) = defs.iter().find(|d| d.swim && d.walkable) {
        return Err(format!(
            "terrain `{}`: deep water (`swim`) is not walkable",
            def.id
        ));
    }
    for def in defs.iter().filter(|d| d.chest.is_some()) {
        let shut = def.is_shut_chest();
        if def.walkable == shut {
            return Err(format!(
                "terrain `{}`: a shut chest is not walkable, an open or smashed one is",
                def.id
            ));
        }
        if def.is_door() || def.stairs.is_some() || def.breaks.is_some() || def.kicked_in.is_some()
        {
            return Err(format!(
                "terrain `{}`: a chest is no door, stairs, or wall (its band says how it breaks)",
                def.id
            ));
        }
    }
    Ok(())
}
