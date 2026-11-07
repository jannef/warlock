//! Vaults (`assets/vaults/*.toml`): hand-made map pieces in the level-file format that generators
//! stamp into generated levels where they fit (spec §7).
//!
//! A vault is a rectangle copied verbatim: terrain, placed lights, objects, traps, chests, and
//! creatures with their patrols. A dungeon vault's chests are of the floor's item level (no
//! `chest` level or band of their own, no mimic), and one given no `objects` is filled by its
//! band's loot table when the level is made (`mapgen/chest.rs::fill_vaults`). Its outer edge must be solid (non-walkable terrain such as walls) except for its
//! **entrances**: doors on the edge, never on a corner. Generators join every entrance to the rest
//! of the level. A vault has no stairs and no player start. The one exception is a layer boss's lair (spec §7 "The endless branch"): one `>`, its
//! [`Vault::down`], which the stamp lays as plain floor and the level keeps hidden until the boss
//! dies (`world/layer_boss.rs`); only a vault an endless branch's boss row names may have it.
//! Its author places the keys to its locks (spec §8 "Doors"). A legend's `loot` is
//! rolled when it is stamped: a grade, or a pool of item classes (`magic/loot.rs`, [`LootPool`]),
//! at the floor's item level. A legend's `monsters` draws creatures at the floor's danger
//! ([`VaultMonsters`], `mapgen/population.rs`), on top of its population and outside the cap.
//!
//! [`LootPool`]: crate::magic::LootPool
//! [`VaultMonsters`]: super::VaultMonsters
//!
//! A vault of kind `outdoor` (ruins for the local areas of the overworld) is stamped onto open
//! ground instead of into rock: its `void` cells (the default legend's space) are the local
//! area's own ground, and every open cell on its edge (walkable, a door, or void) is an entrance.
//! Dungeon generators use only dungeon vaults, local areas only outdoor ones.
//!
//! Where a dungeon vault stamps ([`Vault::stamps_on`]): a shared one (`assets/vaults/`) in every
//! branch, a campaign's (`campaigns/<id>/vaults/`) only in that campaign's branches, and one whose
//! file names `dungeons` (branch ids, or branch levels such as `"cealdhlaw:2"`) only there.
//! A branch level may name vaults (`vaults` in `[[dungeon.levels]]`): each is stamped on that
//! level for certain, and a vault so named ([`Vault::pinned`]) is never drawn at random.
//!
//! A vault file's `danger = { from = 8 }`, `{ to = 6 }`, or both ([`DangerRange`]) keeps it to the
//! floors whose danger lies in that range (the danger the floor's pools draw at; see
//! [`Vault::suits`]): a deep vault stays deep and a shallow one shallow. It is for vaults drawn at
//! random: refused on an outdoor vault (a local area has no danger) and on one a branch level names.
//!
//! A vault drawn at random lies any of eight ways, rotated or mirrored (`vault_turn.rs`), unless
//! its file says `turns = false`, for text that depends on direction or a layout that must not
//! turn. A vault a branch level names keeps the way its file draws it.

use super::dungeon::{DungeonDef, parse_branch_level};
use super::level::{LevelKind, LevelTemplate};
use super::{Content, ContentError, TerrainId};
use crate::geom::{Dir, Pos};
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
    /// Whether a random stamp may rotate or mirror it (a file's `turns`, default true; see
    /// [`Vault::may_turn`]).
    pub turns: bool,
    /// The dangers it stamps at (a file's `danger`; default: every danger).
    pub danger: DangerRange,
    /// A lair's hidden way down: the cell of its one `>`, with the stairs' terrain (a layer
    /// boss's vault only, checked by `Content::load_dungeons`).
    pub down: Option<(Pos, TerrainId)>,
}

/// The dangers a vault drawn at random stamps at: `danger = { from = 8 }`, `{ to = 6 }`, or both,
/// each end included; neither: every danger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DangerRange {
    pub from: Option<u8>,
    pub to: Option<u8>,
}

impl DangerRange {
    /// Whether it holds `danger`.
    pub fn holds(&self, danger: u8) -> bool {
        self.from.is_none_or(|f| danger >= f) && self.to.is_none_or(|t| danger <= t)
    }

    /// Whether it narrows the dangers at all (a file gave `from` or `to`).
    pub fn is_given(&self) -> bool {
        self.from.is_some() || self.to.is_some()
    }
}

/// The vault-only fields of a vault file (the level-file parser reads the rest and ignores them).
#[derive(Deserialize)]
struct VaultFields {
    #[serde(default)]
    dungeons: Vec<String>,
    turns: Option<bool>,
    /// A vault's `{ from, to }`, or a level's danger rule (the level parser reads that one).
    danger: Option<toml::Value>,
}

/// The `dungeons` a vault file (or a level file, which may not give it) names.
pub(super) fn parse_dungeons(path: &str, text: &str) -> Result<Vec<String>, ContentError> {
    Ok(parse_fields(path, text)?.dungeons)
}

/// The `turns` a vault file (or a level file, which may not give it) says.
pub(super) fn parse_turns(path: &str, text: &str) -> Result<Option<bool>, ContentError> {
    Ok(parse_fields(path, text)?.turns)
}

/// The `danger = { from, to }` a vault file (or a level file, which may not give it) says; `None`
/// for none, or for a level's own `danger` rule (`danger = 3`, `{ base = 2, per_depth = 1 }`).
pub(super) fn parse_danger(path: &str, text: &str) -> Result<Option<DangerRange>, ContentError> {
    Ok((parse_fields(path, text)?.danger)
        .and_then(|v| DangerRange::deserialize(v).ok())
        .filter(DangerRange::is_given))
}

fn parse_fields(path: &str, text: &str) -> Result<VaultFields, ContentError> {
    toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))
}

impl Vault {
    /// Checks the vault rules for a parsed vault file.
    pub(super) fn new(mut template: LevelTemplate, content: &Content) -> Result<Self, String> {
        // A lair's one `>` (module docs); any other staircase is refused.
        let down = match template.stairs.as_slice() {
            [] => None,
            [s] if s.dir == super::terrain::StairsDir::Down
                && template.kind == LevelKind::Dungeon =>
            {
                Some((s.pos, template.terrain[s.pos]))
            }
            [s, ..] => return Err(format!("a vault cannot have stairs (at {})", s.pos)),
        };
        template.stairs.clear();
        let terrain = &template.terrain;
        let (w, h) = (terrain.width(), terrain.height());
        if w < 3 || h < 3 {
            return Err(format!("a vault must be at least 3x3, not {w}x{h}"));
        }
        if template.player_start.is_some() {
            return Err("a vault cannot have a player start".into());
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
        // A monster pool is drawn at the floor's danger; a local area has none.
        if let (LevelKind::Outdoor, Some(group)) = (template.kind, template.monsters.first()) {
            return Err(format!(
                "an outdoor vault cannot draw monsters: a local area has no danger (at {})",
                group.cells[0]
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
        // It draws at the floor's danger; its own `danger` says which floors (`DangerRange`).
        if template.spawning.is_some() {
            return Err(
                "a vault has no danger rule of its own: its `danger` is `{ from, to }`, the floors it stamps on"
                    .into(),
            );
        }
        let entrances = match template.kind {
            LevelKind::Dungeon => door_entrances(&template, content)?,
            LevelKind::Outdoor => open_edges(&template, content)?,
            LevelKind::Overworld => return Err("a vault cannot be of kind `overworld`".into()),
        };
        Ok(Self {
            template,
            entrances,
            campaign: None,
            dungeons: Vec::new(),
            pinned: false,
            turns: true,
            danger: DangerRange::default(),
            down,
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

    /// Whether a floor of `danger` may draw it at random (its file's `danger`). The danger is the
    /// one the floor's pools draw at (`Content::spawning`; 0 on a floor with none), so on an
    /// endless branch past its roster's top it stays at the top's, as the draws do.
    pub fn suits(&self, danger: u8) -> bool {
        self.danger.holds(danger)
    }

    /// Checks its `dungeons` once the branches are loaded: each a branch or one of its levels,
    /// of the vault's own campaign, and never on an outdoor vault.
    pub(super) fn check_dungeons(&self, content: &Content) -> Result<(), String> {
        if self.is_outdoor() && !self.dungeons.is_empty() {
            return Err(
                "an outdoor vault stamps in no dungeon: `dungeons` is for dungeon vaults".into(),
            );
        }
        if self.is_outdoor() && self.danger.is_given() {
            return Err("an outdoor vault has no `danger`: a local area has none".into());
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

    /// Checks its monster pools once it knows its campaign: each holds a type it may meet.
    pub(super) fn check_monsters(&self, content: &Content) -> Result<(), String> {
        for group in &self.template.monsters {
            if group.pool.holds_none(content, self.campaign.as_deref()) {
                return Err(format!(
                    "the monster pool at {} holds no type of its campaign",
                    group.cells[0]
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
