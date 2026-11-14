//! Loading creature types and what hangs on them: factions, gear, throws, summons, and drops.

use super::{
    COMMON, Combat, Content, ContentError, ContentFiles, CreatureId, FactionId, Factions,
    GlyphAnim, MEDIUM, PLAYER_FACTION, animated, aura, campaign, creature, creature_spells, drops,
    duplicate_id, faction, gear, load_table, mimic, summon, tag, validate::check_effects,
    weapon_skill,
};
use crate::dice::Dice;
use crate::skills::Skills;
use crate::stats::Stats;
use crate::weapon_skills::WeaponSkills;
use serde::Deserialize;
use std::collections::BTreeMap;

impl Content {
    /// The creature table: `data/creatures.toml` and `data/creatures/*.toml`, numbered in path
    /// order (the single file first); required, with a `player`.
    pub(super) fn load_creatures(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let Some((first_file, _)) = files.table("creatures").next() else {
            return Err(ContentError::new("data/creatures.toml", "file is missing"));
        };
        let mut specs = load_table(files, "creatures", |f: CreatureFile| f.creature)?;
        // Animated types after the shared files, before every campaign's (docs/dev/animated.md),
        // each named by its body's file.
        let generated = self.animated.specs(self)?;
        let at = (specs.iter())
            .position(|(p, _)| campaign::of_path(p).is_some())
            .unwrap_or(specs.len());
        let file = |p: &str| (files.table("animated").map(|(q, _)| q)).find(|q| *q == p);
        let generated =
            (generated.into_iter()).map(|(p, spec)| (file(&p).unwrap_or(animated::RULES), spec));
        // Then the mimics (docs/dev/creatures/mimics.md).
        let mimics = self.mimics.specs(self)?;
        let generated = generated.chain(mimics.into_iter().map(|spec| (mimic::RULES, spec)));
        specs.splice(at..at, generated);
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        for (i, (path, mut spec)) in specs.into_iter().enumerate() {
            let err = |m: String| ContentError::new(path, m);
            let id = CreatureId(u16::try_from(i).map_err(|_| err("too many creatures".into()))?);
            for t in &spec.tags {
                tag::check(t).map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            }
            if spec.plural.is_none() {
                spec.plural = Some(self.plural(&spec.name));
            }
            if let Some(first) = seen.insert(spec.id.clone(), path) {
                return Err(duplicate_id("creature", &spec.id, path, first));
            }
            self.creature_ids.insert(spec.id.clone(), id);
            // A creature of speed 0 never gains the energy to act, and the scheduler would wait
            // on it for ever.
            if spec.speed == 0 {
                return Err(err(format!("creature `{}`: speed 0", spec.id)));
            }
            let light = self.resolve_light(path, spec.light.as_deref())?;
            check_effects(
                path,
                "creature",
                &spec.id,
                spec.anim.as_ref(),
                spec.glow,
                0.0,
                &spec.particles,
            )?;
            if spec.anim.as_ref().is_some_and(GlyphAnim::is_flow) {
                return Err(err(format!(
                    "creature `{}`: only terrain can have a `flow` animation",
                    spec.id
                )));
            }
            if let Some(missing) = spec.leavings.iter().find(|l| self.leaving(**l).is_none()) {
                return Err(err(format!(
                    "creature `{}`: leaving `{}` needs its terrain or object",
                    spec.id,
                    missing.id()
                )));
            }
            // Without races (no characters) a creature's race means nothing, and is not checked.
            let unknown = |r: &&str| self.race(r).is_none() && !self.races.list.is_empty();
            if let Some(race) = spec.race.as_deref().filter(unknown) {
                return Err(err(format!(
                    "creature `{}`: unknown race `{race}`",
                    spec.id
                )));
            }
            let has_stats = !spec.stats.is_empty() || !spec.potential.is_empty();
            if spec.id == "player" && (has_stats || spec.base_hp != 0 || spec.mana != 0) {
                return Err(err(
                    "creature `player`: the player's stats, hit points, and mana come from the \
                     scenario (`stats`, `potential`, `hp`, and `mana` in scenarios/*.toml)"
                        .into(),
                ));
            }
            if spec.id == "player" && !spec.skills.is_empty() {
                return Err(err(
                    "creature `player`: the player's skills come from their race and class".into(),
                ));
            }
            let skills = self
                .creature_skills(&spec.skills)
                .map_err(|m| err(format!("creature `{}`: skills: {m}", spec.id)))?;
            if spec.id == "player" && !spec.weapon_skills.is_empty() {
                return Err(err(
                    "creature `player`: the player's weapon skills come from their race and class"
                        .into(),
                ));
            }
            let ranks: BTreeMap<String, Dice> = (spec.weapon_skills.iter())
                .map(|(id, &rank)| (id.clone(), Dice::fixed(i32::from(rank))))
                .collect();
            let ranks = (self
                .weapon_skills
                .starting_ranks(&ranks, weapon_skill::Ranked::Any))
            .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let ranks = ranks.iter().map(|&(id, rank)| (id, rank.max() as u8));
            let weapon_skills = WeaponSkills::from_ranks(ranks);
            // The status diseases show as comes of a disease, never of a blow.
            let status = |id: &str| {
                let found = self
                    .status_id(id)
                    .filter(|&s| Some(s) != self.disease_status)?;
                let def = self.status(found);
                Some((found, def.max, def.hold.is_some()))
            };
            // One natural attack or several, each struck in turn (`[[creature.attack]]`).
            let written = spec.attack.as_ref().map_or(&[][..], |a| a.0.as_slice());
            let mut attacks = Vec::new();
            for a in written {
                let kinds = (self.resolve_kinds(a.damage_type.as_ref()))
                    .map_err(|m| err(format!("creature `{}`: attack: {m}", spec.id)))?;
                let attack = (a.clone().resolve(status, |id| self.disease_id(id), kinds))
                    .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
                attacks.push(attack);
            }
            let mut attacks = attacks.into_iter();
            let attack = attacks.next();
            let more_attacks: Vec<_> = attacks.collect();
            let breath = (spec.breath.as_ref())
                .map(|b| b.resolve(self, status, |id| self.disease_id(id)))
                .transpose()
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let gaze = (spec.gaze.as_ref())
                .map(|g| g.resolve(self, status, |id| self.disease_id(id)))
                .transpose()
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let burst = (spec.burst.as_ref())
                .map(|b| b.resolve(self))
                .transpose()
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let steals = (spec.steals.as_ref())
                .map(|t| t.resolve(self))
                .transpose()
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let blinks = (spec.blinks.as_ref())
                .map(|b| b.resolve(self))
                .transpose()
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let invisible = (spec.invisible.map(|i| i.resolve(self)))
                .transpose()
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?
                .flatten();
            let throw = (spec.throw.clone())
                .map(|t| {
                    let kinds = self.resolve_kinds(t.damage_type.as_ref())?;
                    t.resolve(status, |id| self.disease_id(id), kinds)
                })
                .transpose()
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let resist = (self.resolve_per_kind("resist", &spec.resist, false))
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let stats = Stats::from_data(&spec.stats, &spec.potential)
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let size_id = spec.size.as_deref().unwrap_or(MEDIUM);
            let size = (self.size_index(size_id))
                .ok_or_else(|| err(format!("creature `{}`: unknown size `{size_id}`", spec.id)))?;
            let rarity_id = spec.rarity.as_deref().unwrap_or(COMMON);
            let rarity = (self.creature_rarities.iter())
                .position(|r| r.id == rarity_id)
                .ok_or_else(|| {
                    err(format!(
                        "creature `{}`: unknown rarity `{rarity_id}`",
                        spec.id
                    ))
                })?;
            let blood = creature::blood(spec.blood.as_deref())
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            let (immune, immune_kinds) = creature::immunities(self, &spec.immune)
                .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            if let Some(animated) = &mut spec.animated {
                animated.family = CreatureId(id.0 - u16::from(animated.member));
            }
            if let Some(mimic) = &mut spec.mimic {
                mimic.family = CreatureId(id.0 - u16::from(mimic.member));
            }
            let resolved = creature::Resolved {
                blood,
                rarity,
                light,
                size,
                stats,
                skills,
                attack,
                more_attacks,
                breath,
                gaze,
                burst,
                throw,
                presence: aura::resolve(self, &spec).map_err(err)?,
                steals,
                blinks,
                invisible,
                weapon_skills,
                resist,
                immune,
                immune_kinds,
                spells: creature_spells::resolve(self, &spec).map_err(err)?,
                faction: self.creature_faction(&spec).map_err(err)?,
                factions: self.creature_factions(&spec).map_err(err)?,
                vision: self.vision_index(&spec.vision).ok_or_else(|| {
                    err(format!(
                        "creature `{}`: unknown vision profile `{}`",
                        spec.id, spec.vision
                    ))
                })?,
            };
            check_combat(&spec).map_err(err)?;
            if let Some(summon) = &spec.summon {
                summon
                    .check()
                    .map_err(|m| err(format!("creature `{}`: {m}", spec.id)))?;
            }
            // A creature without a corpse is never eaten: its `on_eat` would be lost.
            if spec.on_eat.is_some() && (!spec.corpse || spec.id == "player") {
                return Err(err(format!(
                    "creature `{}`: `on_eat` is for creatures that leave a corpse",
                    spec.id
                )));
            }
            // An everlasting corpse is a unique's, and there must be one to keep.
            if spec.everlasting && (!spec.corpse || !spec.unique || spec.id == "player") {
                return Err(err(format!(
                    "creature `{}`: `everlasting` is for a unique creature that leaves a corpse",
                    spec.id
                )));
            }
            let mut def = spec.resolve(resolved);
            def.campaign = campaign::of_path(path).map(str::to_string);
            self.creatures.push(def);
        }
        if !self.creature_ids.contains_key("player") {
            return Err(ContentError::new(
                first_file,
                "a creature with id `player` is required",
            ));
        }
        self.name_families()
    }

    /// A creature type's skills as data lists them (`skills = { listen = 30 }`): fixed values of
    /// skills `data/skills/` has.
    pub(super) fn creature_skills(&self, data: &BTreeMap<String, u16>) -> Result<Skills, String> {
        let mut skills = Skills::default();
        for (name, &value) in data {
            let id = (self.skill_id(name)).ok_or_else(|| format!("unknown skill `{name}`"))?;
            skills.add(id, i32::from(value));
        }
        Ok(skills)
    }

    /// The faction a creature type names (none: it stands neutral to all). With factions
    /// defined the player belongs to [`PLAYER_FACTION`]; without, none may be named.
    pub(super) fn creature_faction(
        &self,
        spec: &creature::CreatureSpec,
    ) -> Result<Option<FactionId>, String> {
        if self.factions.is_empty() {
            return match &spec.faction {
                Some(name) => Err(format!(
                    "creature `{}`: unknown faction `{name}` (there is no data/factions/)",
                    spec.id
                )),
                None => Ok(None),
            };
        }
        let is_player = spec.id == "player";
        if !spec.factions.is_empty() {
            if spec.faction.is_some() || is_player {
                return Err(format!(
                    "creature `{}`: `factions` replaces `faction`, never with it",
                    spec.id
                ));
            }
            // The heaviest; of equal weights the first by id.
            let (_, top) = (self.creature_factions(spec)?.into_iter())
                .map(|(f, w)| (std::cmp::Reverse(w), f))
                .min_by_key(|&(w, f)| (w, f))
                .expect("factions given");
            return Ok(Some(top));
        }
        let name = match &spec.faction {
            Some(name) => name.as_str(),
            None if is_player => PLAYER_FACTION,
            None => return Ok(None),
        };
        if is_player && name != PLAYER_FACTION {
            return Err(format!(
                "the player belongs to the faction `{PLAYER_FACTION}`, not `{name}`"
            ));
        }
        self.factions
            .id(name)
            .map(Some)
            .ok_or_else(|| format!("creature `{}`: unknown faction `{name}`", spec.id))
    }

    /// The factions a creature type's creatures each draw one of (`factions = { seelie = 50,
    /// unseelie = 50 }`), in id order: known ids, weights 1 or more (a draw over them); empty when
    /// it gives none.
    pub(super) fn creature_factions(
        &self,
        spec: &creature::CreatureSpec,
    ) -> Result<Vec<(FactionId, u32)>, String> {
        (spec.factions.iter())
            .map(|(name, &weight)| {
                let id = (self.factions.id(name))
                    .ok_or_else(|| format!("creature `{}`: unknown faction `{name}`", spec.id))?;
                if weight == 0 {
                    return Err(format!("creature `{}`: faction `{name}` weighs 0", spec.id));
                }
                Ok((id, weight))
            })
            .collect()
    }

    /// Status effects and the healing rules (optional: none, and the default healing).
    /// Every status that something `follows` into must end: it eases, something cures it, or
    /// a resource brings it (Raging ends with Rage, spec §8 "Resources"), so the one that
    /// follows it begins. Checked once the resources are loaded.
    pub(super) fn check_follows_end(&self) -> Result<(), ContentError> {
        for def in &self.statuses {
            let brought = (self.resources.iter()).any(|r| {
                r.status
                    .is_some_and(|s| self.statuses[usize::from(s.0)].id == def.id)
            });
            if def.follows.is_some() && def.eases.is_none() && def.cured_by.is_empty() && !brought {
                return Err(ContentError::new(
                    "data/status/",
                    format!(
                        "status `{}`: follows: it never ends (it has no `eases` and no \
                         `cured_by`, and no resource brings it)",
                        def.id
                    ),
                ));
            }
        }
        Ok(())
    }

    /// Factions (optional: none, everyone neutral): a file per faction in `data/factions/`, in
    /// path order (`NNN-<id>.toml`: the number keeps their order).
    pub(super) fn load_factions(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        if files.table("factions").next().is_none() {
            return Ok(());
        }
        let specs = load_table(files, "factions", |f: faction::FactionFile| f.faction)?;
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for (path, spec) in &specs {
            if let Some(first) = seen.insert(&spec.id, path) {
                return Err(duplicate_id("faction", &spec.id, path, first));
            }
        }
        let specs = specs.into_iter().map(|(_, spec)| spec).collect();
        self.factions =
            Factions::resolve(specs).map_err(|m| ContentError::new("data/factions/", m))?;
        Ok(())
    }

    /// The classes', races', and birth signs' starting kits (spec §8 "Character creation"), once
    /// the objects, gems, slots, and weapon skills they name are known.
    /// Resolves the creature types' gear (spec §8 "Creature gear") once objects are known.
    pub(super) fn load_gear(&mut self) -> Result<(), ContentError> {
        let gear = (self.creatures.iter())
            .map(|c| {
                gear::resolve(&c.gear_spec, self).map_err(|m| {
                    ContentError::new("data/creatures/", format!("creature `{}`: {m}", c.id))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (creature, gear) in self.creatures.iter_mut().zip(gear) {
            if creature.id == "player" && !gear.is_empty() {
                return Err(ContentError::new(
                    "data/creatures/",
                    "creature `player`: the player's gear is their starting kit",
                ));
            }
            creature.gear = gear;
        }
        Ok(())
    }

    /// Resolves what the creature types' throws fly as (spec §8 "Ranged combat": "Creatures'
    /// throws") once objects are known, and names a throw by its object unless it has a name.
    pub(super) fn load_throws(&mut self) -> Result<(), ContentError> {
        for i in 0..self.creatures.len() {
            let Some(throw) = &self.creatures[i].throw else {
                continue;
            };
            let object = (self.object_id(&throw.object_id)).ok_or_else(|| {
                ContentError::new(
                    "data/creatures/",
                    format!(
                        "creature `{}`: throw: unknown object `{}`",
                        self.creatures[i].id, throw.object_id
                    ),
                )
            })?;
            if !throw.damage_given && self.object(object).flask.is_none() {
                return Err(ContentError::new(
                    "data/creatures/",
                    format!(
                        "creature `{}`: throw: no `damage`, and `{}` is no flask",
                        self.creatures[i].id, throw.object_id
                    ),
                ));
            }
            let name = self.object(object).name.clone();
            let throw = self.creatures[i].throw.as_mut().expect("a throw");
            throw.object = object;
            if !throw.named {
                throw.attack.name = name;
            }
        }
        Ok(())
    }

    /// Resolves the creature types' summonings (spec §8 "Summoning") once every type is known:
    /// their pools and their cooldown status (`content/summon.rs`).
    pub(super) fn load_summons(&mut self) -> Result<(), ContentError> {
        for i in 0..self.creatures.len() {
            let Some(spec) = self.creatures[i].summon_spec.clone() else {
                continue;
            };
            let me = CreatureId(i as u16);
            let summon = summon::resolve(self, me, &spec).map_err(|m| {
                ContentError::new(
                    "data/creatures/",
                    format!("creature `{}`: {m}", self.creatures[i].id),
                )
            })?;
            self.creatures[i].summon = Some(summon);
        }
        Ok(())
    }

    /// Reads `data/drops/` and resolves every creature type's drop rows (`content/drops.rs`).
    pub(super) fn load_drops(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        self.drops = drops::parse(files.table("drops"), self)?;
        let own: Vec<_> = (self.creatures.iter())
            .map(|c| c.drop_spec.clone())
            .collect();
        let resolved = drops::resolve(self, &self.drops, &own)?;
        for (def, rows) in self.creatures.iter_mut().zip(resolved) {
            def.drops = rows;
        }
        Ok(())
    }
}

/// One file of the creature table (`data/creatures.toml` or `data/creatures/*.toml`): any number
/// of entries, none in a file of comments only (`README.toml`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatureFile {
    #[serde(default)]
    creature: Vec<creature::CreatureSpec>,
}

/// Checks the fields a creature type's `combat` would leave unread: only a dragon breathes
/// (`world/breath.rs`), a burst on touch is struck instead of every melee blow, and only
/// `ranged` and `caster` creatures have a `range`.
fn check_combat(spec: &creature::CreatureSpec) -> Result<(), String> {
    let id = &spec.id;
    if spec.breath.is_some() && spec.combat != Combat::Dragon {
        return Err(format!(
            "creature `{id}`: only a `combat = \"dragon\"` creature has a breath"
        ));
    }
    // A vortex's burst is its whole offence (`world/creature_burst.rs`): no attack beside it.
    if (spec.burst.as_ref()).is_some_and(|b| b.when == crate::content::BurstWhen::Touches)
        && (spec.attack.is_some() || spec.combat != Combat::Melee)
    {
        return Err(format!(
            "creature `{id}`: a burst on touch is its offence: no attack, and `combat` melee"
        ));
    }
    if spec.range.is_some() && matches!(spec.combat, Combat::Melee | Combat::Dragon) {
        return Err(format!(
            "creature `{id}`: only `ranged` and `caster` creatures have a `range`"
        ));
    }
    Ok(())
}
