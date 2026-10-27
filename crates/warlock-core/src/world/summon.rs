//! Summoning (spec §8 "Summoning", #520): a creature with `[creature.summon]` in its type
//! (`content/summon.rs`) calls forth creatures of its side. Enemy-only: no player ability.
//!
//! **When** ([`World::summons_at`], asked first in `ai.rs::fight`): it sees a hostile within its
//! summoning's `range`, its casting is not blocked (Silenced), its cooldown status is off, it has
//! the mana, fewer than `most` of those it summoned live, and a free cell lies within `spread`.
//! Deciding draws nothing.
//!
//! **What** ([`World::creature_summons`]): it pays the mana (`World::spend_mana`), takes the
//! cooldown status, makes a spell's noise, and rolls `count` on the gameplay stream; then each
//! creature is one of the pool (`below(pool.len())`), made (`travel::make_creature`, its gear
//! rolled as a spawn's) on the next of the free cells ([`World::summon_cells`]). Fewer cells than
//! the roll, or than `most` leaves room for: fewer come. The summoned take the summoner's faction and remember who called them
//! ([`super::Actor::summoner`]); a kill of one pays its experience like any other. The action
//! costs a `cast`.
//!
//! **Words**: the player who sees the summoner is told what it calls forth that they see
//! ("The goblin wizard calls forth 2 goblins and a warg!", "something" for none); unseen, only
//! what appears where they see it ("A skeleton appears!").

use super::{ActorId, World};
use crate::action::{AttackKind, Event};
use crate::content::{ActionKind, CreatureId, CreatureSpawn, Summon, and_list, counted, the_start};
use crate::geom::Pos;
use crate::noise::Act;
use crate::sight::LineOfSight;

impl World {
    /// Whether creature `id` summons now, at a hostile it sees at `at` (module docs).
    pub(super) fn summons_at(&self, id: ActorId, at: Pos) -> bool {
        let me = &self.state.actors[&id];
        let Some(summon) = self.content.creature(me.creature).summon.as_ref() else {
            return false;
        };
        me.pos.chebyshev(at) <= summon.range
            && !me.effects.has(summon.status)
            && me
                .effects
                .blocking(&self.content, ActionKind::Cast)
                .is_none()
            && me.mana >= me.mana_cost(&self.content, summon.mana)
            && self.live_summons(id) < usize::from(summon.most)
            && !self.summon_cells(id, summon).is_empty()
    }

    /// How many creatures summoned by `id` live.
    pub(super) fn live_summons(&self, id: ActorId) -> usize {
        (self.state.actors.values())
            .filter(|a| a.summoner == Some(id))
            .count()
    }

    /// Where creature `id`'s summoned may appear: the cells within its `spread` (Chebyshev) it has
    /// a line of sight to, walkable, with no one there, and not a sanctuary it would shun; nearest
    /// first (Chebyshev distance, then squared distance, then row order).
    fn summon_cells(&self, id: ActorId, summon: &Summon) -> Vec<Pos> {
        let me = &self.state.actors[&id];
        let level = self.state.current();
        let spread = summon.spread;
        let radius = LineOfSight::radius_for(2 * spread * spread);
        let los = LineOfSight::compute(me.pos, radius, |p| level.is_opaque(&self.content, p));
        let shuns = self.shuns(id);
        let mut cells: Vec<Pos> = (-spread..=spread)
            .flat_map(|dy| (-spread..=spread).map(move |dx| (dx, dy)))
            .map(|(dx, dy)| Pos::new(me.pos.x + dx, me.pos.y + dy))
            .filter(|&p| p != me.pos && los.contains(p))
            .filter(|&p| level.is_walkable(&self.content, p))
            .filter(|&p| self.state.actor_at(&level.id, p).is_none() && !shuns(p))
            .collect();
        cells.sort_by_key(|p| (p.chebyshev(me.pos), p.dist2(me.pos), p.y, p.x));
        cells
    }

    /// Creature `id` summons (its AI saw to it that it can). Returns the price: a `cast`.
    pub(super) fn creature_summons(&mut self, id: ActorId, events: &mut Vec<Event>) -> i64 {
        let content = self.content.clone();
        let me = &self.state.actors[&id];
        let price = self.price_of(id, ActionKind::Cast);
        let Some(summon) = content.creature(me.creature).summon.as_ref() else {
            return price;
        };
        let cells = self.summon_cells(id, summon);
        if !self.spend_mana(id, summon.mana) {
            return price;
        }
        self.afflict(id, summon.status, summon.degrees, None, events);
        self.make_noise(id, Act::Attack(AttackKind::Spell));
        let me = &self.state.actors[&id];
        let (pos, faction) = (me.pos, me.faction(&content));
        let rolled = summon.count.roll(&mut self.state.rng);
        let count = usize::try_from(rolled.max(1)).unwrap_or(1);
        // Never more live at once than `most`.
        let room = usize::from(summon.most).saturating_sub(self.live_summons(id));
        let count = count.min(room);
        let level = self.state.current_level.clone();
        let mut came: Vec<(CreatureId, Pos)> = Vec::new();
        for &at in cells.iter().take(count) {
            let len = u32::try_from(summon.pool.len()).unwrap_or(u32::MAX);
            let creature = summon.pool[self.state.rng.below(len) as usize];
            let own = content.creature(creature).faction;
            let spawn = CreatureSpawn {
                creature,
                faction: faction.filter(|&f| Some(f) != own),
                alignment: None,
                npc: None,
                pos: at,
                route: Vec::new(),
                carry: Vec::new(),
                equipment: Default::default(),
                rolls_gear: true,
                asleep: false,
            };
            let made = super::travel::make_creature(&mut self.state, &content, &level, &spawn);
            if let Some(actor) = self.state.actors.get_mut(&made) {
                actor.summoner = Some(id);
            }
            self.spawned += 1;
            came.push((creature, at));
        }
        self.say_summoning(id, pos, &came);
        price
    }

    /// What the player is told of creature `id`'s summoning at `pos` that brought `came`
    /// (module docs).
    fn say_summoning(&mut self, id: ActorId, pos: Pos, came: &[(CreatureId, Pos)]) {
        let content = self.content.clone();
        // What the player sees appear, by type in the order they came.
        let mut seen: Vec<(CreatureId, u32)> = Vec::new();
        for &(c, at) in came {
            if !self.perception.sees_actors_at(at) {
                continue;
            }
            match seen.iter_mut().find(|(s, _)| *s == c) {
                Some((_, n)) => *n += 1,
                None => seen.push((c, 1)),
            }
        }
        let words: Vec<String> = (seen.iter())
            .map(|&(c, n)| {
                let def = content.creature(c);
                counted(&def.name, &def.plural, n)
            })
            .collect();
        let list = and_list(&words.iter().map(String::as_str).collect::<Vec<_>>());
        let summoner = &self.state.actors[&id];
        let def = content.creature(summoner.creature);
        let line = if self.perception.sees_actors_at(pos) {
            let verb = def.summon.as_ref().map_or("summons", |s| s.verb.as_str());
            let what = if list.is_empty() { "something" } else { &list };
            format!("{} {verb} {what}!", the_start(&def.name))
        } else if list.is_empty() {
            return;
        } else {
            let one = seen.len() == 1 && seen[0].1 == 1;
            let verb = if one { "appears" } else { "appear" };
            format!("{} {verb}!", capital(&list))
        };
        self.say(line);
    }
}

/// `text` with its first letter upper-case.
fn capital(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
