//! A miscast's rank loss and forgetting (spec §8 "Spells": "Miscasts", the miscasts design
//! §5.1, #1289): what the `rank_loss` and `forget` kinds of `data/miscasts/` do to the caster.
//!
//! - **Rank loss**: the spell cast is known `ranks` ranks lower, never below 1
//!   ([`Actor::set_spell_rank`]); a spell at rank 1 (a class's never deepened among them) loses
//!   nothing more. The grimoire's deepening (`learn.rs`) wins the ranks back.
//! - **Forgetting**: the spell cast is forgotten ([`Actor::forget_spell`]), a class's spell as
//!   much as a learned one; `spells = N` forgets `N − 1` more known spells, each drawn at random
//!   among those left in menu order ([`Actor::known_spells`]); `all` forgets every spell known.
//!   A forgotten spell is learned again only from its grimoire, as the first time, experience
//!   and all (no guard: the owner).
//! - **A creature's** do nothing: its spells are its type's list, never in [`Actor::spells`].
//! - **The words**: the entry's own line says how it felt; the player is then told what was
//!   lost by name ("You know Fireball less well than you did.", "You have forgotten Fireball and
//!   Haste."), nothing more when nothing was.
//!
//! Determinism: one gameplay draw for each further spell forgotten, none otherwise.

use crate::content::SpellId;
use crate::text::and_list;
use crate::world::{ActorId, World};

impl World {
    /// `caster` knows `spell` `ranks` ranks lower, never below 1 (module docs).
    pub(super) fn miscast_rank_loss(&mut self, caster: ActorId, spell: SpellId, ranks: u8) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get_mut(&caster) else {
            return;
        };
        let Some(rank) = me.spell_rank(&content, spell).filter(|&r| r > 1) else {
            return;
        };
        me.set_spell_rank(spell, rank.saturating_sub(ranks).max(1));
        if caster == self.state.player {
            let name = content
                .spell_of(spell)
                .map_or("the spell", |d| d.name.as_str());
            self.say(format!("You know {name} less well than you did."));
        }
    }

    /// `caster` forgets `spell`, and `spells − 1` more drawn at random, or every spell it knows
    /// when `spells` is `None` (module docs).
    pub(super) fn miscast_forget(&mut self, caster: ActorId, spell: SpellId, spells: Option<u8>) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get(&caster) else {
            return;
        };
        if !me.knows_spell(&content, spell) {
            return;
        }
        let mut left: Vec<SpellId> = (me.known_spells(&content).into_iter())
            .filter(|&s| s != spell)
            .collect();
        let mut gone = vec![spell];
        match spells {
            None => gone.append(&mut left),
            Some(n) => {
                for _ in 1..n {
                    if left.is_empty() {
                        break;
                    }
                    let i = self.state.rng.below(left.len() as u32) as usize;
                    gone.push(left.remove(i));
                }
            }
        }
        if let Some(me) = self.state.actors.get_mut(&caster) {
            for &s in &gone {
                me.forget_spell(&content, s);
            }
        }
        if caster == self.state.player {
            let names: Vec<&str> = (gone.iter())
                .filter_map(|&s| content.spell_of(s).map(|d| d.name.as_str()))
                .collect();
            self.say(format!("You have forgotten {}.", and_list(&names)));
        }
    }
}
