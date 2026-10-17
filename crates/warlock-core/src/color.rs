//! Display colors carried by content definitions (sRGB, 8 bits per channel).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// An sRGB color, written in content files as `"#rrggbb"`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb::new(0, 0, 0);
    pub const WHITE: Rgb = Rgb::new(255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl FromStr for Rgb {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s
            .strip_prefix('#')
            .filter(|h| h.len() == 6 && h.is_ascii())
            .ok_or_else(|| format!("expected a color like \"#rrggbb\", got `{s}`"))?;
        let channel = |i: usize| {
            u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| format!("invalid hex color `{s}`"))
        };
        Ok(Rgb::new(channel(0)?, channel(2)?, channel(4)?))
    }
}

impl TryFrom<String> for Rgb {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<Rgb> for String {
    fn from(c: Rgb) -> String {
        c.to_string()
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_hex() {
        let c: Rgb = "#1a2B3c".parse().unwrap();
        assert_eq!(c, Rgb::new(0x1a, 0x2b, 0x3c));
        assert_eq!(c.to_string(), "#1a2b3c");
        assert!("1a2b3c".parse::<Rgb>().is_err());
        assert!("#12345".parse::<Rgb>().is_err());
        assert!("#gg0000".parse::<Rgb>().is_err());
    }
}
