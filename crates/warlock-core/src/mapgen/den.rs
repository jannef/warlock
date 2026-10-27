//! Teeming dens (spec §7 "Special rooms"; `docs/dev/special-rooms.md`): a generated floor's
//! special room whose effect has a `swarm` is filled with sleeping creatures and their hoard.
//!
//! Nothing about a particular den is hand-made. Its sleepers are drawn by the floor's danger from
//! the branch's population by the same draw that lays down the floor's creatures
//! ([`danger::draw`]: level weights, rarity tiers, strays, out of depth), hostile types only and
//! never uniques. Now and then a den is themed (`swarm.themes`, one draw in 0..100): a *warren*,
//! every sleeper of one type drawn first by that same draw, or a *tribe*, every sleeper of the
//! faction of a type drawn first, each drawn from the population and the strays of that faction
//! (should none weigh anything at the danger, the first type again). How many: one per
//! `per_cells` free floor cells of the room, `min` to `max` (never more than its cells). Each lies
//! on a cell of the room's plain floor drawn among those free (no creature, stairs, light, or
//! trap), keeps no post (it has none to return to until woken), and is placed asleep
//! ([`CreatureSpawn::asleep`]). The hoard is one item per four sleepers (at least one) at the
//! floor's item level + `loot_level`, with the finder's luck, each on a plain floor cell of the
//! room drawn anew.
//!
//! A den kind may name `swarm.factions` (a beast den's `wildlife`, `vermin`, and `marsh`): its
//! sleepers are then drawn only among the types of those factions, hostile or not, and the kind
//! is eligible only on a branch whose own population holds one ([`peopled`], asked by
//! `special::pick`). Its hoard may be `corpses` instead of items: per piece, a fresh corpse of a
//! type drawn from the branch's population at large (its prey, any faction, by the same draw;
//! a type that leaves none leaves nothing) and a pile of bones, each on a cell drawn anew.
//!
//! Everything comes from the level's own stream [`Rng::swarm`] (`swarm/<level id>`), drawn after
//! its population, so the rest of the level is the same with or without a den; the sleepers roll
//! their gear after the population's, with the rest ([`crate::gear::arm`]). The population and
//! the generator's patrols keep out of a den (`population.rs`, `special::make`).

use super::{Palette, Visit};
use crate::content::danger::{self, Candidate, Spawning};
use crate::content::{Content, CreatureId, CreatureSpawn, Hoard, LevelTemplate, Stance, Swarm};
use crate::geom::Pos;
use crate::magic::{LootGrade, loot};
use crate::object::{Stack, add_to_pile};
use crate::rng::Rng;
use crate::variant::Source;

/// Fills every teeming den of generated level `template` (see the module docs): `danger` its
/// danger, `spawning` its branch's rules, `visit` whom it is made for (the finder's luck).
pub(super) fn fill(
    template: &mut LevelTemplate,
    content: &Content,
    seed: u64,
    (pal, danger, spawning): (Palette, u8, &Spawning),
    visit: &Visit,
) {
    let dens: Vec<(usize, &Swarm)> = (template.rooms.iter().enumerate())
        .filter_map(|(i, r)| content.room_effect(r.effect).swarm.as_ref().map(|s| (i, s)))
        .collect();
    if danger == 0 || dens.is_empty() {
        return;
    }
    let mut rng = Rng::swarm(seed, &template.id);
    let none = Default::default();
    let all_own = plain(content, content.candidates(&spawning.population, &none));
    let all_generic = plain(content, content.candidates(&spawning.generic, &none));
    for (i, swarm) in dens {
        let (own, generic) = if swarm.kinds.is_empty() {
            (
                sleepers(content, swarm, &all_own),
                sleepers(content, swarm, &all_generic),
            )
        } else {
            (
                plain(content, content.candidates(&swarm.kinds, &none)),
                Vec::new(),
            )
        };
        let danger = deeper(danger, swarm);
        let room = template.rooms[i].clone();
        let floor: Vec<Pos> = (room.cells().into_iter())
            .filter(|&p| template.terrain.get(p) == Some(&pal.floor))
            .filter(|&p| !template.stairs.iter().any(|s| s.pos == p))
            .filter(|&p| !template.lights.contains_key(&p))
            .collect();
        let mut cells: Vec<Pos> = (floor.iter().copied())
            .filter(|p| !template.traps.contains_key(p))
            .filter(|&p| !template.creatures.iter().any(|c| c.pos == p))
            .collect();
        let count = swarm.sleepers(cells.len());
        let theme = Theme::roll(&mut rng, content, (&own, &generic), danger, spawning, swarm);
        let mut placed = 0;
        for _ in 0..count {
            let Some(creature) = theme.draw(&mut rng, (&own, &generic), danger, spawning) else {
                break;
            };
            if cells.is_empty() {
                break;
            }
            let pos = cells.swap_remove(rng.index(cells.len()));
            template.creatures.push(CreatureSpawn {
                creature,
                faction: None,
                alignment: None,
                npc: None,
                pos,
                route: Vec::new(),
                carry: Vec::new(),
                equipment: Default::default(),
                rolls_gear: true,
                asleep: true,
            });
            placed += 1;
        }
        if placed == 0 || floor.is_empty() {
            continue;
        }
        if swarm.hoard == Hoard::Corpses {
            let prey = (all_own.as_slice(), all_generic.as_slice());
            corpses(
                template,
                content,
                &mut rng,
                &floor,
                (placed, prey, danger, spawning),
            );
            continue;
        }
        let level = template.item_level.saturating_add(swarm.loot_level);
        for _ in 0..Swarm::hoard(placed) {
            let p = floor[rng.index(floor.len())];
            let item = loot::roll_with(
                content,
                &mut rng,
                level,
                LootGrade::Any,
                Source::Loot,
                visit.luck,
            );
            if let Some(stack) = item {
                add_to_pile(template.objects.entry(p).or_default(), &[stack], content);
            }
        }
    }
}

/// The candidates of `candidates` that are not unique.
fn plain(content: &Content, candidates: Vec<Candidate>) -> Vec<Candidate> {
    (candidates.into_iter())
        .filter(|&(c, _, _)| !content.creature(c).unique)
        .collect()
}

/// The candidates of `candidates` (no uniques) that may sleep in a den of `swarm`: those of its
/// `factions` if it names any, else those hostile to the player.
fn sleepers(content: &Content, swarm: &Swarm, candidates: &[Candidate]) -> Vec<Candidate> {
    let player = content.creature_id("player");
    (candidates.iter().copied())
        .filter(|&(c, _, _)| {
            if swarm.factions.is_empty() {
                player.is_some_and(|p| content.stance(c, p) == Stance::Hostile)
            } else {
                of_factions(content, swarm, c)
            }
        })
        .collect()
}

/// Whether type `c` is of one of `swarm`'s factions.
fn of_factions(content: &Content, swarm: &Swarm, c: CreatureId) -> bool {
    let faction = content.creature(c).faction;
    (swarm.factions.iter()).any(|name| faction.is_some() && content.factions().id(name) == faction)
}

/// Whether a den of `swarm` may be made on a level whose branch's own population is
/// `population` and whose least danger is `least`: for one with a population of its own (an
/// armoury's), when a type of it weighs something in a draw at `least` made [`deeper`]; else
/// always for one that names no factions, else when a type of the population that is not unique
/// is of one of them.
pub(super) fn peopled(
    content: &Content,
    swarm: &Swarm,
    population: &[CreatureId],
    least: u8,
) -> bool {
    if !swarm.kinds.is_empty() {
        let at = deeper(least, swarm);
        let none = Default::default();
        return (content.candidates(&swarm.kinds, &none).into_iter())
            .any(|c| danger::weighs(c, at));
    }
    swarm.factions.is_empty()
        || (population.iter())
            .any(|&c| !content.creature(c).unique && of_factions(content, swarm, c))
}

/// The danger a den of `swarm` draws its sleepers at on a floor of `danger`: its `danger` deeper
/// (none on a floor without one).
fn deeper(danger: u8, swarm: &Swarm) -> u8 {
    if danger == 0 {
        return 0;
    }
    danger.saturating_add(swarm.danger).min(danger::MAX_DANGER)
}

/// A den's corpse hoard (`hoard = "corpses"`) of `placed` sleepers on `floor`: per piece
/// ([`Swarm::hoard`]), a fresh corpse of prey drawn from `prey` (the branch's population and
/// strays, any faction) and a pile of bones, each on a floor cell drawn anew.
fn corpses(
    template: &mut LevelTemplate,
    content: &Content,
    rng: &mut Rng,
    floor: &[Pos],
    (placed, (own, generic), danger, spawning): (
        usize,
        (&[Candidate], &[Candidate]),
        u8,
        &Spawning,
    ),
) {
    let bones = content.object_id("bones");
    for _ in 0..Swarm::hoard(placed) {
        let kill = danger::draw(rng, own, generic, danger, &spawning.odds);
        if let Some(corpse) = kill.and_then(|c| content.corpse_of(c)) {
            let p = floor[rng.index(floor.len())];
            // Made when the level is first entered (`rot::made_at`): fresh then.
            add_to_pile(
                template.objects.entry(p).or_default(),
                &[Stack::one(corpse)],
                content,
            );
        }
        if let Some(bones) = bones {
            let p = floor[rng.index(floor.len())];
            add_to_pile(
                template.objects.entry(p).or_default(),
                &[Stack::one(bones)],
                content,
            );
        }
    }
}

/// What a den is drawn from.
enum Theme {
    /// The population and the strays, as the floor's own creatures.
    Mixed,
    /// A warren: every sleeper of this type.
    Kind(CreatureId),
    /// A tribe: the population and the strays of this type's faction (it, should none of them
    /// weigh anything at the danger).
    Faction(CreatureId, Vec<Candidate>, Vec<Candidate>),
}

impl Theme {
    /// A den's theme: one draw in 0..100 against `swarm.themes`, then the type it starts from.
    fn roll(
        rng: &mut Rng,
        content: &Content,
        (own, generic): (&[Candidate], &[Candidate]),
        danger: u8,
        spawning: &Spawning,
        swarm: &Swarm,
    ) -> Self {
        let themes = swarm.themes;
        if themes.kind == 0 && themes.faction == 0 {
            return Theme::Mixed;
        }
        let at = rng.below(100);
        let warren = at < u32::from(themes.kind);
        if !warren && at >= u32::from(themes.kind) + u32::from(themes.faction) {
            return Theme::Mixed;
        }
        let Some(first) = danger::draw(rng, own, generic, danger, &spawning.odds) else {
            return Theme::Mixed;
        };
        if warren {
            return Theme::Kind(first);
        }
        let faction = content.creature(first).faction;
        let of = |list: &[Candidate]| -> Vec<Candidate> {
            (list.iter().copied())
                .filter(|&(c, _, _)| content.creature(c).faction == faction)
                .collect()
        };
        Theme::Faction(first, of(own), of(generic))
    }

    /// The type of the den's next sleeper.
    fn draw(
        &self,
        rng: &mut Rng,
        (own, generic): (&[Candidate], &[Candidate]),
        danger: u8,
        spawning: &Spawning,
    ) -> Option<CreatureId> {
        match self {
            Theme::Mixed => danger::draw(rng, own, generic, danger, &spawning.odds),
            Theme::Kind(c) => Some(*c),
            Theme::Faction(first, own, generic) => {
                danger::draw(rng, own, generic, danger, &spawning.odds).or(Some(*first))
            }
        }
    }
}
