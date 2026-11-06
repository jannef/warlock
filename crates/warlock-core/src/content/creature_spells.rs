//! A caster's spells as its type writes them (spec §8 "Creature behavior": "Casters", #1102):
//! `spells = ["spark", "hex_of_weakness"]`, ids in `data/spells/`, in the order it prefers
//! them. Resolved when the creature types load (spells load first); `world/creature_cast.rs`
//! casts them.
//!
//! Checks: a `combat = "caster"` type names at least one spell, and no other type names any;
//! every id is a spell; and each is one a creature can cast: a damage spell (no zone, no corpse,
//! no open sky), a hex that lays a plain status (no zone, no push, no charm, no ground it needs),
//! a heal by touch, or a buff that lays a status. Summons, cleansing, and utilities are the
//! player's (a summoner has its own `[creature.summon]`).

use super::creature::{Combat, CreatureSpec};
use super::{Content, Shape, SpellData, SpellId, SpellPurpose};

/// The spells `spec` names, resolved, or what is wrong with them (module docs).
pub(super) fn resolve(content: &Content, spec: &CreatureSpec) -> Result<Vec<SpellId>, String> {
    let id = &spec.id;
    match (spec.combat, spec.spells.is_empty()) {
        (Combat::Caster, true) => {
            return Err(format!("creature `{id}`: a `caster` needs `spells`"));
        }
        (Combat::Melee | Combat::Ranged | Combat::Dragon, false) => {
            return Err(format!(
                "creature `{id}`: only a `combat = \"caster\"` creature has `spells`"
            ));
        }
        _ => {}
    }
    let mut spells = Vec::with_capacity(spec.spells.len());
    for name in &spec.spells {
        let spell = (content.spell_id(name))
            .ok_or_else(|| format!("creature `{id}`: spells: unknown spell `{name}`"))?;
        if spells.contains(&spell) {
            return Err(format!("creature `{id}`: spells: `{name}` twice"));
        }
        castable(content.spell(spell))
            .map_err(|why| format!("creature `{id}`: spells: `{name}` {why}"))?;
        spells.push(spell);
    }
    Ok(spells)
}

/// Whether a creature can cast `def` (module docs), else why not.
pub fn castable(def: &SpellData) -> Result<(), &'static str> {
    let e = &def.effect;
    let plain = e.zone.is_none() && e.corpse.is_none() && !def.open_sky;
    match def.kind {
        SpellPurpose::Damage if plain => Ok(()),
        SpellPurpose::Damage => Err("lays a zone, needs a corpse or the open sky"),
        SpellPurpose::Hex
            if plain
                && e.status.is_some()
                && e.push.is_none()
                && e.charm.is_none()
                && e.only_terrain.is_none()
                && e.only_tags.is_empty() =>
        {
            Ok(())
        }
        SpellPurpose::Hex => Err("is a hex that lays no plain status"),
        SpellPurpose::Heal if def.shape == Shape::Touch && e.heal.is_some() && plain => Ok(()),
        SpellPurpose::Heal => Err("is a heal not by touch"),
        SpellPurpose::Buff if e.status.is_some() => Ok(()),
        SpellPurpose::Buff => Err("is a buff that lays no status"),
        SpellPurpose::Summon | SpellPurpose::Cleanse | SpellPurpose::Utility => {
            Err("is a summon, a cleansing, or a utility: no creature casts one")
        }
    }
}
