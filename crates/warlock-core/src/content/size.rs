//! Sizes (`assets/data/sizes.toml`, spec §8 "Burden"): how big a race or a creature type is,
//! and so how much it can carry. A size scales the carrying limit by its `carry` percent: small
//! races (halflings, gnomes) carry three quarters of what a medium one does. Its `blood` percent
//! scales how much blood it spills (spec §8 "Blood"; presentation only).
//!
//! Every race and creature type has a size, [`MEDIUM`] unless it names another. Without the file
//! there is only [`SizeDef::medium`].

use super::ContentError;
use serde::Deserialize;

/// The size of every race and creature type that names none; the file must have it.
pub const MEDIUM: &str = "medium";

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
    /// How much blood it spills, in percent of what a medium one does (spec §8 "Blood"); 100 by
    /// default.
    #[serde(default = "full_blood")]
    pub blood: u16,
    /// The chance its material drops are rolled at, in percent of what is written (spec §7
    /// "Material drops"; `content/drops.rs`); 100 by default.
    #[serde(default = "full_drops")]
    pub drops: u16,
}

fn full_blood() -> u16 {
    100
}

fn full_drops() -> u16 {
    100
}

impl SizeDef {
    /// The one size there is without `data/sizes.toml`: medium, carrying as much as anyone.
    pub fn medium() -> Self {
        Self {
            id: MEDIUM.into(),
            name: "Medium".into(),
            carry: 100,
            blood: 100,
            drops: 100,
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
        let text = "[[size]]\nid = \"small\"\nname = \"Small\"\ncarry = 75\nblood = 60\n\
                    [[size]]\nid = \"medium\"\nname = \"Medium\"\ncarry = 100\n";
        let sizes = parse("sizes.toml", text).expect("valid");
        assert_eq!(sizes.len(), 2);
        let small = &sizes[0];
        assert_eq!(
            (small.id.as_str(), small.carry, small.blood),
            ("small", 75, 60)
        );
        assert_eq!(sizes[1], SizeDef::medium());
    }

    #[test]
    fn bad_sizes_are_refused() {
        let medium = "[[size]]\nid = \"medium\"\nname = \"Medium\"\ncarry = 100\n";
        for (text, why) in [
            (medium.replace("medium", "small"), "`medium` is required"),
            (format!("{medium}{medium}"), "duplicate size id `medium`"),
            (medium.replace("carry = 100\n", ""), "missing field `carry`"),
            (format!("{medium}weight = 3\n"), "unknown field `weight`"),
        ] {
            let e = parse("sizes.toml", &text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
