//! Text and JSON dumps of the simulation for developers and agents (spec §9).
//!
//! Every layer except [`Layer::Truth`], [`Layer::Actors`], and [`Layer::Rooms`] is built from
//! [`World::perception`] only, so dumps cannot leak information the player does not have.
//! `Truth`, `Actors`, and `Rooms` read ground truth and are dev-only.

use crate::ability::Aim;
use crate::action::{letter, position_text};
use crate::blood::{self, Stain};
use crate::character::Gender;
use crate::content::WeaponSkillId;
use crate::equipment;
use crate::geom::Pos;
use crate::hunger::Hunger;
use crate::ice::Ice;
use crate::noise::{PerceivedSound, SoundKind, Volume};
use crate::object::format_weight;
use crate::perception::{CellView, PerceivedLight, PerceptionView};
use crate::stats::Stat;
use crate::world::{Cause, World};
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
    /// or stat damage lowers it), then their status effects by name (`Effects Bleeding,
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
    /// Not a map: the player's abilities in the abilities menu's order, one a line with its
    /// source and what it asks for, or why it cannot be used now (`Shield bash (weapon skill):
    /// a direction`, `Open lock (skill): not usable yet`).
    Abilities,
    /// Not a map: the traps the player knows on the level, one a line by position (`(12, 5)
    /// fireball trap`, `(remembered)` when out of sight); never hidden ones.
    Traps,
    /// Not a map: the player's quest log, one quest a line, active first (`Rats in the arena
    /// (active, from the old hermit): Kill 3 giant rats in the Arena. 2 / 3 killed.`).
    Quests,
    /// Ground truth: all terrain, objects, traps (hidden ones too), secret doors (`S`), and
    /// actors on the level.
    /// **Dev-only.**
    Truth,
    /// Not a map: every actor on the level with its weight, what it carries (and of that wears
    /// and wields), its stats (scores and potentials), its trained skills with their marks, its
    /// status effects with their severities, and its diseases and stat damage, the player's
    /// nutrition, then every corpse on the level and in the player's pack with its age (ground
    /// truth). **Dev-only.**
    Actors,
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
        Layer::Abilities,
        Layer::Traps,
        Layer::Quests,
        Layer::Truth,
        Layer::Actors,
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
            Layer::Abilities => "abilities",
            Layer::Traps => "traps",
            Layer::Quests => "quests",
            Layer::Truth => "truth",
            Layer::Actors => "actors",
            Layer::Rooms => "rooms",
            Layer::Ice => "ice",
        }
    }

    /// Whether the layer reads ground truth.
    pub const fn is_dev_only(self) -> bool {
        matches!(
            self,
            Layer::Truth | Layer::Actors | Layer::Rooms | Layer::Ice
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
                     inventory, equipment, stats, skills, weapon_skills, abilities, traps, quests, \
                     truth, actors, rooms, ice)"
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
        Layer::Truth => truth_rows(world),
        Layer::Actors => actor_rows(world),
        Layer::Rooms => room_rows(world),
        Layer::Ice => ice_rows(world),
        Layer::Inventory => inventory_rows(&world.perception()),
        Layer::Equipment => equipment_rows(&world.perception()),
        Layer::Stats => stat_rows(&world.perception()),
        Layer::Skills => skill_rows(&world.perception()),
        Layer::WeaponSkills => weapon_skill_rows(&world.perception()),
        Layer::Abilities => ability_rows(&world.perception()),
        Layer::Traps => trap_rows(&world.perception()),
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
    let known_glyph = |cell: CellView| match (view.trap(p), view.objects(p).first()) {
        (Some(trap), _) => content.trap(trap).glyph,
        (None, Some(top)) => content.object(top.object).glyph,
        (None, None) => terrain_glyph(cell),
    };
    match layer {
        Layer::View => match view.actor_at(p) {
            Some(actor) => content.creature(actor.creature).glyph,
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
        | Layer::Abilities
        | Layer::Traps
        | Layer::Quests
        | Layer::Truth
        | Layer::Actors
        | Layer::Rooms
        | Layer::Ice => unreachable!("not a map built from perception"),
    }
}

/// A heard sound in the `sound` layer: `f` footsteps, `o` a door opening, `c` a door closing,
/// `x` fighting, `b` a bowstring, `m` chanting (magic), `k` banging (a door kicked), `d` digging,
/// `v` a voice, `a` an alarm bell, `e` an explosion, `t` a thunderclap; upper case when loud.
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
            (None, None, None) => level
                .terrain_at(p)
                .map_or(' ', |t| content.terrain(t).glyph),
        }
    })
}

/// The level's special rooms (ground truth): the terrain with each special room's plain ground
/// ([`crate::content::TerrainDef::is_plain_ground`]) drawn as its effect's mark, then one line
/// per room.
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
    for room in level.rooms() {
        let effect = content.room_effect(room.effect);
        let entered = if room.entered { ", entered" } else { "" };
        out.push(format!(
            "{} {} at {}-{}{entered}",
            effect.mark, effect.name, room.min, room.max
        ));
    }
    out
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

/// The ice on the level (ground truth): a line with the temperature there and when the ice was
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
        let name = s.describe(content);
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
                    format!("{} ({})", s.describe(content), position_text(item))
                }),
                None => "-".into(),
            };
            format!("{} {}: {what}", letter(i).unwrap_or('?'), def.name)
        })
        .collect()
}

/// The player's character, if any (`Race Dwarf`, `Gender Female`, `Class Warrior`, `Age 27`,
/// `Born 3 Seedtide 1417`, `Sign Fire Manticore`), then their own scores, one stat a line:
/// `Strength 12`.
fn stat_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    let mut rows = Vec::new();
    if let Some(c) = view.character() {
        let spec = &c.spec;
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
        if let Some(sign) = c.sign_name(content) {
            rows.push(format!("Sign {sign}"));
        }
    }
    rows.extend((view.own_scores().iter()).map(|(stat, score)| format!("{} {score}", stat.name())));
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
    rows
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
    (view.content().weapon_skills().ids())
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

/// The player's abilities, one a line in menu order: its name, its source, and what it asks for
/// or why it cannot be used now.
fn ability_rows(view: &PerceptionView<'_>) -> Vec<String> {
    let content = view.content();
    (view.abilities().iter())
        .map(|own| {
            let a = own.ability;
            let state = match own.unavailable {
                Some(why) => why.reason(),
                None => aim_word(a.aim()),
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
                "aim": aim_word(a.aim()),
                "unavailable": own.unavailable.map(|why| why.id()),
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
/// Protection (`dv 11 pv 3`), weight, what it carries, its burden and carrying limit, its stats
/// as score/potential
/// (`St 12/15`), its trained skills with their marks toward the next point
/// (`stealth 17 (2/5)`), its status effects with their severities (`effects bleeding 2`), the
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
                    let need = content.increase().need(t.value);
                    format!("{} {} ({}/{need})", content.skill(id).id, t.value, t.marks)
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
            let weapons = actor_weapon_skills(content, a)
                .map_or(String::new(), |s| format!("; weapon skills {s}"));
            // Who it is and how far its talk has got (spec §8 "Dialogue"), if anything.
            let npc = (a.npc.as_ref()).map_or(String::new(), |n| format!("; npc {n}"));
            let talk = if a.talk.is_new() {
                String::new()
            } else {
                let met = if a.talk.met { ", met" } else { "" };
                let (said, muttered) = (a.talk.said, a.talk.muttered);
                format!("; talk said {said}, muttered {muttered}{met}")
            };
            format!(
                "{} {} at {}: hp {}/{} mp {}/{} dv {} pv {}; weight {} (base {} + carried {}: {carried})\
                 {equipped}; {} (limit {}){weapons}; stats {}{skills}{effects}{diseases}{hunger}{npc}\
                 {talk}",
                a.id,
                content.creature(a.creature).id,
                a.pos,
                a.hp,
                a.max_hp(content),
                a.mana,
                a.max_mana(content),
                a.defence(content, load.penalty()).total(),
                a.protection(content),
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

/// An actor's diseases and stat damage, as the `actors` dump shows them (ground truth):
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

/// One-line summary used as the dump header; under the open sky it names the weather, and it
/// has the temperature where the player is.
fn summary(world: &World) -> String {
    let view = world.perception();
    // Spelled as in the JSON dump (`heavy_rain`).
    let weather = (view.weather())
        .and_then(|w| serde_json::to_value(w).ok())
        .and_then(|v| v.as_str().map(|w| format!(" weather {w}")))
        .unwrap_or_default();
    let temperature = view.temperature();
    let (hp, max) = view.own_hp();
    let (mp, max_mp) = view.own_mana();
    let dead = if world.death().is_some() { " dead" } else { "" };
    // As the status bar shows it: only when it is not Not hungry.
    let hunger = (view.own_hunger())
        .filter(|h| *h != Hunger::NotHungry)
        .map_or(String::new(), |h| format!(" hunger {}", h.id()));
    format!(
        "turn {} time {} level {} viewer {} hp {hp}/{max} mp {mp}/{max_mp}{dead}{hunger}{weather} \
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
/// sounds, known traps (`known_traps`: position, trap id, and whether in sight), the player's
/// inventory (with what of each stack is unpaid, priced), equipment (by slot id), weight, scores
/// (by stat id), status effects (by id), the diseases they know (by id), recent messages, the
/// requested layers as arrays of rows, and optionally the full
/// [`crate::WorldState`] under `state`.
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
            json!({
                "id": a.id, "kind": a.kind, "creature": def.id, "glyph": def.glyph.to_string(),
                "pos": a.pos, "health": a.health, "effects": effects,
            })
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
                "name": s.describe(content),
                "rarity": s.rarity(),
                "identified": s.is_identified(),
                "magic": magic,
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
    let weapon_skills: serde_json::Map<String, Value> = (content.weapon_skills().ids())
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
    let layer_map: serde_json::Map<String, Value> = layers
        .iter()
        .map(|&l| (l.name().to_string(), json!(layer_rows(world, l))))
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
        "shop": shop,
        "weather": view.weather(),
        // In °C, to a tenth of a degree.
        "temperature": f64::from(view.temperature().tenths()) / 10.0,
        "hash": format!("{:016x}", world.state_hash()),
        "seen_actors": seen,
        "heard_sounds": view.sounds(),
        "known_traps": traps,
        "inventory": inventory,
        "equipment": equipment,
        "weight": { "base": weight.base, "carried": weight.carried, "total": weight.total() },
        "burden": { "level": load.burden(), "limit": load.limit },
        "character": view.character().map(|c| json!({
            "race": c.spec.race,
            "gender": c.spec.gender,
            "class": c.spec.class,
            "raised": c.raised,
            "age": c.age_on(view.time()),
            "born": c.born,
            "sign": c.sign,
        })),
        "stats": view.own_scores(),
        "skills": skills_json(&view),
        "effects": effects,
        "diseases": diseases,
        "weapon_skills": weapon_skills,
        "styles_in_use": (view.own_styles_in_use().into_iter())
            .map(|id| content.weapon_skills().skill(id).id.clone())
            .collect::<Vec<_>>(),
        "abilities": abilities_json(&view),
        "quests": quests_json(&view),
        "hp": { "current": hp, "max": max_hp },
        "mana": { "current": mana, "max": max_mana },
        "experience": view.own_experience(),
        "defence": view.own_defence(),
        "protection": view.own_protection(),
        "hunger": view.own_hunger(),
        "death": death,
        "messages": messages,
        "layers": layer_map,
    });
    if include_state {
        out["state"] = serde_json::to_value(world.state()).expect("state serializes");
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
/// "objective", "status"}]`, active quests first.
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
            json!({ "race": "human", "gender": "female", "class": "thief",
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
            json!({ "appraise": 11, "disable_device": 16, "escape_artist": 7, "open_lock": 18,
                    "search": 9, "spot": 10, "steal": 17, "stealth": 16, "tumble": 16 })
        );
        assert!(json(&world, &[], false).get("state").is_none());
    }

    #[test]
    fn skills_show_the_players_trained_skills_only() {
        let world = showcase();
        assert_eq!(
            layer_rows(&world, Layer::Skills),
            [
                "Appraise 11",
                "Disable device 16",
                "Escape artist 7",
                "Open lock 18",
                "Search 9",
                "Spot 10",
                "Steal 17",
                "Stealth 16",
                "Tumble 16"
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
            shaman.is_some_and(
                |s| s.ends_with("; skills concentration 20 (0/6), spellcraft 25 (0/6)")
            ),
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
                "Presence 9"
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
        // Every actor's stats, then its trained skills with their marks toward the next point.
        assert!(
            actors[0].contains(
                "stats St 9/16 Dx 13/16 To 10/16 In 12/16 Wi 11/16 Pr 9/16; skills appraise 11 \
                 (0/5), disable_device 16 (0/5), escape_artist 7 (0/4), open_lock 18 (0/5), \
                 search 9 (0/4), spot 10 (0/5), steal 17 (0/5), stealth 16 (0/5), tumble 16 (0/5); \
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
        assert_eq!(rows.len(), 24);
        assert_eq!(rows[0], "Daggers and knives 2 (0%)");
        assert_eq!(rows[16], "Duelist 1 (0%)");
        assert_eq!(rows[22], "Brawler 0 (0%), in use");
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
        // The showcase thief: the class's skills used as actions (no use yet), then the shield
        // bash, which needs a shield in hand.
        let mut world = showcase();
        assert_eq!(
            layer_rows(&world, Layer::Abilities),
            [
                "Appraise (skill): not usable yet",
                "Disable device (skill): not usable yet",
                "Open lock (skill): not usable yet",
                "Steal (skill): not usable yet",
                "Shield bash (weapon skill): you hold no shield",
            ]
        );
        let value = json(&world, &[], false);
        assert_eq!(
            value["abilities"][4],
            json!({"id": "shield_bash", "source": "weapon_skill", "aim": "a direction",
                   "unavailable": "no_shield"})
        );
        assert_eq!(value["abilities"][2]["id"], "open_lock");
        // The storeroom's small shield in hand: it can be used, in a direction.
        let walk = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
                    pickup:a equip:off_hand:a";
        for action in crate::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        let rows = layer_rows(&world, Layer::Abilities);
        assert_eq!(rows[4], "Shield bash (weapon skill): a direction");
        assert_eq!(
            json(&world, &[], false)["abilities"][4]["unavailable"],
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
