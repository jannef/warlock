//! Hearing (spec §5 "Hearing"): how loud each noise is where it is made, how far it carries, and
//! what muffles it, from the rules' constants and the content.

use super::text::{article, capital};
use std::fmt::Write as _;
use warlock_core::content::Content;
use warlock_core::noise::{
    self, DIAGONAL_LOSS, DIG_NOISE, KICK_NOISE, MAX_REACH, MELEE_NOISE, SHOT_NOISE, SPELL_NOISE,
    STEP_LOSS, SoundKind, THROW_NOISE, VOICE_NOISE,
};

/// How far a noise of `loudness` carries to a listener whose threshold is `hearing`, in open
/// space: `5 cells (3 diagonally)`; `-` if it never does.
fn reach(loudness: u16, hearing: u16) -> String {
    let Some(over) = loudness.checked_sub(hearing) else {
        return "-".into();
    };
    let straight = (over / STEP_LOSS).min(MAX_REACH as u16);
    let diagonal = (over / DIAGONAL_LOSS).min(MAX_REACH as u16);
    let cells = |n: u16| {
        if n == 1 {
            "1 cell".into()
        } else {
            format!("{n} cells")
        }
    };
    format!("{} ({} diagonally)", cells(straight), diagonal)
}

/// Every noise and how loud it is where it is made ([`noise::noise`]'s constants, the doors'
/// `toggle_noise`, the traps' noise), what the player hears it as, and how far the player hears
/// it in open space with the hearing they start with (the player creature's; the listen skill
/// lowers it).
pub(super) fn noises(content: &Content) -> String {
    let player = content.creature_id("player").map(|p| content.creature(p));
    let hearing = player
        .and_then(|p| p.hearing)
        .unwrap_or(noise::MAX_LOUDNESS);
    let mut rows: Vec<(String, SoundKind, u16)> = Vec::new();
    if let Some(p) = player.filter(|p| p.footsteps > 0) {
        rows.push((
            "Your step onto normal ground".into(),
            SoundKind::Footsteps,
            p.footsteps,
        ));
    }
    for t in content.terrain_defs().iter().filter(|t| t.toggle_noise > 0) {
        let (verb, kind) = if t.is_closed_door() {
            ("Opening", SoundKind::DoorOpened)
        } else {
            ("Closing", SoundKind::DoorClosed)
        };
        let name = t.name.to_lowercase();
        rows.push((
            format!("{verb} {} {name}", article(&name)),
            kind,
            t.toggle_noise,
        ));
    }
    rows.extend([
        ("Kicking a door".to_string(), SoundKind::Banging, KICK_NOISE),
        ("A blow up close".into(), SoundKind::Fighting, MELEE_NOISE),
        ("A shot".into(), SoundKind::Bowstring, SHOT_NOISE),
        ("A throw".into(), SoundKind::Fighting, THROW_NOISE),
        ("A spell".into(), SoundKind::Chanting, SPELL_NOISE),
        ("A turn of digging".into(), SoundKind::Digging, DIG_NOISE),
        ("Muttering".into(), SoundKind::Voice, VOICE_NOISE),
    ]);
    for t in content.trap_defs() {
        if let Some(n) = t.noise {
            let name = t.name.to_lowercase();
            rows.push((
                capital(&format!("{} {name} going off", article(&name))),
                n.sound,
                n.loudness,
            ));
        }
    }
    let mut out = format!(
        "| Noise | Heard as | Loudness where it is made | You hear it up to (hearing {hearing}) \
         |\n|---|---|:---:|---|\n"
    );
    for (what, kind, loudness) in rows {
        let _ = writeln!(
            out,
            "| {what} | {} | {loudness} | {} |",
            kind.heard_as(),
            reach(loudness, hearing)
        );
    }
    out
}

/// How sound travels and how loud it seems, from the rules' constants ([`noise::propagate`],
/// and the volume bands of the perceived sound, `noise::Volume`).
pub(super) fn sound_travel(_content: &Content) -> String {
    format!(
        "- A sound loses {STEP_LOSS} loudness for every cell it travels straight and \
         {DIAGONAL_LOSS} for every diagonal one, more where something muffles it (below), and \
         nothing carries farther than {MAX_REACH} cells.\n\
         - Whoever it reaches at their **hearing** or louder hears it: the lower your hearing, \
         the fainter the sounds you hear.\n\
         - You hear a sound that reaches you at {loud} or more above your hearing as loud, \
         placed where it was made; at {clear} or more above it as clear, placed up to a cell \
         off; otherwise faintly, placed up to two cells off.\n\
         - A sound is no news, and gets no message, if you heard the same kind of sound near the \
         same place within the last {recall} turns.\n",
        loud = 3 * STEP_LOSS,
        clear = STEP_LOSS,
        recall = noise::RECALL_TURNS,
    )
}

/// The terrain sound passes through but loses loudness entering (`muffle`): doors.
pub(super) fn muffling(content: &Content) -> String {
    let mut out = String::from("| Terrain | Muffles by | As much as |\n|---|:---:|---|\n");
    for t in content.terrain_defs() {
        if let Some(m) = t.muffle.filter(|&m| m > 0) {
            let _ = writeln!(
                out,
                "| {} | {m} | {} more cells |",
                capital(&t.name),
                m / STEP_LOSS
            );
        }
    }
    out
}
