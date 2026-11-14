//! Splitting and dying into (docs/design/monster-dread.md §5.5, G5a and G5b; #1129): a thing
//! cut in two is two things, and what lived in a corpse crawls out of it. The data is
//! `content/split.rs`.
//!
//! - **Splitting** ([`World::split`], one line in `fight.rs::resolve`, after a blow that left
//!   its target alive): a blow of a kind its type's `splits` lists (the sword's slashing and
//!   piercing; never the mace's bludgeoning, fire, or acid) that leaves it with at least
//!   `2 × least` hit points halves them: it keeps the greater half, and a second creature of
//!   its type (`travel::make_creature`, no gear, the spawner's cap ignored) takes the other on
//!   the nearest free cell beside it ([`World::summon_cells`] at 1; none free: no split). The
//!   new half shares its side, its summoner, its master if it is a summoned ally (never a kept
//!   beast's place), and its mind (it knows what the first knew). No draw: the placement is
//!   the cells' order.
//! - **Dying into** ([`World::dies_into`], one line in `fight.rs::dies`, before the creature
//!   leaves the world): its type's `dies_into` count is rolled on the gameplay stream, each
//!   one drawn from the pool (`below(pool.len())`) and made on the next of the free cells within
//!   `spread` of the corpse, of the dead one's faction. Only on the player's level.
//! - **The credit** ([`World::offspring_share`]): what was made so remembers it
//!   ([`crate::world::Actor::split_from`]): a half's kill pays a quarter of its type's experience, a
//!   creature that crawled out of a corpse a tenth, and neither counts toward a quest's kills,
//!   so no quest counts the halves twice and a pudding farm is a poor one.
//! - **Words**: the first split of a type the player sees in a run says what to do about it
//!   ("The charnel heap comes apart, and both halves crawl."; remembered in
//!   `Progress::once` as `split:<type>`), later ones "The charnel heap splits in two."; what
//!   crawls out where the player sees it, "3 grave worms crawl out of the wormhost!". Unseen,
//!   nothing. Said with the news, after the blow's own words.

use crate::content::{CORPSE_SHARE, CreatureId, CreatureSpawn, HALF_SHARE, counted};
use crate::content::{the, the_start};
use crate::text::{and_list, capitalized};
use crate::world::{ActorId, World};
use serde::{Deserialize, Serialize};

/// What a creature was made of (docs/design/monster-dread.md §5.5), for its kill's credit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Split {
    /// A half cut off a creature of its type: its kill pays a quarter.
    Half,
    /// It crawled out of a corpse: its kill pays a tenth.
    Corpse,
}

impl World {
    /// Creature `id`, struck by a blow of damage kind `kind` that left it alive, splits if its
    /// type's `splits` say so (module docs).
    pub(in crate::world) fn split(
        &mut self,
        id: ActorId,
        kind: Option<crate::content::DamageTypeId>,
    ) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get(&id) else {
            return;
        };
        let def = content.creature(me.creature);
        let Some(splits) = def.splits.as_ref() else {
            return;
        };
        if !kind.is_some_and(|k| splits.on.contains(&k)) || me.hp < 2 * splits.least {
            return;
        }
        if me.level != self.state.current_level {
            return;
        }
        let Some(&at) = self.summon_cells(id, 1).first() else {
            return;
        };
        let (creature, pos, hp) = (me.creature, me.pos, me.hp);
        let (mind, faction, summoner) = (me.mind, me.faction_override, me.summoner);
        // An ally's half is an ally too; a kept beast is one of its kind and never two.
        let ally = (me.ally.clone()).filter(|a| !a.source.kept());
        let spawn = CreatureSpawn {
            creature,
            faction,
            alignment: None,
            npc: None,
            pos: at,
            route: Vec::new(),
            carry: Vec::new(),
            equipment: Default::default(),
            rolls_gear: false,
            asleep: false,
            status: Vec::new(),
        };
        let level = self.state.current_level.clone();
        let made =
            crate::world::levels::travel::make_creature(&mut self.state, &content, &level, &spawn);
        let half = hp / 2;
        if let Some(new) = self.state.actors.get_mut(&made) {
            new.hp = half;
            new.mind = mind;
            new.summoner = summoner;
            new.ally = ally;
            new.split_from = Some(Split::Half);
        }
        if let Some(me) = self.state.actors.get_mut(&id) {
            me.hp = hp - half;
        }
        self.spawned += 1;
        if !self.perception.sees_actors_at(pos) && !self.perception.sees_actors_at(at) {
            return;
        }
        let yours =
            (self.state.actors.get(&id)).is_some_and(|a| a.master() == Some(self.state.player));
        let name = match yours {
            true => format!("Your {}", def.name),
            false => the_start(&def.name),
        };
        let first = self.state.progress.once.insert(format!("split:{}", def.id));
        self.news.push(if first {
            format!("{name} comes apart, and both halves crawl.")
        } else {
            format!("{name} splits in two.")
        });
    }

    /// Creature `id`, dying, lets out what its type's `dies_into` says (module docs). Returns
    /// what the player is told, said with its death.
    pub(in crate::world) fn dies_into(&mut self, id: ActorId) -> Option<String> {
        let content = self.content.clone();
        let me = self.state.actors.get(&id)?;
        let def = content.creature(me.creature);
        let into = def.dies_into.as_ref()?;
        if me.level != self.state.current_level {
            return None;
        }
        let (pos, faction) = (me.pos, me.faction(&content));
        let cells = self.summon_cells(id, into.spread);
        let rolled = into.count.roll(&mut self.state.rng);
        let count = usize::try_from(rolled.max(1)).unwrap_or(1);
        let level = self.state.current_level.clone();
        let mut seen: Vec<(CreatureId, u32)> = Vec::new();
        for &at in cells.iter().take(count) {
            let len = u32::try_from(into.pool.len()).unwrap_or(u32::MAX);
            let creature = into.pool[self.state.rng.below(len) as usize];
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
                rolls_gear: false,
                asleep: false,
                status: Vec::new(),
            };
            let made = crate::world::levels::travel::make_creature(
                &mut self.state,
                &content,
                &level,
                &spawn,
            );
            if let Some(actor) = self.state.actors.get_mut(&made) {
                actor.split_from = Some(Split::Corpse);
            }
            self.spawned += 1;
            if self.perception.sees_actors_at(at) {
                match seen.iter_mut().find(|(c, _)| *c == creature) {
                    Some((_, n)) => *n += 1,
                    None => seen.push((creature, 1)),
                }
            }
        }
        if seen.is_empty() || !self.perception.sees_actors_at(pos) {
            return None;
        }
        let words: Vec<String> = (seen.iter())
            .map(|&(c, n)| counted(&content.creature(c).name, &content.creature(c).plural, n))
            .collect();
        let list = and_list(&words.iter().map(String::as_str).collect::<Vec<_>>());
        let one = seen.len() == 1 && seen[0].1 == 1;
        let verb = if one { "crawls" } else { "crawl" };
        Some(format!(
            "{} {verb} out of {}!",
            capitalized(&list),
            the(&def.name)
        ))
    }

    /// The share of a kill of actor `id` its master earns, `percent` before what it was made
    /// of (module docs), and whether the kill counts toward a quest's.
    pub(in crate::world) fn offspring_share(&self, id: ActorId, percent: u8) -> (u8, bool) {
        let made = self.state.actors.get(&id).and_then(|a| a.split_from);
        let share = match made {
            None => return (percent, true),
            Some(Split::Half) => HALF_SHARE,
            Some(Split::Corpse) => CORPSE_SHARE,
        };
        let percent = u16::from(percent) * u16::from(share) / 100;
        (u8::try_from(percent).unwrap_or(u8::MAX), false)
    }
}
