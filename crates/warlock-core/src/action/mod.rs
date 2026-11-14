//! Player actions and the events a turn produces.

use crate::combat::Strike;
use crate::content::{
    ActionKind, ClassActive, CombatStance, DamageTypeId, DiseaseId, ObjectId, ResourceId,
    SignActive, StatusId, TalentActive, TrapId, ZoneKind,
};
use crate::geom::{Dir, Pos};
use crate::time::{DAY, Duration};
use crate::weather::Weather;
use crate::world::{ActorId, Conversation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

mod text;

pub use text::{MOST_COORDINATE, MOST_REPEATS, format_actions, parse_actions};

/// Something the player asks to do. Actions are the unit of replays (spec §9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Step in a direction; bumping a closed door opens it (unlocking it first with a key that
    /// fits, if it is locked).
    Move(Dir),
    /// Run in a direction (#1325; spec §6 "Running"): step after step until something worth
    /// stopping for, a long action like a rest (each step a move's time). In the open it holds
    /// the direction; landing in a corridor, it follows it to an intersection or a room. A first
    /// step a run cannot take is a plain [`Action::Move`].
    Run(Dir),
    /// Let a turn pass, looking around: the same as `Search(1)`, one code path (#1326; spec §8
    /// "Searching"). Kept as its own spelling, `wait`, for replays, tests, and the manifests.
    Wait,
    /// Open the door in a direction; a locked one only with a key that fits, which unlocks it
    /// first (spec §8 "Doors").
    Open(Dir),
    Close(Dir),
    /// Kick in a direction (spec §8 "Doors"): a closed door, to break it (a chance of Strength in
    /// its sturdiness), or a creature, an unarmed blow. Takes the time of `kick`; kicking at
    /// empty space or something solid takes none.
    Kick(Dir),
    /// Pick the lock of the closed door in a direction (spec §8 "Doors"; the Pick lock ability,
    /// the Open lock skill's use): a Dexterity check plus what Open lock gives against the lock's
    /// difficulty. Picked, the door opens. Takes the time of `pick_lock` whatever comes of it
    /// (plus `open_door` when it opens); refused, taking none, without the skill or a lock there.
    PickLock(Dir),
    /// Take stairs up.
    Ascend,
    /// Take stairs down.
    Descend,
    /// Use the tool ready in the Tool slot without a direction (`u`; spec §5 "Light sources"):
    /// a light there (a lantern) turns from unlit to lit, lit to hooded, and hooded to unlit
    /// (`world/lamp.rs`). Takes the time of `light` (lowering a hood: `hood`); refused, no time,
    /// without a light ready. A digging tool is used toward a direction: [`Action::Dig`].
    UseTool,
    /// Pick up the stacks at these positions in the pile on the player's cell (0 is the top of
    /// the pile), or everything there with none. Takes a turn (its cost: `pick_up`).
    PickUp(Positions),
    /// Drop the inventory stacks at these positions onto the player's cell, or everything
    /// carried with none. Takes a turn (its cost: `drop`, plus taking off what of it is worn or
    /// wielded). In a shop (spec §7 "Shops") what of them is unpaid goes back; the player's own
    /// things stay, and the shop says what it would pay for those it buys.
    Drop(Positions),
    /// Sell the inventory stacks at these positions (everything with none) to the shop the
    /// player stands in, for what it offers (spec §7 "Shops"): like [`Action::Drop`] there, but
    /// what the shop buys is sold, paid from the keeper's purse. Takes the time of `drop`.
    Sell(Positions),
    /// Pay for everything unpaid in the shop the player stands in (spec §7 "Shops"), from the
    /// gold they carry, which takes no game time; then, with a direction, step that way (out of
    /// the shop, as [`Action::Move`] would, taking its time): paying and leaving in one action.
    /// Refused, with no step, if the player cannot pay.
    Pay(Option<Dir>),
    /// Wear or wield inventory stack `item` (a position, `a` = 0) in `slot` (an id in
    /// `data/slots.toml`), taking off what is there first (a swap). Takes the time of the slot's
    /// action (`wield`, `wear`, `body_armor`, `ready`) for each thing put on or taken off.
    Equip {
        slot: String,
        item: usize,
    },
    /// Take off what is in a slot (an id in `data/slots.toml`). Takes the time of its action.
    Remove(String),
    /// Bash whoever stands in a direction with the shield the player holds (spec §8 "Weapon
    /// skills"): a melee blow with the shield's own damage, drawing on and training shield
    /// bashing. Takes the time of `melee`.
    Bash(Dir),
    /// Shoot the readied missile weapon's ammunition at this cell (spec §8 "Ranged combat"):
    /// one shot flies along the line through it, as far as the weapon shoots. Takes the time of
    /// `shoot` (plus `reload` for a missile weapon that needs it).
    Fire(Pos),
    /// Throw one of inventory stack `item` (a position, `a` = 0), a throwing weapon, at cell
    /// `at`. Takes the time of `throw`.
    Throw {
        item: usize,
        at: Pos,
    },
    /// Wait turn after turn (creatures act) until the goal, stopping early when a creature comes
    /// into view or a message is logged.
    Rest(Rest),
    /// Answer the overworld encounter that is pending (spec §7 "Overworld encounters";
    /// `encounter:ambush`, `evade`, `fight`, `hail`, `hunt`, `pass`): lie in wait (ten minutes),
    /// slip away (a check; half the step's time again), or fight, hail, or hunt (the area opens
    /// as `>` opens it), or pass by (no time). While one pends, every other action that takes
    /// time is refused. Refused (no time) with none pending or a choice it does not offer.
    Encounter(crate::world::EncounterChoice),
    /// Chat with the creature in a direction (spec §8 "Dialogue"): it answers with a line, and
    /// one that has topics opens the dialogue screen. Takes the time of `chat`; refused (no
    /// time) when the player sees no one there.
    Chat(Dir),
    /// Ask the creature in a direction about one of its topics, by id. Takes no time. Quests
    /// (spec §8 "Quests") are asked about with topics of their own: `quest-ID` (be given it,
    /// be reminded, or hand it in).
    Ask {
        dir: Dir,
        topic: String,
    },
    /// Try to steal from the creature in a direction (spec §8 "Stealing"): one stack of what it
    /// carries and neither wears nor wields, if the player's Dexterity and Steal skill beat its
    /// Willpower; caught, it turns on them. Takes the time of `steal`; refused (no time) when the
    /// player sees no one there, or someone hostile.
    Steal(Dir),
    /// Feint at the creature in a direction (spec §8 "Skills": Bluff; `world/feint.rs`): a
    /// hostile one the player sees, with a mind to fool, checks its Willpower against
    /// `data/feint.toml`'s difficulty + the player's Presence modifier + Bluff's `feint` points,
    /// and is Surprised if it fails. Takes the time of `feint`, whatever comes of it; refused (no
    /// time) untrained, at no one seen, at one not hostile, or at one without a mind.
    Feint(Dir),
    /// Tame the beast in a direction (spec §8 "Allies": Tame; `world/tame.rs`): `[tame] turns`
    /// actions beside it (`data/companions.toml`), each the time of `tame`, then a Willpower
    /// check; made, it is the player's companion. Refused (no time) untrained, with a companion
    /// already, at no beast seen there, at one that cannot be tamed, or at one hostile that has
    /// seen them.
    Tame(Dir),
    /// Intimidate the creature on a cell the player sees within the `range` of
    /// `data/intimidate.toml` (spec §8 "Skills": Intimidate; `world/intimidate.rs`): a hostile
    /// one that hears and sees them checks its nerve, and failed, is frightened. Takes the time
    /// of `intimidate`; refused (no time) untrained, at no one seen, out of reach, at one not
    /// hostile, or at one with no vitals.
    Intimidate(Pos),
    /// Throw the player's voice to a cell they see within [`crate::world::VOICE_RANGE`] (spec §5
    /// "Hearing"): a voice sounds there, and creatures that hear it and do not see its cell go
    /// and look. Takes the time of `throw_voice`; refused (no time) at a cell they do not see,
    /// out of reach, solid, or their own.
    ThrowVoice(Pos),
    /// Give every ally of the player's on the level an order (spec §8 "Allies";
    /// `world/ally.rs`): `order:follow` (Come to heel), `order:stay` (Hold here), `order:hunt:X,Y`
    /// (Set on a cell). No time, a voice's noise where the player stands; refused (no time, no
    /// noise) with no ally on the level, and Set on at a cell the player does not see.
    Order(Command),
    /// Eat one of a stack, from the pack or the floor (spec §8 "Hunger"): a meal of as many
    /// turns (each the `eat` action) as the food takes, stopping early like a rest; the food is
    /// eaten and feeds at the end.
    Eat(Meal),
    /// Read one of inventory stack `item` (a position, `a` = 0), a scroll (spec §8 "Scrolls"),
    /// using it up: a spell scroll at the cell `target` names (it flies or runs there from the
    /// player, with the traps' rules), the scroll of identify on the inventory stack it names.
    /// Takes the time of `read`; refused (no time, the scroll kept) without a fitting target.
    /// A grimoire (spec §8 "Spells": "Learning") is read with no target, and its reading is a
    /// study of its spell (`world/learn.rs`): turn after turn of the `study` action, stopping
    /// early like a rest; at the end, above the player's learning, a check decides whether they
    /// learn it or a mishap befalls them. The grimoire is kept either way. Refused (no time) for
    /// a spell already known, or without training in Decipher script.
    Read {
        item: usize,
        target: Option<ReadTarget>,
    },
    /// Write a scroll of `spell` (a spell's id), one the player knows, on the first blank scroll
    /// they carry (spec §8 "Scrolls": the wizard's Scribe, `scribe:fireball`): turn after turn of
    /// the `scribe` action, as many as a study of the spell takes, stopping early like a rest
    /// (the blank and the mana kept); at the end the blank is used up, three times the spell's
    /// mana spent, and the scroll made, its type known. Refused (no time) without the ability, a
    /// spell they know, a blank scroll, or the mana.
    Scribe {
        spell: String,
    },
    /// Take a combat stance (spec §8 "Stances"). Takes no game time, but changes state, so
    /// replays record it.
    SetStance(CombatStance),
    /// Dig into the wall in this direction with the digging tool the player has ready (spec §8
    /// "Digging"): turn after turn (each the `dig` action's time) until the wall gives way,
    /// stopping early like a rest. What was dug stays dug.
    Dig(Dir),
    /// Search for hidden features (secret doors, hidden traps; spec §8 "Searching") for up to
    /// this many turns (each the `search` action's time, a fresh check every turn), stopping at
    /// once when something is found or something interrupts, like a rest. 1 is one search.
    Search(u8),
    /// Try to disarm the trap the player knows of in a direction (spec §8 "Disarming traps"; the
    /// Disarm ability): a Dexterity check plus Disable device against the trap's difficulty takes
    /// it away; a bad failure sets it off at the player. Takes the time of `disarm`, whatever
    /// comes of it; refused (no time) without the skill or a known trap there.
    Disarm(Dir),
    /// Listen at the closed door in a direction (spec §5 "Hearing"; the Listen ability,
    /// `world/listen.rs`): a turn in which the player's hearing through that door is keener.
    Listen(Dir),
    /// Wedge the closed door in a direction shut (spec §8 "Doors"; the Wedge ability,
    /// `world/wedge.rs`): a turn, after which creatures that open doors must kick it in. Refused
    /// (no time) without Woodcraft or a closed door that way.
    Wedge(Dir),
    /// Treat oneself, or an ally beside one, with the Heal skill (spec §8 "Skills"; the Heal
    /// ability): the ailment named (a status id, `heal:bleeding`, or a disease id,
    /// `heal:filth_fever`), else the player's first there is to treat (`heal`); `ally` the
    /// player's ally that way (`heal:bleeding:E`), which needs an ailment named. An Intelligence
    /// check plus Heal: bleeding stops, a poison eases a degree, a disease is eased. Takes the
    /// time of `heal` whatever comes of it, and may be tried again at once; refused (no time)
    /// without the skill, with nothing to treat, or with no ally of the player's that way.
    Heal {
        what: Option<String>,
        ally: Option<Dir>,
    },
    /// Pray at the forgotten altar next to the player (spec §7 "Special rooms"; the Pray
    /// ability, the abilities menu's Here group): a minute of turns of the `pray` action, then one
    /// outcome drawn from the altar's table, and perhaps the altar spent. Interrupted, it gives
    /// nothing. Refused (no time) with no altar next to the player.
    Pray,
    /// Drink at the healing spring next to the player (spec §7 "Special rooms"; the Drink
    /// ability, the abilities menu's Here group): ten seconds of turns of the `drink` action,
    /// then one outcome drawn from the spring's table. A spring never runs dry. Interrupted, it
    /// gives nothing. Refused (no time) with no spring next to the player.
    Drink,
    /// Forage for food in the wild (spec §8 "Foraging"; the Forage ability, Survival's use): turns
    /// of the `forage` action for `data/forage.toml`'s time, then a chance of food by the
    /// landscape and Survival. Interrupted, it finds nothing. Refused (no time) untrained, or
    /// anywhere but on land of the overworld or a local area of it.
    Forage,
    /// Gather herbs from the glowing fungus in a direction (spec §8 "Skills": Herbalism; the
    /// Gather ability, the abilities menu's Here group): turns of the `gather` action for
    /// `data/herbs.toml`'s time, then herbs drawn by the level's depth, more with Herbalism.
    /// Interrupted, it finds nothing. The fungus is never picked over. Refused (no time) with no
    /// fungus that way.
    Gather(Dir),
    /// Make a profession's recipe (spec §8 "Skills"; docs/design/professions.md §8;
    /// `world/craft.rs`): `recipe` its id (`data/recipes/<skill>/<id>.toml`), `picks` the
    /// inventory positions chosen for its steps in order, `None` a skipped optional one
    /// (`craft:long_sword:c,f`, `craft:dagger:a,-`). A built-in recipe's row runs its own action
    /// (`temper:`, `whittle:`), so this is refused for it; a generic one is made by
    /// `world/craft_make.rs` (the forge, the arrows: `craft:arrows:b,c`). Refused, it takes no
    /// time.
    Craft {
        recipe: String,
        picks: Vec<Option<usize>>,
    },
    /// Make out the carried stack at this position, a scroll of a type not yet identified,
    /// without reading it (spec §8 "Scrolls"; the Decipher ability, Decipher script's use;
    /// `world/decipher.rs`): `data/decipher.toml`'s time of turns of the `decipher` action, then
    /// the reading's check; made, the type is learned. Interrupted, nothing comes of it. Refused
    /// (no time) with a stack that is no scroll of a type not yet known.
    Decipher {
        item: usize,
    },
    /// Temper the carried stack at this position at the anvil next to the player (spec §7
    /// "Special rooms"; the Temper ability, the abilities menu's Here group): ten minutes of turns
    /// of the `temper` action, then a Strength check plus Metalcraft, harder for every attempt
    /// ever made on the item: an improvement or damage, or on a damaged item its repair or its
    /// end (`world/temper.rs`). Interrupted, it gives nothing and counts for nothing. Refused (no
    /// time) with no anvil next to the player, a stack with all its improvements, or one that is
    /// not a metal
    /// weapon or armour.
    Temper {
        item: usize,
    },
    /// Whittle the carried stack at this position with a knife, anywhere (the Whittle ability,
    /// Woodcraft's second use): `temper.toml`'s rules for a weapon of wood, ten minutes of turns
    /// of the `whittle` action, then a Dexterity check plus Woodcraft (`world/temper.rs`).
    /// Refused (no time) without a knife carried, a stack with all its improvements, or one that
    /// is not a wooden weapon.
    Whittle {
        item: usize,
    },
    /// Salvage the carried stack at this position (one of it), anywhere (the Salvage ability,
    /// the Skills group; `world/salvage.rs`): ten minutes of turns of the `salvage` action, then
    /// a check per material a profession works, each made one giving its stock; the item is
    /// destroyed either way. Interrupted, nothing happens. Refused (no time) with a stack worn or
    /// wielded, protected, unpaid, a spirit weapon, or with nothing a profession works.
    Salvage {
        item: usize,
    },
    /// Rig a trap of rig `rig` (an id in `data/rigs.toml`) on the cell `dir` of the player (spec
    /// §8 "Traps": "Rigging traps"; the craftsperson's Trapwright, `world/rig.rs`):
    /// `rig:tripwire:E`. Turn after turn of the `craft` action for the rig's time, then a
    /// Dexterity check: made, the trap is placed, known, and the inputs spent. Refused (no time)
    /// without the ability, a direction (`rig:tripwire`), a cell that may hold it, or what it
    /// takes.
    Rig {
        rig: String,
        dir: Option<Dir>,
    },
    /// Give the carried stack at this position (one of it) to the player's companion beside them,
    /// for its neck (spec §8 "Allies"; the Give ability, the abilities menu's Allies group;
    /// `world/collar.rs`): priced as a `wear`, and what it wore falls on its cell. `ally` is the
    /// way to the beast when one is named, `give:b:E`; with none, the first beside the player.
    /// Refused (no time) with none beside them or a stack that does not fit a neck.
    Give {
        item: usize,
        ally: Option<Dir>,
    },
    /// Use a talent's own active (spec §8 "Talents"; the abilities menu's Talents group), with
    /// the verb every ability use has (#644). `use:second_wind` catches a second wind: hit
    /// points back at once, then Winded for a while; refused (no time) without the talent or
    /// while winded. `use:blood_magic` gives some hit points for some mana; refused (no time)
    /// without the talent, when it would cost the last hit point, or while mana is full. Either
    /// takes the time of `wait`.
    UseTalent(TalentActive),
    /// Use a class active (spec §8 "Class abilities"; the abilities menu's Class group): rage,
    /// veteran's wind, lay on hands, holy radiance, and, aimed in a direction, smite
    /// and quivering palm (`use:rage`, `use:smite:E`). Takes the time of its cost's action;
    /// refused (no time) without the ability, while its cooldown lasts, or without the
    /// direction it asks.
    Use(ClassActive, Option<Dir>),
    /// Cast a spell the player knows (spec §8 "Spells"; the abilities menu's Spells group), by
    /// its id in `data/spells/`: on oneself, or aimed at a cell (`use:blessing`,
    /// `use:magic_missile:12,5`). Takes the time of its `cast`s and its mana; refused (no time)
    /// for an unknown spell, without the spell, without the mana, or without the cell it asks.
    /// Any `use:ID` that is no class or birth sign ability is read as this. An id that is no
    /// spell's is a weapon active's (#564; `use:shield_wall`, `use:piercing_shot:12,5`): the
    /// rules resolve it after the spells.
    Cast(String, Option<Pos>),
    /// Use a weapon active aimed in a direction (spec §8 "Weapon skills", "Fighting styles";
    /// #564; the abilities menu's Weapon skills group), by its id in `data/weapon_abilities/`:
    /// `use:lunge:E` (the direction in upper case). Takes its energy's percent of the time of
    /// the action it is and its stamina; refused (no time) without the ability, while what the
    /// player holds is not for it, short of stamina, or with nobody it may strike there.
    Act(String, Dir),
    /// Cast a spell the player knows on a stack they carry, by its position (Identify,
    /// `use:identify:c`). Refused as [`Action::Cast`] is, and for a spell that asks for no
    /// stack.
    CastOn(String, usize),
    /// Use a class active aimed at a cell (spec §8 "Class abilities"): the ranger's Volley
    /// (`use:volley:12,5`). Takes the time of its cost's action; refused (no time) without the
    /// ability or while its cooldown lasts.
    UseAt(ClassActive, Pos),
    /// Use a class active on a stack the player carries, by its position (spec §8 "Class
    /// abilities"): the merchant's Merchant's eye (`use:merchants_eye:c`). Takes the time of its
    /// cost's action; refused (no time) without the ability, or for a stack it may not work on.
    UseOn(ClassActive, usize),
    /// Use a class active that throws a stack the player carries, by its position, at a cell
    /// (spec §8 "Class abilities"): the thief's Thrown exploit (`use:thrown_exploit:c:12,5`).
    /// Takes the time of a throw; refused (no time) without the ability, without an opening, or
    /// for a throw it may not make.
    UseThrow(ClassActive, usize, Pos),
    /// Use a class active in a weapon's shape, by the object's id (spec §8 "Prestige classes"):
    /// the Hunter's Spirit weapon (`use:spirit_weapon:long_sword`) calls a weapon of that shape
    /// into the main hand, for the time of a wield. Refused (no time) without the ability, or
    /// for an id that is no melee weapon's.
    Shape(ClassActive, String),
    /// Use a birth sign's active gift (spec §8 "Birth sign"; the abilities menu's Birth sign
    /// group): Horn's grace, Blaze, and, aimed at a cell, the Transfixing stare
    /// (`use:horns_grace`, `use:blaze`, `use:transfixing_stare:12,5`). Takes the time of its
    /// cost's action; refused (no time) without the gift, while its cooldown lasts, or without the cell it asks.
    Gift(SignActive, Option<Pos>),
    /// Take talent `id` (an id in `data/talents/`), for `choice` if it takes one (spec §8
    /// "Talents"): one of the picks a level brings, made at once (nothing is banked). Refused
    /// with the reason when none is due or a prerequisite fails. Takes no game time; recorded in
    /// replays.
    Talent {
        id: String,
        choice: Option<String>,
    },
    /// Spend one of the current level's skill rolls on trained skill `skill` (spec §8 "Skills";
    /// `world/skill_roll.rs`, `World::advance_skill`; docs/dev/skills.md): throws the skill's
    /// ladder die and adds the result to the skill. A level brings rolls by class, race,
    /// Intelligence, and talents (`skill_roll::roll_count`); refused with the reason when none is left to spend or the
    /// skill is not trained. Takes no game time; recorded in replays.
    Advance(String),
    /// Dev command: switch the player's vision profile (an id in `data/vision.toml`). Takes no
    /// game time, but changes state and perception, so replays record it like any action.
    SetVision(String),
    /// Dev command: move the clock ahead at once; nothing else acts.
    SkipTime(Duration),
    /// Dev command: set the weather for three hours of game time ([`crate::weather::OVERRIDE`]),
    /// then the weather of the day returns. Takes no game time; recorded in replays.
    SetWeather(Weather),
    /// Dev command: one attempt to bring a creature to the current level now (spec §7 "Danger
    /// levels"), whatever its cap and chance: a type by its danger, out of the player's sight.
    /// Refused on a level of danger 0. Takes no game time; recorded in replays.
    Spawn,
    /// Dev command: `count` allies of type `creature` beside the player, following, on their
    /// side (spec §8 "Allies"; `world/ally.rs`), their stay ending after `lasts` if given
    /// (`ally:wolf`, `ally:wolf:2`, `ally:wolf:2:10m`). No game time; refused without
    /// `data/allies.toml`, for an unknown type, at the cap, or with no free cell beside the
    /// player. Recorded in replays.
    Ally {
        creature: String,
        count: u8,
        lasts: Option<Duration>,
    },
    /// Dev command: overworld encounter entry `entry` (an id of the run's campaign's tables) is
    /// met now on the player's overworld cell (spec §7 "Overworld encounters"), whatever its
    /// tables say there: the lookouts as usual, or `outcome` forced (`meet:kobolds:ambushed`).
    /// Refused (no time) off the overworld or on a cell without a local area. Takes no game time;
    /// recorded in replays.
    Meet {
        entry: String,
        outcome: Option<crate::world::EncounterForced>,
    },
    /// Dev command: show spell effect `effect` (an id in `data/effects.toml`) at level cell `at`,
    /// or from `at` toward `to`: a burst around `at` (around `to`, flying there from `at`), or a
    /// bolt from `at` to `to` (cut short by the first opaque cell). It only shows: it hurts no
    /// one and takes no game time, but it is recorded in replays, so captures can show any moment
    /// of an effect (spec §4.8 "Spell effects").
    Effect {
        effect: String,
        at: Pos,
        to: Option<Pos>,
    },
    /// Dev command: identify the inventory stacks at these positions, or everything carried
    /// with none (spec §8 "Magic items"; the scroll of identify does it in play). Takes no
    /// game time; recorded in replays.
    Identify(Positions),
    /// Dev command: the player suffers one degree more of a status effect (an id in
    /// `data/status/`), as a wound or a bite would give it: for trying statuses out and for
    /// captures. Takes no game time; recorded in replays.
    Afflict(String),
    /// Dev command: the player spends this much mana (spec §8 "Mana"), as a spell will, through
    /// the one way to spend it; refused when they have less. For captures. Takes no game time;
    /// recorded in replays.
    Drain(u16),
    /// Dev command: the player is granted an alternative resource (an id in
    /// `data/resources.toml`, spec §8 "Resources") at its start, as the ability that grants it
    /// will, through the one way to grant it. For trying resources out and for captures. Takes no
    /// game time; recorded in replays.
    Resource(String),
    /// Dev command: the player catches a disease (an id in `data/diseases.toml`) as if they
    /// had failed to resist it: it incubates, then shows. Takes no game time; recorded in
    /// replays.
    Infect(String),
    /// Dev command: reveal a hidden location of the overworld (spec §7 "Hidden locations"), by
    /// its id, as a quest would: it is marked on the player's map. For tests and captures. Takes
    /// no game time; recorded in replays.
    Reveal(String),
    /// Dev command: active lure quest `ID`'s creature (spec §8 "Quests": "Lure quests") comes
    /// beside the player, aware of them: made if it is nowhere, else taken from wherever it is
    /// (`world/lure.rs`). For replays, tests, and captures that skip the walk. Takes no game
    /// time; recorded in replays.
    Lure(String),
    /// Dev command: the player is given profane gift `ID` (an id in `data/profane/`) as a draw
    /// would give it, its relations applied (`world/profane.rs`); refused for one a draw would
    /// not offer. For tests and captures. Takes no game time; recorded in replays.
    Profane(String),
    /// Dev command: profane gift `ID` is taken from the player (`World::rescind`). Takes no game
    /// time; recorded in replays.
    Rescind(String),
    /// Dev command: active rolled quest `quest`, with no progress yet, draws `target` instead
    /// (spec §8 "Quests": a target its pool may hold, or a variant's id), its kills counted
    /// from now and a variant's reveal made. For replays, tests, and captures. Takes no game
    /// time; recorded in replays.
    QuestRoll {
        quest: String,
        target: String,
    },
    /// Dev command: spend every choice a level still asks for with the first the player may
    /// take (spec §8 "Talents"), for tests and benchmarks. Takes no game time; recorded in
    /// replays.
    LevelUpAuto,
    /// Dev command: the player gains this much experience, and any levels it brings (spec §8
    /// "Experience"), for tests and captures. Takes no game time; recorded in replays.
    GrantXp(u32),
    /// Dev command: the player is called into prestige class `ID` (spec §8 "Prestige
    /// classes") now, whatever their level, through the calling of the run's campaign that
    /// opens it to their class, as saying its words would: the class is theirs, and the
    /// experience the first prestige level costs with it. Refused with no such prestige class,
    /// one held already, or no calling open to their class. For tests and captures. Takes no
    /// game time; recorded in replays.
    Call(String),
    /// Dev command: the player's rank in weapon skill or fighting style `skill` (an id in
    /// `data/weapon_skills.toml`) becomes `rank` (0 to 10), its marks starting from 0, as a
    /// rank risen would (the skills it raises by default rise too, and what the ranks teach is
    /// said). For tests and captures. Takes no game time; recorded in replays.
    SetRank {
        skill: String,
        rank: u8,
    },
    /// Dev command: an instant frost (spec §5 "Ice"): the water within `radius` of cell `at`, in
    /// its line of sight, freezes at once to ice that holds a laden person
    /// ([`crate::ice::FROST`]). Takes no game time; recorded in replays.
    Freeze {
        at: Pos,
        radius: i32,
    },
    /// Dev command: a burst of heat (standing in for a fireball): the ice within `radius` of
    /// cell `at`, in its line of sight, melts at once, and whoever stood on it falls in. Takes no
    /// game time; recorded in replays.
    Thaw {
        at: Pos,
        radius: i32,
    },
    /// Dev command: a zone (`world/zone.rs`) of `kind` around cell `at`, within `radius` in its
    /// line of sight, by the player, for ten turns. Takes no game time; recorded in replays.
    Zone {
        kind: ZoneKind,
        at: Pos,
        radius: i32,
    },
}

/// The largest radius of the dev frost and heat bursts.
pub const MAX_BURST: i32 = 20;

/// The most turns one search action may last (`search:N`).
pub const MAX_SEARCH: u8 = 100;

/// An order to one's allies ([`Action::Order`]; spec §8 "Allies"): what the three abilities of
/// the Allies group tell every ally on the level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Command {
    /// Come to heel: follow (the default behaviour).
    Heel,
    /// Hold here: each holds the cell it stands on.
    Hold,
    /// Set on: go to that cell and fight what is near it.
    SetOn(Pos),
}

/// How long to rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rest {
    /// For this long (at most a day).
    For(Duration),
    /// Until the next dawn, 06:00.
    Dawn,
    /// Until the next dusk, 18:00.
    Dusk,
}

/// The longest rest; resting until dawn or dusk is never longer either.
pub const MAX_REST: Duration = Duration::seconds(DAY);

/// Where the food of a meal is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Meal {
    /// Inventory stack at this position (`a` = 0): `eat:b`.
    Pack(usize),
    /// The stack at this position in the pile on the player's cell (`a` = the top):
    /// `eat:floor:a`.
    Floor(usize),
}

/// What a scroll is read at ([`Action::Read`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadTarget {
    /// A level cell, aimed at: `read:b:12,5`.
    Cell(Pos),
    /// An inventory stack, by position: `read:b:c` (the scroll of identify).
    Item(usize),
}

/// A set of stacks of a list (a pile, the inventory) by position, `0` the first: the rules name
/// stacks by where they are, never by the letter a screen shows (letters restart on every page
/// of a list, spec §6). Action text writes the set as letters while every position is below 26
/// (`a` = 0: `ac`), else as numbers (`0,2,30`); both forms parse, mixed too (`ac,30`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Positions(BTreeSet<usize>);

/// The letters naming list rows, `a` to `z`: action text writes the positions below this as
/// letters, and a page of a list holds at most this many stacks.
pub const LETTERS: usize = 26;

impl Positions {
    pub fn of(positions: impl IntoIterator<Item = usize>) -> Self {
        Self(positions.into_iter().collect())
    }

    pub fn insert(&mut self, i: usize) {
        self.0.insert(i);
    }

    pub fn remove(&mut self, i: usize) {
        self.0.remove(&i);
    }

    pub fn contains(&self, i: usize) -> bool {
        self.0.contains(&i)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The positions, in order.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter().copied()
    }
}

/// The letter of list row `i`: `a` for 0. Rows past `z` have none.
pub fn letter(i: usize) -> Option<char> {
    (i < LETTERS).then(|| char::from(b'a' + i as u8))
}

/// Position `i` in action text: its letter below 26 (`c`), else its number (`30`).
pub fn position_text(i: usize) -> String {
    letter(i).map_or_else(|| i.to_string(), String::from)
}

/// One position in action text: a letter `a` to `z` (either case) or a number.
fn parse_position(text: &str) -> Option<usize> {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_alphabetic() => {
            Some(usize::from(c.to_ascii_lowercase() as u8 - b'a'))
        }
        _ if text.bytes().all(|b| b.is_ascii_digit()) => text.parse().ok(),
        _ => None,
    }
}

impl fmt::Display for Positions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.iter().all(|i| i < LETTERS) {
            return (self.iter().filter_map(letter)).try_for_each(|c| write!(f, "{c}"));
        }
        let numbers: Vec<String> = self.iter().map(|i| i.to_string()).collect();
        f.write_str(&numbers.join(","))
    }
}

impl FromStr for Positions {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut positions = Positions::default();
        for part in s.split(',') {
            let letters = !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase());
            let number = !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
            if letters {
                part.bytes()
                    .for_each(|b| positions.insert(usize::from(b - b'a')));
            } else if let (true, Ok(i)) = (number, part.parse()) {
                positions.insert(i);
            } else {
                return Err(format!(
                    "`{part}` is neither letters a to z nor a position such as `30`"
                ));
            }
        }
        Ok(positions)
    }
}

/// Something the player perceived happening during a turn. Presentation turns these into
/// animations; they never carry information about things the player could not perceive.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    ActorMoved {
        actor: ActorId,
        from: Pos,
        to: Pos,
    },
    DoorOpened {
        pos: Pos,
    },
    DoorClosed {
        pos: Pos,
    },
    /// A kick at a closed door that held: "WHAMM!" (spec §8 "Doors").
    DoorKicked {
        pos: Pos,
    },
    /// A door kicked in: it broke, and is open for good.
    DoorBroken {
        pos: Pos,
    },
    /// A chest kicked open or blown apart (spec §8 "Chests").
    ChestBroken {
        pos: Pos,
    },
    LevelChanged {
        from: String,
        to: String,
    },
    /// An actor attacked another (spec §8 "Combat"). `from` and `to` are where they stood at
    /// that moment; `strike` is what came of it: a hit (a critical, perhaps) or a miss and why,
    /// or `None` for an intent only (a spell). Kept only when the player saw both of them (or
    /// is one of them); the message line reports more (an attack on the player by something
    /// unseen, an attack the player saw only one end of).
    Attacked {
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        from: Pos,
        to: Pos,
        strike: Option<Strike>,
        /// What a shot or a throw was made with: the missile (the ammunition as it flew, the
        /// thrown thing); for a shield bash, the shield; `None` for other blows and spells.
        with: Option<ObjectId>,
        /// The kind of damage it did (spec §8 "Damage types": what a blow is said with, a
        /// sword's slashing or piercing by what was better); `None` for a miss, an intent, or
        /// an untyped attack.
        damage: Option<DamageTypeId>,
        /// For a melee blow struck with one of the attacker's natural attacks after the first
        /// (a dragon's claws and bite, `[[creature.attack]]`), its index among them all
        /// (`CreatureDef::natural_attacks`), which its words are said with; `None` otherwise.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        natural: Option<u8>,
        /// How the player's attack that rolled to hit (a blow, a shot, a throw, a spell) found
        /// its target, told in the attack's own words ("You hit the sleeping kobold."): `None`
        /// for an aware target and for anyone else's attack (spec §8 "Surprise and sneak
        /// attacks"; `surprise.rs`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caught: Option<Caught>,
    },
    /// A missile flew (spec §8 "Ranged combat"): a shot or a throw. As the player perceives it
    /// (the rest of the turn's events are kept only when seen; this one is cut to what was
    /// seen): `from` is where it was loosed if the player saw that cell, `path` the cells it
    /// flew over, in order, from the first the player saw to the last, and `end` how its flight
    /// ended on the last of them. Kept when the player saw any of its path.
    Missile {
        /// What flew: the object as it flies and lands (a flaming arrow flies burning).
        object: ObjectId,
        from: Option<Pos>,
        path: Vec<Pos>,
        end: MissileEnd,
    },
    /// An actor died where it stood, killed by the attack (or trap) before it, or of status
    /// effect `of` (it bled to death). Kept when the player saw it (their own death always).
    Died {
        actor: ActorId,
        at: Pos,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        of: Option<StatusId>,
    },
    /// A status effect began (`began`) or ended on an actor (spec §8 "Status effects"). Only the
    /// player's own, which they always perceive: said in the message line ("You are bleeding.",
    /// "The bleeding stops.").
    Status {
        actor: ActorId,
        status: StatusId,
        began: bool,
    },
    /// An alternative resource that ends at zero ended on an actor (spec §8 "Resources"): its
    /// bar is gone. Only the player's own, which they always perceive: said in the message line
    /// in the resource's words (its `ended`, "Your rage subsides.").
    ResourceEnded {
        actor: ActorId,
        resource: ResourceId,
    },
    /// A disease of the player's showed itself, went badly at a progression, or left them (spec
    /// §8 "Diseases"). Only the player's own, which they always perceive: said in the message
    /// line in the disease's own words (its `onset`, a symptom, `recovered`).
    Disease {
        actor: ActorId,
        disease: DiseaseId,
        news: DiseaseNews,
    },
    /// A drain touched an actor (spec §8 "Stats": "Drain"): `stat` lost points for good, or
    /// none when it held (or had nothing more to lose). Only the player's own, which they always
    /// perceive: said in the message line in words, never the points.
    Drained {
        actor: ActorId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stat: Option<crate::stats::Stat>,
    },
    /// A trap went off (spec §8 "Traps"): `victim` set off trap `trap` lying on `at`. Kept when
    /// the player perceived it: they set it off, saw its cell, or saw any of its blast.
    TrapSprung {
        trap: TrapId,
        at: Pos,
        victim: ActorId,
    },
    /// What a trap did to `target`, standing on `to`: what came of its attack roll, or a hit for
    /// what is not rolled (a blast, a fall). Kept when the player is the target or saw them.
    TrapStruck {
        trap: TrapId,
        target: ActorId,
        to: Pos,
        strike: Strike,
    },
    /// A spell-like effect the player perceives (spec §4.8 "Spell effects"): traps now, spells
    /// later. As the player perceives it: it is light itself, so its cells are cut to those the
    /// player sees or has in line of sight (in the dark too), and a bolt's path to the stretch
    /// from the first of them to the last. Kept when the player saw any of its cells.
    Spell {
        /// An id in `data/effects.toml` (`fireball`, `lightning_bolt`): what it looks like.
        effect: String,
        /// Where it comes from: the trap's cell, later a caster.
        origin: Pos,
        shape: SpellShape,
        /// The affected cells the player can see; perception-filtered, never ground truth.
        cells: Vec<Pos>,
    },
    /// The player identified inventory stack `item` by the Identify spell (a cast or the scroll
    /// of identify) and learned something of it. The screen for that stack opens (the game's
    /// description screen). Not sent for the other ways of identifying.
    Identified {
        item: usize,
    },
    /// The player chatted with a creature, or asked it about a topic, and it answered (spec §8
    /// "Dialogue"): what it said and what the player may ask it about. One with topics opens
    /// the dialogue screen.
    Talked(Conversation),
    /// The player threw their voice to this cell (spec §5 "Hearing"; a cell they see): where it
    /// sounded.
    VoiceThrown {
        at: Pos,
    },
    /// The player called an order to their allies (spec §8 "Allies") from this cell.
    Ordered {
        at: Pos,
    },
    /// An ally departed (spec §8 "Allies"): its time ran out where the player is. Nothing is
    /// left on its cell; presentation may fade it.
    Departed {
        actor: ActorId,
        at: Pos,
    },
    /// A wall gave way (spec §8 "Digging") and is rubble now. Kept when the player sees the cell
    /// afterwards.
    WallBroken {
        pos: Pos,
        cause: Breach,
    },
    /// An actor broke through the ice and plunged into the water at `at` (spec §5 "Ice"). Kept
    /// when the player saw it (their own always).
    FellThroughIce {
        actor: ActorId,
        at: Pos,
    },
}

/// How the player's attack found its target ([`Event::Attacked`]): what its words say of it,
/// after the fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Caught {
    /// Asleep: "the sleeping kobold".
    Asleep,
    /// Awake but unaware of the player, and Surprised by it (it failed its initiative check),
    /// or a disguised mimic: "the unsuspecting kobold".
    Unaware,
    /// Already suffering this flat-footed status (Surprised, Stunned), said by the status's
    /// `seen` word: "the stunned kobold".
    OffGuard(StatusId),
}

/// What became of a disease ([`Event::Disease`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiseaseNews {
    /// Its incubation is over: it shows.
    Onset,
    /// Its progression number `n` (1 for the first) went badly: a symptom.
    Worse(u16),
    /// It is gone.
    Recovered,
}

/// The shape of a spell effect ([`Event::Spell`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpellShape {
    /// A ball around `center` reaching `radius` cells (the rounded radius, like every radius:
    /// cells at squared distance up to `radius² + radius`).
    Burst { center: Pos, radius: i32 },
    /// A line along `path`, in order, from the cell next to its origin on.
    Bolt { path: Vec<Pos> },
}

/// What broke a wall.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Breach {
    /// Dug through with a tool.
    Dug,
    /// Blasted open by an explosion.
    Blast,
    /// Kicked apart (a statue, spec §8 "Statues").
    Kicked,
}

/// How a missile's flight ended, on the last cell of its path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissileEnd {
    /// It struck someone there.
    Struck,
    /// It came down there: it stopped against something solid, at the end of its range, or at
    /// the cell it was aimed at.
    Fell,
    /// It flew on out of the player's sight.
    Unseen,
}

/// How an attack is made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackKind {
    /// From the next cell.
    Melee,
    /// A shot from a distance, with a missile weapon.
    Ranged,
    /// A throwing weapon thrown from a distance.
    Thrown,
    /// A spell cast from a distance.
    Spell,
}

impl AttackKind {
    /// The action whose cost it has (`data/actions.toml`: `melee`, `shoot`, `throw`, `cast`).
    pub const fn action(self) -> ActionKind {
        match self {
            AttackKind::Melee => ActionKind::Melee,
            AttackKind::Ranged => ActionKind::Shoot,
            AttackKind::Thrown => ActionKind::Throw,
            AttackKind::Spell => ActionKind::Cast,
        }
    }
}

/// The result of applying an action.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Whether at least one turn passed (creatures acted). Refused actions (walking into a wall)
    /// and free actions take none.
    pub took_turn: bool,
    /// Game time that passed: a step, a rest, a skip; zero for refused and free actions.
    pub elapsed: Duration,
    /// Perceived events, in the order they happened.
    pub events: Vec<Event>,
}
