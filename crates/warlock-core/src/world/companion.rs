//! The animal companion (spec §8 "Allies": "The animal companion"; docs/design/ranger-companion.md
//! §5–§8; `docs/dev/companion.md`): a beast that is the player's for good. It is an ally of the
//! core (`ally.rs`: [`Actor::ally`], the side, the arm, the orders, the retinue) with
//! `source = Companion(CompanionData)`, `until = None`, and four things of its own:
//!
//! - **how it is gained**: Animal companion (the ranger's level 1, `TalentMods::companion`)
//!   brings one as the run begins ([`World::bring_companion`]): a beast drawn from the
//!   character's race's list in `data/companions.toml`, or at `rare_per_mille` from the rares,
//!   on the stream `companion/<race>,<gender>,<class>` (two draws); its name on
//!   `companion/name/<actor id>` (one draw). Nothing else moves. A run restored makes none;
//!   with a companion already, none; the design's Tame (C2) is the other way to one;
//! - **growth** ([`CompanionData::grade`]): its level is its master's ([`World::grow_companion`]
//!   at every level-up, "Snel grows stronger."), never below its type's own; the hooks that
//!   read the grade above the type's level sit in `actor.rs` (hit points, the blow, Defence,
//!   Protection, the stats; `data/companions.toml [growth]`);
//! - **it always comes along** (`ally.rs::follow_along`): wherever it stands, held fast or not;
//! - **its nose** ([`World::smelled`]): a type with `nose` smells a hostile of its master's
//!   within that many cells (Chebyshev), however dark and through a closed door (the owner, §20),
//!   when its eyes find none; "Snel growls at the dark." once per thing scented unseen.
//!
//! Its name is said instead of "your hound" wherever an ally is named ([`World::called`],
//! `fight::End::Named`, `ally.rs::pack_phrase`). It dies as any creature does, leaving a corpse.

use super::{Actor, ActorId, AllySource, LevelState, World, WorldState};
use crate::character::CharacterSpec;
use crate::combat::Profile;
use crate::content::{CompanionGrowth, Content, CreatureId, LevelKind};
use crate::geom::Pos;
use crate::rng::Rng;
use crate::sight::LineOfSight;
use crate::stats::Stat;
use serde::{Deserialize, Serialize};

/// What a companion is beyond an ally ([`AllySource::Companion`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanionData {
    /// Its name ("Snel"), said without an article.
    pub name: String,
    /// Its own level: its master's (`[growth] grade_percent` of it), never below its type's.
    pub grade: u8,
    /// What its nose is on: the hostile it last scented without seeing, so the growl is said
    /// once per thing scented.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scent: Option<ActorId>,
    /// What Sic 'em set it on, while that hunt lasts (`sic.rs`): an order clears it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quarry: Option<ActorId>,
}

impl Actor {
    /// Its companion data, if it is someone's companion (not a familiar: the ranger's own
    /// rules read this).
    pub fn companion(&self) -> Option<&CompanionData> {
        match &self.ally.as_ref()?.source {
            AllySource::Companion(data) => Some(data),
            AllySource::Summoned { .. }
            | AllySource::Familiar(_)
            | AllySource::Charmed { .. }
            | AllySource::Escort { .. } => None,
        }
    }

    pub(super) fn companion_mut(&mut self) -> Option<&mut CompanionData> {
        match &mut self.ally.as_mut()?.source {
            AllySource::Companion(data) => Some(data),
            AllySource::Summoned { .. }
            | AllySource::Familiar(_)
            | AllySource::Charmed { .. }
            | AllySource::Escort { .. } => None,
        }
    }

    /// Its data, if it is someone's **kept** beast, a companion or a familiar
    /// ([`AllySource::kept`]): what the shared rules read (the name, growth, always along).
    pub fn kept(&self) -> Option<&CompanionData> {
        self.ally.as_ref()?.source.data()
    }

    pub(super) fn kept_mut(&mut self) -> Option<&mut CompanionData> {
        self.ally.as_mut()?.source.data_mut()
    }

    /// How far its grade stands above its type's level (`[growth]`'s `above`), 0 for anyone
    /// that is not a kept beast.
    pub fn grade_above(&self, content: &Content) -> i32 {
        match self.kept() {
            Some(data) => CompanionGrowth::above(data.grade, content.creature(self.creature).level),
            None => 0,
        }
    }
}

/// The starting beast a character of `spec` draws in the run of `seed`: at `rare_per_mille` one
/// of the rares, else one of the race's list (`None` without the data or a list for the race).
/// Two draws on `companion/<spec>`, and nothing else.
pub fn draw(content: &Content, seed: u64, spec: &CharacterSpec) -> Option<CreatureId> {
    let rules = content.companions()?;
    let list = rules.list_of(&spec.race)?;
    let mut rng = Rng::stream(seed, &format!("companion/{spec}"));
    let rare = rng.below(1000) < rules.rare_per_mille;
    let pool = if rare { rules.rare.as_slice() } else { list };
    let at = rng.below(u32::try_from(pool.len()).unwrap_or(u32::MAX));
    pool.get(at as usize).copied()
}

/// The name of the companion that is actor `id` in the run of `seed`: one draw on
/// `companion/name/<id>`.
pub fn name_for(content: &Content, seed: u64, id: ActorId) -> String {
    let Some(rules) = content.companions() else {
        return String::new();
    };
    let mut rng = Rng::stream(seed, &format!("companion/name/{}", id.0));
    let at = rng.below(u32::try_from(rules.names.len()).unwrap_or(u32::MAX));
    rules.names.get(at as usize).cloned().unwrap_or_default()
}

impl World {
    /// The player's companion, if they have one: their one ally of source `Companion`, on any
    /// level or in the retinue.
    pub fn companion(&self) -> Option<ActorId> {
        (self.allies_of(self.state.player).into_iter())
            .find(|id| self.state.actors[id].companion().is_some())
    }

    /// A kept beast's name and type, as the words say them: `("Snel", "hound")`.
    pub(super) fn companion_words(&self, id: ActorId) -> Option<(String, String)> {
        let actor = self.state.actors.get(&id)?;
        let data = actor.kept()?;
        Some((
            data.name.clone(),
            self.content.creature(actor.creature).name.clone(),
        ))
    }

    /// Ally `id` as the subject of a sentence: its name ("Snel"), else "Your wolf".
    pub(super) fn ally_subject(&self, id: ActorId) -> String {
        if let Some(escort) = (self.state.actors.get(&id)).filter(|a| a.escorting().is_some()) {
            return super::talk::capitalized(&self.called(escort));
        }
        match self.companion_words(id) {
            Some((name, _)) => name,
            None => format!(
                "Your {}",
                self.content.creature(self.state.actors[&id].creature).name
            ),
        }
    }

    /// Animal companion's beast as the run begins (the module docs): drawn, made beside the
    /// player (into the retinue on the overworld), named, and said. Nothing without the talent,
    /// the data, the ally core, a character, or with a companion already.
    pub(super) fn bring_companion(&mut self) {
        let content = self.content.clone();
        if self.companion().is_some() || content.allies().is_none() {
            return;
        }
        let Some(character) = self.state.character.clone() else {
            return;
        };
        if !self.state.player().talent_mods(&content).beast.companion {
            return;
        }
        let Some(rules) = content.companions() else {
            return;
        };
        let Some(creature) = draw(&content, self.state.seed, &character.spec) else {
            return;
        };
        let level = self.state.current_level.clone();
        let overworld = self.state.current().kind == LevelKind::Overworld;
        let at = match overworld {
            true => self.player_pos(),
            false => self.free_cell_near(&level, self.player_pos()),
        };
        let grade = (rules.growth).grade(self.player_level(), content.creature(creature).level);
        let data = CompanionData {
            name: String::new(),
            grade,
            scent: None,
            quarry: None,
        };
        let id = self.make_ally(creature, at, AllySource::Companion(data), None);
        let name = name_for(&content, self.state.seed, id);
        if let Some(actor) = self.state.actors.get_mut(&id) {
            if let Some(data) = actor.kept_mut() {
                data.name = name.clone();
            }
            // Its grade raised its maximum: it starts whole.
            actor.hp = actor.max_hp(&content);
            if overworld && let Some(ally) = actor.ally.as_mut() {
                ally.retinue = true;
            }
        }
        let kind = &content.creature(creature).name;
        self.say(format!("{name}, your {kind}, pads at your heel."));
    }

    /// Creature `id` becomes the player's companion where it stands (Tame, `tame.rs`): it keeps
    /// its actor (hit points, statuses, its place), takes the player's side (their faction as
    /// its `faction_override`, as [`World::make_ally`] gives a made one), forgets what it was
    /// about (`Mind` and its patrol cleared), and is graded at the player's level (its hit
    /// points rising with its maximum) and named (one draw on `companion/name/<id>`). Returns its name; `None`
    /// without the data or the actor.
    pub(super) fn adopt(&mut self, id: ActorId) -> Option<String> {
        let content = self.content.clone();
        let rules = content.companions()?;
        let player = self.state.player;
        let faction = self.state.player().faction(&content);
        let level = self.player_level();
        let name = name_for(&content, self.state.seed, id);
        let actor = self.state.actors.get_mut(&id)?;
        let before = actor.max_hp(&content);
        let grade = (rules.growth).grade(level, content.creature(actor.creature).level);
        actor.faction_override = faction;
        actor.mind = super::Mind::default();
        actor.patrol = None;
        actor.ally = Some(super::Ally {
            master: player,
            source: AllySource::Companion(CompanionData {
                name: name.clone(),
                grade,
                scent: None,
                quarry: None,
            }),
            order: super::Order::Follow,
            retinue: false,
            until: None,
        });
        let after = actor.max_hp(&content);
        actor.hp += (after - before).max(0);
        Some(name)
    }

    /// The player's level changed: the grade of each kept beast of theirs (a companion, a
    /// familiar) follows it (never down), its hit points rising with its maximum; "Snel grows
    /// stronger." among the level's news, in actor id order.
    pub(super) fn grow_companion(&mut self) {
        let content = self.content.clone();
        let Some(rules) = content.companions() else {
            return;
        };
        let level = self.player_level();
        for id in self.allies_of(self.state.player) {
            let Some(actor) = self.state.actors.get_mut(&id) else {
                continue;
            };
            let own = content.creature(actor.creature).level;
            let grade = rules.growth.grade(level, own);
            let before = actor.max_hp(&content);
            let Some(data) = actor.kept_mut().filter(|d| grade > d.grade) else {
                continue;
            };
            data.grade = grade;
            let name = data.name.clone();
            let after = actor.max_hp(&content);
            actor.hp += after - before;
            self.news.push(format!("{name} grows stronger."));
        }
    }

    /// The nearest actor ally `id`'s nose finds (the module docs): one `wanted` within its
    /// type's `nose` (Chebyshev) that a smell reaches (its line of sight with closed doors open,
    /// whatever the light); ties by lower id. `None` for a type without a nose.
    pub(super) fn smelled(
        &mut self,
        id: ActorId,
        wanted: &dyn Fn(&WorldState, &Content, &Actor, &Actor) -> bool,
    ) -> Option<(ActorId, Pos)> {
        let me = &self.state.actors[&id];
        smelt(&self.state, &self.content, me, wanted)
            .into_iter()
            .next()
    }

    /// Ally `id` scented `target` without seeing it (`ally.rs::ally_plan`): a companion growls,
    /// "Snel growls at the dark.", when the player sees it and not what it smells, once per
    /// thing scented ([`CompanionData::scent`] remembers; a summon with a nose hunts in silence).
    pub(super) fn scented(&mut self, id: ActorId, target: ActorId, at: Pos) {
        let Some(me) = self.state.actors.get(&id) else {
            return;
        };
        let pos = me.pos;
        let Some(data) = me.kept() else {
            return;
        };
        let fresh = data.scent != Some(target);
        if let Some(data) = (self.state.actors.get_mut(&id)).and_then(Actor::kept_mut) {
            data.scent = Some(target);
        }
        if fresh && self.perception.sees_actors_at(pos) && !self.perception.sees_actors_at(at) {
            let who = self.ally_subject(id);
            self.say(format!("{who} growls at the dark."));
        }
    }
}

/// Every actor `me`'s nose finds, nearest first (ties by lower id): each `wanted` one on its
/// level, not in the retinue nor disguised (a mimic smells of nothing), within its type's `nose`
/// (Chebyshev) and reached by a smell (a line of sight from `me` with closed doors counted open,
/// whatever the light; never through a wall). Empty for a type without a nose. The ally's arm
/// takes the first ([`World::smelled`]); Hunter's bond shows the player them all
/// (`Perception::compute`). Pure: nothing drawn, nothing changed.
pub(crate) fn smelt(
    state: &WorldState,
    content: &Content,
    me: &Actor,
    wanted: &dyn Fn(&WorldState, &Content, &Actor, &Actor) -> bool,
) -> Vec<(ActorId, Pos)> {
    match content.creature(me.creature).nose {
        Some(nose) => smelt_within(state, content, me, nose, wanted),
        None => Vec::new(),
    }
}

/// [`smelt`] with a reach of `nose` cells, whatever `me`'s type has (a familiar's for Familiar's
/// sight, `Perception::compute`).
pub(crate) fn smelt_within(
    state: &WorldState,
    content: &Content,
    me: &Actor,
    nose: i32,
    wanted: &dyn Fn(&WorldState, &Content, &Actor, &Actor) -> bool,
) -> Vec<(ActorId, Pos)> {
    let mut near: Vec<(i32, ActorId, Pos)> = (state.actors.values())
        .filter(|a| a.level == me.level && a.id != me.id && !a.in_retinue())
        .filter(|a| a.pos.chebyshev(me.pos) <= nose)
        .filter(|a| !a.effects.hides(content) && wanted(state, content, me, a))
        .map(|a| (a.pos.dist2(me.pos), a.id, a.pos))
        .collect();
    if near.is_empty() {
        return Vec::new();
    }
    near.sort_unstable();
    let level: &LevelState = state.current();
    // A smell passes a closed door; the radius covers the Chebyshev square's corners.
    let los = LineOfSight::compute(me.pos, nose * 3 / 2 + 1, |q| {
        level.is_opaque(content, q) && !level.is_closed_door(content, q)
    });
    (near.into_iter())
        .filter(|&(_, _, p)| los.contains(p))
        .map(|(_, other, p)| (other, p))
        .collect()
}

/// The growth hooks (`data/companions.toml [growth]`; the module docs): what a companion's grade
/// above its type's level adds, read by `actor.rs` beside the type's own numbers. Nothing for
/// anyone that is not a companion, or without the data.
impl Actor {
    fn growth(&self, content: &Content) -> Option<(CompanionGrowth, i32)> {
        let above = self.grade_above(content);
        if above <= 0 {
            return None;
        }
        Some((content.companions()?.growth, above))
    }

    /// Hit points above the type's: `hp_per_level` a level ([`Actor::max_hp`]).
    pub(super) fn growth_hp(&self, content: &Content) -> i32 {
        self.growth(content)
            .map_or(0, |(g, above)| g.hp_per_level * above)
    }

    /// What the grade adds to `stat`: Strength, Dexterity, and Toughness +1 per `stat_every`.
    pub(super) fn growth_stat(&self, content: &Content, stat: Stat) -> i32 {
        match stat {
            Stat::Strength | Stat::Dexterity | Stat::Toughness => self
                .growth(content)
                .map_or(0, |(g, above)| above / g.stat_every),
            _ => 0,
        }
    }

    /// Natural Defence the grade adds: +1 per `defence_every` ([`Actor::defence`]).
    pub(super) fn growth_defence(&self, content: &Content) -> i32 {
        self.growth(content)
            .map_or(0, |(g, above)| above / g.defence_every)
    }

    /// The natural Protection of a companion at its grade: its body's by the rule of spec §8
    /// "Creatures" (a hide (grade − 10) ÷ 8, a bony body grade ÷ 6, an armoured one grade ÷ 4 +
    /// 1, by what its hide is called; a body with none, none), for the type's own to be taken
    /// over when it is more ([`Actor::protection_plus`]); 0 for anyone else.
    pub(super) fn growth_protection(&self, content: &Content) -> i32 {
        let Some(data) = self.kept() else {
            return 0;
        };
        let hide = (content.creature(self.creature).natural.as_ref()).map(|h| h.name.as_str());
        body_protection(hide, data.grade)
    }

    /// `profile` with what the grade adds to a companion's blow: +1 to hit per `to_hit_every`,
    /// +1 damage per `damage_bonus_every`, one more die per `damage_dice_every`
    /// ([`Actor::melee`]).
    pub(super) fn grown(&self, content: &Content, mut profile: Profile) -> Profile {
        if let Some((g, above)) = self.growth(content) {
            profile.to_hit += above / g.to_hit_every;
            profile.bonus += above / g.damage_bonus_every;
            let dice = above / g.damage_dice_every;
            if dice > 0 {
                profile.damage = profile.damage.more(u32::try_from(dice).unwrap_or(0));
            }
        }
        profile
    }
}

/// The body rule's Protection at level `grade` for a creature whose hide is called `hide`
/// (`None`: a plain hide): hide (L − 10) ÷ 8, bony L ÷ 6, armoured L ÷ 4 + 1, none 0 (spec §8
/// "Creatures"; `content/creature_body_tests.rs` pins the types to it).
pub fn body_protection(hide: Option<&str>, grade: u8) -> i32 {
    const NONE: &[&str] = &[
        "insubstantial",
        "wind",
        "crackling",
        "flame",
        "flickering",
        "flowing",
        "fey",
        "mud",
    ];
    const BONY: &[&str] = &[
        "bone", "skull", "scales", "chitin", "shell", "carapace", "plates",
    ];
    const ARMOURED: &[&str] = &[
        "stone",
        "stony",
        "iron-hard",
        "frost-hard",
        "ice",
        "slag",
        "cinder",
    ];
    let name = hide.unwrap_or("hide");
    let has = |words: &[&str]| words.iter().any(|w| name.contains(w));
    let level = i32::from(grade);
    if has(NONE) {
        0
    } else if has(ARMOURED) {
        level / 4 + 1
    } else if has(BONY) {
        level / 6
    } else {
        ((level - 10) / 8).max(0)
    }
}

#[cfg(test)]
#[path = "companion_tests.rs"]
mod tests;
