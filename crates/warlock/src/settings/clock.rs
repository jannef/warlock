//! The clock setting: how a time of day is written (presentation only; the core keeps one clock).

use warlock_core::time::GameTime;

/// 24-hour (`14:32`) or 12-hour (`2:32 PM`) time of day.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Clock {
    #[default]
    H24,
    H12,
}

impl Clock {
    pub const ALL: [Clock; 2] = [Clock::H24, Clock::H12];

    /// Name in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            Clock::H24 => "24h",
            Clock::H12 => "12h",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.id() == id)
    }

    /// Name on the settings screen.
    pub fn label(self) -> &'static str {
        match self {
            Clock::H24 => "24-hour",
            Clock::H12 => "12-hour (AM/PM)",
        }
    }

    /// The other form (there are two).
    pub fn toggle(self) -> Self {
        match self {
            Clock::H24 => Clock::H12,
            Clock::H12 => Clock::H24,
        }
    }

    /// The time of day of `time`: `14:32`, or `2:32 PM` (`12:05 AM` just after midnight, `12:00 PM`
    /// at noon).
    pub fn format(self, time: GameTime) -> String {
        self.format_hm(time.hour(), time.minute())
    }

    fn format_hm(self, hour: u32, minute: u32) -> String {
        match self {
            Clock::H24 => format!("{hour:02}:{minute:02}"),
            Clock::H12 => {
                let h = match hour % 12 {
                    0 => 12,
                    h => h,
                };
                let suffix = if hour < 12 { "AM" } else { "PM" };
                format!("{h}:{minute:02} {suffix}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_forms_at_the_edges_of_the_day() {
        let cases = [
            (0, 0, "00:00", "12:00 AM"),
            (0, 5, "00:05", "12:05 AM"),
            (9, 7, "09:07", "9:07 AM"),
            (12, 0, "12:00", "12:00 PM"),
            (12, 59, "12:59", "12:59 PM"),
            (13, 0, "13:00", "1:00 PM"),
            (23, 59, "23:59", "11:59 PM"),
        ];
        for (h, m, h24, h12) in cases {
            assert_eq!(Clock::H24.format_hm(h, m), h24);
            assert_eq!(Clock::H12.format_hm(h, m), h12);
        }
    }

    #[test]
    fn ids_round_trip() {
        for c in Clock::ALL {
            assert_eq!(Clock::from_id(c.id()), Some(c));
        }
        assert_eq!(Clock::from_id("am/pm"), None);
    }
}
