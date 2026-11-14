//! The stealth talents (#1495): what a talent does to the holder's stealth and surprise: noise,
//! ambushes, initiative, sneak attacks, the thief's exploits, and the rogue's skill abilities.
//! [`StealthEffect`] is the data (a talent's `[[effect]]` of one of [`StealthEffect::KINDS`]),
//! [`StealthMods`] the sum every rule reads (`TalentMods::stealth`). A new effect of the area is a
//! variant here, its field and its arm in [`StealthMods::add`] here, its wiki text in
//! `xtask/src/wiki/talents/stealth.rs`, and its one read.

use crate::content::{Content, StatusId};
use crate::dice::Dice;
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StealthEffect {
    /// Footsteps this percent louder (negative: quieter).
    FootstepsPercent { value: i32 },
    /// The doors the actor opens and closes make no noise.
    SilentDoors,
    /// Its blows and shots at a creature that has not seen it do `dice` more damage, after
    /// Protection (Ambusher).
    Ambush { dice: Dice },
    /// This much added to its initiative check when it is caught unawares (Wary;
    /// `World::startle`, `world/combat/surprise.rs`).
    InitiativeBonus { value: i32 },
    /// This percent of the extra cost of the action it takes Surprised is spared (Cool head:
    /// 100, none; `World::surprise_cost`).
    SurpriseEase { value: i32 },
    /// Those it catches unawares find their initiative check this much harder (the thief's
    /// Sneak attack; `World::startle`'s craft).
    SurpriseDifficulty { value: i32 },
    /// Its shots, throws, and attack spells sneak-attack this many cells farther than the
    /// rules' `missile_reach` (Long reach; `World::sneaks`).
    SneakReach { value: i32 },
    /// A sneak attack of its that kills leaves it a degree of `status` (Vanishing act:
    /// Light-footed; read in `World::resolve` where a sneak attack's blow kills, `world/classes/thief.rs`).
    SneakKillStatus { status: String },
    /// The perception snapshot tells which creatures in view have not noticed it, and they are
    /// drawn with the unaware mark (Stalker's eye; `SeenActor::unaware`).
    SeesUnaware,
    /// Its steps make no sound while it is Unburdened (Hide in plain sight).
    SilentUnburdened,
    /// Its attacks are this percent louder (negative: quieter).
    AttackNoisePercent { value: i32 },
    /// It can pick locks (the Pick lock ability) without Open lock.
    PickLock,
    /// It can steal (the Steal ability) without the Steal skill.
    Steal,
    /// It can throw its voice (the Throw your voice ability; the bard's).
    ThrowVoice,
    /// Its attacks on a creature that has not seen it threaten a critical this many points
    /// lower (Master thief).
    AmbushThreat { value: u8 },
    /// The class active Lurk: a wait in ambush, a degree of `status` on it (Lurking: a hostile
    /// stepping up beside it takes its blow at once; `world/classes/thief.rs`).
    Lurk { status: String },
    /// The class active Exploit: every opening held spent on a melee blow in a direction that
    /// is a sneak attack whatever the foe knows (its `resource` cost, the least it needs:
    /// Openings; `world/classes/openings.rs`).
    Exploit,
    /// The class active Thrown exploit: Exploit's blow as a throw at a creature on a cell (its
    /// `resource` cost, the least it needs: Openings; `world/classes/openings.rs`).
    ThrownExploit,
    /// An exploit that hits does `dice` more damage per opening spent, after Protection, never
    /// multiplied (Exploit's own 1d6; dice of one size add up).
    ExploitDice { dice: Dice },
    /// An exploit is `value` likelier to hit per opening spent (Low blow).
    ExploitToHit { value: i32 },
    /// An exploit that hits leaves the foe a degree of `status` (Low blow: Stunned).
    ExploitStatus { status: String },
    /// An exploit threatens a critical `value` lower per opening spent, never below 2, and with
    /// `sure_at` or more spent is a critical whenever it hits, no confirmation (Master thief).
    ExploitThreat { value: u8, sure_at: u16 },
}

impl StealthEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`StealthEffect`]).
    pub const KINDS: [&'static str; 22] = [
        "ambush",
        "ambush_threat",
        "attack_noise_percent",
        "exploit",
        "exploit_dice",
        "exploit_status",
        "exploit_threat",
        "exploit_to_hit",
        "footsteps_percent",
        "initiative_bonus",
        "lurk",
        "pick_lock",
        "sees_unaware",
        "silent_doors",
        "silent_unburdened",
        "sneak_kill_status",
        "sneak_reach",
        "steal",
        "surprise_difficulty",
        "surprise_ease",
        "throw_voice",
        "thrown_exploit",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StealthMods {
    /// Percent louder footsteps (negative: quieter).
    pub footsteps_percent: i32,
    /// The doors it opens and closes make no noise.
    pub silent_doors: bool,
    /// Damage its attacks do a creature that has not seen it, after Protection.
    pub ambush: Option<Dice>,
    /// Added to its initiative check when it is caught unawares (Wary).
    pub initiative_bonus: i32,
    /// Percent of the extra cost of its action taken Surprised that is spared (Cool head).
    pub surprise_ease: i32,
    /// Added to the initiative check's difficulty of those it catches unawares (the thief's
    /// Sneak attack).
    pub surprise_difficulty: i32,
    /// Cells farther its shots, throws, and attack spells sneak-attack (Long reach).
    pub sneak_reach: i32,
    /// The status a sneak attack of its that kills leaves it (Vanishing act).
    pub sneak_kill_status: Option<StatusId>,
    /// It sees which creatures in view have not noticed it (Stalker's eye).
    pub sees_unaware: bool,
    /// Its steps are silent while it is Unburdened (Hide in plain sight).
    pub silent_unburdened: bool,
    /// Percent more loudness of its attacks.
    pub attack_noise_percent: i32,
    /// It has the Pick lock ability without Open lock.
    pub pick_lock: bool,
    /// It has the Steal ability without the Steal skill.
    pub steal: bool,
    /// It has the Throw your voice ability.
    pub throw_voice: bool,
    /// Points lower its attacks on a creature that has not seen it threaten (Master thief).
    pub ambush_threat: u8,
    /// The active Lurk: the status it waits in ambush under (Lurking).
    pub lurk: Option<StatusId>,
    /// The active Exploit (`world/classes/openings.rs`): every opening held spent on a blow that is a
    /// sneak attack whatever the foe knows.
    pub exploit_active: bool,
    /// The active Thrown exploit (`world/classes/openings.rs`): Exploit's blow as a throw.
    pub thrown_exploit: bool,
    /// What an exploit adds per opening spent (Exploit's dice, Low blow's stun and to-hit).
    pub exploit: ExploitMods,
}

impl StealthMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: StealthEffect) {
        match effect {
            StealthEffect::FootstepsPercent { value } => self.footsteps_percent += value,
            StealthEffect::SilentDoors => self.silent_doors = true,
            StealthEffect::Ambush { dice } => {
                self.ambush = Some(self.ambush.map_or(dice, |had| had.and(dice)));
            }
            StealthEffect::InitiativeBonus { value } => self.initiative_bonus += value,
            StealthEffect::SurpriseEase { value } => self.surprise_ease += value,
            StealthEffect::SurpriseDifficulty { value } => self.surprise_difficulty += value,
            StealthEffect::SneakReach { value } => self.sneak_reach += value,
            StealthEffect::SeesUnaware => self.sees_unaware = true,
            StealthEffect::SneakKillStatus { status } => {
                self.sneak_kill_status = content.status_id(&status).or(self.sneak_kill_status);
            }
            StealthEffect::SilentUnburdened => self.silent_unburdened = true,
            StealthEffect::AttackNoisePercent { value } => self.attack_noise_percent += value,
            StealthEffect::PickLock => self.pick_lock = true,
            StealthEffect::Steal => self.steal = true,
            StealthEffect::ThrowVoice => self.throw_voice = true,
            StealthEffect::AmbushThreat { value } => self.ambush_threat += value,
            StealthEffect::Lurk { status } => {
                self.lurk = content.status_id(&status).or(self.lurk);
            }
            StealthEffect::Exploit => self.exploit_active = true,
            StealthEffect::ThrownExploit => self.thrown_exploit = true,
            StealthEffect::ExploitDice { dice } => {
                self.exploit.dice = Some(self.exploit.dice.map_or(dice, |had| had.and(dice)));
            }
            StealthEffect::ExploitToHit { value } => self.exploit.to_hit += value,
            StealthEffect::ExploitStatus { status } => {
                self.exploit.status = content.status_id(&status).or(self.exploit.status);
            }
            StealthEffect::ExploitThreat { value, sure_at } => {
                self.exploit.threat = self.exploit.threat.saturating_add(value);
                let sure = self.exploit.sure_at.map_or(sure_at, |had| had.min(sure_at));
                self.exploit.sure_at = Some(sure);
            }
        }
    }
}

/// What the thief's class talents add to an exploit, per opening spent (the openings design §6;
/// `world/classes/openings.rs::land_exploit`): dice of damage after Protection, never multiplied
/// (`exploit_dice`, summed), to hit (`exploit_to_hit`), a status the foe struck is left a
/// degree of (`exploit_status`), and a lower threat with a sure critical from `sure_at`
/// openings (`exploit_threat`: Master thief's finisher, folded in).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExploitMods {
    pub dice: Option<Dice>,
    pub to_hit: i32,
    pub status: Option<StatusId>,
    /// Points lower the exploit threatens a critical per opening spent (never below 2).
    pub threat: u8,
    /// With this many openings spent or more, every hit is a critical, no confirmation.
    pub sure_at: Option<u16>,
}

impl ExploitMods {
    /// The lowest threat the ladder brings a blow to: every natural roll that is no fumble
    /// threatens a critical.
    pub const LOWEST_THREAT: u8 = 2;

    /// `profile`, an exploit's attack spending `n` openings, with what the ladder does before
    /// the roll: n × its to-hit (Low blow), a threat n × its points lower, never below
    /// [`Self::LOWEST_THREAT`], and with `sure_at` or more spent a critical whenever it hits
    /// (Master thief: the finisher, folded in). The dice after the hit are
    /// `World::land_exploit`'s.
    pub fn profile(&self, n: u16, mut profile: crate::combat::Profile) -> crate::combat::Profile {
        profile.to_hit += self.to_hit * i32::from(n);
        let lower = u32::from(self.threat) * u32::from(n);
        let threat = u32::from(profile.threat)
            .saturating_sub(lower)
            .max(u32::from(Self::LOWEST_THREAT));
        profile.threat = u8::try_from(threat).map_or(profile.threat, |t| t.min(profile.threat));
        if self.sure_at.is_some_and(|at| n >= at) {
            profile.sure_critical = true;
        }
        profile
    }
}
