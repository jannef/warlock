//! The elemental roster (#453): the cinder elemental on the fire line.

use super::Content;

#[test]
fn the_cinder_elemental_is_the_fire_lines_rung() {
    let content = Content::builtin().expect("content");
    let cinder = content.creature_id("cinder_elemental").expect("cinder");
    assert!(content.has_tag(cinder, "elemental") && content.has_tag(cinder, "fire"));
    assert!(content.has_tag(cinder, "fire-thing"), "still the delf's");
}
