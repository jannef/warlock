//! The fiends of no family (#450): no beast friend calms them.

use super::Content;

#[test]
fn no_beast_friend_calms_a_fiend() {
    let content = Content::builtin().expect("content");
    let fiends = content.factions().id("fiends").expect("the fiends faction");
    assert!(!content.factions().def(fiends).animal);
}
