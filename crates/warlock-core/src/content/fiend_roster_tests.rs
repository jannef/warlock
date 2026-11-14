//! The fiends of no family (#450): no beast friend calms them.

use crate::test_support::builtin;

#[test]
fn no_beast_friend_calms_a_fiend() {
    let content = builtin();
    let fiends = content.factions().id("fiends").expect("the fiends faction");
    assert!(!content.factions().def(fiends).animal);
}
