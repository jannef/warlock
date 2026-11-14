//! The map layers: a glyph per cell of the perceived level (view, fov, memory, light, sound, blood, scorch, ice).

use super::*;

pub(super) fn perceived_glyph(view: &PerceptionView<'_>, layer: Layer, p: Pos) -> char {
    let content = view.content();
    let cell = view.cell(p);
    let terrain_glyph = |cell: CellView| match cell {
        CellView::Unknown => ' ',
        CellView::Remembered(t) | CellView::Visible(t) => content.terrain(t).glyph,
    };
    let stairs = |cell: CellView| match cell {
        CellView::Unknown => false,
        CellView::Remembered(t) | CellView::Visible(t) => content.terrain(t).stairs.is_some(),
    };
    // A chest's known trap leaves the chest's glyph (spec §8 "Chests").
    let trap = view.trap(p).filter(|_| view.chest_at(p).is_none());
    let known_glyph = |cell: CellView| match (trap, view.objects(p).first()) {
        (Some(trap), _) => content.trap(trap).glyph,
        // Known stairs keep their glyph over a pile (as the map draws them).
        (None, Some(_)) if stairs(cell) => terrain_glyph(cell),
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
        Layer::Scorch => match view.scorch(p) {
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
        | Layer::Recipes
        | Layer::Spells
        | Layer::Traps
        | Layer::Locations
        | Layer::Quests
        | Layer::Story
        | Layer::Companion
        | Layer::Familiar
        | Layer::Truth
        | Layer::Actors
        | Layer::Progress
        | Layer::Rooms
        | Layer::Ice => unreachable!("not a map built from perception"),
    }
}

/// A heard sound in the `sound` layer: `f` footsteps, `o` a door opening, `c` a door closing,
/// `x` fighting, `b` a bowstring, `m` chanting (magic), `k` banging (a door kicked), `d` digging,
/// `v` a voice, `h` a howl, `r` a roar, `z` buzzing, `g` giggling, `s` a hiss, `l` a rattle, `y` a
/// scream, `w` a wail, `j` a crash of glass (a flask shattering), `q` a heartbeat, `a` an alarm
/// bell, `e` an explosion, `t` a thunderclap; upper case when loud.
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
        SoundKind::Roar => 'r',
        SoundKind::Buzzing => 'z',
        SoundKind::Giggle => 'g',
        SoundKind::Hiss => 's',
        SoundKind::Rattle => 'l',
        SoundKind::Scream => 'y',
        SoundKind::Wail => 'w',
        SoundKind::Shatter => 'j',
        SoundKind::Heartbeat => 'q',
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
pub(super) fn ice_rows(world: &World) -> Vec<String> {
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

pub(super) fn rows(width: i32, height: i32, glyph: impl Fn(Pos) -> char) -> Vec<String> {
    (0..height)
        .map(|y| {
            let row: String = (0..width).map(|x| glyph(Pos::new(x, y))).collect();
            row.trim_end().to_string()
        })
        .collect()
}
