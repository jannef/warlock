//! Vaults (`assets/vaults/*.toml`): hand-made map pieces in the level-file format that generators
//! stamp into generated levels where they fit (spec §7).
//!
//! A vault is a rectangle copied verbatim: terrain, placed lights, objects, traps, chests, and
//! creatures with their patrols. A dungeon vault's chests are of the floor's item level (no
//! `chest` level or band of their own, no mimic), and one given no `objects` is filled by its
//! band's loot table when the level is made (`mapgen/chest.rs::fill_vaults`). Its outer edge must be solid (non-walkable terrain such as walls) except for its
//! **entrances**: doors on the edge, never on a corner. Generators join every entrance to the rest
//! of the level. A vault has no stairs and no player start, and every walkable cell must be
//! reachable from an entrance (doors count as passable), so stamping one never strands anything.
//! Its locks (spec §8 "Doors") come with their keys: a key to every lock lies, or is carried by a
//! creature, within reach of an unlocked entrance without unlocking a door.
//!
//! A vault of kind `outdoor` (ruins for the local areas of the overworld) is stamped onto open
//! ground instead of into rock: its `void` cells (the default legend's space) are the local
//! area's own ground, every open cell on its edge (walkable, a door, or void) is an entrance, and
//! every walkable cell must be reachable from its edge. Dungeon generators use only dungeon
//! vaults, local areas only outdoor ones.
//!
//! Where a dungeon vault stamps ([`Vault::stamps_on`]): a shared one (`assets/vaults/`) in every
//! branch, a campaign's (`campaigns/<id>/vaults/`) only in that campaign's branches, and one whose
//! file names `dungeons` (branch ids, or branch levels such as `"cealdhlaw:2"`) only there.
//! A branch level may name vaults (`vaults` in `[[dungeon.levels]]`): each is stamped on that
//! level for certain, and a vault so named ([`Vault::pinned`]) is never drawn at random.

use super::dungeon::{DungeonDef, parse_branch_level};
use super::level::{LevelKind, LevelTemplate};
use super::{Content, ContentError, TerrainId};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use serde::Deserialize;

/// A validated vault.
#[derive(Clone, Debug)]
pub struct Vault {
    pub template: LevelTemplate,
    /// Entrance doors on the edge (outdoor vaults: every open edge cell), each with the
    /// direction leading out of the vault.
    pub entrances: Vec<(Pos, Dir)>,
    /// The campaign whose `vaults/` it is in (spec §3); `None` for a shared one.
    pub campaign: Option<String>,
    /// The branches (`"cealdhlaw"`) and branch levels (`"cealdhlaw:2"`) it stamps in; empty:
    /// every branch it may (its campaign's, or all of them for a shared vault).
    pub dungeons: Vec<String>,
    /// Named by a branch level's `vaults` ([`DungeonDef::vaults`]): stamped there for certain
    /// and never drawn at random.
    ///
    /// [`DungeonDef::vaults`]: super::DungeonDef::vaults
    pub pinned: bool,
}

/// The vault-only field of a vault file (the level-file parser reads the rest and ignores it).
#[derive(Deserialize)]
struct VaultPlaces {
    #[serde(default)]
    dungeons: Vec<String>,
}

/// The `dungeons` a vault file (or a level file, which may not give it) names.
pub(super) fn parse_dungeons(path: &str, text: &str) -> Result<Vec<String>, ContentError> {
    let places: VaultPlaces =
        toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
    Ok(places.dungeons)
}

impl Vault {
    /// Checks the vault rules for a parsed vault file.
    pub(super) fn new(template: LevelTemplate, content: &Content) -> Result<Self, String> {
        let terrain = &template.terrain;
        let (w, h) = (terrain.width(), terrain.height());
        if w < 3 || h < 3 {
            return Err(format!("a vault must be at least 3x3, not {w}x{h}"));
        }
        if template.player_start.is_some() {
            return Err("a vault cannot have a player start".into());
        }
        if let Some(stairs) = template.stairs.first() {
            return Err(format!("a vault cannot have stairs (at {})", stairs.pos));
        }
        // A vault may be stamped many times; a unique character cannot be.
        if let Some(spawn) = template.creatures.iter().find(|c| c.npc.is_some()) {
            return Err(format!(
                "a vault cannot place a unique npc (at {})",
                spawn.pos
            ));
        }
        if !template.rooms.is_empty() {
            return Err("a vault cannot mark special rooms".into());
        }
        // A vault's chests are of the floor it is stamped on (spec §8 "Chests"): no level or
        // band of their own, and no mimic (its type is its level).
        if let (LevelKind::Outdoor, Some(at)) = (template.kind, template.chests.keys().next()) {
            return Err(format!("an outdoor vault cannot hold chests (at {at})"));
        }
        let own_level = |c: &&crate::chest::Chest| c.level != template.item_level;
        if let Some((at, _)) = template.chests.iter().find(|(_, c)| own_level(c)) {
            return Err(format!(
                "a vault's chest is of the floor's level: no `chest` level or band (at {at})"
            ));
        }
        if let Some(spawn) = (template.creatures.iter()).find(|c| content.is_mimic(c.creature)) {
            return Err(format!(
                "a vault cannot place a mimic: its level is the floor's (at {})",
                spawn.pos
            ));
        }
        if let (LevelKind::Outdoor, Some(at)) = (template.kind, template.traps.keys().next()) {
            return Err(format!("an outdoor vault cannot have traps (at {at})"));
        }
        if let (LevelKind::Outdoor, Some(at)) = (template.kind, template.secrets.keys().next()) {
            return Err(format!(
                "an outdoor vault cannot have secret doors (at {at})"
            ));
        }
        // The level it is stamped into decides its temperature, and so its ice.
        if let Some(at) = template.ice.keys().next() {
            return Err(format!("a vault cannot lay ice (at {at})"));
        }
        if template.temperature.is_some() {
            return Err("a vault has no temperature of its own".into());
        }
        if !template.shops.is_empty() {
            return Err("a vault cannot have shops".into());
        }
        let is_door = |p: Pos| {
            let def = content.terrain(terrain[p]);
            def.is_closed_door() || def.is_open_door()
        };
        let entrances = match template.kind {
            LevelKind::Dungeon => door_entrances(&template, content)?,
            LevelKind::Outdoor => open_edges(&template, content)?,
            LevelKind::Overworld => return Err("a vault cannot be of kind `overworld`".into()),
        };

        // Everything walkable is reachable from the first entrance (4-connected, doors passable);
        // in outdoor vaults from any open edge cell, over the open ground too.
        let ground = outdoor_ground(&template, content);
        let passable = |p: Pos| {
            terrain
                .get(p)
                .is_some_and(|t| content.terrain(*t).walkable || is_door(p) || ground(*t))
        };
        let seeds = match template.kind {
            LevelKind::Outdoor => entrances.len(),
            _ => 1,
        };
        let mut seen = Grid::new(w, h, false);
        let mut stack: Vec<Pos> = entrances[..seeds].iter().map(|(p, _)| *p).collect();
        for p in &stack {
            seen[*p] = true;
        }
        while let Some(p) = stack.pop() {
            for dir in [Dir::N, Dir::E, Dir::S, Dir::W] {
                let q = p.step(dir);
                if passable(q) && !seen[q] {
                    seen[q] = true;
                    stack.push(q);
                }
            }
        }
        if let Some((p, _)) = terrain.iter().find(|(p, _)| passable(*p) && !seen[*p]) {
            return Err(format!("{p} cannot be reached from the entrances"));
        }
        check_keys(&template, &entrances, passable, content)?;
        Ok(Self {
            template,
            entrances,
            campaign: None,
            dungeons: Vec::new(),
            pinned: false,
        })
    }

    /// Whether the vault may be stamped on branch `def`'s level at `depth`: a dungeon vault,
    /// shared or of the branch's campaign, and on a branch or level its `dungeons` name, if any.
    pub fn stamps_on(&self, def: &DungeonDef, depth: u32) -> bool {
        if self.is_outdoor() || (self.campaign.is_some() && self.campaign != def.campaign) {
            return false;
        }
        self.dungeons.is_empty()
            || (self.dungeons.iter()).any(|d| *d == def.id || *d == def.level_id(depth))
    }

    /// Checks its `dungeons` once the branches are loaded: each a branch or one of its levels,
    /// of the vault's own campaign, and never on an outdoor vault.
    pub(super) fn check_dungeons(&self, content: &Content) -> Result<(), String> {
        if self.is_outdoor() && !self.dungeons.is_empty() {
            return Err(
                "an outdoor vault stamps in no dungeon: `dungeons` is for dungeon vaults".into(),
            );
        }
        for name in &self.dungeons {
            let def = match parse_branch_level(name) {
                Some(_) => content.branch_level(name).map(|(def, _)| def),
                None => content.dungeon(name),
            }
            .ok_or_else(|| format!("`dungeons`: unknown branch or branch level `{name}`"))?;
            if def.campaign != self.campaign {
                let theirs = def.campaign.as_deref().unwrap_or("no campaign");
                return Err(format!(
                    "`dungeons`: `{name}` is {theirs}'s; a vault stamps only in its own campaign's branches"
                ));
            }
        }
        Ok(())
    }

    pub fn id(&self) -> &str {
        &self.template.id
    }

    /// The file it was read from, for an error.
    pub(super) fn file(&self) -> String {
        match &self.campaign {
            Some(c) => format!(
                "{}/{c}/vaults/{}.toml",
                super::campaign::CAMPAIGNS,
                self.id()
            ),
            None => format!("vaults/{}.toml", self.id()),
        }
    }

    pub fn width(&self) -> i32 {
        self.template.terrain.width()
    }

    pub fn height(&self) -> i32 {
        self.template.terrain.height()
    }

    /// Whether the vault is for the local areas of the overworld (kind `outdoor`).
    pub fn is_outdoor(&self) -> bool {
        self.template.kind == LevelKind::Outdoor
    }
}

/// A vault's locks (spec §8 "Doors") come with their keys: for every lock, a key that opens it
/// lies, or is carried by a creature, within reach of an unlocked entrance without unlocking any
/// door, so a generator never has to place a vault's keys.
fn check_keys(
    template: &LevelTemplate,
    entrances: &[(Pos, Dir)],
    passable: impl Fn(Pos) -> bool,
    content: &Content,
) -> Result<(), String> {
    if template.locks.is_empty() {
        return Ok(());
    }
    let terrain = &template.terrain;
    let unlocked = |p: Pos| passable(p) && !template.locks.contains_key(&p);
    let mut seen = Grid::new(terrain.width(), terrain.height(), false);
    let mut stack: Vec<Pos> = (entrances.iter())
        .map(|(p, _)| *p)
        .filter(|p| unlocked(*p))
        .collect();
    for p in &stack {
        seen[*p] = true;
    }
    while let Some(p) = stack.pop() {
        for dir in [Dir::N, Dir::E, Dir::S, Dir::W] {
            let q = p.step(dir);
            if unlocked(q) && !seen[q] {
                seen[q] = true;
                stack.push(q);
            }
        }
    }
    // A chest's lock wants no key: Pick lock, Knock, or a kick open it, and a chest is never in
    // the way (spec §8 "Chests").
    let chest = |p: Pos| content.terrain(terrain[p]).is_shut_chest();
    match (template.locks.iter())
        .find(|(p, lock)| !chest(**p) && !template.has_key_for(content, lock, |q| seen[q]))
    {
        Some((p, _)) => Err(format!(
            "no key to the lock at {p} lies or is carried within reach of an unlocked entrance \
             (without unlocking a door)"
        )),
        None => Ok(()),
    }
}

/// Whether a terrain is an outdoor vault's open ground: `void`, which takes the local area's own
/// ground when stamped. Never in a dungeon vault.
fn outdoor_ground(
    template: &LevelTemplate,
    content: &Content,
) -> impl Fn(TerrainId) -> bool + use<> {
    let void = content
        .terrain_id("void")
        .filter(|_| template.kind == LevelKind::Outdoor);
    move |t| Some(t) == void
}

/// The outward direction of an edge cell of a `w`×`h` vault (`None` inside).
fn outward(p: Pos, w: i32, h: i32) -> Option<Dir> {
    if p.y == 0 {
        Some(Dir::N)
    } else if p.y == h - 1 {
        Some(Dir::S)
    } else if p.x == 0 {
        Some(Dir::W)
    } else if p.x == w - 1 {
        Some(Dir::E)
    } else {
        None
    }
}

/// Outdoor vaults: every open edge cell (walkable, a door, or open ground) is an entrance.
fn open_edges(template: &LevelTemplate, content: &Content) -> Result<Vec<(Pos, Dir)>, String> {
    let terrain = &template.terrain;
    let ground = outdoor_ground(template, content);
    let entrances: Vec<(Pos, Dir)> = terrain
        .iter()
        .filter(|(_, t)| {
            let def = content.terrain(**t);
            def.walkable || def.is_closed_door() || ground(**t)
        })
        .filter_map(|(p, _)| outward(p, terrain.width(), terrain.height()).map(|d| (p, d)))
        .collect();
    if entrances.is_empty() {
        return Err("an outdoor vault needs an open cell on its edge".into());
    }
    Ok(entrances)
}

/// Dungeon vaults: the edge is solid except for entrance doors, never on a corner.
fn door_entrances(template: &LevelTemplate, content: &Content) -> Result<Vec<(Pos, Dir)>, String> {
    let terrain = &template.terrain;
    let (w, h) = (terrain.width(), terrain.height());
    let is_door = |p: Pos| {
        let def = content.terrain(terrain[p]);
        def.is_closed_door() || def.is_open_door()
    };
    let mut entrances = Vec::new();
    for (p, t) in terrain.iter() {
        let Some(out) = outward(p, w, h) else {
            continue;
        };
        let corner = (p.x == 0 || p.x == w - 1) && (p.y == 0 || p.y == h - 1);
        if is_door(p) && !corner {
            entrances.push((p, out));
        } else if content.terrain(*t).walkable || is_door(p) {
            return Err(format!(
                "edge cell {p} must be solid or an entrance door (not on a corner)"
            ));
        }
    }
    if entrances.is_empty() {
        return Err("a vault needs at least one entrance: a door on its edge".into());
    }
    Ok(entrances)
}
