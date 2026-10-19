//! Sizes (`assets/data/sizes.toml`, spec §8 "Burden"): how big a race or a creature type is,
//! and so how much it can carry. A size scales the carrying limit by its `carry` percent: small
//! races (halflings, gnomes) carry three quarters of what a medium one does.
//!
//! Every race and creature type has a size, [`MEDIUM`] unless it names another. Without the file
//! there is only [`SizeDef::medium`].

use super::ContentError;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// The size of every race and creature type that names none; the file must have it.
pub const MEDIUM: &str = "medium";
/// What a size may carry, in percent of a medium one: a tenth to ten times as much.
pub const CARRY: RangeInclusive<u16> = 10..=1000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SizeFile {
    size: Vec<SizeDef>,
}

/// A size, in file order.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SizeDef {
    pub id: String,
    /// Its name, as the wiki shows it ("Small").
    pub name: String,
    /// How much it carries, in percent of what a medium one does (spec §8 "Burden").
    pub carry: u16,
}

impl SizeDef {
    /// The one size there is without `data/sizes.toml`: medium, carrying as much as anyone.
    pub fn medium() -> Self {
        Self {
            id: MEDIUM.into(),
            name: "Medium".into(),
            carry: 100,
        }
    }
}

/// Parses `data/sizes.toml`.
pub(super) fn parse(file: &str, text: &str) -> Result<Vec<SizeDef>, ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: SizeFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let mut sizes: Vec<SizeDef> = Vec::new();
    for size in parsed.size {
        if sizes.iter().any(|s| s.id == size.id) {
            return Err(err(format!("duplicate size id `{}`", size.id)));
        }
        if !CARRY.contains(&size.carry) {
            return Err(err(format!(
                "size `{}`: carry {} is out of range ({} to {})",
                size.id,
                size.carry,
                CARRY.start(),
                CARRY.end()
            )));
        }
        sizes.push(size);
    }
    if !sizes.iter().any(|s| s.id == MEDIUM) {
        return Err(err(format!(
            "a size with id `{MEDIUM}` is required: it is the size of whatever names none"
        )));
    }
    Ok(sizes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_parse_in_file_order() {
        let text = "[[size]]\nid = \"small\"\nname = \"Small\"\ncarry = 75\n\
                    [[size]]\nid = \"medium\"\nname = \"Medium\"\ncarry = 100\n";
        let sizes = parse("sizes.toml", text).expect("valid");
        assert_eq!(sizes.len(), 2);
        assert_eq!((sizes[0].id.as_str(), sizes[0].carry), ("small", 75));
        assert_eq!(sizes[1], SizeDef::medium());
    }

    #[test]
    fn bad_sizes_are_refused() {
        let medium = "[[size]]\nid = \"medium\"\nname = \"Medium\"\ncarry = 100\n";
        for (text, why) in [
            (medium.replace("medium", "small"), "`medium` is required"),
            (format!("{medium}{medium}"), "duplicate size id `medium`"),
            (medium.replace("100", "9"), "carry 9 is out of range"),
            (medium.replace("100", "1001"), "carry 1001 is out of range"),
            (medium.replace("carry = 100\n", ""), "missing field `carry`"),
            (format!("{medium}weight = 3\n"), "unknown field `weight`"),
        ] {
            let e = parse("sizes.toml", &text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
