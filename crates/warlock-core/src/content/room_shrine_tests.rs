//! A shrine of a side's data (`shrine = { ... }`, spec §7 "Special rooms"): parsed with its
//! status resolved, and refused without `sides` or with an unknown status.

use super::{Content, RoomEffectId, RoomEffects};
use crate::test_support::builtin;

const SHRINE: &str = "[[effect]]\nid = \"shrine\"\nname = \"shrine\"\nmark = \"i\"\n\
                      enter = \"Ignored.\"\nfeature = \"shrine_stone\"\n";

fn parse(content: &Content, text: &str) -> Result<RoomEffects, String> {
    RoomEffects::parse_with(
        "e.toml",
        text,
        |name| {
            let id = content.terrain_id(name)?;
            Some((id, content.terrain(id)))
        },
        |name| {
            let id = content.status_id(name)?;
            Some((id, content.status(id)))
        },
    )
    .map_err(|e| e.message)
}

fn shrine(sides: bool, fields: &str) -> String {
    let sides = if sides {
        "sides = { good = \"altar_good\", evil = \"altar_evil\" }\n"
    } else {
        ""
    };
    format!("{SHRINE}{sides}shrine = {{ welcome = \"In.\", unwelcome = \"Out.\", {fields} }}\n")
}

#[test]
fn a_shrine_parses_with_its_status() {
    let content = builtin();
    let effects = parse(
        &content,
        &shrine(true, "regen = 200, status = \"unwelcome\""),
    )
    .expect("valid");
    let def = effects.get(RoomEffectId(0));
    let s = def.shrine.as_ref().expect("a shrine");
    assert_eq!(s.regen, 200);
    assert_eq!(Some(s.status), content.status_id("unwelcome"));
    assert_eq!((s.welcome.as_str(), s.unwelcome.as_str()), ("In.", "Out."));
    let plain = parse(&content, SHRINE).expect("valid");
    assert!(
        plain.get(RoomEffectId(0)).shrine.is_none(),
        "none by default"
    );
}

#[test]
fn a_bad_shrine_is_refused() {
    let content = builtin();
    for (sides, fields, why) in [
        (
            false,
            "regen = 200, status = \"unwelcome\"",
            "needs `sides`",
        ),
        (
            true,
            "regen = 200, status = \"nothing\"",
            "unknown status `nothing`",
        ),
    ] {
        let e = parse(&content, &shrine(sides, fields)).unwrap_err();
        assert!(e.contains(why), "{fields}: {e}");
    }
}
