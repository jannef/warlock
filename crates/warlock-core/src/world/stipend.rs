//! The aristocrat's stipend and servant (spec §8 "Class abilities": Stipend; #1405).
//!
//! A character with the Stipend class ability (`FortuneMods::stipend`, the aristocrat's from
//! level 1) has their family put something by at every level they reach, a prestige class's
//! too ([`World::grow_stipend`], one line in `experience.rs::rise_from`): the gold of
//! `data/stipend.toml` (`each` plus its dice, Landed noble's percent more), and one loot roll
//! at the level as item level with the file's luck (Landed noble's magic odds added), kept
//! when magic or rare or of a `keep` class. Each level draws from its own stream,
//! `stipend/<level>`, so nothing else moves. It accumulates in [`WorldState::stipend`] (saved
//! and hashed; left out while empty).
//!
//! A town's `servant` for `stipend` (`content/town.rs`) is placed as its level is made
//! ([`place_servant`], one line in `travel.rs::instantiate_level`), only in a run with the
//! ability: the creature type for the player's gender speaking the npc dialogue for their race.
//! The same placing serves every class helper keyed by its class ability (a merchant's business
//! partner, `invest.rs`; a craftsperson's patron, `patron.rs`, whose ability comes at level 12 and
//! who is placed as it is given when the town is made already, [`place_given`]). Chatting with
//! anyone
//! whose dialogue has `[dialogue.stipend]` words ([`World::servant_chat`], one line in
//! `talk.rs::chat`) hands over everything put by, with the `handover` line: the gold and the
//! items into the pack, each set down at the player's feet instead if it would leave them
//! Overloaded; with nothing put by, a one-liner. `{address}` in either becomes the word for
//! the player's gender. Messages call the servant "your squire" (`talk.rs::called`).
//!
//! If the servant dies, no one else brings the stipend: it still grows, and is never handed
//! over.

use super::{ActorId, Spent, World, WorldState};
use crate::action::Event;
use crate::content::{
    ActionKind, Content, CreatureSpawn, Dialogue, DialogueId, MAX_ITEM_LEVEL, Rarity, StatUse,
    TalentId, TownServant,
};
use crate::equipment::Equipment;
use crate::geom::Pos;
use crate::magic::loot::{self, LootGrade, Luck};
use crate::object::{self, Stack};
use crate::rng::Rng;
use crate::variant::Source;
use serde::{Deserialize, Serialize};

/// What the family has put by and the servant holds.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stipend {
    /// Gold coins.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub gold: u32,
    /// The finds kept, in the order they came.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<Stack>,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl Stipend {
    pub fn is_empty(&self) -> bool {
        self.gold == 0 && self.items.is_empty()
    }
}

/// What level `level` puts by for a character with `mods`: its gold, and its find if kept.
/// Drawn from the level's own stream.
pub(crate) fn of_level(
    content: &Content,
    seed: u64,
    level: u8,
    mods: &crate::talent::TalentMods,
) -> Option<(u32, Option<Stack>)> {
    let rules = content.stipend()?;
    let mut rng = Rng::stream(seed, &format!("stipend/{level}"));
    let rolled = u32::try_from(rules.dice.roll(&mut rng).max(0)).unwrap_or(0);
    let gold = u64::from(rules.each + rolled)
        * u64::try_from(100 + mods.fortune.stipend_gold_percent).unwrap_or(0)
        / 100;
    let luck = Luck {
        magic: rules.magic + mods.fortune.stipend_magic_percent,
        gems: rules.gems,
    };
    let item_level = level.clamp(1, MAX_ITEM_LEVEL);
    let chance = u32::try_from(i64::from(rules.chance) + i64::from(mods.fortune.stipend_chance))
        .unwrap_or(0)
        .min(100);
    let find = (rng.chance(chance))
        .then(|| {
            loot::roll_with(
                content,
                &mut rng,
                item_level,
                LootGrade::Any,
                Source::Loot,
                luck,
            )
        })
        .flatten();
    let kept = find.filter(|s| {
        s.rarity() != Rarity::Normal || rules.keep.contains(&content.object(s.object).class)
    });
    let kept = kept.map(|mut s| {
        s.draw_wand_rung(content, &mut rng);
        s
    });
    Some((u32::try_from(gold).unwrap_or(u32::MAX), kept))
}

/// Places every town's class helper that waits on level `level` (just made) whose class ability
/// the character holds: an aristocrat's servant, a merchant's business partner (`invest.rs`), a
/// craftsperson's patron (`patron.rs`). The module docs.
pub(super) fn place_servant(state: &mut WorldState, content: &Content, level: &str) {
    let helpers = content.towns().iter().flat_map(|t| &t.servants);
    for servant in helpers.filter(|s| s.level == level) {
        place(state, content, servant);
    }
}

/// Places the town helpers of class abilities `given`, just given (a patron's at level 12), whose
/// town is made already; a town made later places them as it is made ([`place_servant`]).
pub(super) fn place_given(state: &mut WorldState, content: &Content, given: &[TalentId]) {
    let book = content.talents();
    let helpers = content.towns().iter().flat_map(|t| &t.servants);
    for servant in helpers {
        let new = (book.id(&servant.talent)).is_some_and(|t| given.contains(&t));
        if new && state.levels.contains_key(&servant.level) {
            place(state, content, servant);
        }
    }
}

/// Places town helper `servant` on its level (which must be made) if the character holds its
/// class ability and no actor of the run is it yet.
fn place(state: &mut WorldState, content: &Content, servant: &TownServant) {
    let Some(character) = state.character.as_ref() else {
        return;
    };
    let (race, gender) = (character.spec.race.clone(), character.spec.gender);
    if !state.player().talents.has(&servant.talent) {
        return;
    }
    let npc = servant.npc_for(&race).to_string();
    if (state.actors.values()).any(|a| a.npc.as_deref() == Some(npc.as_str())) {
        return;
    }
    let Some(creature) = content.creature_id(servant.creature.of(gender)) else {
        return;
    };
    let spawn = CreatureSpawn {
        creature,
        faction: None,
        alignment: None,
        npc: Some(npc),
        pos: Pos::new(servant.at[0], servant.at[1]),
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Equipment::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    super::travel::make_creature(state, content, &servant.level, &spawn);
}

impl World {
    /// The player rose from level `before` to their level now: each level reached adds to the
    /// stipend, if they have the ability (the module docs).
    pub(super) fn grow_stipend(&mut self, before: u8, after: u8) {
        let content = self.content.clone();
        let mods = self.state.player().talent_mods(&content).into_owned();
        if !mods.fortune.stipend || content.stipend().is_none() {
            return;
        }
        for level in before + 1..=after {
            let Some((gold, find)) = of_level(&content, self.state.seed, level, &mods) else {
                return;
            };
            let stipend = &mut self.state.stipend;
            stipend.gold = stipend.gold.saturating_add(gold);
            if let Some(find) = find {
                object::add_to_pile(&mut stipend.items, &[find], &content);
            }
        }
        self.news
            .push("Word comes from home: your stipend has grown.".into());
    }

    /// The `progress` dump's row (dev-only): what the stipend holds, `stipend 412 gold: a
    /// potion of healing`; `None` while it holds nothing.
    pub fn stipend_row(&self) -> Option<String> {
        let stipend = &self.state.stipend;
        if stipend.is_empty() {
            return None;
        }
        let items = if stipend.items.is_empty() {
            "nothing else".to_string()
        } else {
            self.listed(&stipend.items)
        };
        Some(format!("stipend {} gold: {items}", stipend.gold))
    }

    /// The dialogue `d`'s words with `{address}` made the player's.
    fn addressed(&self, d: &Dialogue, line: &str) -> String {
        let (Some(words), Some(character)) = (&d.stipend, &self.state.character) else {
            return line.to_string();
        };
        line.replace("{address}", words.address.of(character.spec.gender))
    }

    /// Chats with creature `id`, who speaks `dialogue`, if it is a family servant: hands over
    /// the stipend, or says a one-liner (the module docs). `None` for anyone else.
    pub(super) fn servant_chat(
        &mut self,
        id: ActorId,
        dialogue: DialogueId,
        events: &mut Vec<Event>,
    ) -> Option<Spent> {
        let content = self.content.clone();
        let d = content.dialogue(dialogue);
        let words = d.stipend.as_ref()?;
        let price = self.price_of(self.state.player, ActionKind::Chat);
        if self.state.stipend.is_empty() {
            let said = self.state.actors[&id].talk.said;
            let line = self.one_liner(id, d, said);
            let line = self.addressed(d, &line);
            if let Some(actor) = self.state.actors.get_mut(&id) {
                actor.talk.said += 1;
                actor.talk.met = true;
            }
            self.answer(id, line, None, events);
        } else {
            let line = self.addressed(d, &words.handover);
            if let Some(actor) = self.state.actors.get_mut(&id) {
                actor.talk.met = true;
            }
            self.answer(id, line, None, events);
            self.hand_over_stipend();
        }
        self.train_stat(StatUse::Chat);
        Some(Spent::Turn(price))
    }

    /// Everything put by into the player's pack, or at their feet what would overload them.
    fn hand_over_stipend(&mut self) {
        let content = self.content.clone();
        let stipend = std::mem::take(&mut self.state.stipend);
        let mut all = Vec::new();
        if let Some(coin) = crate::chest::currency(&content).filter(|_| stipend.gold > 0) {
            all.push(Stack::new(coin, stipend.gold));
        }
        all.extend(stipend.items);
        self.receive(all);
    }

    /// `all` into the player's pack, or at their feet each stack that would overload them, and
    /// says so: a servant's handover, a partner's withdrawal (`invest.rs`).
    pub(super) fn receive(&mut self, all: Vec<Stack>) {
        let content = self.content.clone();
        let (mut taken, mut left) = (Vec::new(), Vec::new());
        for stack in all {
            let load = self.load_of(self.state.player);
            if load.carried + stack.weight(&content) > load.limit {
                left.push(stack);
            } else {
                object::add_to_inventory(self.state.inventory_mut(), stack.clone(), &content);
                taken.push(stack);
            }
        }
        if !taken.is_empty() {
            let what = self.listed(&taken);
            self.say(format!("You receive {what}."));
        }
        if !left.is_empty() {
            let here = self.player_pos();
            let level = self.state.current_mut();
            let mut pile = level.objects_at(here).to_vec();
            object::add_to_pile(&mut pile, &left, &content);
            level.set_pile(here, pile);
            let what = self.listed(&left);
            self.say(format!("Too much to carry, set down at your feet: {what}."));
        }
    }

    /// Every stack of `stacks` named, however many: "412 gold coins, a potion, and a ruby".
    pub(super) fn listed(&self, stacks: &[Stack]) -> String {
        let names: Vec<String> = stacks.iter().map(|s| self.lore().describe(s)).collect();
        if names.is_empty() {
            return "nothing".into();
        }
        crate::text::and_list(&names)
    }
}
