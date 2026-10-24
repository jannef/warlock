//! Alignment (spec §8 "Alignment"): the bands with their scores, what moves the score and by how
//! much, what reads it, and the score and band each race and class starts with. The game shows
//! only the band; the wiki states the numbers exactly (the game is vague, the wiki is accurate).

use super::text::{capital, signed};
use std::fmt::Write as _;
use warlock_core::alignment::{Band, Score};
use warlock_core::content::{AlignmentRules, Content};

/// What a deed of worth `amount` does to the score, in words: "very strongly evil", "a little
/// good", "nothing".
fn worth(amount: i32) -> String {
    let way = if amount > 0 { "good" } else { "evil" };
    let how = match amount.abs() {
        0 => return "nothing".into(),
        1..=10 => "a little",
        11..=50 => "somewhat",
        51..=120 => "strongly",
        _ => "very strongly",
    };
    format!("{how} {way}")
}

/// A score or a worth with its sign (a real minus), 0 bare.
fn signed_or_zero(n: i32) -> String {
    if n == 0 { "0".into() } else { signed(n) }
}

fn rules(content: &Content) -> Option<&AlignmentRules> {
    content.alignment()
}

/// The seven bands, best first, with what each means.
pub(super) fn alignment_bands(content: &Content) -> String {
    let Some(rules) = rules(content) else {
        return "There is no alignment.\n".into();
    };
    let mut out = String::from("| Band | Scores | What it means |\n|---|:---:|---|\n");
    let mut top = warlock_core::alignment::MAX;
    for def in rules.bands() {
        let scores = format!("{} to {}", signed_or_zero(def.from), signed_or_zero(top));
        let _ = writeln!(
            out,
            "| {} | {scores} | {} |",
            def.band.name(),
            def.description
        );
        top = def.from - 1;
    }
    out
}

/// What each deed does to the score, in words, before it diminishes.
pub(super) fn alignment_deeds(content: &Content) -> String {
    let Some(rules) = rules(content) else {
        return "There is no alignment.\n".into();
    };
    let s = &rules.shift;
    let rows = [
        (
            "Killing a good creature that is not hostile to you",
            s.kill_peaceful.good,
        ),
        (
            "Killing a neutral creature that talks and is not hostile to you",
            s.kill_peaceful.neutral,
        ),
        (
            "Killing an evil creature that is not hostile to you",
            s.kill_peaceful.evil,
        ),
        (
            "Killing a beast that is not hostile to you (hunting)",
            s.kill_beast,
        ),
        (
            "Killing an evil creature that is hostile to you",
            s.kill_hostile.evil,
        ),
        (
            "Killing a neutral creature that is hostile to you",
            s.kill_hostile.neutral,
        ),
        (
            "Killing a good creature that is hostile to you",
            s.kill_hostile.good,
        ),
        (
            "Attacking a shopkeeper who trades with you",
            s.attack_shopkeeper,
        ),
        (
            "Leaving a shop with goods you have not paid for, or stealing unnoticed",
            s.theft,
        ),
        ("Paying a shopkeeper what you owe", s.pay_debt),
        (
            "Eating one of your own race, or a peaceful creature who talks",
            s.eat_kin,
        ),
    ];
    let mut out = String::from("| Deed | Moves you | Worth |\n|---|---|:---:|\n");
    for (deed, amount) in rows {
        let _ = writeln!(
            out,
            "| {deed} | {} | {} |",
            worth(amount),
            signed_or_zero(amount)
        );
    }
    out
}

/// The quests that move the score or ask for an alignment, by name.
pub(super) fn alignment_quests(content: &Content) -> String {
    let mut quests: Vec<_> = (content.quests().iter())
        .filter(|q| q.alignment != 0 || q.requires_alignment.is_some())
        .collect();
    quests.sort_by(|a, b| a.name.cmp(&b.name));
    if quests.is_empty() {
        return "No quest moves your alignment.\n".into();
    }
    let mut out = String::from(
        "| Quest | Given by | Completing it is | Worth | Offered to |\n|---|---|---|:---:|---|\n",
    );
    for q in quests {
        let offered = q.requires_alignment.map_or("anyone".into(), |r| r.words());
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {offered} |",
            q.name,
            capital(&content.speaker_name(q.giver)),
            worth(q.alignment),
            signed_or_zero(q.alignment),
        );
    }
    out
}

/// How a good shopkeeper's prices follow the band, in words, for the bands that change them.
pub(super) fn alignment_shops(content: &Content) -> String {
    let Some(rules) = rules(content) else {
        return "There is no alignment.\n".into();
    };
    let mut out =
        String::from("| Band | A good shopkeeper's prices | In your favour |\n|---|---|:---:|\n");
    for band in Band::ALL {
        let favour = rules.favour(band);
        let how = if favour.abs() <= 10 {
            "a little"
        } else {
            "much"
        };
        let way = if favour > 0 { "lower" } else { "higher" };
        if favour != 0 {
            let _ = writeln!(
                out,
                "| {} | {how} {way} | {} % |",
                band.name(),
                signed(favour)
            );
        }
    }
    out.push_str("| Any other | as for anyone | 0 % |\n");
    out
}

/// The score and band a new character starts with, by class (rows) and race (columns), both by
/// name: the race's `alignment` plus the class's.
pub(super) fn alignment_start(content: &Content) -> String {
    let Some(rules) = rules(content) else {
        return "There is no alignment.\n".into();
    };
    let mut races: Vec<_> = content.races().list.iter().collect();
    races.sort_by(|a, b| a.name.cmp(&b.name));
    let mut classes: Vec<_> = content.classes().iter().collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from("| Class |");
    for r in &races {
        let _ = write!(out, " {} |", r.name);
    }
    out.push_str("\n|---|");
    out.push_str(&":---:|".repeat(races.len()));
    out.push('\n');
    for c in classes {
        let _ = write!(out, "| {} |", c.name);
        for r in &races {
            let score = Score::new(r.alignment + c.alignment);
            let band = rules.band(score);
            let _ = write!(
                out,
                " {} ({}) |",
                band.name(),
                signed_or_zero(score.value())
            );
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deeds_are_told_in_words_and_numbers() {
        assert_eq!(worth(0), "nothing");
        assert_eq!(worth(5), "a little good");
        assert_eq!(worth(-100), "strongly evil");
        assert_eq!(worth(-200), "very strongly evil");
        let content = Content::builtin().expect("content");
        let start = alignment_start(&content);
        assert!(start.contains("| Paladin | Good (+450) |"), "{start}");
        let bands = alignment_bands(&content);
        assert!(bands.contains("| Saintly | +750 to +1000 |"), "{bands}");
        assert!(
            bands.contains("| Neutral | \u{2212}149 to +149 |"),
            "{bands}"
        );
        assert!(
            bands.contains("| Wicked | \u{2212}1000 to \u{2212}750 |"),
            "{bands}"
        );
        let deeds = alignment_deeds(&content);
        assert!(
            deeds.contains("| very strongly evil | \u{2212}150 |"),
            "{deeds}"
        );
    }
}
