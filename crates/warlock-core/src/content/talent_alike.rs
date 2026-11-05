//! The identical-effects check on the talent table (#790; docs/dev/progression.md "Coverage
//! policy", the #776 review's rule 3), run at load after every talent has parsed: no talent does
//! exactly what another does (a class ability that copies a pickable, Song of rest beside Fast
//! healer). A class table names the pickable's id instead.
//!
//! `alike` in `data/talents.toml` lists the pairs kept on purpose (Cleave and Cleaving strike:
//! two cleave sources make a great cleave; the birth signs' gifts, which the owner keeps whole).
//! A listed pair that is no longer alike is an error too, so the list stays honest.

use super::{ContentError, StatusId, TalentBook, TalentDef, TalentEffect};

/// One effect with the status it counts only under (`while`), if any: the condition is part of
/// the effect's identity, so `protection 2 while raging` is not Bulwark's plain `protection 2`.
type Effect = (Option<StatusId>, TalentEffect);

/// Whether two effect lists hold the same effects, in any order.
fn same(a: &[Effect], b: &[Effect]) -> bool {
    a.len() == b.len() && a.iter().all(|e| b.contains(e)) && b.iter().all(|e| a.contains(e))
}

/// What `def` adds, plain and conditional.
fn signature(def: &TalentDef) -> Vec<Effect> {
    (def.effects.iter().cloned().map(|e| (None, e)))
        .chain((def.effects_while.iter().cloned()).map(|(s, e)| (Some(s), e)))
        .collect()
}

/// Whether `a` does what `b` does, and does something. Talents taken for different kinds of
/// choice never match (Stylist's `weapon` reads a style, Schooled's a school).
fn clone_of((a, sa): (&TalentDef, &[Effect]), (b, sb): (&TalentDef, &[Effect])) -> bool {
    a.choice == b.choice && !sa.is_empty() && same(sa, sb)
}

/// The check: the first clone not listed in `alike`, or the first listed pair that is none, in
/// words, against `file` (`data/talents.toml`).
pub(super) fn check(
    book: &TalentBook,
    alike: &[[String; 2]],
    file: &str,
) -> Result<(), ContentError> {
    let listed =
        |a: &str, b: &str| (alike.iter()).any(|[x, y]| (x == a && y == b) || (x == b && y == a));
    let defs: Vec<(&TalentDef, Vec<Effect>)> =
        book.iter().map(|(_, d)| (d, signature(d))).collect();
    let mut found: Vec<(&str, &str)> = Vec::new();
    for (i, (a, sa)) in defs.iter().enumerate() {
        for (b, sb) in &defs[i + 1..] {
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
    fn the_builtin_talents_have_no_clone_unlisted() {
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
    fn an_effect_under_a_status_is_not_alike_the_plain_one() {
        // Bulwark's `protection 2`, raging only: not a clone of Bulwark.
        let raging = r#"
[[talent]]
id = "raging_wall"
name = "Raging wall"
tree = "class"
granted = true
description = "Raging, you take a blow."
effects = [{ kind = "protection", value = 2, while = "raging" }]
systems = ["protection"]
"#;
        load(&[("raging_wall", raging)], str::to_string).expect("not alike: it loads");
        // The same under the same status is a clone of it.
        let twin = raging.replace("raging_wall", "raging_twin");
        let e = refusal(load(
            &[("raging_wall", raging), ("raging_twin", &twin)],
            str::to_string,
        ));
        assert!(
            e.contains("have the same effects") && e.contains("`raging_twin`"),
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
