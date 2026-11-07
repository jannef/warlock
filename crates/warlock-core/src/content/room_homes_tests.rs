//! A special room's `dressing` and a den's own `population` and `danger` (#487, animated
//! objects A5: the armoury and the workshop): parsed and resolved against the
//! creatures.

use super::{Content, RoomEffectId, RoomEffects};
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> = LazyLock::new(|| Content::builtin().expect("content"));

const DEN: &str = "swarm = { per_cells = 4, min = 1, max = 4, wake = 200, cry = 0, loot_level = 0";

fn effect(extra: &str) -> String {
    format!(
        "[[effect]]\nid = \"home\"\nname = \"home\"\nmark = \"H\"\nenter = \"In.\"\n\
         needs_danger = true\n{extra}\n"
    )
}

fn parse(extra: &str) -> Result<RoomEffects, String> {
    let content = &*CONTENT;
    let terrain = |name: &str| content.terrain_id(name).map(|id| (id, content.terrain(id)));
    RoomEffects::parse("e.toml", &effect(extra), terrain).map_err(|e| e.message)
}

#[test]
fn dressing_parses_and_divides_by_its_cells() {
    let effects =
        parse("dressing = { terrain = \"weapon_rack\", per_cells = 8, most = 6 }").expect("valid");
    let dressing = effects.get(RoomEffectId(0)).dressing.expect("dressing");
    assert_eq!(
        dressing.terrain,
        CONTENT.terrain_id("weapon_rack").expect("rack")
    );
    // One per eight cells, at least one, at most six.
    assert_eq!(
        (
            dressing.pieces(3),
            dressing.pieces(20),
            dressing.pieces(200)
        ),
        (1, 2, 6)
    );
    for bad in ["per_cells = 0, most = 6", "per_cells = 8, most = 0"] {
        let text = format!("dressing = {{ terrain = \"workbench\", {bad} }}");
        assert!(parse(&text).is_err(), "{bad}");
    }
}

#[test]
fn a_dens_own_population_and_factions_do_not_go_together() {
    assert!(parse(&format!("{DEN}, population = [\"kobold\"], danger = 10 }}")).is_ok());
    let both = parse(&format!(
        "{DEN}, population = [\"kobold\"], factions = [\"vermin\"] }}"
    ));
    assert!(both.unwrap_err().contains("do not go together"));
}

#[test]
fn a_dens_own_population_resolves_against_the_creatures() {
    let content = &*CONTENT;
    let creature =
        |name: &str| (content.creature_id(name)).map(|c| (c, content.creature(c).unique));
    let resolve = |names: &str| {
        let mut effects = parse(&format!("{DEN}, population = [{names}] }}")).expect("parses");
        effects
            .resolve_populations(creature)
            .map(|()| effects)
            .map_err(|e| e.message)
    };
    let effects = resolve("\"kobold\", \"animated_blade\"").expect("valid");
    let swarm = effects.get(RoomEffectId(0)).swarm.clone().expect("a den");
    let blade = content.creature_id("animated_blade").expect("blades");
    assert_eq!(
        swarm.kinds,
        vec![content.creature_id("kobold").expect("kobold"), blade]
    );
    let unknown = resolve("\"no_such_thing\"").unwrap_err();
    assert!(unknown.contains("`no_such_thing` is unknown"), "{unknown}");
}
