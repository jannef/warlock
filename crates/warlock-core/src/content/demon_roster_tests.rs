//! The demon roster (#449): the demons' stance toward the others.

use super::Content;

#[test]
fn the_demons_hunt_the_player_and_the_devils_and_leave_the_rest_be() {
    let content = Content::builtin().expect("content");
    let id = |n: &str| content.creature_id(n).expect(n);
    use super::Stance::{Hostile, Neutral};
    for (other, want) in [
        ("player", Hostile),
        ("kobold", Neutral),
        ("goblin", Neutral),
        ("duergar_warrior", Neutral),
        ("marsh_troll", Neutral),
        ("imp", Hostile),
        ("skeleton", Neutral),
        ("ember_beetle", Neutral),
        ("rat", Neutral),
        ("balor", Neutral),
    ] {
        assert_eq!(content.stance(id("dretch"), id(other)), want, "{other}");
    }
}
