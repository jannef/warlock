//! The hags (#451): every hag carries the `hag` tag, the bog hag (of the `marsh`) too.

use crate::test_support::builtin;

#[test]
fn every_hag_carries_the_hag_tag() {
    let content = builtin();
    let tagged: Vec<String> = (content.tagged("hag"))
        .map(|c| content.creature(c).id.clone())
        .collect();
    for id in ["bog_hag", "green_hag", "sea_hag", "annis_hag", "bheur_hag"] {
        assert!(tagged.contains(&id.to_string()), "{id}: tag it `hag`");
    }
}
