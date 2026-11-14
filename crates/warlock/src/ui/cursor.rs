//! A list's selection stepping up and down, wrapping round at the ends: the one rule every
//! screen's list follows (#1568).

/// The row above `selected` in a list of `len` rows, the last row from the first; 0 in an empty
/// list.
pub fn up(selected: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (selected % len + len - 1) % len
    }
}

/// The row below `selected` in a list of `len` rows, the first row from the last; 0 in an empty
/// list.
pub fn down(selected: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (selected + 1) % len }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_wrap_round_and_an_empty_list_stays_at_zero() {
        assert_eq!(up(0, 3), 2);
        assert_eq!(up(2, 3), 1);
        assert_eq!(down(2, 3), 0);
        assert_eq!(down(0, 3), 1);
        assert_eq!((up(0, 0), down(0, 0)), (0, 0));
        assert_eq!((up(0, 1), down(0, 1)), (0, 0));
    }
}
