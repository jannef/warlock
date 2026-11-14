//! A release build's content (`--no-default-features`, #331): the test world is not in it, and
//! New run lists exactly the campaigns' starts. Run by CI's release-configuration job
//! (`cargo test --no-default-features -p warlock-assets --test release_content`): the content the
//! binaries embed, in one small harness (#1593, #1587). A default build compiles it empty.
#![cfg(not(feature = "showcase"))]

use warlock_core::content::CampaignKind;

#[test]
fn release_content() {
    let content = warlock_assets::content().expect("a release build's content loads");
    for c in content.campaigns() {
        assert_eq!(c.kind, CampaignKind::Campaign, "test campaign `{}`", c.id);
    }
    assert!(content.campaign("showcase").is_none());
    assert!(content.scenario("showcase").is_none());
    // Whatever names a campaign names a playable one; scenarios and hand-made levels belong
    // to one (nothing is left outside `campaigns/`). So New run, which lists each campaign's
    // scenarios under it, lists nothing of the test world, with `--dev` too.
    let playable = |c: Option<&str>| {
        c.is_none_or(|id| (content.campaign(id)).is_some_and(|c| c.kind == CampaignKind::Campaign))
    };
    for s in content.scenarios() {
        let campaign = s.campaign.as_deref();
        assert!(
            campaign.is_some() && playable(campaign),
            "scenario {}",
            s.id
        );
    }
    for l in content.levels() {
        let campaign = content.level_campaign(&l.id);
        assert!(campaign.is_some() && playable(campaign), "level {}", l.id);
    }
    for d in content.dungeons() {
        assert!(playable(d.campaign.as_deref()), "dungeon {}", d.id);
    }
    for q in content.quests() {
        assert!(playable(q.campaign.as_deref()), "quest {}", q.id);
    }
    for d in content.dialogues() {
        assert!(playable(d.campaign.as_deref()), "dialogue {:?}", d.speaker);
    }
    for c in content.creature_defs() {
        assert!(playable(c.campaign.as_deref()), "creature {}", c.id);
    }
    for v in content.vaults() {
        assert!(playable(v.campaign.as_deref()), "vault {}", v.template.id);
    }
    // A run that names no scenario starts the first campaign's start.
    assert_eq!(
        content.default_scenario(),
        content.campaigns().first().map(|c| c.start.as_str())
    );
    // One campaign: New run lists one start, so its step is skipped, straight on to the race
    // (`NewRunScreen::skips`, tested on fixture content in `new_run_screen.rs`).
    let ids: Vec<&str> = content.campaigns().iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids.len(), 1, "{ids:?}");
}
