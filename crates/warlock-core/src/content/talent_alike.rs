//! The identical-effects check on the talent table (#790; docs/dev/progression.md "Coverage
//! policy", the #776 review's rules 3 and 4), run at load after every talent has parsed.
//!
//! - **No clones** (rule 3): no talent does at each of its ranks exactly what another does at
//!   the same ranks (a class ability that copies a pickable, Song of rest beside Fast healer, or
//!   a pickable copying another's first ranks). A class table names the pickable's id instead.
//! - **No chains of one effect** (rule 4): a talent whose one talent prerequisite shares an
//!   effect kind with it (Weapon specialization after Weapon focus, both `weapon`) is that
//!   talent's next rank, not an id of its own.
//!
//! `alike` in `data/talents.toml` lists the pairs kept on purpose (Cleave and Cleaving strike:
//! two cleave sources make a great cleave; the birth signs' gifts, which the owner keeps whole).
//! A listed pair that is no longer alike is an error too, so the list stays honest.

use super::{ContentError, TalentBook, TalentDef, TalentEffect};
use std::mem::discriminant;

/// What `def` does at each of its ranks: the effects that rank adds.
fn signature(def: &TalentDef) -> Vec<Vec<TalentEffect>> {
    (1..=def.rank_count())
        .map(|r| def.effects_at(r).collect())
        .collect()
}

/// Whether two effect lists hold the same effects, in any order.
fn same(a: &[TalentEffect], b: &[TalentEffect]) -> bool {
    a.len() == b.len() && a.iter().all(|e| b.contains(e)) && b.iter().all(|e| a.contains(e))
}

/// Whether `a` (no more ranks than `b`) does at each of its ranks what `b` does at the same
/// rank, and does something: a clone of `b` or of its first ranks. Talents taken for different
/// kinds of choice never match (Stylist's `weapon` reads a style, Schooled's a school).
fn clone_of(
    (a, sa): (&TalentDef, &[Vec<TalentEffect>]),
    (b, sb): (&TalentDef, &[Vec<TalentEffect>]),
) -> bool {
    a.choice == b.choice
        && sa.len() <= sb.len()
        && sa.iter().any(|r| !r.is_empty())
        && sa.iter().zip(sb).all(|(x, y)| same(x, y))
}

/// Whether `a` needs exactly one other talent, `b`, and shares an effect kind with it.
fn chained(a: &TalentDef, b: &TalentDef) -> bool {
    let kinds = |d: &TalentDef| {
        (1..=d.rank_count())
            .flat_map(|r| {
                d.effects_at(r)
                    .map(|e| discriminant(&e))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    let (ka, kb) = (kinds(a), kinds(b));
    ka.iter().any(|k| kb.contains(k))
}

/// The talents `def` needs, every rank's, each once.
fn needed(def: &TalentDef) -> Vec<super::TalentId> {
    let mut ids: Vec<_> = (def.needs.talents.iter())
        .chain(def.ranks.iter().flat_map(|r| r.needs.talents.iter()))
        .map(|&(id, _)| id)
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// The check: the first clone or chain not listed in `alike`, or the first listed pair that is
/// neither, in words, against `file` (`data/talents.toml`).
pub(super) fn check(
    book: &TalentBook,
    alike: &[[String; 2]],
    file: &str,
) -> Result<(), ContentError> {
    let listed =
        |a: &str, b: &str| (alike.iter()).any(|[x, y]| (x == a && y == b) || (x == b && y == a));
    let defs: Vec<(&TalentDef, Vec<Vec<TalentEffect>>)> =
        book.iter().map(|(_, d)| (d, signature(d))).collect();
    let mut found: Vec<(&str, &str)> = Vec::new();
    for (i, (a, sa)) in defs.iter().enumerate() {
        for (j, (b, sb)) in defs.iter().enumerate() {
            // Each pair once, but a shorter clone of a longer talent either way round.
            if i == j || (sa.len() == sb.len() && j < i) {
                continue;
            }
            if clone_of((a, sa), (b, sb)) {
                found.push((&a.id, &b.id));
                if !listed(&a.id, &b.id) {
                    return Err(ContentError::new(
                        file,
                        format!(
                            "talents `{}` and `{}` have the same effects: a class table names \
                             the pickable's id instead of a clone, and a talent that adds \
                             nothing new is cut (list the pair in `alike` if it is deliberate)",
                            a.id, b.id
                        ),
                    ));
                }
            }
        }
        if let [only] = needed(a)[..] {
            let b = book.get(only);
            if chained(a, b) {
                found.push((&a.id, &b.id));
                if !listed(&a.id, &b.id) {
                    return Err(ContentError::new(
                        file,
                        format!(
                            "talent `{}` needs only `{}` and shares an effect kind with it: make \
                             it that talent's next rank (list the pair in `alike` if it is \
                             deliberate)",
                            a.id, b.id
                        ),
                    ));
                }
            }
        }
    }
    for [x, y] in alike {
        for id in [x, y] {
            if book.id(id).is_none() {
                return Err(ContentError::new(
                    file,
                    format!("`alike` names an unknown talent `{id}`"),
                ));
            }
        }
        let is = |a: &str, b: &str| {
            found
                .iter()
                .any(|&(p, q)| (p == a && q == b) || (p == b && q == a))
        };
        if !is(x, y) {
            return Err(ContentError::new(
                file,
                format!("`alike` lists `{x}` and `{y}`, which are not alike: take the pair off"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentError, ContentFiles};

    /// The built-in content with `talents` added (`(file name, text)`) and the settings (the
    /// `alike` list) edited by `settings`.
    fn load(
        talents: &[(&str, &str)],
        settings: impl Fn(&str) -> String,
    ) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        for (name, text) in talents {
            files.insert(&format!("data/talents/{name}.toml"), text);
        }
        let edited = settings(files.get("data/talents.toml").expect("the settings"));
        files.insert("data/talents.toml", &edited);
        Content::load(&files)
    }

    fn refusal(result: Result<Content, ContentError>) -> String {
        result.expect_err("refused").to_string()
    }

    /// `alike` with the pair `[a, b]` listed first.
    fn listing(a: &'static str, b: &'static str) -> impl Fn(&str) -> String {
        move |s: &str| s.replace("alike = [", &format!("alike = [\n    [\"{a}\", \"{b}\"],"))
    }

    /// The bard's old Song of rest, Fast healer under another id.
    const SONG: &str = r#"
[[talent]]
id = "song_of_rest"
name = "Song of rest"
tree = "class"
granted = true
description = "A song while you rest."
effects = [{ kind = "resting_percent", value = 50 }]
systems = ["hit_points"]
"#;

    #[test]
    fn the_builtin_talents_have_no_clone_or_chain_unlisted() {
        Content::builtin().expect("the built-in content loads");
    }

    #[test]
    fn a_clone_of_a_pickable_is_refused() {
        let e = refusal(load(&[("song_of_rest", SONG)], str::to_string));
        assert!(
            e.contains("`fast_healer` and `song_of_rest` have the same effects"),
            "{e}"
        );
    }

    #[test]
    fn a_clone_of_a_talents_first_rank_is_refused() {
        // Brawn's first rank under another id (the cut Whetstone).
        let whetstone = r#"
[[talent]]
id = "whetstone"
name = "Whetstone"
tree = "class"
granted = true
description = "Sharper blows."
effects = [{ kind = "melee_damage", value = 1 }]
systems = ["combat"]
"#;
        let e = refusal(load(&[("whetstone", whetstone)], str::to_string));
        assert!(e.contains("`whetstone` and `brawn`"), "{e}");
    }

    #[test]
    fn only_talents_of_the_same_choice_kind_are_alike() {
        // Schooled and Stylist both read `weapon { ranks = 1 }`, for a school and a style, and
        // the built-in content loads; a second talent for a school is refused.
        let twin = r#"
[[talent]]
id = "schooled_twin"
name = "Schooled twin"
tree = "spellcraft"
choice = "school_skill"
description = "A copy."
needs = { chosen = 2 }
effects = [{ kind = "weapon", ranks = 1 }]
systems = ["schools"]
"#;
        let e = refusal(load(&[("schooled_twin", twin)], str::to_string));
        assert!(e.contains("`schooled` and `schooled_twin`"), "{e}");
    }

    #[test]
    fn a_chain_of_one_effect_kind_is_refused() {
        let more = r#"
[[talent]]
id = "faster_healer"
name = "Faster healer"
tree = "vigour"
description = "Rest mends you more quickly still."
needs = { talents = { fast_healer = 1 }, level = 9 }
effects = [{ kind = "resting_percent", value = 25 }]
systems = ["hit_points"]
"#;
        let e = refusal(load(&[("faster_healer", more)], str::to_string));
        assert!(
            e.contains("`faster_healer` needs only `fast_healer` and shares an effect kind"),
            "{e}"
        );
    }

    #[test]
    fn a_listed_pair_is_let_be_and_a_stale_or_unknown_one_refused() {
        let pair = listing("song_of_rest", "fast_healer");
        load(&[("song_of_rest", SONG)], pair).expect("listed: it loads");
        let e = refusal(load(&[], listing("healthy", "hardy")));
        assert!(
            e.contains("`alike` lists `healthy` and `hardy`, which are not alike"),
            "{e}"
        );
        let e = refusal(load(&[], listing("healthy", "nobody")));
        assert!(e.contains("unknown talent `nobody`"), "{e}");
    }
}
