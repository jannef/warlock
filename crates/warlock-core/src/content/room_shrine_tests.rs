//! A shrine of a side's data (`shrine = { ... }`, spec §7 "Special rooms"): parsed with its
//! status resolved, and refused without `sides`, with a pace out of range, an empty line, or a
//! status that does not ease slowly enough.

use super::{Content, RoomEffectId, RoomEffects};

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
    let content = Content::builtin().expect("content");
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
    let content = Content::builtin().expect("content");
    for (sides, fields, why) in [
        (
            false,
            "regen = 200, status = \"unwelcome\"",
            "needs `sides`",
        ),
        (true, "regen = 0, status = \"unwelcome\"", "shrine.regen 0"),
        (
            true,
            "regen = 1001, status = \"unwelcome\"",
            "shrine.regen 1001",
        ),
        (
            true,
            "regen = 200, status = \"nothing\"",
            "unknown status `nothing`",
        ),
        (true, "regen = 200, status = \"asleep\"", "must ease"),
    ] {
        let e = parse(&content, &shrine(sides, fields)).unwrap_err();
        assert!(e.contains(why), "{fields}: {e}");
    }
    let blank = format!(
        "{SHRINE}sides = {{ good = \"altar_good\" }}\nshrine = {{ welcome = \" \", unwelcome = \
         \"Out.\", regen = 200, status = \"unwelcome\" }}\n"
    );
    assert!(
        parse(&content, &blank)
            .unwrap_err()
            .contains("must not be empty")
    );
}
