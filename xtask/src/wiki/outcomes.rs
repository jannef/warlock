//! Outcome tables (spec §7 "Special rooms"): what a forgotten altar's prayer may bring, exactly,
//! and how often, per prayer and per altar, computed from `data/outcomes/forgotten_altar.toml`;
//! and what a healing spring's drink may bring, per drink (`data/outcomes/spring.toml`).

use super::text::capital;
use std::fmt::Write as _;
use warlock_core::alignment::Alignment;
use warlock_core::content::{
    Amount, Content, Near, OutcomeDef, OutcomeEffect, OutcomeKind, OutcomeTable, OutcomeTier,
};
use warlock_core::magic::loot::LootGrade;

/// The forgotten altar's table, if the content has it.
fn altar(content: &Content) -> Option<&OutcomeTable> {
    content.outcomes().get("forgotten_altar")
}

/// A share in percent, to two decimals.
fn percent(p: f64) -> String {
    format!("{:.2}%", p * 100.0)
}

/// What `effect` does, exactly, in words.
fn does(content: &Content, effect: &OutcomeEffect) -> String {
    let amount = |a: Amount| match a {
        Amount::Dice(d) => d.to_string(),
        Amount::Half => "half the maximum of".to_string(),
        Amount::Full => "all".to_string(),
    };
    let damage = |kind: Option<warlock_core::content::DamageTypeId>| {
        kind.map_or(String::new(), |k| format!(" {}", content.damage_type(k).id))
    };
    let mut out: Vec<String> = Vec::new();
    if let Some(heal) = effect.heal {
        out.push(format!("{} hit points back", amount(heal)));
    }
    if !effect.cure.is_empty() {
        let names: Vec<String> = (effect.cure.iter())
            .map(|&s| content.status(s).name.to_lowercase())
            .collect();
        out.push(format!("{} cured", names.join(" and ")));
    }
    if let Some(means) = &effect.cure_by {
        out.push(format!(
            "every disease (and status) cured by `{means}` cured"
        ));
    }
    if effect.mend {
        out.push("all stat damage mended, and drained points restored".into());
    }
    if let Some(mana) = effect.mana {
        out.push(format!("{} mana back", amount(mana)));
    }
    if let Some(drain) = effect.drain {
        out.push(format!("{drain} mana lost (never below 0)"));
    }
    if let Some((status, degrees)) = effect.status {
        let def = content.status(status);
        let lasts = def
            .eases
            .map_or(String::new(), |e| format!(", easing a degree every {e} s"));
        out.push(format!(
            "{} ({degrees} degree{}{lasts})",
            def.name,
            if degrees == 1 { "" } else { "s" }
        ));
    }
    match effect.hunger {
        0 => {}
        n if n > 0 => out.push(format!("{n} hunger state better (never past Satiated)")),
        n => out.push(format!("{} hunger state worse (never into Starving)", -n)),
    }
    if !effect.objects.is_empty() {
        let total: u32 = effect.objects.iter().map(|o| o.2).sum();
        let each: Vec<String> = (effect.objects.iter())
            .map(|&(o, count, w)| {
                format!(
                    "{count} {} ({}%)",
                    content.object(o).plural,
                    w * 100 / total.max(1)
                )
            })
            .collect();
        out.push(format!("at the altar's foot: {}", each.join(" or ")));
    }
    if let Some(bonus) = effect.loot {
        let level = if bonus == 0 {
            "the floor's item level".to_string()
        } else {
            format!("the floor's item level + {bonus}")
        };
        let floor = match effect.rarity {
            LootGrade::Any => "",
            LootGrade::Magic => ", magic at least",
            LootGrade::Rare => ", rare (the top rarity its base allows)",
        };
        out.push(format!(
            "an item of loot at {level}{floor} at the altar's foot"
        ));
    }
    if let Some(reach) = effect.know_traps {
        out.push(format!("every trap within {reach} cells known"));
    }
    if effect.identify {
        out.push("one unidentified item carried identified".into());
    }
    if effect.forget {
        out.push("one identified magic item carried unidentified again".into());
    }
    if let Some(burn) = effect.burn {
        out.push(format!(
            "{}{} damage, never the last hit point (resistance counts)",
            burn.damage,
            damage(burn.kind)
        ));
    }
    if let Some(smite) = effect.smite {
        out.push(format!(
            "{}{} damage from above, which can kill (resistance counts)",
            smite.damage,
            damage(smite.kind)
        ));
    }
    if effect.lose_coins > 0 {
        out.push(format!(
            "{}% of every stack of coins carried lost",
            effect.lose_coins
        ));
    }
    if effect.snuff {
        out.push("your lantern goes out".into());
    }
    if let Some(summon) = effect.summon {
        let danger = if summon.danger == 0 {
            "the floor's danger".to_string()
        } else {
            format!("the floor's danger + {}", summon.danger)
        };
        let where_ = match summon.near {
            Near::Edge => "at the room's edge, out of view if it can be",
            Near::Feature => "beside the altar",
        };
        out.push(format!(
            "{} hostile creature(s) of {danger}, awake, {where_} (none on a level without a danger)",
            summon.count
        ));
    }
    if effect.disease {
        out.push("a random disease, the deadly ones too".into());
    }
    if let Some(disease) = effect.catch {
        out.push(format!("{} caught", content.disease(disease).name));
    }
    if let Some(wither) = effect.wither {
        out.push(format!("{wither} stat damage to a random stat"));
    }
    if effect.xp > 0 {
        out.push(format!(
            "{}% of the experience to your next level",
            effect.xp
        ));
    }
    if effect.tithe {
        out.push("the most valuable stack carried (worn and wielded too) gone".into());
    }
    if effect.teleport {
        out.push("moved to a random free spot of the level".into());
    }
    if let Some(x) = &effect.explode {
        out.push(format!(
            "a {} of radius {} on the altar ({}{}, half at the edge) hurting everyone in it, you \
             too, which can kill",
            x.burst.spell,
            x.burst.radius,
            x.burst.damage,
            damage(x.burst.kind)
        ));
    }
    if let Some(t) = effect.becomes {
        out.push(format!("the altar becomes {}", content.terrain(t).name));
    }
    if let Some(gift) = effect.gift {
        out.push(format!(
            "+{gift} to a random stat's score and potential, for good"
        ));
    }
    if let Some(wrath) = effect.wrath {
        out.push(format!(
            "−{wrath} to a random stat's potential and score, for good, never below 3"
        ));
    }
    match effect.levels {
        0 => {}
        1 => out.push("experience to the next level's threshold: a whole level".into()),
        -1 => out.push(
            "one experience level lost: experience to the threshold of the level below, its \
             hit points and mana taken back (never the last hit point); its choices, once \
             made, are kept, and the level gained again gives none"
                .into(),
        ),
        n if n > 0 => out.push(format!("{n} experience levels gained")),
        n => out.push(format!("{} experience levels lost", -n)),
    }
    if effect.talent {
        out.push(
            "a talent you may take and do not hold, drawn, given free and kept for life (never \
             one of your picks)"
                .into(),
        );
    }
    if effect.flaw {
        out.push("a flaw talent of the birth signs you lack, drawn, kept for life".into());
    }
    if out.is_empty() {
        return "nothing".into();
    }
    capital(&out.join("; "))
}

/// The lines an outcome says: its one line, or its lines (one drawn).
fn lines(o: &OutcomeDef) -> String {
    let quoted: Vec<String> = o.messages.iter().map(|m| format!("\"{m}\"")).collect();
    let mut text = quoted.join(" or ");
    if let Some(otherwise) = &o.otherwise {
        let _ = write!(text, " (when it finds nothing to do: \"{otherwise}\")");
    }
    text
}

/// Every outcome of the forgotten altar's table: its kind, tier, weight within its kind, share
/// of its kind, what it does exactly, and what it says.
pub(super) fn altar_outcomes(content: &Content) -> String {
    let Some(table) = altar(content) else {
        return String::new();
    };
    let mut out = String::from(
        "| Kind | Tier | Outcome | Weight | Of its kind | What it does | Message |\n\
         |---|---|---|:---:|:---:|---|---|\n",
    );
    for kind in OutcomeKind::ALL {
        let total = table.total(kind);
        let mut rows: Vec<&OutcomeDef> = table.of(kind).collect();
        rows.sort_by_key(|o| (o.tier, o.id.clone()));
        for o in rows {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} |",
                capital(kind.id()),
                o.tier.id(),
                capital(&o.id.replace('_', " ")),
                o.weight,
                percent(f64::from(o.weight) / f64::from(total.max(1))),
                does(content, &o.effect),
                lines(o),
            );
        }
    }
    out
}

/// How often a prayer brings each tier, by the side's lean, and what an altar gives over its
/// prayers: a row per lean (none, agreement, opposition, and Devout's agreement), each with the
/// kinds' weights, the shares per prayer of nothing, minor, moderate, and great outcomes and of
/// the altar exploding, and per altar the prayers it takes on average and the chance it
/// explodes. Then the facts the table and the room's sides give.
pub(super) fn altar_shares(content: &Content) -> String {
    let Some(table) = altar(content) else {
        return String::new();
    };
    let e = f64::from(table.exhaust) / 100.0;
    let lean = i32::try_from(table.lean).unwrap_or(0);
    let mut out = String::from(
        "| Lean | Boon / nothing / bane | Nothing | Minor | Moderate | Great | The altar \
         explodes | Prayers an altar takes | An altar explodes |\n\
         |---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|\n",
    );
    for (name, by) in [
        ("None (a neutral altar, or a neutral you)", 0),
        ("Agreement (good at good, evil at evil)", lean),
        ("Opposition (good at evil, evil at good)", -lean),
        ("Agreement, Devout", 2 * lean),
    ] {
        let kinds = table.leaned(by);
        let sum = f64::from(kinds.iter().sum::<u32>().max(1));
        let mut tiers = [0.0f64; 3];
        let mut nothing = 0.0;
        let mut burst = 0.0;
        for (kind, w) in OutcomeKind::ALL.into_iter().zip(kinds) {
            let total = f64::from(table.total(kind).max(1));
            for o in table.of(kind) {
                let p = f64::from(w) / sum * f64::from(o.weight) / total;
                if kind == OutcomeKind::Nothing {
                    nothing += p;
                } else {
                    tiers[o.tier as usize] += p;
                }
                if o.effect.explode.is_some() {
                    burst += p;
                }
            }
        }
        // Each prayer goes on to another unless the altar explodes or is spent.
        let go_on = (1.0 - burst) * (1.0 - e);
        let prayers = 1.0 / (1.0 - go_on);
        let explodes = burst / (1.0 - go_on);
        let _ = writeln!(
            out,
            "| {name} | {} / {} / {} | {} | {} | {} | {} | {} | {prayers:.2} | {} |",
            kinds[0],
            kinds[1],
            kinds[2],
            percent(nothing),
            percent(tiers[OutcomeTier::Minor as usize]),
            percent(tiers[OutcomeTier::Moderate as usize]),
            percent(tiers[OutcomeTier::Great as usize]),
            percent(burst),
            percent(explodes),
        );
    }
    let colour = |side: Alignment| match side {
        Alignment::Good => "gold",
        Alignment::Neutral => "grey",
        Alignment::Evil => "red",
    };
    let sides: Vec<String> = (Alignment::ALL.iter())
        .map(|&s| format!("{} {}", s.id(), colour(s)))
        .collect();
    let shift = content.alignment().map(|r| r.shift);
    let _ = write!(
        out,
        "\nA prayer takes {} seconds. After a completed prayer the altar is spent with {}% \
         chance, whatever the prayer gave, unless it exploded. The lean moves {} points between \
         boon and bane. An altar's side is drawn one good to two neutral to one evil, and its \
         glow shows it: {}. Praying moves your alignment by {} at a good altar and {} at an evil \
         one.\n",
        table.seconds,
        table.exhaust,
        table.lean,
        sides.join(", "),
        shift.map_or(0, |s| s.prayed_good),
        shift.map_or(0, |s| s.prayed_evil),
    );
    out
}

/// Every outcome of the healing spring's table: its kind, tier, chance per drink, what it does
/// exactly, and what it says; then how long a drink takes.
pub(super) fn spring_outcomes(content: &Content) -> String {
    let Some(table) = content.outcomes().get("spring") else {
        return String::new();
    };
    let sum = f64::from(table.kinds.iter().sum::<u32>().max(1));
    let mut out = String::from(
        "| Kind | Tier | Outcome | Per drink | What it does | Message |\n\
         |---|---|---|:---:|---|---|\n",
    );
    for (kind, w) in OutcomeKind::ALL.into_iter().zip(table.kinds) {
        let total = f64::from(table.total(kind).max(1));
        let mut rows: Vec<&OutcomeDef> = table.of(kind).collect();
        rows.sort_by_key(|o| (o.tier, o.id.clone()));
        for o in rows {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} |",
                capital(kind.id()),
                o.tier.id(),
                capital(&o.id.replace('_', " ")),
                percent(f64::from(w) / sum * f64::from(o.weight) / total),
                does(content, &o.effect),
                lines(o),
            );
        }
    }
    let dry = if table.exhaust == 0 {
        "A spring never runs dry: drink as often as you like.".to_string()
    } else {
        format!(
            "After a drink the spring runs dry with {}% chance.",
            table.exhaust
        )
    };
    let _ = write!(out, "\nA drink takes {} seconds. {dry}\n", table.seconds);
    out
}
