//! Wall-clock time, which only the app reads (core never does): the time since 1970 and a moment
//! of it split into its UTC date and time of day, for log names, crash reports, and when a
//! character was last played.

use std::time::Duration;

/// The time since 1970 (UTC), now; on the web to the millisecond.
pub fn since_epoch() -> Duration {
    #[cfg(target_arch = "wasm32")]
    {
        Duration::from_millis(web_sys::js_sys::Date::now() as u64)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
    }
}

/// A moment in UTC, split up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UtcParts {
    pub year: i64,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

/// `secs` since 1970 as a UTC date and time of day (civil-from-days, Howard Hinnant).
pub fn utc_parts(secs: u64) -> UtcParts {
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    UtcParts {
        year: yoe + era * 400 + i64::from(month <= 2),
        month: month as u32,
        day: day as u32,
        hour: (rem / 3600) as u32,
        minute: (rem % 3600 / 60) as u32,
        second: (rem % 60) as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_a_moment_in_utc() {
        let at = |year, month, day, hour, minute, second| UtcParts {
            year,
            month,
            day,
            hour,
            minute,
            second,
        };
        assert_eq!(utc_parts(0), at(1970, 1, 1, 0, 0, 0));
        assert_eq!(utc_parts(1_781_654_400 + 3661), at(2026, 6, 17, 1, 1, 1));
        // A leap day.
        assert_eq!(utc_parts(951_782_400), at(2000, 2, 29, 0, 0, 0));
    }
}
