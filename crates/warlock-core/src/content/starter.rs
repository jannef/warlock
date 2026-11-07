//! The starter monsters (#1200): the first five levels are cut, level 1 hardest. A generated
//! family's member of level 1 to 5 (`animated.rs`) has [`hp_percent`] of the roster formulas'
//! hit points (Toughness and `base_hp` alike); the hand-written types carry the cut in their
//! files.

/// The share, in percent, of the roster formulas' hit points a monster of `level` has: 40 at
/// level 1, rising to 90 at level 5, the formulas' own from level 6.
pub fn hp_percent(level: u8) -> u32 {
    match level {
        0 | 6.. => 100,
        1 => 40,
        2 => 55,
        3 => 70,
        4 => 80,
        5 => 90,
    }
}

/// `value` cut to [`hp_percent`] of itself at `level`, rounded to the nearest.
pub fn cut(value: i64, level: u8) -> i64 {
    let percent = i64::from(hp_percent(level));
    (value * percent + 50).div_euclid(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_share_rises_to_the_formulas_own() {
        let shares: Vec<u32> = (0..=7).map(hp_percent).collect();
        assert_eq!(shares, [100, 40, 55, 70, 80, 90, 100, 100]);
        assert_eq!(cut(12, 1), 5);
        assert_eq!(cut(19, 5), 17);
        assert_eq!(cut(30, 6), 30);
    }
}
