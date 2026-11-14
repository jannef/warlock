//! The `companion` and `familiar` layers: the player's kept beasts as they know them.

use super::*;

/// Which kept beast a dump says: the companion or the familiar.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kept {
    Companion,
    Familiar,
}

impl Kept {
    fn of<'v>(
        self,
        view: &'v PerceptionView<'_>,
    ) -> Option<(
        &'v crate::perception::AllyView,
        &'v crate::world::CompanionData,
    )> {
        match self {
            Kept::Companion => view.companion(),
            Kept::Familiar => view.familiar(),
        }
    }

    const fn id(self) -> &'static str {
        match self {
            Kept::Companion => "companion",
            Kept::Familiar => "familiar",
        }
    }
}

/// The player's companion or familiar as they know it (spec §8 "Allies"): `Snel, a hound
/// (grade 12) at (12, 5): unhurt, order follow`, `Snel, a hound (grade 12): with you` in the
/// retinue (a cell only where the player sees it); `no companion` (`no familiar`) without one.
pub(super) fn kept_json(view: &PerceptionView<'_>, kept: Kept) -> Value {
    let content = view.content();
    let Some((ally, data)) = kept.of(view) else {
        return Value::Null;
    };
    let pos = (view.actors().iter())
        .find(|a| a.id == ally.id)
        .map(|a| a.pos);
    json!({
        "id": ally.id,
        "name": data.name,
        "creature": content.creature(ally.creature).id,
        "grade": data.grade,
        "health": ally.health,
        "order": ally.order.id(),
        "at": ally.order.at(),
        "pos": pos,
        "retinue": ally.retinue,
        "wears": wears(view, ally),
    })
}

/// What a companion wears as it reads (`an amber amulet`; `world/collar.rs`), or null.
pub(super) fn wears(
    view: &PerceptionView<'_>,
    ally: &crate::perception::AllyView,
) -> Option<String> {
    (!ally.wears.is_empty()).then(|| crate::object::phrase(&ally.wears, view.lore()))
}

pub(super) fn kept_rows(view: &PerceptionView<'_>, kept: Kept) -> Vec<String> {
    let content = view.content();
    let Some((ally, data)) = kept.of(view) else {
        let remembered = (view.character())
            .and_then(|c| c.familiar.as_ref())
            .filter(|_| kept == Kept::Familiar);
        let Some(lost) = remembered else {
            return vec![format!("no {}", kept.id())];
        };
        let kind = &content.creature(lost.creature).name;
        return vec![format!(
            "no familiar; {}, {} {kind}, answers a calling",
            lost.name,
            article(kind)
        )];
    };
    let kind = &content.creature(ally.creature).name;
    let head = format!(
        "{}, {} {kind} (grade {})",
        data.name,
        article(kind),
        data.grade
    );
    let collar = wears(view, ally).map_or(String::new(), |w| format!(", wearing {w}"));
    if ally.retinue {
        return vec![format!("{head}: with you{collar}")];
    }
    let seen = (view.actors().iter()).find(|a| a.id == ally.id);
    let at = seen.map_or(String::new(), |a| format!(" at ({}, {})", a.pos.x, a.pos.y));
    let order = match ally.order.at() {
        Some(at) => format!("{} at ({}, {})", ally.order.id(), at.x, at.y),
        None => ally.order.id().to_string(),
    };
    vec![format!(
        "{head}{at}: {}, order {order}{collar}",
        ally.health.word()
    )]
}
