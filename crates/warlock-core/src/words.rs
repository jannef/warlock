//! Words the player says to someone (spec §8 "Dialogue", "Saying words"; docs/design/
//! prestige-classes.md §6): what they type is matched against the words a speaker knows
//! (`[[dialogue.word]]`) once both are normalised, so case, punctuation, and spacing never
//! matter. Pure: the rules are in `world/words.rs`.

/// The longest phrase a dialogue's word may have, and so the most the say field takes, in
/// characters: a longer line could match nothing.
pub const MAX_SAID: usize = 40;

/// `text` as words are compared: lower case (Unicode), every character that is not a letter or
/// a digit (`char::is_alphanumeric`, so `ǣ` and `þ` stay) a space, runs of spaces one, and no
/// space at either end. "Life before death!", "LIFE, BEFORE... DEATH", and "life  before death"
/// are all `life before death`. Idempotent.
pub fn normalise(text: &str) -> String {
    let lower = text.to_lowercase();
    let spaced = lower
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' });
    spaced
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_punctuation_and_spacing_do_not_matter() {
        for said in [
            "Life before death!",
            "LIFE, BEFORE... DEATH",
            "life  before death",
            "  life\tbefore-death. ",
        ] {
            assert_eq!(normalise(said), "life before death", "{said}");
        }
        assert_eq!(normalise(""), "");
        assert_eq!(normalise("?! ..."), "");
        assert_eq!(
            normalise("_"),
            "",
            "the action text's joiner is never a word"
        );
    }

    #[test]
    fn letters_beyond_ascii_and_digits_stay() {
        assert_eq!(normalise("Ǣþelstān's Gate"), "ǣþelstān s gate");
        assert_eq!(normalise("Door 7"), "door 7");
    }

    #[test]
    fn normalising_twice_changes_nothing() {
        for said in ["Life before death!", "Ǣ  þ", "a_b", "", "x"] {
            let once = normalise(said);
            assert_eq!(normalise(&once), once, "{said}");
        }
    }
}
