//! The accessors of [`PerceptionView`] that read **ground truth**: each reads the live level
//! (`self.level`) or the player's actor (`self.player`), not only the snapshot.
//!
//! **Review note.** Rendering, UI, and dumps read the perception snapshot only (spec §5,
//! CLAUDE.md "Hard rules"), and nothing in these types enforces it: a `Stack` or a cell handed
//! out here is safe because `Stack`'s sensitive fields are `pub(crate)` and each accessor checks
//! what the player sees, remembers, or holds before it reads. A change here is a change to what
//! the player may know, so review each one for that: the player's own actor gives what they
//! carry, wear, and know of themselves (never a modifier, a potential, or a mark), and the level
//! only cells the snapshot shows or memory keeps. A new accessor that reads `self.level` or
//! `self.player` goes in this file, so `view.rs` and `shop.rs` keep to the snapshot. Features
//! that extend the view in their own modules and read ground truth too: `beside.rs` (`ways`),
//! `world/rig.rs` (`rigged`, `rigs`), `world/patron.rs` (`commission`), and `world/quest.rs`
//! (`quests`).

use super::{
    CellView, Freshness, NeedView, OwnResource, OwnScribe, OwnSkill, OwnStudy, OwnTalent,
    PerceivedLight, PerceptionView, Sale, SeenActor, ShopView, TalentView,
};
use crate::ability::{Ability, OwnAbility};
use crate::action::Positions;
use crate::blood::Stain;
use crate::burden::{self, Load};
use crate::chest::{ChestLook, ChestState};
use crate::content::{
    CombatStance, DamageTypeId, DiseaseId, Flow, LevelKind, ObjectId, Place, RoomEffectDef, Stance,
    StatusId, TrapId, WeaponSkillId, ZoneKind,
};
use crate::equipment::Equipment;
use crate::geom::Pos;
use crate::hunger::Hunger;
use crate::object::{Stack, Weight};
use crate::skills::{self, SkillId};
use crate::stats::PerStat;
use crate::talent::{self, Candidate};
use crate::weapon_skills::WeaponSkills;
use crate::world::{
    self, ActorKind, RollCount, SkillRoll, count_of, favour, part_of, unpaid_parts,
};

impl<'a> PerceptionView<'a> {
    pub fn level_id(&self) -> &'a str {
        &self.level.id
    }

    pub fn level_name(&self) -> &'a str {
        &self.level.name
    }

    pub fn level_kind(&self) -> LevelKind {
        self.level.kind
    }

    /// The named region the player is in: on a level with regions (the overworld), the one
    /// around the viewer; in a local area, the one around its overworld cell.
    /// The overworld encounter the player must answer now (spec §7 "Overworld encounters"),
    /// as the one dialog asks it: its title, text, and rows; `None` while none is pending. The
    /// app opens the dialog from it after any action and on load, and it cannot be closed
    /// without an answer (the owner, #796 Q9).
    pub fn encounter(&self) -> Option<crate::world::EncounterView> {
        let pending = self.encounter?;
        let (_, entry) = self
            .content
            .encounter_entry(self.campaign, &pending.entry)?;
        let id = crate::content::local_area_id(pending.cell);
        let area = self.content.local_area(self.campaign, &id);
        let choices = crate::world::encounter_choices(pending, entry, self.player, self.content);
        Some(crate::world::encounter_view(
            pending,
            &entry.name,
            area.map(|a| a.landscape),
            &choices,
        ))
    }

    pub fn region(&self) -> Option<&'a str> {
        let (level, at) = match &self.level.exit {
            Some(exit) => (exit.to.as_str(), exit.pos),
            None => (self.level.id.as_str(), self.snapshot.viewer),
        };
        self.content.level(level)?.region_at(at)
    }

    /// The effect covering the whole level (spec §7 "Special rooms"), which the player was told
    /// of on arrival.
    pub fn pervading(&self) -> Option<&'a RoomEffectDef> {
        (self.level.pervading).map(|id| self.content.room_effect(id))
    }

    /// The special room at `p` (spec §7 "Special rooms"), if the player knows it: they have been
    /// inside it, and `p` is a cell they see or remember.
    pub fn room(&self, p: Pos) -> Option<&'a RoomEffectDef> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        let room = self.level.room_at(p)?;
        room.entered.then(|| self.content.room_effect(room.effect))
    }

    /// Whether `p` is hallowed ground (Sanctuary, `world/sanctuary.rs`), and whether the dark
    /// powers laid it (`Some(true)`: dreaded ground): the player laid it, so they know it where
    /// they see or remember the cell.
    pub fn hallow(&self, p: Pos) -> Option<bool> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        self.level.hallows_at(p, self.time).next().map(|h| h.dark)
    }

    /// The zone on `p` as the player knows it (`world/zone.rs`): what lies there where they see
    /// the cell, what they last saw there where they remember it, and nothing on a hidden cell.
    pub fn zone_at(&self, p: Pos) -> Option<ZoneKind> {
        match self.cell(p) {
            CellView::Unknown => None,
            CellView::Visible(_) => self.snapshot.zones.get(&p).copied(),
            CellView::Remembered(_) => self.level.zone_memory.get(&p).copied(),
        }
    }

    /// The percent of all light a special room keeps at a visible cell (a shadowed room's
    /// darkness, which the player sees; 100 elsewhere), for light that presentation adds of its
    /// own (a burning missile in flight); `None` for cells not in view.
    pub fn light_kept(&self, p: Pos) -> Option<u16> {
        if !self.snapshot.is_visible(p) {
            return None;
        }
        Some(self.level.effects_at(self.content, p).light())
    }

    /// Where the stairs or location at `p` lead, by name — only for cells the player sees or
    /// remembers (the staircase itself is known; its destination is public knowledge).
    pub fn leads_to(&self, p: Pos) -> Option<String> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        let link = self.level.stairs_at(p)?;
        // A named location (a legend's `name`, a revealed hidden location's) goes by its own
        // name, even while it leads nowhere yet (spec §7 "Hidden locations").
        (link.name.clone()).or_else(|| self.content.level_display_name(link.to.as_deref()?))
    }

    /// The named place at `p` (a legend's `name`: a location, a landmark), if the player sees
    /// or remembers the cell.
    pub fn place(&self, p: Pos) -> Option<&'a Place> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        self.level.place(self.content, p)
    }

    /// The locations the player knows on the overworld, by position, each with its name: the
    /// ways into other places on cells they see or remember (a hidden location only once
    /// revealed, when it is one of them). Empty on any other level.
    pub fn known_locations(&self) -> Vec<(Pos, String)> {
        if self.level.kind() != LevelKind::Overworld {
            return Vec::new();
        }
        let mut known: Vec<(Pos, String)> = (self.level.stairs.iter())
            .filter_map(|s| Some((s.pos, self.leads_to(s.pos)?)))
            .collect();
        known.sort_by_key(|(p, _)| (p.y, p.x));
        known
    }

    pub fn width(&self) -> i32 {
        self.level.width()
    }

    pub fn height(&self) -> i32 {
        self.level.height()
    }

    pub fn cell(&self, p: Pos) -> CellView {
        if self.snapshot.is_visible(p) {
            // A disguised mimic shows a shut chest (`world/mimic.rs`).
            let shown = self.snapshot.disguise_at(p).map(|d| d.terrain);
            CellView::Visible(shown.unwrap_or(self.level.terrain[p]))
        } else {
            match self.level.memory.get(p) {
                Some(Some(t)) => CellView::Remembered(*t),
                _ => CellView::Unknown,
            }
        }
    }

    /// The flow of running water at a visible cell (presentation-only); `None` for cells that are
    /// not in view or not running water.
    pub fn flow(&self, p: Pos) -> Option<Flow> {
        if !self.snapshot.is_visible(p) {
            return None;
        }
        self.level.flow.get(&p).copied()
    }

    /// The pile at `p` as the player knows it, top first: what lies there now on a cell in sight
    /// (where creatures would be seen), as last seen on any other cell; empty where nothing is
    /// known to lie.
    pub fn objects(&self, p: Pos) -> &'a [Stack] {
        let pile = if self.snapshot.sees_actors_at(p) {
            self.level.objects.get(&p)
        } else {
            self.level.object_memory.get(&p)
        };
        pile.map_or(&[], Vec::as_slice)
    }

    /// The cells where the player sees a corpse now, nearest first (ties: row order).
    pub fn corpses_in_sight(&self) -> Vec<Pos> {
        let me = self.viewer();
        let mut cells: Vec<Pos> = (self.level.objects.keys().copied())
            .filter(|&p| self.corpse_at(p))
            .collect();
        cells.sort_by_key(|p| (p.dist2(me), p.y, p.x));
        cells
    }

    /// The blood at `p` as the player knows it (spec §8 "Blood"): what is there now where they
    /// see it (an open cell in sight, a wall's stained face), as last seen on a cell out of
    /// sight; `None` where they know of none (a visible wall seen only from its clean side
    /// shows none).
    pub fn blood(&self, p: Pos) -> Option<&'a Stain> {
        if self.snapshot.sees_blood_at(p) {
            self.level.blood.get(&p)
        } else if self.snapshot.is_visible(p) {
            None
        } else {
            self.level.blood_memory.get(&p)
        }
    }

    /// The char at `p` as the player knows it (spec §8 "Blood", [`crate::scorch`]): by
    /// [`Self::blood`]'s rules.
    pub fn scorch(&self, p: Pos) -> Option<&'a Stain> {
        if self.snapshot.sees_scorch_at(p) {
            self.level.scorch.get(&p)
        } else if self.snapshot.is_visible(p) {
            None
        } else {
            self.level.scorch_memory.get(&p)
        }
    }

    /// The chest at `p` as the player sees or remembers it (spec §8 "Chests"): its band and
    /// whether it is shut, open, or smashed (as the cell looks to them), and, with Chest-wise,
    /// on a cell in sight, whether a shut one is locked. Never its level or what it holds.
    /// `None` where they know no chest (or without chest bands).
    pub fn chest_at(&self, p: Pos) -> Option<ChestLook> {
        let terrain = match self.cell(p) {
            CellView::Visible(t) | CellView::Remembered(t) => t,
            CellView::Unknown => return None,
        };
        let state = self.content.terrain(terrain).chest?;
        // A disguised mimic's chest is of its level, as the chest it stands in for would be; a
        // chest remembered where none stands now (a mimic found out out of sight) is of the
        // floor's.
        let level = match (self.level.chests.get(&p), self.snapshot.disguise_at(p)) {
            (Some(chest), _) => chest.level,
            (None, Some(d)) => d.level,
            (None, None) => self.content.item_level(&self.level.id),
        };
        let band = self.content.chests().band_index(level)?;
        let wise =
            || !self.player.talents.is_empty() && self.player.talent_mods(self.content).chest_wise;
        let locked = (state == ChestState::Shut && self.snapshot.is_visible(p) && wise())
            .then(|| self.level.locks.contains_key(&p));
        Some(ChestLook {
            band,
            state,
            locked,
        })
    }

    /// What the statue at `p` was, as the player sees or remembers it (spec §8 "Statues"): the
    /// creature type a stoned statue shows (its shape is plain to see), never what it holds.
    /// `None` where they know no statue of the stoned.
    pub fn statue_at(&self, p: Pos) -> Option<crate::content::CreatureId> {
        let terrain = match self.cell(p) {
            CellView::Visible(t) | CellView::Remembered(t) => t,
            CellView::Unknown => return None,
        };
        self.content.terrain(terrain).statue?;
        self.level.statues.get(&p).map(|s| s.of)
    }

    /// What is cut in the gravestone at `p` (spec §7 "Graves"; `world/graves.rs`), where the
    /// player sees or remembers one: "Here lies Ælfric Wudu, 1381–1427." `None` elsewhere, or
    /// on a blank stone.
    pub fn epitaph(&self, p: Pos) -> Option<&'a str> {
        let terrain = match self.cell(p) {
            CellView::Visible(t) | CellView::Remembered(t) => t,
            CellView::Unknown => return None,
        };
        self.content
            .terrain(terrain)
            .grave
            .then(|| self.level.graves.get(&p).map(String::as_str))
            .flatten()
    }

    /// Whom the statue at `p` is of, where it bears a past character's name from the graveyard
    /// (`world/graveyard.rs`): "Brakka the dwarf". `None` where they know no statue there, or
    /// it bears no name.
    pub fn statue_name(&self, p: Pos) -> Option<String> {
        self.statue_at(p)?;
        let statue = self.level.statues.get(&p)?;
        statue.name.as_ref().map(|f| f.words(self.content))
    }

    /// The chests the player sees or remembers on this level ([`PerceptionView::chest_at`]), by
    /// position.
    pub fn known_chests(&self) -> Vec<(Pos, ChestLook)> {
        let mut cells: Vec<Pos> = (self.level.chests.keys().copied())
            .chain(self.snapshot.disguises.iter().map(|d| d.pos))
            .collect();
        cells.sort_unstable();
        cells.dedup();
        (cells.into_iter())
            .filter_map(|p| Some((p, self.chest_at(p)?)))
            .collect()
    }

    /// The trap the player knows at `p` (spec §8 "Traps"): on a cell in sight, a known trap lying
    /// there now; anywhere else, the trap remembered there. Hidden traps are never here.
    pub fn trap(&self, p: Pos) -> Option<TrapId> {
        if self.snapshot.sees_actors_at(p) {
            (self.level.traps.get(&p))
                .filter(|t| t.known)
                .map(|t| t.kind)
        } else {
            self.level.trap_memory.get(&p).copied()
        }
    }

    /// Whether the wall at `p` is one the player found sounds hollow (spec §7 "Special rooms": a
    /// greater treasure room's breach, found by searching), until it is broken.
    pub fn hollow(&self, p: Pos) -> bool {
        self.level.hollows.contains(&p)
    }

    /// The hollow walls the player has found on this level ([`PerceptionView::hollow`]), by
    /// position.
    pub fn known_hollows(&self) -> Vec<Pos> {
        self.level.hollows.iter().copied().collect()
    }

    /// Every trap the player knows on this level ([`PerceptionView::trap`]), by position.
    pub fn known_traps(&self) -> Vec<(Pos, TrapId)> {
        let cells = (self.level.traps.keys()).chain(self.level.trap_memory.keys());
        let mut cells: Vec<Pos> = cells.copied().collect();
        cells.sort_unstable();
        cells.dedup();
        (cells.into_iter())
            .filter_map(|p| Some((p, self.trap(p)?)))
            .collect()
    }

    /// What the player carries, by position (0 first: the order they picked it up in).
    pub fn inventory(&self) -> &'a [Stack] {
        &self.player.inventory
    }

    /// What item descriptions show the player of worth (spec §8 "Talents": Gem lore) and of
    /// make (an item's capacity, to a holder of Eye for the make or one trained in Appraise:
    /// docs/dev/enchanting.md), by their own abilities.
    pub fn appraisal(&self) -> crate::magic::describe::Appraisal {
        let talents = self.player.talent_mods(self.content);
        let appraise = self.content.skill_id("appraise");
        let trained = appraise.and_then(|s| self.player.skills.value(s));
        crate::magic::describe::Appraisal {
            gems: talents.gem_worth,
            capacity: talents.eye_for_make || trained.is_some_and(|v| v > 0),
        }
    }

    /// `stack`'s description as the player knows it (spec §6 "Item description"):
    /// [`crate::magic::describe::appraised`] with their eye for worth, a wand whose type they
    /// know saying what it does in their hands ([`PerceptionView::wand_does`]).
    /// With Eye for the make (the craftsperson's), what it reads of the item's make follows
    /// ([`crate::magic::describe::make_lines`]), after the capacity line, before a unique's
    /// power and lore.
    pub fn describe_item(&self, stack: &Stack) -> Vec<crate::magic::describe::Line> {
        use crate::magic::describe::{self, LineKind};
        let maker = self.character.map(|c| c.name.as_str());
        let mut lines =
            describe::appraised_for(stack, self.lore, self.appraisal(), Some(self.player), maker);
        if self.player.talent_mods(self.content).eye_for_make {
            let percent = crate::world::salvage_percent(self.content, self.player);
            let protected = crate::world::protected_in(self.content, self.quests, stack);
            let mends = self.player.talent_mods(self.content).make_do.is_some();
            let make = describe::make_lines(stack, self.lore, percent, protected, mends);
            let at = (lines.iter())
                .rposition(|l| !matches!(l.kind, LineKind::Power | LineKind::Lore))
                .map_or(lines.len(), |i| i + 1);
            lines.splice(at..at, make);
        }
        lines
    }

    /// What the wand `stack` does in the player's hands once they know its type (its rung comes
    /// with the type, the spell scaling design §7.4): `4d6 fire, radius 2`; `None` before, for
    /// anything else, and for a wand whose spell rolls nothing.
    pub fn wand_does(&self, stack: &Stack) -> Option<String> {
        crate::magic::describe::wand_does(stack, self.lore, self.player)
    }

    /// The player's own weight: their base weight plus what they carry, worn and wielded things
    /// included.
    pub fn own_weight(&self) -> Weight {
        self.player.weight(self.content)
    }

    /// What of their inventory the player wears and wields, by slot.
    pub fn equipment(&self) -> &'a Equipment {
        &self.player.equipment
    }

    /// What the player has ready to shoot (spec §8 "Ranged combat"): their missile weapon, where
    /// in their inventory the ammunition for it is, and how far it shoots; `None` unless they
    /// have both.
    pub fn ready_to_fire(&self) -> Option<(ObjectId, usize, i32)> {
        let (launcher, _) = self.player.launcher(self.content)?;
        let item = self.player.ammunition_item(self.content)?;
        Some((launcher, item, self.player.launcher_range(self.content)?))
    }

    /// Where in their inventory what the player has ready to throw is (a stack of rocks, or a
    /// sword in the missile weapon slot).
    pub fn ready_to_throw(&self) -> Option<usize> {
        self.player.ready_to_throw(self.content)
    }

    /// Where in their inventory the wand the player has ready is (spec §8 "Magic items":
    /// "Wands"), if any.
    pub fn ready_wand(&self) -> Option<usize> {
        self.player.wand_item(self.content)
    }

    /// Where in their inventory the rod the player has ready is (spec §8 "Magic items":
    /// "Rods"), if any.
    pub fn ready_rod(&self) -> Option<usize> {
        self.player.rod_item(self.content)
    }

    /// What a use of the rod in inventory stack `item` would cost the player in mana and how
    /// far it would reach for them, if it is a rod whose type they know (spec §8 "Magic items":
    /// "Rods").
    pub fn own_rod(&self, item: usize) -> Option<(i32, i32)> {
        let stack = self.player.inventory.get(item)?;
        if !self.lore.is_identified(stack.object) {
            return None;
        }
        let rod = self.content.object(stack.object).rod?;
        let effect = self.content.rod_effect(rod.effect);
        let cost = self.player.rod_cost(self.content, &rod);
        Some((cost, self.player.rod_reach(self.content, effect)))
    }

    /// The digging tool the player has ready (spec §8 "Digging"), if any.
    #[cfg(test)]
    pub fn ready_to_dig(&self) -> Option<ObjectId> {
        self.player.dig_tool(self.content).map(|(tool, _)| tool)
    }

    /// Why the player cannot dig now, in the rules' words (no digging tool ready, or untrained
    /// in Mining: `world::dig_refusal`), or `None` when `u` may ask for a direction.
    pub fn dig_refusal(&self) -> Option<&'static str> {
        crate::world::dig_refusal(self.player, self.content)
    }

    /// The carried light the player has ready (spec §5 "Light sources": a lantern in the Tool
    /// slot), by its inventory position, lit or not: `u` uses it at once, without a direction.
    pub fn ready_light(&self) -> Option<usize> {
        self.player.ready_light(self.content)
    }

    /// Whether the wall at `p`, which the player sees, has been dug into and still stands.
    pub fn is_dug(&self, p: Pos) -> bool {
        self.is_visible(p) && self.level.dug.contains_key(&p)
    }

    /// How far the player throws inventory stack `item`, if it can be thrown (a throwing
    /// weapon, or another weapon, badly).
    pub fn throw_range(&self, item: usize) -> Option<i32> {
        self.player.throw_range(item, self.content)
    }

    /// How far the player throws inventory stack `item` in a thrown exploit (its throw range,
    /// no farther than their sneak dice reach); `None` for what cannot be thrown so (a flask).
    pub fn exploit_throw_range(&self, item: usize) -> Option<i32> {
        self.player.exploit_throw_range(item, self.content)
    }

    /// Whether inventory stack `item` is made for throwing (what the throw choice lists): a
    /// throwing weapon, or a potion, which shatters where it lands (spec §8 "Potions": "Flasks").
    pub fn is_throwing_weapon(&self, item: usize) -> bool {
        (self.player.inventory.get(item)).is_some_and(|s| {
            let def = self.content.object(s.object);
            def.is_thrown() || def.shatters()
        })
    }

    /// The player's own scores, by stat (spec §8 "Stats"), as the rules read them: raised by
    /// what they wear and wield (spec §8 "Magic items"), lowered while a status effect or stat
    /// damage lowers them (spec §8 "Status effects", "Diseases") and while hunger leaves them
    /// Weak or worse (spec §8 "Hunger"). That is all the player
    /// knows of stats: never their potentials or modifiers, and nothing of a creature's.
    pub fn own_scores(&self) -> PerStat<u8> {
        self.player.effective_stats(self.content).scores()
    }

    /// How hungry the player is (spec §8 "Hunger"): the state, never the nutrition counter;
    /// `None` when nobody hungers.
    pub fn own_hunger(&self) -> Option<Hunger> {
        self.player.hunger(self.content)
    }

    /// The player's scores as they are without their status effects, hunger, or stat damage
    /// (raised by what they wear and wield all the same): what [`PerceptionView::own_scores`]
    /// would be with none (the character screen marks the lowered ones).
    pub fn own_base_scores(&self) -> PerStat<u8> {
        self.player.raised_stats(self.content).scores()
    }

    /// The player's own trained skills, in skill order (spec §8 "Skills"): each one's value and
    /// how practised it is, coarsely. Untrained skills are not here; nor is anything of a
    /// creature's skills, a mark, or a benefit.
    pub fn own_skills(&self) -> Vec<OwnSkill> {
        let ladder = self.content.ladder();
        (self.player.skills.iter())
            .map(|(skill, trained)| OwnSkill {
                skill,
                value: trained.value,
                progress: skills::progress(trained, ladder),
            })
            .collect()
    }

    /// The player's own status effects, the most important first (spec §8 "Status effects"):
    /// what they suffer, never how badly or for how long.
    pub fn own_effects(&self) -> impl Iterator<Item = StatusId> + 'a {
        self.player.effects.iter().map(|e| e.status)
    }

    /// The diseases the player knows they have, in disease order (spec §8 "Diseases"): those
    /// that have progressed at least once. One that incubates, or has only just shown, is not
    /// here (a showing one makes the player Diseased, [`PerceptionView::own_effects`]).
    pub fn own_diseases(&self) -> impl Iterator<Item = DiseaseId> + 'a {
        self.player.diseases.known()
    }

    /// What the player may treat now with the Heal skill (spec §8 "Skills"), each by how they
    /// know it ("Sickness" for a disease they cannot name yet) and the action that treats it:
    /// the Heal ability's choice. Their own first, then their allies' beside them ("Your
    /// wolf's bleeding").
    pub fn own_ailments(&self) -> Vec<crate::heal::OwnAilment> {
        let mut list = crate::heal::own(self.player, self.content);
        list.extend(self.snapshot.ally_ailments.iter().cloned());
        list
    }

    /// The player's abilities, in menu order (grouped by source, [`crate::ability`]), each with
    /// why it cannot be used now, if it cannot: what the abilities menu (#156) lists. They know
    /// what they can do and what they hold.
    pub fn abilities(&self) -> Vec<OwnAbility> {
        let here = Ability::here(self.level, self.content, self.player.pos);
        let escorts = (self.snapshot.allies.iter()).any(|a| a.source.is_escort());
        let orders = Ability::orders(
            !self.snapshot.allies.is_empty(),
            self.kept().next().is_some() || escorts,
        );
        let allies_here = self.snapshot.allies.iter().any(|a| a.here);
        let companion_here = (self.snapshot.allies.iter())
            .any(|a| a.here && matches!(a.source, crate::world::AllySource::Companion(_)));
        let beside = |ability| self.ways(crate::beside::Beside::Use(ability));
        (Ability::of(self.player, self.content).into_iter())
            .chain(here)
            .chain(orders)
            // Listen and Wedge only beside a closed door (the owner, #1327).
            .filter(|&ability| {
                !matches!(ability, Ability::Listen | Ability::Wedge)
                    || beside(ability).is_some_and(|ways| !ways.is_empty())
            })
            .map(|ability| {
                let unavailable = self.unavailable(ability, allies_here, companion_here);
                // One aimed in a direction with nothing beside it may act on (#1327).
                let no_target = (unavailable.is_none())
                    .then(|| beside(ability))
                    .flatten()
                    .is_some_and(|ways| ways.is_empty());
                let target = crate::beside::Beside::Use(ability).target(self.content);
                OwnAbility {
                    ability,
                    unavailable: unavailable.or(target
                        .filter(|_| no_target)
                        .map(crate::ability::Unavailable::NoTarget)),
                }
            })
            .collect()
    }

    /// Why the player cannot use `ability` now, before what is beside them is asked
    /// ([`PerceptionView::abilities`]).
    fn unavailable(
        &self,
        ability: Ability,
        allies_here: bool,
        companion_here: bool,
    ) -> Option<crate::ability::Unavailable> {
        match ability {
            // An order needs someone on the level to hear it (spec §8 "Allies").
            Ability::Heel | Ability::Hold | Ability::SetOn if !allies_here => {
                Some(crate::ability::Unavailable::NoAllies)
            }
            // Give needs the beast at hand (`world/collar.rs`).
            Ability::Give if self.snapshot.kept_beside.is_empty() => {
                Some(crate::ability::Unavailable::NotBeside)
            }
            // Decipher needs a scroll of an unidentified type (`world/decipher.rs`).
            Ability::Decipher
                if !(self.player.inventory.iter())
                    .any(|stack| crate::world::undeciphered(self.content, &self.lore(), stack)) =>
            {
                Some(crate::ability::Unavailable::NoUnknownScroll)
            }
            // Make do and mend needs a damaged thing carried, not worn (`world/make_do.rs`).
            Ability::Class(crate::content::ClassActive::MakeDo)
                if !crate::world::mends_any(self.player) =>
            {
                Some(crate::ability::Unavailable::NothingToMend)
            }
            // Sic 'em needs its beast on the level (`world/sic.rs`).
            Ability::Class(crate::content::ClassActive::Sic) if !companion_here => {
                Some(crate::ability::Unavailable::NoCompanion)
            }
            // Trapwright rigs on a level, never the overworld (`world/rig.rs`).
            Ability::Class(crate::content::ClassActive::Rig) => {
                (ability.unavailable(self.player, self.content)).or_else(|| {
                    (self.level.kind() == crate::content::LevelKind::Overworld)
                        .then_some(crate::ability::Unavailable::NotHere)
                })
            }
            // A companion is one at a time (spec §8 "Allies": Tame).
            Ability::Tame if self.companion().is_some() => {
                Some(crate::ability::Unavailable::HaveCompanion)
            }
            Ability::Spell(spell) => {
                crate::ability::spell_unavailable(self.player, self.content, spell)
            }
            // An ally beside the player may need tending (`world/heal.rs`).
            Ability::Heal if !self.snapshot.ally_ailments.is_empty() => (ability
                .unavailable(self.player, self.content))
            .filter(|&why| why != crate::ability::Unavailable::NothingToTreat),
            // Calling a familiar asks for none alive and no hostile in sight.
            Ability::Class(crate::content::ClassActive::CallFamiliar) => self
                .calling_barred()
                .or_else(|| ability.unavailable(self.player, self.content)),
            // Field forge sets an anvil on a level, never the overworld (`world/field_forge.rs`).
            Ability::Class(crate::content::ClassActive::FieldForge)
                if self.level.kind() == crate::content::LevelKind::Overworld =>
            {
                Some(crate::ability::Unavailable::NotHere)
            }
            // Foraging asks for the wild underfoot (spec §8 "Foraging").
            Ability::Forage
                if crate::world::forage_ground(self.content, self.level, self.player.pos)
                    .is_none() =>
            {
                Some(crate::ability::Unavailable::NotHere)
            }
            _ => ability.unavailable(self.player, self.content),
        }
    }

    /// The ways to the glowing fungus beside the player, in compass order (the Gather ability,
    /// `world/gather.rs`): the menu gathers at once from the one, and asks the way when there
    /// are more. The player sees what stands beside them.
    pub fn gather_ways(&self) -> Vec<crate::geom::Dir> {
        crate::world::fungus_ways(self.content, self.level, self.player.pos)
    }

    /// The name the abilities menu gives `ability`: its own, but the priest's prayer by its
    /// highest tier as the face that answers them gives it ("Orison", "Whisper"; they know their
    /// band and their own talents).
    pub fn ability_name(&self, ability: Ability) -> std::borrow::Cow<'a, str> {
        match ability {
            Ability::Class(crate::content::ClassActive::Orison) => {
                match self.player.prayer_title(self.content) {
                    Some((name, _)) => name.into(),
                    None => ability.name(self.content),
                }
            }
            _ => ability.name(self.content),
        }
    }

    /// What the abilities menu says `ability` does: its own line, but the priest's prayer's
    /// that of its highest tier.
    pub fn ability_description(&self, ability: Ability) -> &'a str {
        match ability {
            Ability::Class(crate::content::ClassActive::Orison) => {
                (self.player.prayer_title(self.content))
                    .map_or_else(|| ability.description(self.content), |(_, d)| d)
            }
            _ => ability.description(self.content),
        }
    }

    /// The spells the player knows (spec §8 "Spells": learned, or their class gives them), in
    /// menu order.
    pub fn own_spells(&self) -> Vec<crate::content::SpellId> {
        self.player.known_spells(self.content)
    }

    /// The grimoires the player could learn from now (spec §8 "Spells": "Learning"), in pack
    /// order: those that teach a spell they do not know, and those of a spell they know below
    /// its last rank ("Ranks": a reading would deepen it), each with its chance in words; none
    /// for a character not trained in Decipher script. What the read list tells of a grimoire.
    pub fn studies(&self) -> Vec<OwnStudy> {
        let Some(rules) = self.content.study_rules() else {
            return Vec::new();
        };
        (self.player.inventory.iter().enumerate())
            .filter_map(|(item, stack)| {
                let spell = self.content.object(stack.object).grimoire?;
                let odds =
                    crate::world::study_odds(self.player, self.content, self.character, spell)?;
                Some(OwnStudy {
                    item,
                    spell,
                    rank: odds.rank,
                    chance: rules.words(odds.ways()),
                })
            })
            .collect()
    }

    /// The rank the player knows `spell` at (spec §8 "Spells": "Ranks"; they know their own
    /// spells), 1 to `MAX_SPELL_RANK`; `None` for a spell they do not know.
    pub fn own_spell_rank(&self, spell: crate::content::SpellId) -> Option<u8> {
        self.player.spell_rank(self.content, spell)
    }

    /// The spells the player could write a scroll of (spec §8 "Scrolls": Scribe), in menu
    /// order: those they know that a scroll holds, each with its price and time; empty without
    /// Scribe. What the scribe list offers.
    pub fn scribes(&self) -> Vec<OwnScribe> {
        let costs = self.content.action_costs();
        let turn = crate::content::price(
            costs.cost(crate::content::ActionKind::Scribe),
            crate::content::BASE_TURN,
        );
        (self.own_spells().into_iter())
            .filter(|&spell| self.content.scroll_of(spell).is_some())
            .filter_map(|spell| {
                let def = self.content.spell_of(spell)?;
                let mana = self.player.scribe_cost(self.content, def)?;
                let turns = crate::world::scribe_turns(self.content, def)?;
                let seconds = u64::try_from(turn).unwrap_or(0) * u64::from(turns) / 1000;
                Some(OwnScribe {
                    spell,
                    mana,
                    time: crate::time::Duration::seconds(seconds),
                })
            })
            .collect()
    }

    /// How many blank scrolls the player carries (spec §8 "Scrolls"): what Scribe writes on.
    pub fn blank_scrolls(&self) -> u32 {
        let blank = self.content.blank_scroll();
        (self.player.inventory.iter())
            .filter(|s| Some(s.object) == blank)
            .map(|s| s.count)
            .sum()
    }

    /// The recipes of profession `skill` the player sees, in path order, each with what it takes
    /// and why it cannot be made now (`world/craft.rs`): what the profession's screen lists.
    pub fn recipes(&self, skill: crate::skills::SkillId) -> Vec<crate::world::RecipeRow> {
        crate::world::recipe_rows(self.content, self.lore, self.level, self.player, skill)
    }

    /// The choices of generic recipe `recipe`, in order, each with the stacks carried that it may
    /// take (`world/craft_make.rs`): the forging wizard's steps. Empty for a recipe with none
    /// (or one not made yet).
    pub fn craft_steps(&self, recipe: crate::content::RecipeId) -> Vec<crate::world::CraftStep> {
        crate::world::craft_steps(self.content, self.lore, self.level, self.player, recipe)
    }

    /// [`Self::craft_steps`] as the picks made so far (in step order) narrow them: an
    /// Enchanting recipe's seals follow its crystals and item, its affix kept its item.
    pub fn craft_steps_given(
        &self,
        recipe: crate::content::RecipeId,
        picks: &[Option<usize>],
    ) -> Vec<crate::world::CraftStep> {
        crate::world::craft_steps_given(
            self.content,
            self.lore,
            self.level,
            self.player,
            recipe,
            picks,
        )
    }

    /// What `recipe` made with `picks` (inventory positions in step order, `None` a skipped
    /// optional) would make and use, in words; or why it cannot, as the rules would say it.
    pub fn craft_plan(
        &self,
        recipe: crate::content::RecipeId,
        picks: &[Option<usize>],
    ) -> Result<crate::world::CraftPlan, String> {
        crate::world::craft_plan(
            self.content,
            self.lore,
            self.level,
            self.player,
            recipe,
            picks,
        )
    }

    /// What the player's spell `spell` costs them now, in mana (spec §8 "Spells": they know
    /// what their own spells cost); `None` for a spell they cannot cast.
    pub fn own_spell_cost(&self, spell: crate::content::SpellId) -> Option<i32> {
        let def = self.content.spell_of(spell)?;
        Some((self.player).spell_cost(self.content, def))
    }

    /// What the player's weapon active `id` costs them in stamina now, their talents' discounts
    /// counted (they know their own costs, as a spell's mana).
    pub fn own_weapon_stamina(&self, id: crate::content::WeaponAbilityId) -> u16 {
        self.player.weapon_stamina(self.content, id)
    }

    /// The roll the player's spell `spell` makes now, harm or healing, as they would cast it:
    /// the rung their scores reach on its ladder, their talents' and items' hooks applied
    /// ([`crate::world::Actor::spell_roll`]), with its damage type or "healed" and the text the
    /// abilities menu shows (`4d6 fire`). They know their own spells (the owner's exception,
    /// the spell scaling design decision 11); `None` for one that rolls nothing.
    pub fn own_spell_roll(
        &self,
        spell: crate::content::SpellId,
    ) -> Option<crate::world::SpellRoll> {
        let def = self.content.spell_of(spell)?;
        (self.player).spell_roll_words(self.content, def, crate::world::CastBy::Own)
    }

    /// The faces of the player's spell `spell`'s dice that are rolled again (Sure hand: 1, so
    /// "Your 1s are rolled again."); 0 for none, and for a spell that rolls nothing.
    pub fn own_spell_reroll(&self, spell: crate::content::SpellId) -> u32 {
        let Some(def) = self.content.spell_of(spell) else {
            return 0;
        };
        if def.roll().is_none() {
            return 0;
        }
        self.player.spell_mods(self.content, def).reroll
    }

    /// How far the player's spell `spell` reaches now, its reach ladders climbed: the farthest
    /// cell it may be aimed at, and a burst's radius. They know their own spells.
    pub fn own_spell_reach(&self, spell: crate::content::SpellId) -> Option<crate::world::Reach> {
        let def = self.content.spell_of(spell)?;
        Some(self.player.spell_reach(self.content, def))
    }

    /// The player's ranks and marks in weapon skills (spec §8 "Weapon skills"): they know their
    /// ranks, and see their marks as a bar toward the next. Never what a rank gives.
    pub fn own_weapon_skills(&self) -> &'a WeaponSkills {
        &self.player.weapon_skills
    }

    /// The fighting styles the player fights in now (spec §8 "Fighting styles"), in skill
    /// order: their hands', their readied missile weapon's, and throwing's while something is
    /// ready to throw. They know how they hold their weapons.
    pub fn own_styles_in_use(&self) -> Vec<WeaponSkillId> {
        self.player.styles_in_use(self.content)
    }

    /// The attacks the player has ready (spec §6 "Character screen"): the main hand's blow, the
    /// off hand's with two weapons, and the missile weapon's shot or throw, each with its to-hit,
    /// damage, and range, from the profile a strike would roll now in the fighting style in use.
    /// The one place the game shows a to-hit (the owner's exception to hidden numbers: the
    /// player's own attacks only); unidentified magic counts for nothing here.
    pub fn own_attacks(&self) -> Vec<crate::world::OwnAttack> {
        self.player.known_attacks(self.content)
    }

    /// The player's hit points: what they have left (at most their maximum, never below 0) and
    /// their maximum (spec §8 "Combat").
    pub fn own_hp(&self) -> (i32, i32) {
        let max = self.player.max_hp(self.content);
        (self.player.hp.clamp(0, max), max)
    }

    /// The player's mana: what they have left (at most their maximum, never below 0) and their
    /// maximum (spec §8 "Mana"). Never how fast it comes back.
    pub fn own_mana(&self) -> (i32, i32) {
        let max = self.player.max_mana(self.content);
        (self.player.mana.clamp(0, max), max)
    }

    /// The player's alternative resources (spec §8 "Resources"), in resource order: each one
    /// they hold, what it holds now (at most its maximum), and its maximum. Never how fast it
    /// changes.
    pub fn own_resources(&self) -> Vec<OwnResource> {
        (self.player.resources.iter())
            .map(|pool| {
                let level = self.own_experience().level;
                let max = self.player.resource_max(self.content, pool.resource, level);
                OwnResource {
                    resource: pool.resource,
                    value: pool.value.min(max),
                    max,
                    marked: self.content.resource(pool.resource).marked,
                }
            })
            .collect()
    }

    /// The player's mark, the creature their openings are in (`world/mark.rs`), as they know it:
    /// "the kobold" while they see it, "an unseen foe" while they do not; `None` without one.
    /// Their own choice, so nothing hidden.
    pub fn own_mark(&self) -> Option<String> {
        let mark = self.player.mark?;
        let seen = (self.snapshot.actors.iter()).find(|a| a.id == mark);
        Some(seen.map_or_else(
            || "an unseen foe".to_string(),
            |a| crate::content::the(&self.content.creature(a.creature).name),
        ))
    }

    /// Where a level's skill rolls come from now (spec §8 "Skills"): the level-up screen names
    /// it (the owner's call: numbers on that screen).
    pub fn roll_count(&self) -> RollCount {
        world::roll_count(self.content, self.player, self.character, self.due_level())
    }

    /// What a level-up roll on trained skill `skill` would be: the dice and what set them (the
    /// level-up screen shows them); `None` for a skill the player is not trained in.
    pub fn skill_roll(&self, skill: SkillId) -> Option<SkillRoll> {
        let trained = self.player.skills.get(skill)?;
        let trades = self.player.talent_mods(self.content).trades.clone();
        Some(world::skill_roll(
            self.content,
            self.character,
            &trades,
            skill,
            trained,
        ))
    }

    /// Whether the player's own bursts spare them (spec §8 "Talents": Scroll savant), so a
    /// scroll read close needs no warning.
    pub fn bursts_spare_me(&self) -> bool {
        self.player.talent_mods(self.content).spares_self
    }

    /// The player as prerequisites see them.
    fn candidate(&self) -> Candidate<'a> {
        let level = self.own_experience().level;
        Candidate::player(self.content, self.player, self.character, level)
    }

    /// The player's own talents, in the order taken (spec §8 "Talents"): each by its name for
    /// its choice, its tree, the rank held of how many, and what it does in the game's words;
    /// never a rule's number.
    pub fn own_talents(&self) -> Vec<OwnTalent> {
        let book = self.content.talents();
        let level = self.own_experience().level;
        // The class abilities held: every table's (the prestige class's too), to its level.
        let class: Vec<crate::content::TalentId> = (self.character)
            .map(|c| c.class_levels(self.content, level))
            .unwrap_or_default()
            .into_iter()
            .flat_map(|(table, own)| table.abilities_to(book, own))
            .collect();
        // The rising is spent once it is used up: the first with no Second rising, else both.
        let risen = self.progress.happened(crate::world::RISEN);
        let twice = self.progress.happened(crate::world::RISEN_TWICE);
        let second = (self.player.talent_mods(self.content)).second_rising;
        let spent = |e: &crate::content::TalentEffect| match e {
            crate::content::TalentEffect::RiseFromAshes { .. } => risen && (!second || twice),
            crate::content::TalentEffect::SecondRising => twice,
            _ => false,
        };
        (self.player.talents.iter())
            .filter_map(|taken| {
                let id = book.id(&taken.id)?;
                let def = book.get(id);
                let boon = (self.player.boons.iter())
                    .any(|b| b.id == taken.id && b.choice == taken.choice);
                let story = boon
                    && self.character.is_some_and(|c| {
                        let choice = taken.choice.as_deref();
                        crate::background::gave(self.content, self.campaign, c, &taken.id, choice)
                    });
                Some(OwnTalent {
                    id,
                    name: def.name_for(self.content, taken.choice.as_deref()),
                    tree: def.tree,
                    choice: taken.choice.clone(),
                    description: def.description.clone(),
                    granted: def.granted && !book.tree(def.tree).class,
                    race: def.granted && Some(def.tree) == book.tree_id("racial"),
                    flaw: def.flaw,
                    class: class.contains(&id),
                    spent: def.effects.iter().any(&spent),
                    boon: boon && !story,
                    story,
                })
            })
            .collect()
    }

    /// Every talent of the game, tree by tree in display order (the Talents page and the pick
    /// list): whether the player holds it, its prerequisites with whether each holds, and the ways it may be taken now (none: not now). A birth sign's pick for another
    /// sign is left out: the stars a character was born under never change (spec §8 "Birth
    /// sign"); so is one that asks for a hidden weapon skill or style (Two Shields) the player
    /// has no marks in yet.
    pub fn talents(&self) -> Vec<TalentView> {
        use crate::content::Need;
        let content = self.content;
        let who = self.candidate();
        let born_for = |def: &crate::content::TalentDef| {
            (def.needs.iter())
                .filter(|n| matches!(n, Need::Animal(_) | Need::Element(_) | Need::Lucky))
                .all(|n| who.meets(content, &n, def.choice, None))
        };
        let table = content.weapon_skills();
        let found = |def: &crate::content::TalentDef| {
            (def.needs.iter()).all(|n| match n {
                Need::WeaponSkill(id, _) => who.weapon_skills.shows(table, id),
                _ => true,
            })
        };
        (content.talents().by_tree())
            .filter(|(_, def)| born_for(def) && found(def))
            .map(|(id, def)| {
                let needs = (def.needs.iter())
                    .map(|need| NeedView {
                        met: who.meets(content, &need, def.choice, None),
                        words: need.words(content),
                    })
                    .collect();
                TalentView {
                    id,
                    name: def.name.clone(),
                    tree: def.tree,
                    choice: def.choice,
                    held: self.player.talents.has(&def.id),
                    granted: def.granted,
                    description: def.description.clone(),
                    needs,
                    openings: talent::openings(content, &who, def),
                }
            })
            .collect()
    }

    /// The player's alignment band (spec §8 "Alignment"), which they know; `None` without
    /// alignment rules. Never the score.
    pub fn own_alignment(&self) -> Option<crate::alignment::Band> {
        (self.content.alignment()).map(|_| self.player.band(self.content))
    }

    /// What the player carries against what they can (spec §8 "Burden"): their burden, and the
    /// weights at which it changes, which follow from scores they know.
    pub fn own_load(&self) -> Load {
        let room = (self.level.effects_at(self.content, self.player.pos)).burden();
        burden::load(self.player, self.own_race(), self.content, room)
    }

    /// The player's combat stance (spec §8 "Stances"): only its name is ever shown.
    pub fn own_stance(&self) -> CombatStance {
        self.player.stance
    }

    /// The player's Defence ("Dv"): the total an attack roll must reach (their burden capping
    /// their Dexterity). Only the total: its Dexterity part would give away a modifier.
    pub fn own_defence(&self) -> i32 {
        let burden = self.own_load().penalty();
        self.player
            .defence(self.content, burden, self.own_race())
            .total()
    }

    /// The player's Protection ("Pv"): the damage what they wear soaks off every hit, the total
    /// only (never what each piece does against a kind, nor a creature's).
    pub fn own_protection(&self) -> i32 {
        self.player.protection(self.content, self.own_race()).base
    }

    /// What the player's Knowledge tells of a creature they see (spec §8 "Skills": Knowledge),
    /// in words only: what its kind resists, is immune to, or is vulnerable to, in damage type
    /// order; nothing while their Knowledge does not stand above its level, and never what it
    /// wears or what has been done to it.
    pub fn known_resistances(
        &self,
        actor: &SeenActor,
    ) -> Vec<(DamageTypeId, crate::damage::ResistanceWord)> {
        match actor.kind {
            ActorKind::Player => Vec::new(),
            _ => (self.player).known_resistances(self.content, actor.creature),
        }
    }

    /// How a seen creature stands toward the player: its faction's stance (spec §7 "Factions"),
    /// which the player knows from its type; neutral for an animal a Beast-friend has not
    /// attacked one of (spec §8 "Talents"). Nothing else about factions is perceived.
    pub fn stance(&self, actor: &SeenActor) -> Stance {
        let animal = actor
            .faction
            .is_some_and(|f| self.content.factions().def(f).animal);
        if animal
            && actor.kind == ActorKind::Creature
            && self.player.talent_mods(self.content).beast.beast_friend
            && !(self.progress.empathy_broken).contains(&self.content.creature(actor.creature).id)
        {
            return Stance::Neutral;
        }
        match self.content.ids().player {
            Some(player) => self
                .content
                .stance_of(self.content.creature(player).faction, actor.faction),
            None => Stance::Neutral,
        }
    }

    /// Perceived light at a visible cell (wall-face rule applied); `None` for cells not in view.
    pub fn light(&self, p: Pos) -> Option<PerceivedLight> {
        if !self.snapshot.is_visible(p) {
            return None;
        }
        let light = self.snapshot.light[p];
        let vision = self.vision();
        let overworld = self.level.kind == LevelKind::Overworld;
        Some(PerceivedLight {
            rgb: light.rgb,
            uv: light.uv,
            darkvision: !overworld
                && vision.darkvision > 0
                && light.brightness() < vision.threshold,
            dist2: p.dist2(self.snapshot.viewer),
        })
    }

    /// How fresh `stack` smells to the player, if it is food and they can tell by smell (spec §8
    /// "Birth sign": Hound's nose); `None` otherwise.
    pub fn smell(&self, stack: &Stack) -> Option<Freshness> {
        if self.player.talents.is_empty() || !self.player.talent_mods(self.content).smell_rot {
            return None;
        }
        Some(match crate::rot::stage(self.content, stack, self.time)? {
            crate::rot::Stage::Fresh => Freshness::Fresh,
            crate::rot::Stage::Old => Freshness::Turning,
            crate::rot::Stage::Rotten | crate::rot::Stage::Gone => Freshness::Rotten,
        })
    }

    /// How fresh `stack`, a corpse or food the player has in front of them, looks to an eye
    /// trained in Appraise or Survival (spec §8 "Hunger"; `world/freshness.rs`): `fresh`, `old`,
    /// or `rotten` when their check passes, `None` otherwise.
    pub fn judge(&self, stack: &Stack) -> Option<&'static str> {
        let stage = crate::world::freshness::judged(
            self.content,
            self.player,
            self.seed,
            stack,
            self.time,
        )?;
        Some(stage.id())
    }

    /// The open shop whose floor `p` is, if the player sees or remembers `p` (its sign and goods
    /// are in plain sight) and its keeper still trades with them.
    pub fn shop_at(&self, p: Pos) -> Option<ShopView<'a>> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        let shops = self.content.shops()?;
        let shop = (self.level.shops.iter()).find(|s| s.is_open() && s.contains(p))?;
        Some(ShopView {
            shop,
            shops,
            content: self.content,
            favour: favour(self.content, shops, self.player),
            now: self.time,
        })
    }

    /// What the player has not paid for of each inventory stack, priced, in the shop they stand
    /// in (`None` for what is theirs; spec §7 "Shops").
    pub fn unpaid(&self) -> Vec<Option<u32>> {
        let inventory = &self.player.inventory;
        let Some(view) = self.shop_here() else {
            return vec![None; inventory.len()];
        };
        let parts = unpaid_parts(inventory, &view.shop.unpaid);
        (inventory.iter().zip(parts))
            .map(|(s, n)| (n > 0).then(|| view.bill(&part_of(s, n))))
            .collect()
    }

    /// Whether a step onto `to` would dive into deep water (spec §8 "Skills": Swim): from anything
    /// else, into water the player sees or remembers where no one they see is, and open to them:
    /// an untrained or a Strained player is refused by the rules, with no question to ask. Water
    /// they do not know is never stepped into: the rules stop them at it first (`world/swim.rs`).
    pub fn dives_at(&self, to: Pos) -> bool {
        let swim = |p: Pos| match self.cell(p) {
            CellView::Visible(t) | CellView::Remembered(t) => self.content.terrain(t).swim,
            CellView::Unknown => false,
        };
        swim(to)
            && !swim(self.viewer())
            && self.actor_at(to).is_none()
            && crate::world::shore_refusal(self.content, self.player, self.own_load().penalty())
                .is_none()
    }

    /// What the shop the player stands in offers for their own things among the inventory stacks
    /// `positions` names (all of them for none), if it buys any: what dropping them there would
    /// sell ([`crate::Action::Sell`]).
    pub fn sale(&self, positions: &Positions) -> Option<Sale> {
        let view = self.shop_here()?;
        let inventory = &self.player.inventory;
        let parts = unpaid_parts(inventory, &view.shop.unpaid);
        let all = positions.is_empty();
        let mut sale = Sale {
            offer: 0,
            stacks: Vec::new(),
        };
        for (i, stack) in inventory.iter().enumerate() {
            let own = part_of(stack, stack.count - parts[i]);
            if !(all || positions.contains(i)) || own.count == 0 {
                continue;
            }
            if let Some(offer) = view.offer(&own) {
                sale.offer += offer;
                sale.stacks.push(own);
            }
        }
        (!sale.stacks.is_empty()).then_some(sale)
    }

    /// How much gold the player carries (the shops' currency; none without shops).
    pub fn own_gold(&self) -> u32 {
        (self.content.shops()).map_or(0, |s| count_of(&self.player.inventory, s.currency))
    }
}
