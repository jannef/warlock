//! Every level of every branch generates for every seed (#582): a player must never meet an
//! error generating a floor. Before the fix the caves floors that name a vault (Fyrdelf's
//! bottom, the drake's hoard; Wulfeshol's and Wyrtholm's fourth) failed for a few seeds in a
//! hundred: no rock was left big enough to hold the vault on any of the 16 attempts.

use super::*;
use crate::content::ContentFiles;
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> =
    LazyLock::new(|| Content::load(&ContentFiles::builtin()).expect("builtin content"));

/// The levels of every branch (`bottom`: only each branch's last), each with the campaign whose
/// run reaches it.
fn levels(bottom: bool) -> Vec<(String, String)> {
    (CONTENT.dungeons())
        .flat_map(|def| {
            let campaign = def.campaign.clone().unwrap_or_else(|| "showcase".into());
            let first = if bottom { def.depth } else { 1 };
            (first..=def.depth).map(move |depth| (campaign.clone(), def.level_id(depth)))
        })
        .collect()
}

/// The levels among `levels` that fail to generate for one of `seeds`, with the seed and why.
fn failures(levels: &[(String, String)], seeds: std::ops::Range<u64>) -> Vec<String> {
    let mut failed = Vec::new();
    for (campaign, id) in levels {
        for seed in seeds.clone() {
            if let Err(e) = generate(&CONTENT, (campaign, seed), id, None) {
                failed.push(format!("{id} seed {seed}: {e}"));
            }
        }
    }
    failed
}

#[test]
fn fyrdelf_bottom_generates_for_seed_zero() {
    let level =
        generate(&CONTENT, ("ascmearc", 0), "fyrdelf:12", Some("ascmearc")).expect("generates");
    let drake = CONTENT.creature_id("fire_drake").expect("the drake");
    assert!(level.creatures.iter().any(|c| c.creature == drake));
}

#[test]
fn every_bottom_floor_generates_for_hundreds_of_seeds() {
    let failed = failures(&levels(true), 0..300);
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

#[test]
fn every_floor_generates_for_many_seeds() {
    let failed = failures(&levels(false), 0..100);
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
