//! The dump layers: their names, which are dev-only, and the layer list's parser.

use super::*;

/// Number of recent messages included in a dump.
pub(super) const RECENT_MESSAGES: usize = 10;

/// A map layer to render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// What the player sees: actors on visible cells, then known traps, then the top object of
    /// each pile in sight or remembered, then terrain seen or remembered; unknown as space.
    View,
    /// `*` on every currently visible cell.
    Fov,
    /// Terrain glyphs for every cell the player has seen (visible or remembered), with the top
    /// object of each known pile and the known traps over them.
    Memory,
    /// Perceived light on every visible cell as a digit, `0` (dark) to `9` (fully lit or
    /// brighter); see [`light_digit`].
    Light,
    /// Sounds the player heard from unseen sources, at their rough places ([`sound_char`]: `f`
    /// footsteps, `o` a door opening, `c` a door closing, `v` a voice; upper case when loud),
    /// over the view.
    Sound,
    /// Blood the player sees or remembers ([`blood_digit`]: `1` a few drops to `9` soaked), over
    /// the view.
    Blood,
    /// Char the player sees or remembers ([`blood_digit`]: `1` a little soot to `9` charred
    /// through), over the view.
    Scorch,
    /// Not a map: the player's inventory, one line per stack (marked when worn or wielded), then
    /// what they carry and weigh.
    Inventory,
    /// Not a map: the player's equipment, one slot a line (`c Body: a leather jerkin`, `-` when
    /// empty).
    Equipment,
    /// Not a map: the player's character (`Race Dwarf`, `Gender Female`, `Class Warrior`), if
    /// any, then their own scores, one stat a line (`Strength 12`, lowered while a status effect
    /// or stat damage lowers it), then their alignment band (`Alignment Kind`; spec §8
    /// "Alignment", never the score), then their status effects by name (`Effects Bleeding,
    /// Diseased`), the creature their openings are in (`Mark the kobold`), and the diseases they know they have (`Diseases filth fever`) if any; never
    /// potentials, modifiers, how severe an effect is, or a disease's timers.
    Stats,
    /// Not a map: the player's own trained skills, one a line in skill order (`Stealth 17`);
    /// never an untrained skill, a mark, or a benefit.
    Skills,
    /// Not a map: the player's weapon skills and fighting styles, one a line with its rank and
    /// how far its marks are toward the next (`Long blades 2 (40%)`), the styles they fight in
    /// now marked (`Duelist 1 (0%), in use`), the caster styles among them; never what a rank
    /// gives.
    WeaponSkills,
    /// Not a map: the player's attacks as the character screen shows them, one a line: where it
    /// comes from, what it is made with, its to-hit, damage, and range (`Main hand long sword: +4
    /// to hit, 1d8+2 slashing`, `Missile short bow: +3 to hit, 1d6 piercing, range 10`).
    Attacks,
    /// Not a map: the player's talents in the order taken, one a line with its choice
    /// (`Skill focus (stealth)`, `Weapon focus (long blades)`).
    Talents,
    /// Not a map: the player's abilities in the abilities menu's order, one a line with its
    /// source and what it asks for, or why it cannot be used now (`Shield bash (weapon skill):
    /// a direction`, `Steal (skill): not usable yet`).
    Abilities,
    /// Not a map: every profession's recipes as its screen lists them for the player (spec §8
    /// "Skills"; `world/crafting/craft.rs`), a `Metalcraft:` line per profession they have, then one a
    /// line with what each takes and why it cannot be made now (`Temper: a metal weapon or
    /// armour, an anvil, about 10 minutes (no anvil beside you)`).
    Recipes,
    /// Not a map: the spells the player knows (spec §8 "Spells"), one a line in menu order with
    /// its tier and what it costs them now (`Frost: tier 2, 14 mana`), then the scrolls they
    /// could study, by position, with the chance in words (`study c, scroll of fireball: Fire
    /// burst, far beyond you`); never the number.
    Spells,
    /// Not a map: the traps the player knows on the level, one a line by position (`(12, 5)
    /// fireball trap`, `(remembered)` when out of sight); never hidden ones.
    Traps,
    /// Not a map: the overworld locations the player knows, one a line by position (`(45, 23)
    /// The Hall`), from what they see and remember (a hidden location only once revealed);
    /// `no known locations` elsewhere or when there are none.
    Locations,
    /// Not a map: the player's background story (spec §8 "Background story"), a paragraph a
    /// line as the story screen tells it, a blank line between; `no story` without one.
    Story,
    /// The player's companion as they know it (spec §8 "Allies"): its name, type, grade, cell
    /// (where seen), health in words, and order; `no companion` without one.
    Companion,
    /// The player's familiar, as the companion layer says a companion (`world/allies/familiar.rs`);
    /// `no familiar` without one, `no familiar; Tibb, a cat, answers a calling` while the one
    /// she remembers is dead (`Character::familiar`).
    Familiar,
    /// Not a map: the player's quest log, one quest a line, active first (`Rats in the arena
    /// (active, from the old hermit): Kill 3 giant rats in the Arena. 2 / 3 killed.`).
    Quests,
    /// Ground truth: all terrain, objects, traps (hidden ones too), secret doors (`S`), hidden
    /// locations (by their entrance's glyph), and actors on the level, then a line per chest
    /// (its level, band, state, lock, and how many stacks it holds). **Dev-only.**
    Truth,
    /// Not a map: every actor on the level with its weight, what it carries (and of that wears
    /// and wields), its stats (scores and potentials), its trained skills with their marks, its
    /// status effects with their severities, and its diseases, stat damage, and drained points,
    /// the player's nutrition, then every corpse on the level and in the player's pack with its
    /// age (ground truth). **Dev-only.**
    Actors,
    /// Not a map: the player's progression (spec §8 "Talents"): the choices a level still asks
    /// for (`due level 3: 3 skill rolls, 1 talent`), where a level's skill rolls come from
    /// (`rolls a level 3 (class 2, race 1, intelligence 0, talents 0)`) and the dice a roll on
    /// each trained skill adds (`roll dice athletics 1d3, balance +2`; `/2` past the soft cap),
    /// the talents they may take now by id with the choices
    /// each may be taken for (`takeable healthy, weapon_focus:{bows,slings}`), and the creature
    /// types whose first kill has paid (`discovered kobold, rat`), the animal types a
    /// Beast-friend attacked (`empathy broken rat`), and the stat training marks the player holds
    /// (`stat marks St 3, To 1`), each rolled quest's draw with the size of the pool it could
    /// draw from now (`roll verses scroll_of_frost x1 of 4`), what an aristocrat's stipend holds
    /// (`stipend 412 gold: a potion of healing`), a merchant's stake with their partner
    /// (`invest 98.24 gold, week 74260, bonus 0 bp`), a craftsperson's commissions from their
    /// patron (`patron 3 taken, open long_sword`), and the named monsters dead
    /// (`slain fire_drake`). **Dev-only.**
    Progress,
    /// The level's special rooms (spec §7 "Special rooms"; ground truth): the terrain, with the
    /// plain ground of each special room drawn as its effect's `mark`, then a line per room
    /// (`s shadowed room at (12, 3)-(20, 9), entered`). **Dev-only.**
    Rooms,
    /// Ground truth: the ice on the level's water, its thickness in whole centimetres as a
    /// character ([`ice_char`]), over the terrain; first a line with the temperature there and
    /// when the ice was last brought up to date. **Dev-only.**
    Ice,
}

impl Layer {
    /// A slice, not a fixed-size array, so features adding layers don't collide on the length.
    const ALL: &[Layer] = &[
        Layer::View,
        Layer::Fov,
        Layer::Memory,
        Layer::Light,
        Layer::Sound,
        Layer::Blood,
        Layer::Scorch,
        Layer::Inventory,
        Layer::Equipment,
        Layer::Stats,
        Layer::Skills,
        Layer::WeaponSkills,
        Layer::Attacks,
        Layer::Talents,
        Layer::Abilities,
        Layer::Recipes,
        Layer::Spells,
        Layer::Traps,
        Layer::Locations,
        Layer::Quests,
        Layer::Story,
        Layer::Companion,
        Layer::Familiar,
        Layer::Truth,
        Layer::Actors,
        Layer::Progress,
        Layer::Rooms,
        Layer::Ice,
    ];

    /// Layer name as used on the command line and in JSON.
    pub const fn name(self) -> &'static str {
        match self {
            Layer::View => "view",
            Layer::Fov => "fov",
            Layer::Memory => "memory",
            Layer::Light => "light",
            Layer::Sound => "sound",
            Layer::Blood => "blood",
            Layer::Scorch => "scorch",
            Layer::Inventory => "inventory",
            Layer::Equipment => "equipment",
            Layer::Stats => "stats",
            Layer::Skills => "skills",
            Layer::WeaponSkills => "weapon_skills",
            Layer::Attacks => "attacks",
            Layer::Talents => "talents",
            Layer::Abilities => "abilities",
            Layer::Recipes => "recipes",
            Layer::Spells => "spells",
            Layer::Traps => "traps",
            Layer::Locations => "locations",
            Layer::Quests => "quests",
            Layer::Story => "story",
            Layer::Companion => "companion",
            Layer::Familiar => "familiar",
            Layer::Truth => "truth",
            Layer::Actors => "actors",
            Layer::Progress => "progress",
            Layer::Rooms => "rooms",
            Layer::Ice => "ice",
        }
    }

    /// Whether the layer reads ground truth.
    pub const fn is_dev_only(self) -> bool {
        matches!(
            self,
            Layer::Truth | Layer::Actors | Layer::Progress | Layer::Rooms | Layer::Ice
        )
    }
}

impl FromStr for Layer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Layer::ALL
            .iter()
            .copied()
            .find(|l| l.name() == s)
            .ok_or_else(|| {
                format!(
                    "unknown layer `{s}` (expected view, fov, memory, light, sound, blood, \
                     scorch, inventory, equipment, stats, skills, weapon_skills, attacks, talents, \
                     abilities, recipes, traps, locations, quests, story, companion, familiar, \
                     truth, actors, progress, rooms, ice)"
                )
            })
    }
}

/// Parses a comma-separated layer list such as `view,fov`.
pub fn parse_layers(text: &str) -> Result<Vec<Layer>, String> {
    text.split(',').map(|l| l.trim().parse()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `docs/dev/cli.md` lists every dump layer by hand (#1667).
    #[test]
    fn every_layer_is_in_the_cli_doc() {
        let missing =
            crate::test_support::missing_from_cli_doc(Layer::ALL.iter().map(|l| l.name()));
        assert!(
            missing.is_empty(),
            "docs/dev/cli.md lacks the dump layers {missing:?}"
        );
    }
}
