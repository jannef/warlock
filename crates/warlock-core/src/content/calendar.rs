//! The calendar's flavor (`assets/data/calendar.toml`): month names. The calendar's structure
//! (12 months of 30 days, 24-hour days) is fixed in [`crate::time`].

use super::ContentError;
use crate::time::{GameTime, MONTHS_PER_YEAR};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CalendarFile {
    months: Vec<String>,
}

/// Month names, in calendar order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Calendar {
    months: Vec<String>,
}

impl Default for Calendar {
    /// Without a calendar file months are numbered: `Month 1` to `Month 12`.
    fn default() -> Self {
        Self {
            months: (1..=MONTHS_PER_YEAR)
                .map(|m| format!("Month {m}"))
                .collect(),
        }
    }
}

impl Calendar {
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: CalendarFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if parsed.months.len() as u64 != MONTHS_PER_YEAR {
            return Err(err(format!(
                "expected {MONTHS_PER_YEAR} month names, found {}",
                parsed.months.len()
            )));
        }
        if let Some(i) = parsed.months.iter().position(|m| m.trim().is_empty()) {
            return Err(err(format!("month {} has an empty name", i + 1)));
        }
        Ok(Self {
            months: parsed.months,
        })
    }

    /// The name of the month `time` falls in.
    pub fn month_name(&self, time: GameTime) -> &str {
        &self.months[time.month_index()]
    }

    /// Day and month, e.g. `12 Rainmoon`.
    pub fn date(&self, time: GameTime) -> String {
        format!("{} {}", time.day(), self.month_name(time))
    }

    /// Day, month, and year, e.g. `12 Rainmoon, Year 1`.
    pub fn full_date(&self, time: GameTime) -> String {
        format!("{}, Year {}", self.date(time), time.year())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_months_and_validates_the_file() {
        let t = GameTime::from_calendar(1, 4, 12, 14, 32).expect("valid");
        assert_eq!(Calendar::default().date(t), "12 Month 4");
        assert_eq!(Calendar::default().full_date(t), "12 Month 4, Year 1");
        let names: Vec<String> = (1..=12).map(|m| format!("\"M{m}\"")).collect();
        let text = format!("months = [{}]", names.join(", "));
        let calendar = Calendar::parse("c", &text).expect("valid");
        assert_eq!(calendar.month_name(t), "M4");
        assert!(Calendar::parse("c", "months = [\"A\"]").is_err());
        let blank = text.replace("\"M3\"", "\" \"");
        assert!(
            Calendar::parse("c", &blank)
                .unwrap_err()
                .message
                .contains("month 3")
        );
    }
}
