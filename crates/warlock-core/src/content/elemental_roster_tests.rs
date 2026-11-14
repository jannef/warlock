//! The elemental roster (#453): the cinder elemental on the fire line.

use crate::test_support::builtin;

#[test]
fn the_cinder_elemental_is_the_fire_lines_rung() {
    let content = builtin();
    let cinder = content.creature_id("cinder_elemental").expect("cinder");
    assert!(content.has_tag(cinder, "elemental") && content.has_tag(cinder, "fire"));
    assert!(content.has_tag(cinder, "fire-thing"), "still the delf's");
}
