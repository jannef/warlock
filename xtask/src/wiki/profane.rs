//! The profane gifts (docs/design/profane-gifts.md; spec §8 "Hunger"): what each keeper's corpse
//! gives (its signature gift, its rolls, and their tier odds) and every gift a roll may draw,
//! with its odds within its tier, what it does, its relations, and the races it is never drawn
//! for.

use super::talents::{effects_text, talent_link};
use super::util::{Term, anchored, link, percent_of};
use std::fmt::Write as _;
use warlock_core::content::{Content, GiftDef, GiftTier, TalentId};

/// A gift's name, linked to its row.
fn gift_link(content: &Content, talent: TalentId) -> String {
    let def = content.talents().get(talent);
    talent_link(content, &def.id, &def.name)
}

/// The tier's word, in the tables.
fn tier_name(tier: GiftTier) -> &'static str {
    match tier {
        GiftTier::Lesser => "Lesser",
        GiftTier::Greater => "Greater",
        GiftTier::Dire => "Dire",
    }
}

/// Every profane corpse, by its creature's level: the keeper, its signature gift with what it
/// does, the rolls on the table, and each roll's odds of each tier.
pub(super) fn profane_corpses(content: &Content) -> String {
    let book = content.talents();
    let mut rows: Vec<_> = (content.creature_defs().iter())
        .filter_map(|c| {
            let eat = c.on_eat.as_ref()?;
            Some((c, eat.gives.as_deref()?, eat.profane.as_ref()?))
        })
        .collect();
    rows.sort_by(|a, b| (a.0.level, &a.0.id).cmp(&(b.0.level, &b.0.id)));
    if rows.is_empty() {
        return "No corpse is profane.\n".into();
    }
    let mut out = String::from(
        "| Keeper | Signature gift | Kind | What it does | Rolls | Lesser | Greater | Dire |\n\
         |---|---|---|---|:---:|:---:|:---:|:---:|\n",
    );
    for (creature, gives, rolls) in rows {
        let Some(talent) = book.id(gives) else {
            continue;
        };
        let gift = content
            .profane()
            .gift(talent)
            .expect("a signature is a gift");
        let def = book.get(talent);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {}% | {}% | {}% |",
            link(Term::Creature, &creature.id, &creature.name),
            anchored(Term::Talent, &def.id, &def.name),
            gift.kind.id(),
            effects_text(content, def),
            rolls.rolls,
            rolls.weight(GiftTier::Lesser),
            rolls.weight(GiftTier::Greater),
            rolls.weight(GiftTier::Dire),
        );
    }
    out
}

/// What a gift does to the others: the gifts it replaces, undoes, or cannot be held with.
fn relations(content: &Content, gift: &GiftDef) -> String {
    let names = |list: &[TalentId]| {
        (list.iter())
            .map(|&t| gift_link(content, t))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut parts = Vec::new();
    if !gift.replaces.is_empty() {
        parts.push(format!("replaces {}", names(&gift.replaces)));
    }
    if !gift.undoes.is_empty() {
        parts.push(format!(
            "undoes {} instead of taking, if held",
            names(&gift.undoes)
        ));
    }
    if !gift.excludes.is_empty() {
        parts.push(format!("never with {}", names(&gift.excludes)));
    }
    if parts.is_empty() {
        "-".into()
    } else {
        parts.join("; ")
    }
}

/// Every gift a roll may draw, by tier and then name: its kind, its odds within its tier while
/// none of the tier is held or barred, what it does, its relations, and the races it is never
/// drawn for.
pub(super) fn profane_gifts(content: &Content) -> String {
    let book = content.talents();
    let mut gifts: Vec<&GiftDef> = content.profane().iter().filter(|g| g.drawn()).collect();
    if gifts.is_empty() {
        return "No gift is drawn.\n".into();
    }
    gifts.sort_by(|a, b| {
        let name = |g: &GiftDef| book.get(g.talent).name.clone();
        (a.tier, name(a)).cmp(&(b.tier, name(b)))
    });
    let mut out = String::from(
        "| Gift | Tier | Kind | Odds in its tier | What it does | With other gifts | Never for |\n\
         |---|---|---|:---:|---|---|---|\n",
    );
    for gift in gifts {
        let total: u32 = (content.profane().iter())
            .filter(|g| g.tier == gift.tier && g.drawn())
            .map(|g| u32::from(g.weight))
            .sum();
        let def = book.get(gift.talent);
        let races = if gift.not_for.is_empty() {
            "-".to_string()
        } else {
            (gift.not_for.iter())
                .map(|r| {
                    let name = content
                        .race(r)
                        .map_or(r.as_str(), |race| race.name.as_str());
                    link(Term::Race, r, name)
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {races} |",
            anchored(Term::Talent, &def.id, &def.name),
            tier_name(gift.tier),
            gift.kind.id(),
            percent_of(u32::from(gift.weight), total),
            effects_text(content, def),
            relations(content, gift),
        );
    }
    out
}
