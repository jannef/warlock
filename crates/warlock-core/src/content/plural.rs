//! The plural dictionary (`assets/data/plurals.toml`, and a campaign's `data/plurals.toml`;
//! spec §7 "Data"): the plurals of words that do not take a plain "s", kept by hand, one line
//! each (`wolf = "wolves"`). [`Plurals::plural`] is the one place a plural is made when content
//! gives none: a creature's or an object's name, a gem's material, a tag ("Kill 100 goblins.").
//!
//! A word is looked up whole, then by its last word (after a space or a hyphen: "giant wolf" is
//! "giant wolves", "marsh-thing" "marsh-things"), else it takes an "s".

use super::ContentError;
use std::collections::BTreeMap;

/// The longest word, or plural, in the dictionary.
pub const MAX_WORD: usize = 32;

/// Words and their plurals.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plurals {
    words: BTreeMap<String, String>,
}

impl Plurals {
    /// The plural of `word`: the dictionary's for the whole of it, else its last word's with the
    /// rest kept, else `word` and an "s".
    pub fn plural(&self, word: &str) -> String {
        if let Some(plural) = self.words.get(word) {
            return plural.clone();
        }
        if let Some(at) = word.rfind([' ', '-'])
            && let Some(plural) = self.words.get(&word[at + 1..])
        {
            return format!("{}{plural}", &word[..=at]);
        }
        format!("{word}s")
    }

    /// Reads one dictionary file (`singular = "plural"` lines) into these words. A word another
    /// file gave another plural is an error.
    pub(super) fn add(&mut self, path: &str, text: &str) -> Result<(), ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let words: BTreeMap<String, String> =
            toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for (word, plural) in words {
            let fits = |w: &str| !w.is_empty() && w.chars().count() <= MAX_WORD;
            if !fits(&word) || !fits(&plural) || word.chars().any(|c| c.is_uppercase()) {
                return Err(err(format!(
                    "`{word}`: a word and its plural are 1 to {MAX_WORD} characters, the word \
                     lower-case"
                )));
            }
            match self.words.get(&word) {
                Some(had) if *had != plural => {
                    return Err(err(format!(
                        "`{word}`: its plural is `{had}` already, not `{plural}`"
                    )));
                }
                _ => {
                    self.words.insert(word, plural);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dictionary() -> Plurals {
        let mut plurals = Plurals::default();
        plurals
            .add(
                "data/plurals.toml",
                "undead = \"undead\"\nwolf = \"wolves\"\n\"marsh-thing\" = \"fen things\"\n",
            )
            .expect("valid");
        plurals
    }

    #[test]
    fn whole_words_then_last_words_then_an_s() {
        let p = dictionary();
        assert_eq!(p.plural("undead"), "undead");
        assert_eq!(p.plural("wolf"), "wolves");
        assert_eq!(p.plural("skeletal wolf"), "skeletal wolves");
        assert_eq!(p.plural("were-wolf"), "were-wolves");
        assert_eq!(
            p.plural("marsh-thing"),
            "fen things",
            "the whole word first"
        );
        assert_eq!(p.plural("goblin"), "goblins");
        assert_eq!(p.plural("giant rat"), "giant rats");
        assert_eq!(p.plural("fire-thing"), "fire-things");
        assert_eq!(
            Plurals::default().plural("wolf"),
            "wolfs",
            "no dictionary: an s"
        );
    }

    #[test]
    fn a_word_given_two_plurals_is_refused() {
        let mut p = dictionary();
        assert!(
            p.add("b.toml", "wolf = \"wolves\"\n").is_ok(),
            "the same again"
        );
        let error = p.add("b.toml", "wolf = \"wolfs\"\n").expect_err("refused");
        assert!(error.to_string().contains("`wolves` already"), "{error}");
        for bad in ["Wolf = \"wolves\"", "wolf = \"\"", "wolf = 3"] {
            assert!(p.add("c.toml", bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_builtin_dictionary() {
        let content = super::super::Content::builtin().expect("content");
        assert_eq!(content.plural("undead"), "undead");
        assert_eq!(content.plural("vermin"), "vermin");
        assert_eq!(content.plural("dwarf"), "dwarves");
        assert_eq!(content.plural("wolf"), "wolves");
        assert_eq!(content.plural("goblin"), "goblins");
    }
}
