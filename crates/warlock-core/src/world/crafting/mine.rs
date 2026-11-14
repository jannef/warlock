//! Mining (spec §8 "Digging"; #1389, docs/design/professions.md §9): what a wall a miner digs
//! through gives, by `data/mining.toml` ([`crate::content::MiningRules`]).
//!
//! - **When**: the player breaks through a wall by digging ([`World::dig`]) on a floor of a
//!   dungeon branch, trained in the file's skill (Mining). Untrained, on a hand-made level, the
//!   overworld or its areas, by a blast or a rod, or through a statue (no rock), nothing.
//! - **The chance**: the wall's hardness band (the hardest band not harder than it) gives the
//!   percent chance (one draw) that a check is made at all.
//! - **The check**: two draws (2d12, [`combat::roll_natural`]) plus the file's stat modifier
//!   (Strength), the skill's `craft` benefit ([`crate::world::Actor::skill_effect_of`]), and the
//!   talents' checks of that stat, against the band's difficulty: a natural 2 fails, a natural 24 makes it.
//!   Failed, nothing, and nothing is said: the player never knew there was a chance.
//! - **The find**: one draw by weight from the table at the floor's danger
//!   ([`crate::content::Content::floor_danger`]: an endless branch's keeps rising), the branch's
//!   `ores` allow-list leaving out the other ores. An ore or stone row gives one lump or block,
//!   one more for each `more_every` of the margin, at most `most`; a gem row one uncut loose gem
//!   at the floor's item level ([`crate::gem::uncut`], two draws). It lies on the rubble, and is
//!   said: "You break out a lump of iron ore."
//!
//! - **A vein** (#1390; `mapgen/vein.rs` lays them): a wall of a vein's terrain
//!   ([`MiningRules::vein_of`]) always gives its ore, with no chance draw and no table: the
//!   check is made all the same, and sets only the count, one lump failed, else as above.
//!
//! Every draw is the gameplay stream's, in that order. Mining's mark is the wall's own
//! ([`crate::skills::Use::Dug`], `world/crafting/dig.rs`): no mark of its own.

use crate::combat;
use crate::content::{Content, MaterialId, Mined, MiningRules, TerrainId};
use crate::geom::Pos;
use crate::item_parts::ItemPart;
use crate::magic::loot::pick_weighted;
use crate::object::{self, Stack};
use crate::rng::draw;
use crate::skills::Effect;
use crate::world::World;

/// `count` lumps of `metal`'s ore: the ore object, made of it.
pub(crate) fn ore(content: &Content, rules: &MiningRules, metal: MaterialId, count: u32) -> Stack {
    let stack = Stack::new(rules.ore, count);
    if content.object(rules.ore).material() == metal {
        stack
    } else {
        stack.with_parts(vec![ItemPart::material(content, None, metal)])
    }
}

impl World {
    /// The player has dug through `wall` (the terrain it was) at `at`: what Mining breaks out of
    /// it (the module docs).
    pub(super) fn mine(&mut self, at: Pos, wall: TerrainId) {
        let content = self.content.clone();
        let Some(rules) = content.mining() else {
            return;
        };
        let here = self.state.current_level.clone();
        let Some((branch, _)) = content.branch_level(&here) else {
            return;
        };
        let def = content.terrain(wall);
        // A statue or a makeshift anvil (a station set by hand) holds no rock.
        let Some(breaks) = (def.breaks).filter(|_| def.statue.is_none() && def.works_as.is_none())
        else {
            return;
        };
        let me = self.state.player();
        if me.skills.get(rules.skill).is_none() {
            return;
        }
        let Some(band) = rules.band(breaks.hardness).copied() else {
            return;
        };
        let modifier = me.check_modifier(&content, rules.stat)
            + me.skill_effect_of(&content, rules.skill, Effect::Craft);
        let vein = rules.vein_of(wall);
        // The strike: whether anything is there, the check, what, and a gem: one stream.
        let mut rng = draw!(self, "mine/player");
        if vein.is_none() && !rng.chance(u32::from(band.chance)) {
            return;
        }
        let natural = combat::roll_natural(&mut rng);
        let margin = natural + modifier - band.difficulty;
        let made = natural == combat::HIGHEST || (natural != combat::FUMBLE && margin >= 0);
        let count = (1 + margin.max(0) / rules.more_every).clamp(1, rules.most as i32) as u32;
        if let Some(metal) = vein {
            let found = ore(&content, rules, metal, if made { count } else { 1 });
            self.lay_mined(at, found);
            return;
        }
        if !made {
            return;
        }
        let ores: Option<Vec<MaterialId>> = (branch.ores.as_ref())
            .map(|list| list.iter().filter_map(|m| content.material_id(m)).collect());
        let danger = content.floor_danger(&here, self.player_level());
        let table = rules.at(danger, ores.as_deref());
        let Some(&row) = pick_weighted(&mut rng, &table) else {
            return;
        };
        let found = match rules.finds[row].what {
            Mined::Ore(metal) => Some(ore(&content, rules, metal, count)),
            Mined::Piece(material) => crate::piece::make(&content, material, count),
            Mined::Gem => crate::gem::uncut(&content, &mut rng, content.item_level(&here)),
        };
        if let Some(found) = found {
            self.lay_mined(at, found);
        }
    }

    /// Lays what was broken out of the wall at `at` on its rubble, and says so.
    fn lay_mined(&mut self, at: Pos, found: Stack) {
        let content = self.content.clone();
        let text = format!("You break out {}.", found.describe(&content));
        let level = self.state.current_mut();
        let mut pile = vec![found];
        object::add_to_pile(&mut pile, level.objects_at(at), &content);
        level.set_pile(at, pile);
        self.say(text);
    }
}
