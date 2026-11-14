//! The `story` layer: the player's background story, a paragraph a line.

use super::*;

/// The player's background story, a line an entry: `{chapter, line, text}`, the text with its
/// slots filled; empty without a story.
pub(super) fn story_json(world: &World) -> Vec<Value> {
    let view = world.perception();
    let Some(c) = view.character() else {
        return Vec::new();
    };
    (crate::background::lines(world.content(), world.campaign(), c).into_iter())
        .map(|(t, _, text)| json!({ "chapter": t.chapter, "line": t.line, "text": text }))
        .collect()
}

/// The player's background story, a paragraph a line with a blank line between, as the story
/// screen tells it: `no story` without one ([`crate::perception::PerceptionView::story`]).
pub(super) fn story_rows(world: &World) -> Vec<String> {
    let paragraphs = world.perception().story();
    if paragraphs.is_empty() {
        return vec!["no story".into()];
    }
    let mut rows = Vec::new();
    for (i, p) in paragraphs.into_iter().enumerate() {
        if i > 0 {
            rows.push(String::new());
        }
        rows.push(p);
    }
    rows
}
