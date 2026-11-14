//! Small text helpers for the words the player reads: a first letter in capitals, `a` or `an`,
//! a list joined with the Oxford comma ("a, b, and c"; the owner, #1548), and a small count in a
//! word. The one copy of each: core, the app, and the wiki generator all call these.

/// `text` with its first letter in upper case ("Long sword").
pub fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// `a` or `an` for a noun, by its first letter (lower case: [`capitalized`] it to open a
/// sentence).
pub fn article(noun: &str) -> &'static str {
    match noun.chars().next() {
        Some(c) if "aeiouAEIOU".contains(c) => "an",
        _ => "a",
    }
}

/// Words in a list: "gold", "gold and ruby", "gold, ruby, and emerald".
pub fn and_list<S: AsRef<str>>(words: &[S]) -> String {
    joined(words, "and")
}

/// Words as alternatives: "gold", "gold or ruby", "gold, ruby, or emerald".
pub fn or_list<S: AsRef<str>>(words: &[S]) -> String {
    joined(words, "or")
}

/// Words in a list with `conjunction` before the last, and the Oxford comma before it from three
/// words on.
pub fn joined<S: AsRef<str>>(words: &[S], conjunction: &str) -> String {
    match words {
        [] => String::new(),
        [one] => one.as_ref().to_string(),
        [a, b] => format!("{} {conjunction} {}", a.as_ref(), b.as_ref()),
        [init @ .., last] => {
            let init: Vec<&str> = init.iter().map(AsRef::as_ref).collect();
            format!("{}, {conjunction} {}", init.join(", "), last.as_ref())
        }
    }
}

/// A small count in a word: "no", "one" to "twelve", digits beyond.
pub fn number_word(n: usize) -> String {
    const WORDS: [&str; 13] = [
        "no", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
        "eleven", "twelve",
    ];
    WORDS
        .get(n)
        .map_or_else(|| n.to_string(), |w| (*w).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitals() {
        assert_eq!(capitalized("long sword"), "Long sword");
        assert_eq!(capitalized("élan"), "Élan");
        assert_eq!(capitalized(""), "");
    }

    #[test]
    fn articles() {
        assert_eq!(article("arrow"), "an");
        assert_eq!(article("Elf"), "an");
        assert_eq!(article("sword"), "a");
        assert_eq!(article(""), "a");
    }

    #[test]
    fn lists_take_the_oxford_comma() {
        assert_eq!(and_list::<&str>(&[]), "");
        assert_eq!(and_list(&["gold"]), "gold");
        assert_eq!(and_list(&["gold", "ruby"]), "gold and ruby");
        assert_eq!(and_list(&["gold", "ruby", "jet"]), "gold, ruby, and jet");
        let owned = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(or_list(&owned), "a, b, or c");
    }

    #[test]
    fn number_words() {
        assert_eq!(number_word(0), "no");
        assert_eq!(number_word(3), "three");
        assert_eq!(number_word(12), "twelve");
        assert_eq!(number_word(13), "13");
    }
}
