//! [`PerceptionView`], the read-only view of what the player perceives, and its accessors that
//! read only the snapshot ([`Perception`]), the content, and the records the player knows (their
//! character, experience, quests, messages, the clock). The accessors that read ground truth
//! (the live level or the player's actor) are in `truth.rs`.

use super::{AllyView, CellView, Perception, Progress, SeenActor, SeenGear, SeenLight};
use crate::character::Character;
use crate::content::TreeId;
use crate::content::{
    Content, DamageTypeId, Factor, ItemClassId, ObjectId, RaceDef, SlotDef, SlotId, Stance,
    StatusId, VisionProfile,
};
use crate::geom::Pos;
use crate::light::{Light, Sky, Sun};
use crate::noise::PerceivedSound;
use crate::object::Stack;
use crate::temperature::Temperature;
use crate::time::GameTime;
use crate::weather::Weather;
use crate::world::{Actor, ActorId, ActorKind, Death, Due, LevelState};

/// Read-only view of what the player perceives. Obtain one with [`crate::World::perception`].
#[derive(Clone, Copy, Debug)]
pub struct PerceptionView<'a> {
    pub(crate) content: &'a Content,
    pub(crate) level: &'a LevelState,
    /// The player, for what they carry.
    pub(crate) player: &'a Actor,
    /// The player's character, which they know.
    pub(crate) character: Option<&'a Character>,
    /// The campaign the run is of: whose chapters tell the player's story.
    pub(crate) campaign: &'a str,
    /// The time on the clock, which the player knows.
    pub(crate) time: GameTime,
    /// How the run ended, once the player has died.
    pub(crate) death: Option<&'a Death>,
    /// The player's experience total.
    pub(crate) experience: u32,
    /// The player's progression: what their levels still ask them to choose.
    pub(crate) progress: &'a crate::world::Progress,
    /// The player's quests, which they know.
    pub(crate) quests: &'a crate::world::Quests,
    /// The patron's open commission, which the player knows (`world/crafting/patron.rs`).
    pub(crate) patron: &'a crate::world::crafting::patron::Patron,
    /// Everything the player was told, which they know.
    pub(crate) messages: &'a crate::messages::MessageLog,
    /// What the player knows of scroll types: how they name what they see.
    pub(crate) lore: crate::scroll_label::Lore<'a>,
    pub(crate) snapshot: &'a Perception,
    /// The overworld encounter the player must answer, which they know.
    pub(crate) encounter: Option<&'a crate::world::Pending>,
    /// The run's seed, for the freshness check (`world/health/freshness.rs`), which only draws a stream
    /// of its own.
    pub(crate) seed: u64,
}

impl<'a> PerceptionView<'a> {
    pub fn content(&self) -> &'a Content {
        self.content
    }

    /// Whether `stack` is a magic crystal, one the Consume list (`e`) offers beside food and
    /// potions (`data/crystals.toml`; the rare materials are not; `world/crafting/crystal.rs`).
    pub fn is_crystal(&self, stack: &crate::object::Stack) -> bool {
        (self.content.crystals()).is_some_and(|c| c.colour_of_object(stack.object).is_some())
    }

    /// What the player knows of scroll types (spec §8 "Scrolls"): every name presentation shows
    /// of a stack goes through it ([`crate::scroll_label::Lore::describe`]), so an unidentified
    /// scroll shows by its label.
    pub fn lore(&self) -> crate::scroll_label::Lore<'a> {
        self.lore
    }

    /// The message log: what the player was told, oldest first, at most
    /// [`crate::messages::CAPACITY`] entries (the message line and the message history).
    pub fn messages(&self) -> &'a crate::messages::MessageLog {
        self.messages
    }

    /// The pending encounter's entry id and outcome (dumps); `None` while none is pending.
    pub fn encounter_pending(&self) -> Option<(&'a str, crate::world::EncounterOutcome)> {
        self.encounter.map(|p| (p.entry.as_str(), p.outcome))
    }

    /// Where the player is looking from.
    pub fn viewer(&self) -> Pos {
        self.snapshot.viewer
    }

    pub fn is_visible(&self, p: Pos) -> bool {
        self.snapshot.is_visible(p)
    }

    /// Creatures in view, in ascending id order (the player included).
    pub fn actors(&self) -> &'a [SeenActor] {
        &self.snapshot.actors
    }

    pub fn actor_at(&self, p: Pos) -> Option<&'a SeenActor> {
        self.snapshot.actors.iter().find(|a| a.pos == p)
    }

    /// Whether a corpse lies at `p` where the player sees it now (what a spell aimed at a corpse
    /// asks, `world/magic/corpse_spell.rs`).
    pub fn corpse_at(&self, p: Pos) -> bool {
        self.snapshot.sees_actors_at(p)
            && (self.objects(p).iter()).any(|s| self.content.object(s.object).corpse.is_some())
    }

    /// The leavings the player sees or remembers on this level (`content/leaving.rs`), in row
    /// order: statues, webs, and acid-pitted floor by the terrain they know, torn cloth by the
    /// pile they know. What each means is never told.
    pub fn known_leavings(&self) -> Vec<(Pos, crate::content::Leaving)> {
        let mut known = Vec::new();
        for y in 0..self.height() {
            for x in 0..self.width() {
                let p = Pos::new(x, y);
                if let CellView::Visible(t) | CellView::Remembered(t) = self.cell(p)
                    && let Some(leaving) = self.content.leaving_terrain(t)
                {
                    known.push((p, leaving));
                }
                if (self.objects(p).iter()).any(|s| self.content.is_leaving_object(s.object)) {
                    known.push((p, crate::content::Leaving::TornCloth));
                }
            }
        }
        known
    }

    /// Whether the pile at `p` is in sight now (rather than remembered or unknown).
    pub fn sees_objects_at(&self, p: Pos) -> bool {
        self.snapshot.sees_actors_at(p)
    }

    /// The item class `stack` shows as (spec §7 "Objects and materials"): its base item's, which
    /// the player knows of anything they see, identified or not.
    pub fn item_class(&self, stack: &Stack) -> ItemClassId {
        self.content.object(stack.object).class
    }

    /// The positions of `stacks` as item lists show them (spec §6 "Item lists"): grouped by item
    /// class in class order, each class in the stacks' own order.
    pub fn by_class(&self, stacks: &[Stack]) -> Vec<usize> {
        let mut order: Vec<usize> = (0..stacks.len()).collect();
        order.sort_by_key(|&i| self.item_class(&stacks[i]));
        order
    }

    /// Who the player is (spec §8 "Character creation"): their race, gender, and class, the
    /// stat their gender raised, their birth date, and their birth sign; `None` when the
    /// scenario gave no character.
    pub fn character(&self) -> Option<&'a Character> {
        self.character
    }

    /// The player's background story (spec §8 "Background story"), which they know: its
    /// paragraphs, told again from the content with the slots filled; empty for a character
    /// without one (a scenario's own, an old save) or a run without a character.
    pub fn story(&self) -> Vec<String> {
        (self.character)
            .map(|c| crate::background::retell(self.content, self.campaign, c))
            .unwrap_or_default()
    }

    /// The time on the clock.
    pub fn time(&self) -> GameTime {
        self.time
    }

    /// The slots the player has, in display order: every slot but another class's own (the
    /// aristocrat's extra rings, #1404; `Content::slots_for`), unless a talent of theirs opens it
    /// (Many-fingered; `world/items/body_slots.rs`).
    pub fn own_slots(&self) -> Vec<(SlotId, &'a SlotDef)> {
        let class = self.character.map(|c| c.spec.class.as_str());
        let mods = self.player.talent_mods(self.content);
        (self.content.slot_ids())
            .filter(|&(slot, _)| crate::world::has_slot(self.content, class, &mods, slot))
            .collect()
    }

    /// The equipment screen's note for slot `slot` when a talent of the player's bars it, which
    /// they know: `horns`, shown `— (horns)` (Horned; `world/items/body_slots.rs`).
    pub fn barred(&self, slot: SlotId) -> Option<String> {
        if self.player.talents.is_empty() {
            return None;
        }
        let mods = self.player.talent_mods(self.content);
        mods.slots.barred(slot).map(|b| b.mark.clone())
    }

    /// The words that refuse anything put in slot `slot` while a talent of the player's bars it
    /// ("Your horns are in the way."), which the equipment choice shows in place of a list.
    pub fn barred_words(&self, slot: SlotId) -> Option<String> {
        if self.player.talents.is_empty() {
            return None;
        }
        let mods = self.player.talent_mods(self.content);
        mods.slots.barred(slot).map(|b| b.refused.clone())
    }

    /// What seen creature `actor` wears and wields, each thing once, in slot order: all the
    /// player sees of what a creature carries. Empty for creatures not in view and the player.
    pub fn gear(&self, actor: ActorId) -> impl Iterator<Item = &'a SeenGear> + 'a {
        self.snapshot.gear.iter().filter(move |g| g.actor == actor)
    }

    /// The status effects that show on seen creature `actor` (`seen` in `data/status/`:
    /// bleeding), the most important first; none for creatures not in view and the player.
    pub fn effects_of(&self, actor: ActorId) -> impl Iterator<Item = StatusId> + 'a {
        (self.snapshot.effects.iter())
            .filter(move |(a, _)| *a == actor)
            .map(|(_, s)| *s)
    }

    /// The shapes a spirit weapon may be called in (the Hunter's Spirit weapon,
    /// `use:spirit_weapon:ID`): every melee weapon there is, by weapon skill then id. They are
    /// the game's objects, nothing of the run.
    pub fn weapon_shapes(&self) -> Vec<ObjectId> {
        crate::world::weapon_shapes(self.content)
    }

    /// What keeps the player from calling a familiar now (`world/allies/familiar.rs`): one alive,
    /// wherever it is (they know their own), or a hostile they see.
    pub(crate) fn calling_barred(&self) -> Option<crate::ability::Unavailable> {
        use crate::ability::Unavailable;
        let lives = (self.snapshot.allies.iter())
            .any(|a| matches!(a.source, crate::world::AllySource::Familiar(_)));
        let watched = (self.actors().iter())
            .any(|a| a.kind == ActorKind::Creature && self.stance(a) == Stance::Hostile);
        (lives.then_some(Unavailable::FamiliarLives)).or(watched.then_some(Unavailable::Watched))
    }

    /// The player's level, experience total, and the total at which the next level comes (none
    /// at the highest level) (spec §8 "Experience"). All of it is shown to the player.
    pub fn own_experience(&self) -> Progress {
        let rules = self.content.experience();
        let factor = Factor::of(self.content, self.character.map(|c| &c.spec));
        // The level stops at the classes' cap (spec §8 "Prestige classes"): no next level
        // there, as at the curve's top.
        let cap = (self.character).map_or(rules.class_max(), |c| c.level_cap(self.content));
        let level = rules.level_for(self.experience, factor, cap);
        Progress {
            level,
            xp: self.experience,
            next: (level < cap).then(|| rules.threshold_for(level + 1, factor)),
        }
    }

    /// The level whose choices the level-up screen spends now: the first due's, else the
    /// player's own (the class of that level sets the rolls and the bonus trees, spec §8
    /// "Prestige classes").
    pub(super) fn due_level(&self) -> u8 {
        (self.due()).map_or_else(|| self.own_experience().level, |d| d.level)
    }

    /// The choices a level still asks of the player (spec §8 "Talents"), if any: the level and
    /// how many talent picks are left. Nothing else may be done until they are made.
    pub fn due(&self) -> Option<Due> {
        self.progress.due.first().copied()
    }

    /// Every level's choices still due, the lowest level's first (several levels gained in one
    /// turn are one sequence on the level-up screen).
    pub fn dues(&self) -> &[Due] {
        &self.progress.due
    }

    /// The trees the bonus pick due now is of (the `bonus_talents` of the class of the level
    /// that brought it); `None`: any.
    pub fn bonus_trees(&self) -> Option<Vec<TreeId>> {
        let (table, _) = self.character?.class_of(self.content, self.due_level())?;
        table.bonus_trees(self.content.talents())
    }

    /// The player's race, when the run has a character.
    pub(super) fn own_race(&self) -> Option<&'a RaceDef> {
        self.character.and_then(|c| c.spec.race(self.content).ok())
    }

    /// What the player resists (spec §8 "Damage types"), in words only: each kind they resist,
    /// are vulnerable to, or are immune to, in damage type order; never a percentage, and never
    /// a creature's.
    pub fn own_resistances(&self) -> Vec<(DamageTypeId, crate::damage::ResistanceWord)> {
        (self
            .player
            .resistances(self.content, self.own_race())
            .iter())
        .filter_map(|(kind, percent)| Some((kind, crate::damage::ResistanceWord::of(percent)?)))
        .collect()
    }

    /// How the run ended, once the player has died (what killed them only if they saw it).
    pub fn death(&self) -> Option<&'a Death> {
        self.death
    }

    /// Light sources whose light reaches visible cells: static lights by position, then carried
    /// lights by actor id, then the sun, then the moons in content order.
    pub fn light_sources(&self) -> &'a [SeenLight] {
        &self.snapshot.sources
    }

    /// The player's eyes: their active vision profile as their talents change it (its id and
    /// name the base profile's).
    pub fn vision(&self) -> &'a VisionProfile {
        &self.snapshot.eye
    }

    /// Whether strong light dazzles the player (light-blind eyes in glare): they see less far,
    /// and fight worse. Only that it does, never how much.
    pub fn dazzled(&self) -> bool {
        self.snapshot.dazzled
    }

    /// Whether the player averts their eyes from a gaze they see (`world/creatures/avert.rs`): they fight
    /// half-blind. Only that they do, never how much.
    pub fn averted(&self) -> bool {
        self.snapshot.averted
    }

    /// The player's allies (spec §8 "Allies"), wherever they are, in actor id order.
    pub fn allies(&self) -> &[AllyView] {
        &self.snapshot.allies
    }

    /// The player's companion among their allies (`world/allies/companion.rs`), with its name and
    /// grade; `None` without one.
    pub fn companion(&self) -> Option<(&AllyView, &crate::world::CompanionData)> {
        (self.snapshot.allies.iter()).find_map(|a| match &a.source {
            crate::world::AllySource::Companion(data) => Some((a, data)),
            _ => None,
        })
    }

    /// The player's familiar among their allies (`world/allies/familiar.rs`), with its name and
    /// grade; `None` without one.
    pub fn familiar(&self) -> Option<(&AllyView, &crate::world::CompanionData)> {
        (self.snapshot.allies.iter()).find_map(|a| match &a.source {
            crate::world::AllySource::Familiar(data) => Some((a, data)),
            _ => None,
        })
    }

    /// The player's kept beasts (a companion, a familiar; `AllySource::kept`), with their names
    /// and grades, in actor id order: what the status bar shows a line of each.
    pub fn kept(&self) -> impl Iterator<Item = (&AllyView, &crate::world::CompanionData)> {
        (self.snapshot.allies.iter()).filter_map(|a| a.source.data().map(|d| (a, d)))
    }

    /// The ways to the player's kept beasts and escorts beside them, in direction order: with
    /// two or more, tending one or giving one something asks which way (spec §8 "Allies"; the
    /// owner, #907 Q4).
    pub fn kept_beside(&self) -> &[crate::geom::Dir] {
        &self.snapshot.kept_beside
    }

    /// Whether an escort of the player's stands beside them (`world/people/escort.rs`): the Give list
    /// holds anything it could put on.
    pub fn escort_beside(&self) -> bool {
        self.snapshot.escort_beside
    }

    /// Whether the escort of quest `quest` is with the player, on their level or in their
    /// retinue (`world/people/escort.rs`): the quest log's status.
    pub fn escorting(&self, quest: &str) -> bool {
        (self.snapshot.allies.iter()).any(|a| {
            (a.here || a.retinue)
                && matches!(&a.source, crate::world::AllySource::Escort { quest: q, .. } if q == quest)
        })
    }

    /// How many allies are with the player: on their level or in their retinue.
    pub fn allies_with(&self) -> usize {
        (self.snapshot.allies.iter())
            .filter(|a| a.here || a.retinue)
            .count()
    }

    /// The level's ambient light, as the player perceives it.
    pub fn ambient(&self) -> Light {
        self.snapshot.ambient
    }

    /// The sun and the sky over the player's level right now (they may be down); `None` on a
    /// level not under the open sky.
    pub fn sun(&self) -> Option<Sun> {
        self.snapshot.sky.as_ref().map(|s| s.sun)
    }

    /// The sun, the moons, and the sky over the player's level right now (they may be down),
    /// under the weather; `None` on a level not under the open sky.
    pub fn sky(&self) -> Option<&'a Sky> {
        self.snapshot.sky.as_ref()
    }

    /// The weather, which the player knows only under the open sky; `None` elsewhere.
    pub fn weather(&self) -> Option<Weather> {
        self.snapshot.weather()
    }

    /// The temperature of the air where the player is (spec §5 "Temperature"), which they feel,
    /// in dungeons and under the open sky alike, a hot or a cold room's included (the status bar
    /// shows it).
    pub fn temperature(&self) -> Temperature {
        self.snapshot.temperature
    }

    /// Steady light at a visible cell that no single source gives: ambient light plus sky light
    /// (under the open sky: the sun's and the moons') plus bounce light; `None` for cells not in
    /// view. With every [`SeenLight`] contribution to the cell added, it is
    /// [`PerceptionView::light`].
    pub fn base_light(&self, p: Pos) -> Option<Light> {
        self.snapshot.is_visible(p).then(|| self.snapshot.base[p])
    }

    /// Sounds the player heard during the last turn from sources they do not see (spec §5
    /// "Hearing"), in the order heard: the kind, how loud, and roughly where (a cell the player
    /// does not see) — never who or what made them. Sounds whose source the player sees are not
    /// here: the player sees what happens.
    pub fn sounds(&self) -> &'a [PerceivedSound] {
        &self.snapshot.sounds
    }

    /// The loudest sound perceived at `p`, if any.
    pub fn sound_at(&self, p: Pos) -> Option<&'a PerceivedSound> {
        self.snapshot
            .sounds
            .iter()
            .filter(|s| s.pos == p)
            .max_by_key(|s| s.volume)
    }
}
