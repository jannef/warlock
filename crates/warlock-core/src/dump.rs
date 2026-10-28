//! Text and JSON dumps of the simulation for developers and agents (spec §9).
//!
//! Every layer except [`Layer::Truth`], [`Layer::Actors`], and [`Layer::Rooms`] is built from
//! [`World::perception`] only, so dumps cannot leak information the player does not have.
//! `Truth`, `Actors`, and `Rooms` read ground truth and are dev-only.

use crate::ability::Aim;
use crate::action::{letter, position_text};
use crate::blood::{self, Stain};
use crate::character::Gender;
use crate::content::{Content, SpecialRoom, WeaponSkillId};
use crate::equipment;
use crate::geom::Pos;
use crate::hunger::Hunger;
use crate::ice::Ice;
use crate::noise::{PerceivedSound, SoundKind, Volume};
use crate::object::{Stack, format_weight};
use crate::perception::{CellView, PerceivedLight, PerceptionView};
use crate::stats::Stat;
use crate::weather::Weather;
use crate::world::{Cause, LevelState, World};
use serde_json::{Value, json};
use std::fmt::Write;
use std::str::FromStr;

/// Number of recent messages included in a dump.
const RECENT_MESSAGES: usize = 10;

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
    /// Not a map: the player's inventory, one line per stack (marked when worn or wielded), then
    /// what they carry and weigh.
    Inventory,
    /// Not a map: the player's equipment, one slot a line (`c Body: a leather jerkin`, `-` when
    /// empty).
    Equipment,
    /// Not a map: the player's character (`Race Dwarf`, `Gender Female`, `Class Warrior`), if
    /// any, then their own scores, one stat a line (`Strength 12`, lowered while a status effect
    /// or stat damage lowers it), then their caster level (`Caster level 3`; spec §8 "Spells"),
    /// then their alignment band (`Alignment Kind`; spec §8
    /// "Alignment", never the score), then their status effects by name (`Effects Bleeding,
    /// Diseased`) and the diseases they know they have (`Diseases filth fever`) if any; never
    /// potentials, modifiers, how severe an effect is, or a disease's timers.
    Stats,
    /// Not a map: the player's own trained skills, one a line in skill order (`Stealth 17`);
    /// never an untrained skill, a mark, or a benefit.
    Skills,
    /// Not a map: the player's weapon skills and fighting styles, one a line with its rank and
    /// how far its marks are toward the next (`Long blades 2 (40%)`), the styles they fight in
    /// now marked (`Duelist 1 (0%), in use`); never what a rank gives.
    WeaponSkills,
    /// Not a map: the player's attacks as the character screen shows them, one a line: where it
    /// comes from, what it is made with, its to-hit, damage, and range (`Main hand long sword: +4
    /// to hit, 1d8+2 slashing`, `Missile short bow: +3 to hit, 1d6 piercing, range 10`).
    Attacks,
    /// Not a map: the player's talents in the order taken, one a line with its rank of how many
    /// and its choice (`Hardy 2/3`, `Weapon focus (long blades)`); never what a rank gives.
    Talents,
    /// Not a map: the player's abilities in the abilities menu's order, one a line with its
    /// source and what it asks for, or why it cannot be used now (`Shield bash (weapon skill):
    /// a direction`, `Steal (skill): not usable yet`).
    Abilities,
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
    /// draw from now (`roll verses scroll_of_frost x1 of 4`), and the named monsters dead
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
        Layer::Inventory,
        Layer::Equipment,
        Layer::Stats,
        Layer::Skills,
        Layer::WeaponSkills,
        Layer::Attacks,
        Layer::Talents,
        Layer::Abilities,
        Layer::Spells,
        Layer::Traps,
        Layer::Locations,
        Layer::Quests,
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
            Layer::Inventory => "inventory",
            Layer::Equipment => "equipment",
            Layer::Stats => "stats",
            Layer::Skills => "skills",
            Layer::WeaponSkills => "weapon_skills",
            Layer::Attacks => "attacks",
            Layer::Talents => "talents",
            Layer::Abilities => "abilities",
            Layer::Spells => "spells",
            Layer::Traps => "traps",
            Layer::Locations => "locations",
            Layer::Quests => "quests",
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
                     inventory, equipment, stats, skills, weapon_skills, attacks, talents, \
                     abilities, traps, locations, quests, truth, actors, progress, rooms, ice)"
                )
            })
    }
}

/// Parses a comma-separated layer list such as `view,fov`.
pub fn parse_layers(text: &str) -> Result<Vec<Layer>, String> {
    text.split(',').map(|l| l.trim().parse()).collect()
}

/// Renders one layer as rows of text with trailing spaces trimmed.
pub fn layer_rows(world: &World, layer: Layer) -> Vec<String> {
    match layer {
        Layer::Truth => [spawn_header(world), truth_rows(world), chest_rows(world)].concat(),
        Layer::Actors => [spawn_header(world), actor_rows(world), rising_rows(world)].concat(),
        Layer::Progress => progress_rows(world),
        Layer::Rooms => room_rows(world),
        Layer::Ice => ice_rows(world),
        Layer::Inventory => inventory_rows(&world.perception()),
        Layer::Equipment => equipment_rows(&world.perception()),
        Layer::Stats => stat_rows(&world.perception()),
        Layer::Skills => skill_rows(&world.perception()),
        Layer::WeaponSkills => weapon_skill_rows(&world.perception()),
        Layer::Attacks => attack_rows(&world.perception()),
        Layer::Talents => talent_rows(&world.perception()),
        Layer::Abilities => ability_rows(&world.perception()),
        Layer::Spells => spell_rows(&world.perception()),
        Layer::Traps => trap_rows(&world.perception()),
        Layer::Locations => location_rows(&world.perception()),
        Layer::Quests => quest_rows(&world.perception()),
        _ => {
            let view = world.perception();
            rows(view.width(), view.height(), |p| {
                perceived_glyph(&view, layer, p)
            })
        }
    }
}

fn perceived_glyph(view: &PerceptionView<'_>, layer: Layer, p: Pos) -> char {
    let content = view.content();
    let cell = view.cell(p);
    let terrain_glyph = |cell: CellView| match cell {
        CellView::Unknown => ' ',
        CellView::Remembered(t) | CellView::Visible(t) => content.terrain(t).glyph,
    };
    // A chest's known trap leaves the chest's glyph (spec §8 "Chests").
    let trap = view.trap(p).filter(|_| view.chest_at(p).is_none());
    let known_glyph = |cell: CellView| match (trap, view.objects(p).first()) {
        (Some(trap), _) => content.trap(trap).glyph,
        (None, Some(top)) => content.object(top.object).glyph,
        (None, None) => terrain_glyph(cell),
    };
    match layer {
        Layer::View => match view.actor_at(p) {
            Some(actor) => actor.glyph(content),
            None => known_glyph(cell),
        },
        Layer::Fov => {
            if view.is_visible(p) {
                '*'
            } else {
                ' '
            }
        }
        Layer::Memory => known_glyph(cell),
        Layer::Light => view.light(p).map_or(' ', light_digit),
        Layer::Sound => match view.sound_at(p) {
            Some(sound) => sound_char(sound),
            None => perceived_glyph(view, Layer::View, p),
        },
        Layer::Blood => match view.blood(p) {
            Some(stain) => blood_digit(stain),
            None => perceived_glyph(view, Layer::View, p),
        },
        Layer::Inventory
        | Layer::Equipment
        | Layer::Stats
        | Layer::Skills
        | Layer::WeaponSkills
        | Layer::Attacks
        | Layer::Talents
        | Layer::Abilities
        | Layer::Spells
        | Layer::Traps
        | Layer::Locations
        | Layer::Quests
        | Layer::Truth
        | Layer::Actors
        | Layer::Progress
        | Layer::Rooms
        | Layer::Ice => unreachable!("not a map built from perception"),
    }
}

/// A heard sound in the `sound` layer: `f` footsteps, `o` a door opening, `c` a door closing,
/// `x` fighting, `b` a bowstring, `m` chanting (magic), `k` banging (a door kicked), `d` digging,
/// `v` a voice, `h` a howl, `a` an alarm bell, `e` an explosion, `t` a thunderclap; upper case
/// when loud.
pub fn sound_char(sound: &PerceivedSound) -> char {
    let c = match sound.kind {
        SoundKind::Footsteps => 'f',
        SoundKind::DoorOpened => 'o',
        SoundKind::DoorClosed => 'c',
        SoundKind::Fighting => 'x',
        SoundKind::Bowstring => 'b',
        SoundKind::Chanting => 'm',
        SoundKind::Banging => 'k',
        SoundKind::Digging => 'd',
        SoundKind::Voice => 'v',
        SoundKind::Howl => 'h',
        SoundKind::Bell => 'a',
        SoundKind::Explosion => 'e',
        SoundKind::Thunder => 't',
    };
    if sound.volume == Volume::Loud {
        c.to_ascii_uppercase()
    } else {
        c
    }
}

/// A stain in the `blood` layer: its amount in ninths of [`blood::CAP`], rounded up, `1` (a few
/// drops) to `9` (soaked).
pub fn blood_digit(stain: &Stain) -> char {
    let ninths = (u32::from(stain.amount) * 9).div_ceil(u32::from(blood::CAP));
    char::from_digit(ninths.clamp(1, 9), 10).unwrap_or('9')
}

/// Perceived brightness ([`PerceivedLight::brightness`]: luma, plus UV for eyes that see it) in
/// tenths of full light: `0` below 26, `9` from 230 up (overbright included).
pub fn light_digit(light: PerceivedLight) -> char {
    let tenths = (u32::from(light.brightness()) * 10 / 256).min(9);
    char::from_digit(tenths, 10).unwrap_or('9')
}

/// Dev: the header of the `truth` and `actors` layers on a level with a danger rule (spec §7
/// "Danger levels"): `danger 3 population 2/6 spawn next 08:15:00`, its creatures hostile to
/// the player against its cap; none elsewhere.
fn spawn_header(world: &World) -> Vec<String> {
    let Some((danger, hostiles, cap, next)) = world.spawn_status() else {
        return Vec::new();
    };
    let next = format!(
        "{:02}:{:02}:{:02}",
        next.hour(),
        next.minute(),
        next.second()
    );
    vec![format!(
        "danger {danger} population {hostiles}/{cap} spawn next {next}"
    )]
}

/// Dev: the risings due on the current level (spec §7 "Special rooms": a cursed room), one a
/// line in the order they died: `rising goblin at (12, 5) as zombie at 08:15:00`.
fn rising_rows(world: &World) -> Vec<String> {
    let content = world.content();
    (world.risings().iter())
        .map(|r| {
            let at = r.at();
            format!(
                "rising {} at {} as {} at {:02}:{:02}:{:02}",
                content.creature(r.dead()).id,
                r.pos(),
                content.creature(r.risen()).id,
                at.hour(),
                at.minute(),
                at.second()
            )
        })
        .collect()
}

fn truth_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, actors) = world.ground_truth();
    let actors: Vec<_> = actors.collect();
    rows(level.width(), level.height(), |p| {
        match (
            actors.iter().find(|a| a.pos == p),
            level.trap_at(p),
            level.objects_at(p).first(),
        ) {
            (Some(actor), _, _) => content.creature(actor.creature).glyph,
            (None, Some(trap), _) => content.trap(trap.kind).glyph,
            (None, None, Some(top)) => content.object(top.object).glyph,
            (None, None, None) if level.secret_at(p).is_some() => 'S',
            // A sealed room's breach while it stands (spec §7 "Special rooms").
            (None, None, None) if level.breach_at(content, p) => 'H',
            // A hidden location by its entrance's glyph (spec §7 "Hidden locations").
            (None, None, None) => (level.hidden_at(p).map(|h| h.terrain))
                .or(level.terrain_at(p))
                .map_or(' ', |t| content.terrain(t).glyph),
        }
    })
}

/// Dev: the chests on the current level (spec §8 "Chests"; ground truth), one a line by
/// position: `chest 7 (hutch) at (12, 5): shut, locked, trapped (arrow), holds 3 stacks`.
fn chest_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, _) = world.ground_truth();
    (level.chests().iter())
        .map(|(&p, chest)| {
            let band = (content.chests().band_of(chest.level)).map_or("?", |b| b.id.as_str());
            let state = (level.terrain_at(p))
                .and_then(|t| content.terrain(t).chest)
                .map_or("gone", |s| s.id());
            let locked = if level.lock_at(p).is_some() {
                ", locked"
            } else {
                ""
            };
            let trapped = match level.trap_at(p) {
                Some(trap) => format!(", trapped ({})", content.trap(trap.kind).id),
                None => String::new(),
            };
            let holds = match chest.holds.len() {
                0 => String::new(),
                1 => ", holds 1 stack".to_string(),
                n => format!(", holds {n} stacks"),
            };
            format!(
                "chest {} ({band}) at {p}: {state}{locked}{trapped}{holds}",
                chest.level
            )
        })
        .collect()
}

/// The level's special rooms (ground truth): the terrain with each special room's plain ground
/// ([`crate::content::TerrainDef::is_plain_ground`]) drawn as its effect's mark, then one line
/// per room (glimpsed, entered, a hoard's stacks lifted and whether it sealed, a sealed room's
/// breach); headed by `pervading <id>` when an effect covers the level.
fn room_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, _) = world.ground_truth();
    let mut out = rows(level.width(), level.height(), |p| {
        let Some(t) = level.terrain_at(p) else {
            return ' ';
        };
        let room = level.rooms().iter().find(|r| r.contains(p));
        match room {
            Some(room) if content.terrain(t).is_plain_ground() => {
                content.room_effect(room.effect).mark
            }
            _ => content.terrain(t).glyph,
        }
    });
    if let Some(id) = level.pervading() {
        out.insert(0, format!("pervading {}", content.room_effect(id).id));
    }
    for room in level.rooms() {
        let effect = content.room_effect(room.effect);
        let glimpsed = if room.glimpsed { ", glimpsed" } else { "" };
        let entered = if room.entered { ", entered" } else { "" };
        let side = room
            .side
            .map_or(String::new(), |s| format!(", side {}", s.id()));
        let lifted = match room.lifted {
            0 => String::new(),
            n => format!(", lifted {n}"),
        };
        let sealed = if room.sealed { ", sealed" } else { "" };
        let breach = (room.breach).map_or(String::new(), |b| format!(", breach {b}"));
        out.push(format!(
            "{} {} at {}-{}{side}{}{glimpsed}{entered}{lifted}{sealed}{breach}",
            effect.mark,
            effect.name,
            room.min,
            room.max,
            feature_state(level, content, room)
        ));
    }
    out
}

/// What became of a special room's feature, for the `rooms` layer: `, spent` once its cell is
/// the spent terrain of the table it offers, `, destroyed` once it is neither (an altar blown
/// apart); nothing while it stands, or for a room without one.
fn feature_state(level: &LevelState, content: &Content, room: &SpecialRoom) -> &'static str {
    let effect = content.room_effect(room.effect);
    let (Some(feature), Some((_, table))) = (effect.feature, effect.offers.as_ref()) else {
        return "";
    };
    let spent = content.outcomes().get(table).and_then(|t| t.spent);
    let cells = room.cells();
    let terrains: Vec<_> = cells.iter().filter_map(|&p| level.terrain_at(p)).collect();
    if terrains.contains(&feature) {
        ""
    } else if spent.is_some_and(|s| terrains.contains(&s)) {
        ", spent"
    } else {
        ", destroyed"
    }
}

/// The overworld locations the player knows, one a line by position: `(45, 23) The Hall`; `no
/// known locations` when there are none (and on any other level).
fn location_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let rows: Vec<String> = (view.known_locations().into_iter())
        .map(|(p, name)| format!("{p} {name}"))
        .collect();
    if rows.is_empty() {
        vec!["no known locations".into()]
    } else {
        rows
    }
}

/// The traps the player knows on the level, one a line by position: `(12, 5) fireball trap`,
/// `(40, 2) alarm trap (remembered)` when out of sight; `no known traps` when there are none.
fn trap_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let rows: Vec<String> = (view.known_traps().into_iter())
        .map(|(p, trap)| {
            let seen = if view.sees_objects_at(p) {
                ""
            } else {
                " (remembered)"
            };
            format!("{p} {}{seen}", content.trap(trap).name)
        })
        .collect();
    if rows.is_empty() {
        vec!["no known traps".into()]
    } else {
        rows
    }
}

/// Ice of `ice` in the `ice` layer: its thickness in whole centimetres, `0` to `9`, then `a` for
/// 10 cm to `z` for 35 cm, `#` for more.
pub fn ice_char(ice: Ice) -> char {
    let cm = ice.um / 10_000;
    match cm {
        0..=9 => char::from_digit(cm, 10).unwrap_or('#'),
        10..=35 => char::from(b'a' + (cm - 10) as u8),
        _ => '#',
    }
}

/// The ice on the level (ground truth): a line with the temperature where the player stands (a
/// hot or a cold room's included) and when the ice was
/// brought up to date, then the map, each iced cell's thickness ([`ice_char`]) over the terrain.
fn ice_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, _) = world.ground_truth();
    let mut out = vec![format!(
        "temperature {}, ice as of {}",
        world.temperature(),
        level.ice_until()
    )];
    out.extend(rows(level.width(), level.height(), |p| {
        match level.ice_at(p) {
            Some(ice) => ice_char(ice),
            None => level
                .terrain_at(p)
                .map_or(' ', |t| content.terrain(t).glyph),
        }
    }));
    out
}

/// The player's inventory grouped by item class as the screens group it, a header line a class
/// (`Melee weapons:`), each stack by its position as action text names it (`a - a dagger (450
/// g)`, `b - a leather jerkin (being worn) (3.5 kg)`, `26 - a rock (1 kg)`), then what they
/// carry and weigh.
/// What `stack` gives in numbers, as the player knows it (`magic::numbers`), for JSON:
/// `{"to_hit": 0, "damage": "1d8+2", "range": null, "dv": 1, "pv": 1}` (ammunition's damage is
/// what it adds, `+1d4 fire`, or empty), each part null where it
/// has none; null for an item that gives nothing.
fn numbers_json(stack: &Stack, content: &Content) -> Value {
    let numbers = crate::magic::Numbers::of(stack, content);
    if numbers.is_empty() {
        return Value::Null;
    }
    let attack = numbers.attack.as_ref();
    let defence = numbers.defence;
    json!({
        "to_hit": attack.map(|a| a.to_hit),
        "damage": attack.map(|a| a.damage_text(content)),
        "range": attack.and_then(|a| a.range),
        "dv": defence.map(|d| d.dv),
        "pv": defence.and_then(|d| d.pv),
    })
}

fn inventory_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let unpaid = view.unpaid();
    let inventory = view.inventory();
    let mut rows: Vec<String> = Vec::new();
    let mut class = None;
    for i in view.by_class(inventory) {
        let s = &inventory[i];
        let here = view.item_class(s);
        if class != Some(here) {
            class = Some(here);
            rows.push(format!("{}:", content.item_class(here).name));
        }
        let name = view.lore().listed(s);
        let weight = format_weight(s.weight(content));
        let mark = equipment::mark(content, view.equipment(), i)
            .map_or(String::new(), |m| format!(" ({m})"));
        let owed = (unpaid.get(i).copied().flatten())
            .map_or(String::new(), |p| format!(" (unpaid, {p} gold)"));
        rows.push(format!(
            "{} - {name}{mark}{owed} ({weight})",
            position_text(i)
        ));
    }
    let weight = view.own_weight();
    let load = view.own_load();
    rows.push(format!(
        "carried {} ({}: {}), total weight {}",
        format_weight(weight.carried),
        load.burden().id(),
        load.over().join(", "),
        format_weight(weight.total())
    ));
    rows
}

/// The player's equipment, one slot a line by its letter: `c Body: a leather jerkin (b)` (the
/// inventory position in parentheses, as action text names it), `a Head: -` when empty.
fn equipment_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let worn = view.equipment();
    (content.slot_ids())
        .enumerate()
        .map(|(i, (slot, def))| {
            let what = match worn.get(slot) {
                Some(item) => view.inventory().get(item).map_or("?".into(), |s| {
                    format!("{} ({})", view.lore().describe(s), position_text(item))
                }),
                None => "-".into(),
            };
            format!("{} {}: {what}", letter(i).unwrap_or('?'), def.name)
        })
        .collect()
}

/// The player's character, if any (`Name Brakka`, `Race Dwarf`, `Gender Female`, `Class
/// Warrior`, `Age 27`, `Born 3 Seedtide 1417`, `Sign Fire Manticore`, with a lucky day's title
/// `Sign Metal Manticore, the Iron Scourge`), then their own scores, one stat a line:
/// `Strength 12`, then their caster level: `Caster level 3`.
fn stat_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let mut rows = Vec::new();
    if let Some(c) = view.character() {
        let spec = &c.spec;
        rows.push(format!("Name {}", c.name));
        let race = spec
            .race(content)
            .map_or(spec.race.as_str(), |r| r.name.as_str());
        let class = spec
            .class(content)
            .map_or(spec.class.as_str(), |k| k.name.as_str());
        rows.push(format!("Race {race}"));
        rows.push(match spec.gender {
            Gender::Other => format!("Gender Other ({} +1)", c.raised.name()),
            gender => format!("Gender {}", gender.name()),
        });
        rows.push(format!("Class {class}"));
        rows.push(format!("Age {}", c.age_on(view.time())));
        rows.push(format!("Born {}", c.born.text(content.calendar())));
        if let Some(sign) = c.sign_title(content) {
            rows.push(format!("Sign {sign}"));
        }
    }
    rows.extend((view.own_scores().iter()).map(|(stat, score)| format!("{} {score}", stat.name())));
    rows.push(format!("Caster level {}", view.own_caster_level()));
    if let Some(band) = view.own_alignment() {
        rows.push(format!("Alignment {}", band.name()));
    }
    // The guilds they belong to and their ranks (spec §8 "Quests": "Guilds"), a title only.
    for g in view.own_guilds() {
        rows.push(format!("Guild {}, {}", g.guild.bare_name(), g.title()));
    }
    let effects: Vec<&str> = (view.own_effects())
        .map(|s| content.status(s).name.as_str())
        .collect();
    if !effects.is_empty() {
        rows.push(format!("Effects {}", effects.join(", ")));
    }
    let diseases: Vec<&str> = (view.own_diseases())
        .map(|d| content.disease(d).name.as_str())
        .collect();
    if !diseases.is_empty() {
        rows.push(format!("Diseases {}", diseases.join(", ")));
    }
    rows.extend(resistance_rows(view));
    rows
}

/// The player's resistances in words (spec §8 "Damage types"), a row per word: `Resistant to
/// fire, cold`, `Vulnerable to acid`, `Immune to poison`; never a percentage.
fn resistance_rows(view: &PerceptionView<'_>) -> Vec<String> {
    use crate::damage::ResistanceWord;
    let content = view.content();
    let own = view.own_resistances();
    [
        ResistanceWord::Immune,
        ResistanceWord::Resistant,
        ResistanceWord::Vulnerable,
    ]
    .into_iter()
    .filter_map(|word| {
        let kinds: Vec<&str> = (own.iter())
            .filter(|(_, w)| *w == word)
            .map(|(k, _)| content.damage_type(*k).name.as_str())
            .collect();
        (!kinds.is_empty()).then(|| format!("{} {}", word.label(), kinds.join(", ")))
    })
    .collect()
}

/// The player's own trained skills, one a line in skill order: `Stealth 17`.
fn skill_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    (view.own_skills().iter())
        .map(|s| format!("{} {}", content.skill(s.skill).name, s.value))
        .collect()
}

/// How far the marks are toward the next rank, in whole percent; `None` at the last rank.
fn progress_percent(view: &PerceptionView<'_>, id: WeaponSkillId) -> Option<u32> {
    let table = view.content().weapon_skills();
    let (marks, need) = view.own_weapon_skills().progress(table, id)?;
    Some(marks * 100 / need.max(1))
}

/// The player's weapon skills and fighting styles, in file order, with their ranks and how far
/// toward the next (`Long blades 2 (40%)`, `Daggers and knives 10` at the last rank), and the
/// styles they fight in now marked (`Duelist 1 (0%), in use`).
fn weapon_skill_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let skills = view.own_weapon_skills();
    let in_use = view.own_styles_in_use();
    let table = view.content().weapon_skills();
    (table.ids())
        .filter(|&(id, _)| skills.shows(table, id))
        .map(|(id, def)| {
            let rank = skills.rank(id);
            let row = match progress_percent(view, id) {
                Some(percent) => format!("{} {rank} ({percent}%)", def.name),
                None => format!("{} {rank}", def.name),
            };
            if in_use.contains(&id) {
                format!("{row}, in use")
            } else {
                row
            }
        })
        .collect()
}

/// The player's attacks, one a line (`Main hand long sword: +4 to hit, 1d8+2 slashing`; see
/// [`Layer::Attacks`]).
fn attack_rows(view: &PerceptionView<'_>) -> Vec<String> {
    (view.own_attacks().iter())
        .map(|a| {
            let line = a.summary(view.content());
            format!("{} {}: {line}", a.source.label(), a.name)
        })
        .collect()
}

/// The player's attacks for JSON: `source`, `kind`, `name`, `to_hit`, `damage` (the text the
/// screen shows), and `range` (null for a blow).
fn attacks_json(view: &PerceptionView<'_>) -> Value {
    (view.own_attacks().iter())
        .map(|a| {
            json!({
                "source": a.source.id(),
                "kind": a.kind,
                "name": a.name,
                "to_hit": a.to_hit,
                "damage": a.damage_text(view.content()),
                "range": a.range,
            })
        })
        .collect()
}

/// The player's talents in the order taken, one a line: its name for its choice and, for a
/// ranked talent, the rank held of how many (`Hardy 2/3`); a race's and a birth sign's gifts
/// first, marked `(race)` and `(sign)`, the sign's flaw `(sign, flaw)`; a forgotten altar's
/// `(altar)` and `(altar, flaw)`.
fn talent_rows(view: &PerceptionView<'_>) -> Vec<String> {
    (view.own_talents().iter())
        .map(|t| {
            let name = if t.ranks > 1 {
                format!("{} {}/{}", t.name, t.rank, t.ranks)
            } else {
                t.name.clone()
            };
            match (t.granted, t.flaw, t.class) {
                _ if t.boon && t.flaw => format!("{name} (altar, flaw)"),
                _ if t.boon => format!("{name} (altar)"),
                (_, true, _) => format!("{name} (sign, flaw)"),
                (true, false, _) if t.race => format!("{name} (race)"),
                (true, false, _) if t.spent => format!("{name} (sign, spent)"),
                (true, false, _) => format!("{name} (sign)"),
                (false, false, true) => format!("{name} (class)"),
                _ => name,
            }
        })
        .collect()
}

/// The player's progression (dev): the choices due, the talents they may take now with the
/// choices each may be taken for, and the creature types whose first kill has paid.
fn progress_rows(world: &World) -> Vec<String> {
    let view = world.perception();
    let content = view.content();
    let progress = &world.state().progress;
    let mut rows: Vec<String> = (progress.due.iter())
        .map(|due| {
            let count = |n: u8, what: &str| match n {
                0 => None,
                1 => Some(format!("1 {what}")),
                n => Some(format!("{n} {what}s")),
            };
            let parts: Vec<String> = [
                count(due.rolls, "skill roll"),
                count(due.talents, "talent"),
                count(due.bonus, "bonus talent"),
            ]
            .into_iter()
            .flatten()
            .collect();
            format!("due level {}: {}", due.level, parts.join(", "))
        })
        .collect();
    // What the level-up screen shows of skill rolls (#543): where a level's come from, and
    // the dice a roll on each trained skill adds (`/2`: halved past the soft cap).
    let count = view.roll_count();
    if count.total() > 0 {
        rows.push(format!(
            "rolls a level {} (class {}, race {}, intelligence {}, talents {})",
            count.total(),
            count.class,
            count.race,
            count.intelligence,
            count.talents
        ));
    }
    let dice: Vec<String> = (view.own_skills().iter())
        .filter_map(|s| {
            let roll = view.skill_roll(s.skill)?;
            let half = if roll.halved { "/2" } else { "" };
            let id = &content.skill(s.skill).id;
            Some(format!("{id} {}{half}", roll.dice()))
        })
        .collect();
    if !dice.is_empty() {
        rows.push(format!("roll dice {}", dice.join(", ")));
    }
    let takeable: Vec<String> = (view.talents().iter())
        .filter(|t| !t.openings.is_empty())
        .map(|t| {
            let id = &content.talents().get(t.id).id;
            let ways: Vec<&str> = t.openings.iter().flatten().map(String::as_str).collect();
            if ways.is_empty() {
                id.clone()
            } else {
                format!("{id}:{{{}}}", ways.join(","))
            }
        })
        .collect();
    if !takeable.is_empty() {
        rows.push(format!("takeable {}", takeable.join(", ")));
    }
    let kills: Vec<&str> = progress
        .discovered
        .kills
        .iter()
        .map(String::as_str)
        .collect();
    if !kills.is_empty() {
        rows.push(format!("discovered {}", kills.join(", ")));
    }
    // Beast-friend: the animal types the player attacked, theirs to fight again.
    let broken: Vec<&str> = (progress.empathy_broken.iter())
        .map(String::as_str)
        .collect();
    if !broken.is_empty() {
        rows.push(format!("empathy broken {}", broken.join(", ")));
    }
    // Rolled quests: what each drew, of how many it could draw now; the named monsters dead.
    for (id, roll, pool) in world.quest_rolls() {
        let variant = roll.variant.map_or_else(String::new, |v| format!(" ({v})"));
        rows.push(format!(
            "roll {id} {}{variant} x{} of {pool}",
            roll.target, roll.count
        ));
    }
    let slain: Vec<&str> = world.quests().slain().collect();
    if !slain.is_empty() {
        rows.push(format!("slain {}", slain.join(", ")));
    }
    let marks = &world.state().player().stat_marks;
    let marked: Vec<String> = (marks.iter())
        .filter(|&(_, n)| n > 0)
        .map(|(stat, n)| format!("{} {n}", stat.abbrev()))
        .collect();
    if !marked.is_empty() {
        rows.push(format!("stat marks {}", marked.join(", ")));
    }
    // The study check's exact odds (spec §8 "Spells"), for each scroll the player could study.
    for study in view.studies() {
        if let Some(odds) = world.player_study_odds(study.spell) {
            let check = match odds.check {
                None => "certain".to_string(),
                Some((bonus, difficulty)) => {
                    format!("{bonus:+} against {difficulty}, {}/144", odds.ways())
                }
            };
            rows.push(format!(
                "study {} {}: overreach {}, {check}",
                position_text(study.item),
                world.content().spell_of(study.spell).map_or("", |d| &d.id),
                odds.overreach
            ));
        }
    }
    rows
}

/// The spells the player knows, one a line with its tier and cost, then the scrolls they could
/// study with the chance in words.
fn spell_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let known = (view.own_spells().into_iter()).filter_map(|spell| {
        let def = content.spell_of(spell)?;
        let cost = view.own_spell_cost(spell)?;
        Some(format!("{}: tier {}, {cost} mana", def.name, def.tier))
    });
    let studies = (view.studies().into_iter()).filter_map(|s| {
        let def = content.spell_of(s.spell)?;
        let stack = view.inventory().get(s.item)?;
        Some(format!(
            "study {}, {}: {}, {}",
            position_text(s.item),
            view.lore().describe(&stack.single()),
            def.name,
            s.chance
        ))
    });
    let rows: Vec<String> = known.chain(studies).collect();
    if rows.is_empty() {
        vec!["no spells".into()]
    } else {
        rows
    }
}

/// The spells the player knows (`[{"id", "tier", "cost"}]`) and the scrolls they could study
/// (`[{"letter", "spell", "chance"}]`).
fn spells_json(view: &PerceptionView<'_>) -> (Value, Value) {
    let content = view.content();
    let known: Vec<Value> = (view.own_spells().into_iter())
        .filter_map(|spell| {
            let def = content.spell_of(spell)?;
            Some(json!({
                "id": def.id,
                "tier": def.tier,
                "cost": view.own_spell_cost(spell),
            }))
        })
        .collect();
    let studies: Vec<Value> = (view.studies().into_iter())
        .map(|s| {
            json!({
                "letter": position_text(s.item),
                "spell": content.spell_of(s.spell).map(|d| d.id.clone()),
                "chance": s.chance,
            })
        })
        .collect();
    (Value::Array(known), Value::Array(studies))
}

/// The player's abilities, one a line in menu order: its name, its source, and what it asks for
/// or why it cannot be used now.
fn ability_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    (view.abilities().iter())
        .map(|own| {
            let a = own.ability;
            let state = match own.unavailable {
                Some(why) => why.reason(content),
                None => aim_word(a.aim(content)).into(),
            };
            let source = a.source().id().replace('_', " ");
            format!("{} ({source}): {state}", a.name(content))
        })
        .collect()
}

/// What an ability asks for, in a word or two.
fn aim_word(aim: Aim) -> &'static str {
    match aim {
        Aim::Nothing => "nothing",
        Aim::Direction => "a direction",
        Aim::Cell => "a cell",
        Aim::Ailment => "what to treat",
        Aim::Item => "an item you carry",
    }
}

/// The player's abilities in menu order: `[{"id", "source", "aim", "unavailable"}]`
/// (`unavailable` the reason's id, `null` when it can be used now).
fn abilities_json(view: &PerceptionView<'_>) -> Value {
    let content = view.content();
    (view.abilities().iter())
        .map(|own| {
            let a = own.ability;
            json!({
                "id": a.id(content),
                "source": a.source().id(),
                "aim": aim_word(a.aim(content)),
                "unavailable": own.unavailable.map(|why| why.id(content)),
            })
        })
        .collect()
}

/// The player's quest log, one quest a line: its name, whether it is active or done, its giver,
/// what it asks, and how far it has got.
fn quest_rows(view: &PerceptionView<'_>) -> Vec<String> {
    (view.quests().iter())
        .map(|q| {
            let stage = if q.active { "active" } else { "done" };
            let status = if q.active {
                format!(" {}", q.status)
            } else {
                String::new()
            };
            format!(
                "{} ({stage}, from {}): {}{status}",
                q.name, q.giver, q.objective
            )
        })
        .collect()
}

/// An actor's weapon skills (ground truth): `daggers 2 (13/30), thrown 1 (0/10)`, the trained
/// ones only; `None` without any.
fn actor_weapon_skills(
    content: &crate::content::Content,
    a: &crate::world::Actor,
) -> Option<String> {
    let table = content.weapon_skills();
    let skills: Vec<String> = (a.weapon_skills.trained())
        .map(|(id, t)| match table.need(id, t.rank) {
            Some(need) => format!("{} {} ({}/{need})", table.skill(id).id, t.rank, t.marks),
            None => format!("{} {}", table.skill(id).id, t.rank),
        })
        .collect();
    (!skills.is_empty()).then(|| skills.join(", "))
}

/// Every actor on the level (ground truth): id, creature, position, hit points, Defence and
/// Protection (`dv 11 pv 3`), its alignment (`alignment evil`; the player's score and band,
/// `alignment 450 (good)`), the court it joined for a type of several factions (`court
/// unseelie`), weight, what it carries, its burden and carrying limit, its stats
/// as score/potential
/// (`St 12/15`), its trained skills with their marks and the die those set on the ladder
/// (`stealth 17 (7 1d3)`), its status effects with their severities (`effects bleeding 2`), the
/// player's nutrition and hunger (`nutrition 3540 (not_hungry)`), and, if it has any, the unique
/// character it is and how far its talk has got; then all food on the level and in the packs of
/// those on it, with when it was made (a corpse: when its creature died) and how it has rotted
/// (`corpse kobold_corpse at (12, 5): died 1444-11-11 08:12:00, 2 hours ago, fresh`, `food
/// bread*2 carried by 0: made ..., 70 hours ago, old`): the stage food never shows the player.
fn actor_rows(world: &World) -> Vec<String> {
    let content = world.content();
    let (level, actors) = world.ground_truth();
    let now = world.time();
    let food = |where_: String, s: &crate::object::Stack| {
        let made = s.died?;
        let stage = crate::rot::stage(content, s, now)?;
        let (what, verb) = match content.object(s.object).corpse {
            Some(_) => ("corpse", "died"),
            None => ("food", "made"),
        };
        let count = match s.count {
            1 => String::new(),
            n => format!("*{n}"),
        };
        Some(format!(
            "{what} {}{count} {where_}: {verb} {made}, {} ago, {}",
            content.object(s.object).id,
            now.since(made).spoken(),
            stage.id()
        ))
    };
    let on_level = (level.objects.iter())
        .flat_map(|(p, pile)| pile.iter().map(move |s| (*p, s)))
        .filter_map(|(p, s)| food(format!("at {p}"), s));
    let carried = (world.ground_truth().1)
        .flat_map(|a| a.inventory.iter().map(move |s| (a.id, s)))
        .filter_map(|(id, s)| food(format!("carried by {}", id.0), s))
        .collect::<Vec<_>>();
    let foods: Vec<String> = on_level.chain(carried).collect();
    let images = world.images();
    let rows: Vec<String> = actors
        .map(|a| {
            let w = a.weight(content);
            let load = world.actor_load(a.id).expect("the actor exists");
            let carried: Vec<String> = (a.inventory.iter())
                .map(|s| s.describe_truth(content))
                .collect();
            let carried = if carried.is_empty() {
                "nothing".to_string()
            } else {
                carried.join(", ")
            };
            let equipped: Vec<String> = (a.equipment.items().into_iter())
                .filter_map(|(slot, item)| {
                    let stack = a.inventory.get(item)?;
                    Some(format!(
                        "{} {}",
                        content.slot(slot).id,
                        stack.describe_truth(content)
                    ))
                })
                .collect();
            let equipped = if equipped.is_empty() {
                String::new()
            } else {
                format!("; equipped {}", equipped.join(", "))
            };
            // What its magic items do to it (spec §8 "Magic items"), by effect id.
            let mods = a.mods(content);
            let mut magic: Vec<String> = (mods.iter())
                .map(|(effect, value)| format!("{} {value:+}", effect.id()))
                .collect();
            if let Some(light) = mods.light {
                magic.push(format!("light {}", content.light(light).id));
            }
            for &(skill, value) in mods.skills() {
                magic.push(format!("skill {} {value:+}", content.skill(skill).id));
            }
            for &(effect, pool, value) in mods.pools() {
                let pool = &content.resource(pool).id;
                magic.push(format!("{} {pool} {value:+}", effect.id()));
            }
            let equipped = if magic.is_empty() {
                equipped
            } else {
                format!("{equipped}; magic {}", magic.join(" "))
            };
            let stats: Vec<String> = Stat::ALL
                .iter()
                .map(|&s| {
                    let (score, potential) = (a.stats.score(s), a.stats.potential(s));
                    format!("{} {score}/{potential}", s.abbrev())
                })
                .collect();
            let skills: Vec<String> = (a.skills.iter())
                .map(|(id, t)| {
                    let ladder = content.ladder();
                    let die = ladder.die(ladder.step(t.marks));
                    format!("{} {} ({} {die})", content.skill(id).id, t.value, t.marks)
                })
                .collect();
            let skills = if skills.is_empty() {
                String::new()
            } else {
                format!("; skills {}", skills.join(", "))
            };
            let hunger = match (a.satiation, a.hunger(content)) {
                (Some(s), Some(h)) => format!("; nutrition {} ({})", s.nutrition, h.id()),
                _ => String::new(),
            };
            let effects: Vec<String> = (a.effects.iter())
                .map(|e| format!("{} {}", content.status(e.status).id, e.severity))
                .collect();
            let effects = if effects.is_empty() {
                String::new()
            } else {
                format!("; effects {}", effects.join(", "))
            };
            let diseases = disease_text(a, content);
            // Its alternative resources (spec §8 "Resources"): value/maximum, and the progress
            // toward the next point over time in hundredths of a second.
            let resources: Vec<String> = (a.resources.iter())
                .map(|p| {
                    let id = &content.resource(p.resource).id;
                    let max = a.resource_max(content, p.resource, world.holder_level(a.id));
                    format!("{id} {}/{max} ({})", p.value, p.progress)
                })
                .collect();
            let resources = if resources.is_empty() {
                String::new()
            } else {
                format!(" resources {}", resources.join(", "))
            };
            let weapons = actor_weapon_skills(content, a)
                .map_or(String::new(), |s| format!("; weapon skills {s}"));
            // Who it is and how far its talk has got (spec §8 "Dialogue"), if anything.
            let npc = (a.npc.as_ref()).map_or(String::new(), |n| format!("; npc {n}"));
            // Where it stands between good and evil (spec §8 "Alignment"): its score and band
            // (the player's), or its nature.
            let alignment = match a.alignment {
                Some(score) => format!("; alignment {} ({})", score.value(), a.band(content).id()),
                None => format!("; alignment {}", a.nature(content).id()),
            };
            // The court it joined, for a type of several factions (spec §7 "Factions": the fey).
            let court = match a.faction(content) {
                Some(f) if !content.creature(a.creature).factions.is_empty() => {
                    format!("; court {}", content.factions().def(f).id)
                }
                _ => String::new(),
            };
            // Who summoned it (spec §8 "Summoning"), and so whose side it took.
            let court = match a.summoner {
                Some(by) => format!("{court}; summoned by {by}"),
                None => court,
            };
            let talk = if a.talk.is_new() {
                String::new()
            } else {
                let met = if a.talk.met { ", met" } else { "" };
                let (said, muttered) = (a.talk.said, a.talk.muttered);
                format!("; talk said {said}, muttered {muttered}{met}")
            };
            // What it resists, in percent by kind (ground truth: never shown in play).
            let resists: Vec<String> = (world.resistances_of(a.id).iter())
                .map(|(k, p)| format!("{} {p}", content.damage_type(k).id))
                .collect();
            let resists = if resists.is_empty() {
                String::new()
            } else {
                format!("; resists {}", resists.join(" "))
            };
            // Its reflections in a hall of mirrors (`world/mirrors.rs`), wherever the player is.
            let mirrored: Vec<String> = (images.iter())
                .filter(|i| i.of == a.id)
                .map(|i| i.pos.to_string())
                .collect();
            let resists = if mirrored.is_empty() {
                resists
            } else {
                format!("{resists}; images {}", mirrored.join(" "))
            };
            // Its caster level (spec §8 "Spells"), for an actor with mana: its magic items'
            // `caster_level` included, as for the player.
            let max_mana = a.max_mana(content);
            let caster = if max_mana > 0 {
                format!(" cl {}", world.caster_level(a.id))
            } else {
                String::new()
            };
            format!(
                "{} {} at {}: hp {}/{} mp {}/{}{caster}{resources} dv {} pv {}{alignment}{court}; weight {} (base {} + carried {}: \
                 {carried}){equipped}; {} (limit {}){weapons}; stats {}{skills}{effects}{diseases}\
                 {hunger}{npc}{talk}{resists}",
                a.id,
                content.creature(a.creature).id,
                a.pos,
                a.hp,
                a.max_hp(content),
                a.mana,
                max_mana,
                world.defence_of(a.id).total(),
                world.protection_of(a.id).base,
                format_weight(w.total()),
                format_weight(w.base),
                format_weight(w.carried),
                load.burden().id(),
                format_weight(load.limit),
                stats.join(" "),
            )
        })
        .collect();
    rows.into_iter().chain(foods).collect()
}

/// An actor's diseases, stat damage, and drained points, as the `actors` dump shows them
/// (ground truth; `; drained St 2` after the rest):
/// `; diseases filth_fever incubating 2d, red_ache 3 (1 won) 1d; damage St 2`, each infection
/// with its progressions so far (and checks won in a row) and the time to its next turn, in
/// days and hours; empty without any.
fn disease_text(a: &crate::world::Actor, content: &crate::content::Content) -> String {
    let mut out = String::new();
    let infections: Vec<String> = (a.diseases.iter())
        .map(|i| {
            let id = &content.disease(i.disease).id;
            let left = u64::from(i.left);
            let (days, hours) = (left / crate::time::DAY, left % crate::time::DAY / 3600);
            let stage = if i.shown {
                let won = match i.streak {
                    0 => String::new(),
                    n => format!(" ({n} won)"),
                };
                format!("{}{won}", i.progressions)
            } else {
                "incubating".to_string()
            };
            format!("{id} {stage} {days}d{hours}h")
        })
        .collect();
    if !infections.is_empty() {
        let _ = write!(out, "; diseases {}", infections.join(", "));
    }
    let damaged: Vec<String> = (Stat::ALL.iter())
        .filter(|&&s| a.damage.get(s) > 0)
        .map(|&s| format!("{} {}", s.abbrev(), a.damage.get(s)))
        .collect();
    if !damaged.is_empty() {
        let _ = write!(out, "; damage {}", damaged.join(" "));
    }
    let drained: Vec<String> = (Stat::ALL.iter())
        .filter(|&&s| a.drained[s] > 0)
        .map(|&s| format!("{} {}", s.abbrev(), a.drained[s]))
        .collect();
    if !drained.is_empty() {
        let _ = write!(out, "; drained {}", drained.join(" "));
    }
    out
}

fn rows(width: i32, height: i32, glyph: impl Fn(Pos) -> char) -> Vec<String> {
    (0..height)
        .map(|y| {
            let row: String = (0..width).map(|x| glyph(Pos::new(x, y))).collect();
            row.trim_end().to_string()
        })
        .collect()
}

/// One-line summary used as the dump header; under the open sky it names the weather (and the
/// weather coming, `then rain`, for a Weather-wise player), and it has the temperature where the
/// player is (and `dazzled` while strong light dazzles them).
fn summary(world: &World) -> String {
    let view = world.perception();
    // Spelled as in the JSON dump (`heavy_rain`).
    let spelled = |w: Weather| serde_json::to_value(w).ok()?.as_str().map(str::to_string);
    let weather = (view.weather())
        .and_then(spelled)
        .map(|w| format!(" weather {w}"))
        .unwrap_or_default();
    // And the weather coming, for a Weather-wise player.
    let forecast = (view.forecast())
        .and_then(spelled)
        .map(|w| format!(" then {w}"))
        .unwrap_or_default();
    let temperature = view.temperature();
    let (hp, max) = view.own_hp();
    let (mp, max_mp) = view.own_mana();
    // The resources the player holds, as the status bar shows them (spec §8 "Resources").
    let resources: String = (view.own_resources().iter())
        .map(|r| {
            let id = &view.content().resource(r.resource).id;
            format!(" {id} {}/{}", r.value, r.max)
        })
        .collect();
    let dead = if world.death().is_some() { " dead" } else { "" };
    // As the status bar shows it: only when it is not Not hungry.
    let hunger = (view.own_hunger())
        .filter(|h| *h != Hunger::NotHungry)
        .map_or(String::new(), |h| format!(" hunger {}", h.id()));
    let dazzled = if view.dazzled() { " dazzled" } else { "" };
    format!(
        "turn {} time {} level {} viewer {} hp {hp}/{max} mp {mp}/{max_mp}{resources}{dead}{hunger}{dazzled}{weather}{forecast} \
         temperature {temperature} hash {:016x}",
        world.turn(),
        world.time(),
        view.level_id(),
        view.viewer(),
        world.state_hash()
    )
}

/// Plain-text dump: a header line, then each requested layer under a `== name ==` heading.
/// Dev-only layers are headed like `== truth (DEV ONLY: ground truth) ==`.
pub fn ascii(world: &World, layers: &[Layer]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", summary(world));
    for &layer in layers {
        let note = if layer.is_dev_only() {
            " (DEV ONLY: ground truth)"
        } else {
            ""
        };
        let _ = writeln!(out, "== {}{note} ==", layer.name());
        for row in layer_rows(world, layer) {
            let _ = writeln!(out, "{row}");
        }
    }
    out
}

/// Structured dump: turn, game time, level, viewer, the special room the player stands in if they
/// know it (its effect id; else null), the open shop they stand in (`shop`: its sign, keeper,
/// and what they owe there; else null), the weather (under the open sky; else null), the
/// temperature (°C, to a tenth), seen actors (with the status effects that show on them), heard
/// sounds, known traps (`known_traps`: position, trap id, and whether in sight), the hollow walls
/// found (`known_hollows`: positions), the chests seen
/// or remembered (`chests`: position, band id, state, and, with Chest-wise, whether a shut one
/// in sight is locked, else null), known overworld
/// locations (`known_locations`: position and name), the player's inventory (with what of each
/// stack is unpaid, priced), equipment (by slot id), weight, scores (by stat id), status effects
/// (by id), the diseases they know (by id), recent messages, the requested layers as arrays of
/// rows, and optionally the full [`crate::WorldState`] under `state` (with the current level's
/// `danger`, ground truth).
pub fn json(world: &World, layers: &[Layer], include_state: bool) -> Value {
    let view = world.perception();
    let content = world.content();
    let seen: Vec<Value> = view
        .actors()
        .iter()
        .map(|a| {
            let def = content.creature(a.creature);
            let effects: Vec<&str> = (view.effects_of(a.id))
                .map(|s| content.status(s).id.as_str())
                .collect();
            let mut seen = json!({
                "id": a.id, "kind": a.kind, "creature": def.id, "glyph": a.glyph(content).to_string(),
                "fg": a.fg(content).to_string(), "pos": a.pos, "health": a.health, "effects": effects,
            });
            // A composite's colours, top band first (spec §8 "Animated objects").
            let bands = a.shape.as_ref().map_or(&[][..], |s| s.bands());
            if !bands.is_empty() {
                let bands: Vec<String> = bands.iter().map(ToString::to_string).collect();
                seen["bands"] = json!(bands);
            }
            seen
        })
        .collect();
    let messages: Vec<&crate::messages::Message> = {
        let all: Vec<_> = world.messages().iter().collect();
        all[all.len().saturating_sub(RECENT_MESSAGES)..].to_vec()
    };
    let unpaid = view.unpaid();
    let inventory: Vec<Value> = view
        .inventory()
        .iter()
        .enumerate()
        .map(|(i, s)| {
            // An item's affixes only once the player has identified it.
            let magic = s
                .known_magic()
                .map(|m| json!({ "level": m.level, "name": m.name, "affixes": m.affixes }));
            let mut entry = json!({
                "letter": position_text(i),
                "object": content.object(s.object).id,
                "class": content.item_class(view.item_class(s)).id,
                "count": s.count,
                "weight": s.weight(content),
                "name": view.lore().describe(s),
                "rarity": s.rarity(),
                "identified": view.lore().knows(s),
                "magic": magic,
                "numbers": numbers_json(s, content),
            });
            if let Some(price) = unpaid.get(i).copied().flatten() {
                entry["unpaid"] = json!(price);
            }
            entry
        })
        .collect();
    let shop = view
        .shop_here()
        .map(|s| json!({ "sign": s.sign(), "keeper": s.keeper(), "owed": s.owed() }));
    let equipment: serde_json::Map<String, Value> = (view.equipment().iter())
        .filter_map(|(slot, item)| {
            let s = view.inventory().get(item)?;
            let entry = json!({
                "letter": position_text(item),
                "object": content.object(s.object).id,
                "count": s.count,
            });
            Some((content.slot(slot).id.clone(), entry))
        })
        .collect();
    let weight = view.own_weight();
    let load = view.own_load();
    let (hp, max_hp) = view.own_hp();
    let (mana, max_mana) = view.own_mana();
    let death = world.death().map(|d| {
        json!({
            "turn": d.turn,
            "time": d.time.to_string(),
            "place": d.place,
            "by": d.by.map(|c| content.creature(c).id.clone()),
            "trap": match d.cause {
                Some(Cause::Trap(t)) => Some(content.trap(t).id.clone()),
                _ => None,
            },
            "cause": d.cause.map(|c| c.epitaph(content)),
            "of": d.of.map(|s| content.status(s).id.clone()),
        })
    });
    let effects: Vec<&str> = (view.own_effects())
        .map(|s| content.status(s).id.as_str())
        .collect();
    let diseases: Vec<&str> = (view.own_diseases())
        .map(|d| content.disease(d).id.as_str())
        .collect();
    let table = content.weapon_skills();
    let weapon_skills: serde_json::Map<String, Value> = (table.ids())
        .filter(|&(id, _)| view.own_weapon_skills().shows(table, id))
        .map(|(id, def)| {
            let rank = view.own_weapon_skills().rank(id);
            let progress = progress_percent(&view, id);
            (
                def.id.clone(),
                json!({ "rank": rank, "progress": progress }),
            )
        })
        .collect();
    let traps: Vec<Value> = (view.known_traps().into_iter())
        .map(|(p, trap)| {
            json!({ "pos": p, "trap": content.trap(trap).id, "seen": view.sees_objects_at(p) })
        })
        .collect();
    let locations: Vec<Value> = (view.known_locations().into_iter())
        .map(|(p, name)| json!({ "pos": p, "name": name }))
        .collect();
    // The chests the player sees or remembers (spec §8 "Chests"), as they see them.
    let chests: Vec<Value> = (view.known_chests().into_iter())
        .map(|(p, look)| {
            let band = (content.chests().bands.get(look.band)).map(|b| b.id.as_str());
            json!({ "pos": p, "band": band, "state": look.state.id(), "locked": look.locked })
        })
        .collect();
    let layer_map: serde_json::Map<String, Value> = layers
        .iter()
        .map(|&l| (l.name().to_string(), json!(layer_rows(world, l))))
        .collect();
    let talents: Vec<Value> = (view.own_talents().iter())
        .map(|t| {
            json!({
                "id": content.talents().get(t.id).id,
                "name": t.name,
                "rank": t.rank,
                "ranks": t.ranks,
                "choice": t.choice,
                "granted": t.granted,
                "flaw": t.flaw,
                "class": t.class,
            })
        })
        .collect();
    let mut out = json!({
        "game_version": crate::GAME_VERSION,
        "scenario": world.scenario(),
        "seed": world.seed(),
        "turn": world.turn(),
        "time": time_json(world),
        "level": view.level_id(),
        "viewer": view.viewer(),
        "room": view.room(view.viewer()).map(|e| e.id.as_str()),
        "pervading": view.pervading().map(|e| e.id.as_str()),
        "shop": shop,
        "weather": view.weather(),
        // In °C, to a tenth of a degree.
        "temperature": f64::from(view.temperature().tenths()) / 10.0,
        "hash": format!("{:016x}", world.state_hash()),
        "seen_actors": seen,
        "heard_sounds": view.sounds(),
        "known_traps": traps,
        "known_hollows": view.known_hollows(),
        "known_locations": locations,
        "chests": chests,
        "inventory": inventory,
        "equipment": equipment,
        "weight": { "base": weight.base, "carried": weight.carried, "total": weight.total() },
        "burden": { "level": load.burden(), "limit": load.limit },
        "character": view.character().map(|c| {
            let mut character = json!({
                "name": c.name,
                "race": c.spec.race,
                "gender": c.spec.gender,
                "class": c.spec.class,
                "raised": c.raised,
                "age": c.age_on(view.time()),
                "born": c.born,
                "sign": c.sign,
            });
            // A lucky day's title (spec §8 "Birth sign"), only for one born on it.
            if let Some(title) = c.title(content) {
                character["lucky"] = json!(title);
            }
            character
        }),
        "stats": view.own_scores(),
        "alignment": view.own_alignment().map(|b| b.id()),
        "skills": skills_json(&view),
        "effects": effects,
        "diseases": diseases,
        "weapon_skills": weapon_skills,
        "styles_in_use": (view.own_styles_in_use().into_iter())
            .map(|id| content.weapon_skills().skill(id).id.clone())
            .collect::<Vec<_>>(),
        "attacks": attacks_json(&view),
        "talents": talents,
        "due": view.due(),
        "abilities": abilities_json(&view),
        "spells": spells_json(&view).0,
        "studies": spells_json(&view).1,
        "quests": quests_json(&view),
    });
    // A second `json!`: one macro call this long passes the recursion limit.
    let Value::Object(rest) = json!({
        "hp": { "current": hp, "max": max_hp },
        "mana": { "current": mana, "max": max_mana },
        "caster_level": view.own_caster_level(),
        "resources": (view.own_resources().iter())
            .map(|r| json!({
                "id": content.resource(r.resource).id,
                "current": r.value,
                "max": r.max,
            }))
            .collect::<Vec<_>>(),
        "experience": view.own_experience(),
        "defence": view.own_defence(),
        "protection": view.own_protection(),
        "resistances": (view.own_resistances().into_iter())
            .map(|(k, w)| (content.damage_type(k).id.clone(), json!(w.word())))
            .collect::<serde_json::Map<String, Value>>(),
        "hunger": view.own_hunger(),
        "dazzled": view.dazzled(),
        "death": death,
        "messages": messages,
        "layers": layer_map,
    }) else {
        unreachable!("an object literal")
    };
    out.as_object_mut().expect("an object").extend(rest);
    // The guilds they belong to and their ranks (left out before joining one).
    let guilds = view.own_guilds();
    if !guilds.is_empty() {
        out["guilds"] = (guilds.iter())
            .map(|g| json!({ "id": g.guild.id, "name": g.guild.name, "rank": g.title() }))
            .collect();
    }
    // The weather coming, which a Weather-wise player knows (left out otherwise).
    if let Some(coming) = view.forecast() {
        out["forecast"] = json!(coming);
    }
    if include_state {
        out["state"] = serde_json::to_value(world.state()).expect("state serializes");
        // The current level's danger (spec §7 "Danger levels"): ground truth, never stored.
        out["danger"] = json!(world.danger(&world.state().current_level));
    }
    out
}

/// The player's own trained skills by skill id, with their values: `{"stealth": 17}`.
fn skills_json(view: &PerceptionView<'_>) -> Value {
    let content = view.content();
    let map: serde_json::Map<String, Value> = (view.own_skills().iter())
        .map(|s| (content.skill(s.skill).id.clone(), json!(s.value)))
        .collect();
    Value::Object(map)
}

/// The player's quest log: `[{"id", "name", "kind", "active", "completed", "giver",
/// "objective", "target", "status"}]`, active quests first (`target`: what a rolled quest drew,
/// else null).
fn quests_json(view: &PerceptionView<'_>) -> Value {
    (view.quests().iter())
        .map(|q| {
            json!({
                "id": q.id,
                "name": q.name,
                "kind": q.kind.id(),
                "active": q.active,
                "completed": q.completed,
                "giver": q.giver,
                "objective": q.objective,
                "target": q.target,
                "status": q.status,
            })
        })
        .collect()
}

/// The clock in seconds since the epoch, and as a calendar date and time of day.
fn time_json(world: &World) -> Value {
    let t = world.time();
    json!({
        "seconds": t.seconds(),
        "text": t.to_string(),
        "year": t.year(),
        "month": t.month(),
        "month_name": world.content().calendar().month_name(t),
        "day": t.day(),
        "hour": t.hour(),
        "minute": t.minute(),
        "second": t.second(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, parse_actions};
    use crate::content::Content;
    use std::sync::Arc;

    /// Walks north out of the start room and through the first door, then along the corridor.
    const SHOWCASE_RUN: &str = "move:N move:N move:N move:N move:N move:N move:E move:E";

    fn showcase() -> World {
        let content = Arc::new(Content::builtin().expect("builtin content"));
        World::new(content, "showcase", None).expect("world")
    }

    fn run(world: &mut World, actions: &str) {
        for action in parse_actions(actions).expect("actions") {
            world.apply(&action);
        }
    }

    /// A dump with its state hash redacted, for the insta snapshots: the hash covers every
    /// field of the state and changes with any content or rules change, so leaving it in would
    /// churn the snapshots on every feature (determinism is `replay.rs`'s tests' job).
    fn snapshot(world: &World, layers: &[Layer]) -> String {
        let text = ascii(world, layers);
        let (header, rest) = text.split_once('\n').expect("a header line");
        let (before, hash) = header.rsplit_once(" hash ").expect("a hash in the header");
        assert_eq!(hash.len(), 16, "{header}");
        format!("{before} hash [redacted]\n{rest}")
    }

    #[test]
    fn parses_layer_lists() {
        assert_eq!(
            parse_layers("view, fov,truth"),
            Ok(vec![Layer::View, Layer::Fov, Layer::Truth])
        );
        assert_eq!(parse_layers("light"), Ok(vec![Layer::Light]));
        assert!(parse_layers("view,lights").is_err());
    }

    #[test]
    fn view_hides_unseen_actors_that_truth_shows() {
        let world = showcase();
        let view = layer_rows(&world, Layer::View).join("\n");
        let truth = layer_rows(&world, Layer::Truth).join("\n");
        assert!(truth.contains('r') || truth.contains('k'));
        assert_eq!(view.matches('@').count(), 1);
        assert!(view.chars().filter(|c| *c != '\n').all(|c| c != 'k'));
    }

    #[test]
    fn json_has_documented_fields() {
        let mut world = showcase();
        world.apply(&Action::Wait);
        let value = json(&world, &[Layer::View, Layer::Fov], true);
        assert_eq!(value["turn"], 1);
        assert_eq!(value["time"]["text"], "1444-11-11 08:00:06");
        assert_eq!(value["time"]["month_name"], "Darkening");
        assert_eq!(value["level"], "hub");
        assert_eq!(value["layers"]["fov"].as_array().map(Vec::len), Some(21));
        assert!(value["layers"].get("memory").is_none());
        assert_eq!(value["state"]["turn"], 1);
        assert!(value["room"].is_null(), "not in a special room");
        assert!(value["shop"].is_null(), "not in a shop");
        assert!(value["seen_actors"][0]["kind"] == "player");
        // The player's character, and their own scores by stat id; nothing else about stats.
        assert_eq!(
            value["character"],
            json!({ "name": "Tamsin", "race": "human", "gender": "female", "class": "thief",
                    "raised": "dexterity", "age": 34,
                    "born": { "year": 1410, "month": 5, "day": 29 },
                    "sign": { "animal": "basilisk", "element": "water" } })
        );
        assert_eq!(
            value["stats"],
            json!({ "strength": 9, "dexterity": 13, "toughness": 10, "intelligence": 12,
                    "willpower": 11, "presence": 9 })
        );
        // Their own trained skills by skill id, with their values: no marks, nothing untrained.
        assert_eq!(
            value["skills"],
            json!({ "appraise": 19, "detect_traps": 7, "disable_device": 7, "escape_artist": 10,
                    "listen": 6, "open_lock": 18, "search": 10, "spot": 9, "steal": 16, "stealth": 16,
                    "tumble": 17 })
        );
        assert!(json(&world, &[], false).get("state").is_none());
    }

    #[test]
    fn skills_show_the_players_trained_skills_only() {
        let world = showcase();
        assert_eq!(
            layer_rows(&world, Layer::Skills),
            [
                "Appraise 19",
                "Detect traps 7",
                "Disable device 7",
                "Escape artist 10",
                "Listen 6",
                "Open lock 18",
                "Search 10",
                "Spot 9",
                "Steal 16",
                "Stealth 16",
                "Tumble 17"
            ]
        );
        assert_eq!(
            parse_layers("skills,stats"),
            Ok(vec![Layer::Skills, Layer::Stats])
        );
        assert!(!Layer::Skills.is_dev_only());
        // A creature's skills show only in the dev layer: the arena's shaman's.
        let content = Arc::new(Content::builtin().expect("content"));
        let arena = World::new(content, "arena", None).expect("world");
        let actors = layer_rows(&arena, Layer::Actors);
        let shaman = actors.iter().find(|a| a.contains(" kobold_shaman at "));
        assert!(
            shaman.is_some_and(|s| s.ends_with(
                "; skills concentration 20 (0 +1), spellcraft 25 (0 +1); resists fire 25"
            )),
            "{actors:#?}"
        );
        let text = ascii(&arena, &[Layer::Skills, Layer::View]);
        assert!(
            !text.contains("spellcraft") && !text.contains("Spellcraft"),
            "{text}"
        );
    }

    #[test]
    fn showcase_snapshots() {
        let mut world = showcase();
        let layers = [Layer::View, Layer::Fov];
        insta::assert_snapshot!("showcase_start", snapshot(&world, &layers));
        run(&mut world, &SHOWCASE_RUN[..SHOWCASE_RUN.len() / 2]);
        insta::assert_snapshot!("showcase_mid", snapshot(&world, &layers));
        let mut world = showcase();
        run(&mut world, SHOWCASE_RUN);
        insta::assert_snapshot!(
            "showcase_end",
            snapshot(&world, &[Layer::View, Layer::Memory])
        );
    }

    #[test]
    fn stats_show_the_players_scores_and_actors_show_everyones() {
        let world = showcase();
        assert_eq!(
            layer_rows(&world, Layer::Stats),
            [
                "Name Tamsin",
                "Race Human",
                "Gender Female",
                "Class Thief",
                "Age 34",
                "Born 29 Greenleaf 1410",
                "Sign Water Basilisk",
                "Strength 9",
                "Dexterity 13",
                "Toughness 10",
                "Intelligence 12",
                "Willpower 11",
                "Presence 9",
                "Caster level 1",
                "Alignment Callous"
            ]
        );
        // Other names the stat it raised.
        let content = Arc::new(Content::builtin().expect("content"));
        let other = "gnome,other,bard".parse().expect("a character");
        let gnome =
            World::with_character(content, "showcase", Some(3), Some(&other)).expect("world");
        let raised = crate::character::other_stat(3).name();
        assert!(
            layer_rows(&gnome, Layer::Stats).contains(&format!("Gender Other ({raised} +1)")),
            "{:?}",
            layer_rows(&gnome, Layer::Stats)
        );
        let actors = layer_rows(&world, Layer::Actors);
        // Every actor's stats, then its trained skills with their marks and the ladder's die.
        assert!(
            actors[0].contains(
                "stats St 9/16 Dx 13/16 To 10/16 In 12/16 Wi 11/16 Pr 9/16; skills appraise 19 \
                 (0 +1), detect_traps 7 (0 +1), disable_device 7 (0 +1), escape_artist 10 (0 +1), \
                 listen 6 (0 +1), open_lock 18 (0 +1), search 10 (0 +1), spot 9 (0 +1), steal 16 (0 +1), stealth 16 \
                 (0 +1), tumble 17 (0 +1); \
                 nutrition 3600 (not_hungry)"
            ),
            "{}",
            actors[0]
        );
        let kobold = actors.iter().find(|a| a.contains(" kobold at "));
        assert!(
            kobold.is_some_and(|k| k.ends_with("St 9/9 Dx 13/13 To 10/10 In 10/10 Wi 9/9 Pr 8/8")),
            "{actors:#?}"
        );
        assert!(Layer::Actors.is_dev_only() && !Layer::Stats.is_dev_only());
    }

    #[test]
    fn weapon_skills_show_the_players_ranks_and_progress_and_actors_show_everyones() {
        // The showcase thief: daggers 2 and thrown 1 from the class, and (the seed's pick)
        // Duelist 1. With nothing in their hands they fight as a brawler.
        let mut world = showcase();
        let rows = layer_rows(&world, Layer::WeaponSkills);
        assert_eq!(rows.len(), 28);
        assert_eq!(rows[0], "Daggers and knives 2 (0%)");
        assert_eq!(rows[16], "Spell attacks 0 (0%)");
        assert_eq!(rows[17], "Duelist 1 (0%)");
        assert_eq!(rows[23], "Brawler 0 (0%), in use");
        assert!(
            rows.contains(&"Thrown weapons 1 (0%)".to_string()),
            "{rows:#?}"
        );
        assert!(rows.contains(&"Long blades 0 (0%)".to_string()));
        let actors = layer_rows(&world, Layer::Actors);
        assert!(
            actors[0].contains(
                "; weapon skills daggers 2 (0/60), thrown 1 (0/20), duelist 1 (0/30); stats"
            ),
            "{}",
            actors[0]
        );
        let kobold = actors.iter().find(|a| a.contains(" kobold at "));
        assert!(kobold.is_some_and(|k| k.contains("weapon skills spears 1 (0/30)")));
        // Marks show as a share of the next rank.
        let content = world.content().clone();
        let table = content.weapon_skills();
        let daggers = table.id("daggers").expect("daggers");
        let mut state = world.state().clone();
        let me = state.actors.get_mut(&world.player_id()).expect("player");
        me.weapon_skills.train(table, daggers, 24);
        world = World::from_state(content.clone(), state).expect("valid");
        assert_eq!(
            layer_rows(&world, Layer::WeaponSkills)[0],
            "Daggers and knives 2 (40%)"
        );
        let value = json(&world, &[], false);
        assert_eq!(value["weapon_skills"]["daggers"]["rank"], 2);
        assert_eq!(value["weapon_skills"]["daggers"]["progress"], 40);
        assert_eq!(value["weapon_skills"]["bows"]["rank"], 0);
        assert_eq!(value["weapon_skills"]["duelist"]["rank"], 1);
        assert_eq!(value["styles_in_use"], json!(["brawler"]));
        assert!(!Layer::WeaponSkills.is_dev_only());
    }

    #[test]
    fn abilities_show_what_the_player_can_do_and_why_not() {
        // The showcase thief: the class's skills used as actions (no use yet, but Disable
        // device's, disarming a trap, Open lock's, picking a lock, and Steal's, stealing, each in
        // a direction), the shield bash, which needs a shield in hand;
        // first their birth sign's stare (a Basilisk).
        let mut world = showcase();
        assert_eq!(
            layer_rows(&world, Layer::Abilities),
            [
                "Transfixing stare (sign): a cell",
                "Appraise (skill): not usable yet",
                "Disarm a trap (skill): a direction",
                "Pick lock (skill): a direction",
                "Steal (skill): a direction",
                "Shield bash (weapon skill): you hold no shield",
            ]
        );
        let value = json(&world, &[], false);
        assert_eq!(
            value["abilities"][5],
            json!({"id": "shield_bash", "source": "weapon_skill", "aim": "a direction",
                   "unavailable": "no_shield"})
        );
        assert_eq!(
            value["abilities"][2],
            json!({"id": "disarm", "source": "skill", "aim": "a direction", "unavailable": null})
        );
        assert_eq!(
            value["abilities"][3],
            json!({"id": "pick_lock", "source": "skill", "aim": "a direction",
                   "unavailable": null})
        );
        assert_eq!(value["abilities"][0]["aim"], "a cell");
        assert_eq!(
            value["abilities"][4],
            json!({"id": "steal", "source": "skill", "aim": "a direction", "unavailable": null})
        );
        // The storeroom's small shield in hand: it can be used, in a direction.
        let walk = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
                    pickup:a equip:off_hand:b";
        for action in crate::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        let rows = layer_rows(&world, Layer::Abilities);
        assert_eq!(rows[5], "Shield bash (weapon skill): a direction");
        assert_eq!(
            json(&world, &[], false)["abilities"][5]["unavailable"],
            Value::Null
        );
        assert!(!Layer::Abilities.is_dev_only());
    }

    #[test]
    fn light_digits_are_tenths_of_full_light() {
        let light = |rgb: [u16; 3], uv: u16| PerceivedLight {
            rgb,
            uv,
            darkvision: false,
            dist2: 0,
        };
        assert_eq!(light_digit(light([0, 0, 0], 0)), '0');
        assert_eq!(light_digit(light([25, 25, 25], 0)), '0');
        assert_eq!(light_digit(light([26, 26, 26], 0)), '1');
        assert_eq!(light_digit(light([128, 128, 128], 0)), '5');
        assert_eq!(light_digit(light([255, 255, 255], 0)), '9');
        assert_eq!(light_digit(light([900, 900, 900], 0)), '9');
        assert_eq!(
            light_digit(light([0, 0, 0], 128)),
            '5',
            "perceived UV counts"
        );
    }

    #[test]
    fn light_layer_shows_visible_cells_only() {
        let world = showcase();
        insta::assert_snapshot!("showcase_light", snapshot(&world, &[Layer::Light]));
        let rows = layer_rows(&world, Layer::Light);
        let fov = layer_rows(&world, Layer::Fov);
        for (light, fov) in rows.iter().zip(&fov) {
            assert_eq!(light.len(), fov.len());
            for (l, f) in light.chars().zip(fov.chars()) {
                assert_eq!(l.is_ascii_digit(), f == '*');
            }
        }
    }
}
