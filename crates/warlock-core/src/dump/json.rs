//! The structured (JSON) dump.

use super::*;

/// Structured dump: turn, game time, level, viewer, the special room the player stands in if they
/// know it (its effect id; else null), the open shop they stand in (`shop`: its sign, keeper,
/// and what they owe there; else null), the weather (under the open sky; else null), the
/// temperature (°C, to a tenth), seen actors (with the status effects that show on them, and
/// `unaware` with Stalker's eye), heard
/// sounds, known traps (`known_traps`: position, trap id, and whether in sight), the hollow walls
/// found (`known_hollows`: positions), the chests seen
/// or remembered (`chests`: position, band id, state, and, with Chest-wise, whether a shut one
/// in sight is locked, else null), known overworld
/// locations (`known_locations`: position and name), the player's inventory (with what of each
/// stack is unpaid, priced), equipment (by slot id), weight, scores (by stat id), status effects
/// (by id), the diseases they know (by id), recent messages, the requested layers as arrays of
/// rows, and optionally the full [`crate::WorldState`] under `state` (with the current level's
/// `danger`, ground truth).
pub fn json(world: &World, layers: &[Layer], include_state: bool) -> Value {
    let view = world.perception();
    let content = world.content();
    let seen: Vec<Value> = view
        .actors()
        .iter()
        .map(|a| {
            let def = content.creature(a.creature);
            let effects: Vec<&str> = (view.effects_of(a.id))
                .map(|s| content.status(s).id.as_str())
                .collect();
            let mut seen = json!({
                "id": a.id, "kind": a.kind, "creature": def.id, "glyph": a.glyph(content).to_string(),
                "fg": a.fg(content).to_string(), "pos": a.pos, "health": a.health, "effects": effects,
            });
            // What the player's Knowledge tells of it, in words (spec §8 "Skills").
            let known = view.known_resistances(a);
            if !known.is_empty() {
                let known: serde_json::Map<String, Value> = (known.into_iter())
                    .map(|(k, w)| (content.damage_type(k).id.clone(), json!(w.word())))
                    .collect();
                seen["known"] = Value::Object(known);
            }
            // A composite's colours, top band first (spec §8 "Animated objects").
            let bands = a.shape.as_ref().map_or(&[][..], |s| s.bands());
            if !bands.is_empty() {
                let bands: Vec<String> = bands.iter().map(ToString::to_string).collect();
                seen["bands"] = json!(bands);
            }
            // Its grade, the word (spec §8 "Grades").
            if let Some(word) = crate::grade::word(def, a.grade) {
                seen["grade"] = json!(word);
            }
            // Stalker's eye: it has not noticed the player (never told without the talent).
            if a.unaware {
                seen["unaware"] = json!(true);
            }
            // The player's mark (`world/classes/mark.rs`).
            if a.marked {
                seen["marked"] = json!(true);
            }
            // An ally of the player's (spec §8 "Allies").
            if a.ally {
                seen["ally"] = json!(true);
            }
            seen
        })
        .collect();
    let messages: Vec<&crate::messages::Message> = {
        let all: Vec<_> = world.messages().iter().collect();
        all[all.len().saturating_sub(RECENT_MESSAGES)..].to_vec()
    };
    let unpaid = view.unpaid();
    let inventory: Vec<Value> = view
        .inventory()
        .iter()
        .enumerate()
        .map(|(i, s)| {
            // An item's affixes only once the player has identified it.
            let magic = s
                .known_magic()
                .map(|m| json!({ "level": m.level, "name": m.name, "affixes": m.affixes }));
            let mut entry = json!({
                "letter": position_text(i),
                "object": content.object(s.object).id,
                "class": content.item_class(view.item_class(s)).id,
                "count": s.count,
                "weight": s.weight(content),
                "name": view.lore().describe(s),
                "rarity": s.rarity(),
                "identified": view.lore().knows(s),
                "magic": magic,
                "numbers": numbers_json(s, content),
            });
            if let Some(price) = unpaid.get(i).copied().flatten() {
                entry["unpaid"] = json!(price);
            }
            if let Some(rung) = s.known_wand_rung(view.lore()) {
                entry["wand_rung"] = json!(rung);
            }
            if let Some(range) = known_flask_range(&view, i) {
                entry["throw"] = json!({ "range": range });
            }
            entry
        })
        .collect();
    let shop = view
        .shop_here()
        .map(|s| json!({ "sign": s.sign(), "keeper": s.keeper(), "owed": s.owed() }));
    let equipment: serde_json::Map<String, Value> = (view.equipment().iter())
        .filter_map(|(slot, item)| {
            let s = view.inventory().get(item)?;
            let entry = json!({
                "letter": position_text(item),
                "object": content.object(s.object).id,
                "count": s.count,
            });
            Some((content.slot(slot).id.clone(), entry))
        })
        .collect();
    let weight = view.own_weight();
    let load = view.own_load();
    let (hp, max_hp) = view.own_hp();
    let (mana, max_mana) = view.own_mana();
    let death = world.death().map(|d| {
        json!({
            "turn": d.turn,
            "time": d.time.to_string(),
            "place": d.place,
            "by": d.by.map(|c| content.creature(c).id.clone()),
            "trap": match d.cause {
                Some(Cause::Trap(t)) => Some(content.trap(t).id.clone()),
                _ => None,
            },
            "cause": d.cause.map(|c| c.epitaph(content)),
            "of": d.of.map(|s| content.status(s).id.clone()),
        })
    });
    let effects: Vec<&str> = (view.own_effects())
        .map(|s| content.status(s).id.as_str())
        .collect();
    let diseases: Vec<&str> = (view.own_diseases())
        .map(|d| content.disease(d).id.as_str())
        .collect();
    let table = content.weapon_skills();
    let weapon_skills: serde_json::Map<String, Value> = (table.ids())
        .filter(|&(id, _)| view.own_weapon_skills().shows(table, id))
        .map(|(id, def)| {
            let rank = view.own_weapon_skills().rank(id);
            let progress = progress_percent(&view, id);
            (
                def.id.clone(),
                json!({ "rank": rank, "progress": progress }),
            )
        })
        .collect();
    // The caster styles, as `weapon_skills` holds them too.
    let ranked = |id: WeaponSkillId| {
        let def = table.skill(id);
        json!({
            "id": def.id,
            "rank": view.own_weapon_skills().rank(id),
            "progress": progress_percent(&view, id),
        })
    };
    let caster_styles: Vec<Value> = (table.styles())
        .filter(|(id, _, style)| style.grip.casts() && view.own_weapon_skills().shows(table, *id))
        .map(|(id, ..)| ranked(id))
        .collect();
    let traps: Vec<Value> = (view.known_traps().into_iter())
        .map(|(p, trap)| {
            json!({
                "pos": p,
                "trap": content.trap(trap).id,
                "seen": view.sees_objects_at(p),
                "rigged": view.rigged(p),
            })
        })
        .collect();
    let locations: Vec<Value> = (view.known_locations().into_iter())
        .map(|(p, name)| json!({ "pos": p, "name": name }))
        .collect();
    // The chests the player sees or remembers (spec §8 "Chests"), as they see them.
    let chests: Vec<Value> = (view.known_chests().into_iter())
        .map(|(p, look)| {
            let band = (content.chests().bands.get(look.band)).map(|b| b.id.as_str());
            json!({ "pos": p, "band": band, "state": look.state.id(), "locked": look.locked })
        })
        .collect();
    // The leavings the player sees or remembers (`content/leaving.rs`).
    let leavings: Vec<Value> = (view.known_leavings().into_iter())
        .map(|(p, leaving)| json!({ "pos": p, "kind": leaving.id() }))
        .collect();
    let layer_map: serde_json::Map<String, Value> = layers
        .iter()
        .map(|&l| (l.name().to_string(), json!(layer_rows(world, l))))
        .collect();
    let talents: Vec<Value> = (view.own_talents().iter())
        .map(|t| {
            let mut talent = json!({
                "id": content.talents().get(t.id).id,
                "name": t.name,
                "choice": t.choice,
                "granted": t.granted,
                "flaw": t.flaw,
                "class": t.class,
            });
            // A profane gift's kind (docs/design/profane-gifts.md §11), left out otherwise.
            if let Some(kind) = t.profane {
                talent["profane"] = json!(kind.id());
            }
            talent
        })
        .collect();
    let mut out = json!({
        "game_version": crate::GAME_VERSION,
        "scenario": world.scenario(),
        "seed": world.seed(),
        "turn": world.turn(),
        "time": time_json(world),
        "level": view.level_id(),
        "viewer": view.viewer(),
        "room": view.room(view.viewer()).map(|e| e.id.as_str()),
        "pervading": view.pervading().map(|e| e.id.as_str()),
        "shop": shop,
        "weather": view.weather(),
        // In °C, to a tenth of a degree.
        "temperature": f64::from(view.temperature().tenths()) / 10.0,
        "hash": format!("{:016x}", world.state_hash()),
        "seen_actors": seen,
        "heard_sounds": view.sounds(),
        "known_traps": traps,
        "known_hollows": view.known_hollows(),
        "known_locations": locations,
        "chests": chests,
        "inventory": inventory,
        "equipment": equipment,
        "weight": { "base": weight.base, "carried": weight.carried, "total": weight.total() },
        "burden": { "level": load.burden(), "limit": load.limit },
        "character": view.character().map(|c| {
            let mut character = json!({
                "name": c.name,
                "race": c.spec.race,
                "gender": c.spec.gender,
                "class": c.spec.class,
                "raised": c.raised,
                "age": c.age_on(view.time()),
                "born": c.born,
                "sign": c.sign,
            });
            // A lucky day's title (spec §8 "Birth sign"), only for one born on it.
            if let Some(title) = c.title(content) {
                character["lucky"] = json!(title);
            }
            // The prestige class once called, and every class with its own level (spec §8
            // "Prestige classes").
            if let Some(prestige) = &c.prestige {
                character["prestige"] = json!(prestige);
            }
            let level = view.own_experience().level;
            let classes: Vec<Value> = (c.class_levels(content, level).into_iter())
                .map(|(table, own)| json!({ "class": table.id(), "level": own }))
                .collect();
            character["classes"] = json!(classes);
            // The background story's lines by id (spec §8 "Background story"), if told.
            if !c.story.is_empty() {
                character["story"] = c.story.iter().map(|t| t.id()).collect();
            }
            character
        }),
        "stats": view.own_scores(),
        "alignment": view.own_alignment().map(|b| b.id()),
        "skills": skills_json(&view),
        "effects": effects,
        "diseases": diseases,
        "weapon_skills": weapon_skills,
        "caster_styles": caster_styles,
        "styles_in_use": (view.own_styles_in_use().into_iter())
            .map(|id| content.weapon_skills().skill(id).id.clone())
            .collect::<Vec<_>>(),
        "attacks": attacks_json(&view),
        "talents": talents,
        "due": view.due(),
        "abilities": abilities_json(&view),
        "spells": spells_json(&view),
        "quests": quests_json(&view),
        "story": story_json(world),
    });
    // A second `json!`: one macro call this long passes the recursion limit.
    let Value::Object(rest) = json!({
        "hp": { "current": hp, "max": max_hp },
        "mana": { "current": mana, "max": max_mana },
        "resources": (view.own_resources().iter())
            .map(|r| json!({
                "id": content.resource(r.resource).id,
                "current": r.value,
                "max": r.max,
                "on": view.own_mark().filter(|_| r.marked),
            }))
            .collect::<Vec<_>>(),
        "experience": view.own_experience(),
        "defence": view.own_defence(),
        "protection": view.own_protection(),
        "resistances": (view.own_resistances().into_iter())
            .map(|(k, w)| (content.damage_type(k).id.clone(), json!(w.word())))
            .collect::<serde_json::Map<String, Value>>(),
        "hunger": view.own_hunger(),
        "stance": view.own_stance().id(),
        "dazzled": view.dazzled(),
        "averted": view.averted(),
        // The player's allies, wherever they are (spec §8 "Allies").
        "allies": (view.allies().iter())
            .map(|a| json!({
                "id": a.id,
                "creature": content.creature(a.creature).id,
                "name": a.name,
                "health": a.health,
                "order": a.order.id(),
                "at": a.order.at(),
                "source": a.source.id(),
                "lasts": a.lasts,
                "retinue": a.retinue,
                "here": a.here,
            }))
            .collect::<Vec<_>>(),
        // The player's companion (spec §8 "Allies"; `world/allies/companion.rs`), or null.
        "companion": kept_json(&view, Kept::Companion),
        // The player's familiar (`world/allies/familiar.rs`), or null.
        "familiar": kept_json(&view, Kept::Familiar),
        // The overworld encounter to answer (spec §7 "Overworld encounters"), or null.
        "encounter": view.encounter().map(|e| {
            let (entry, outcome) = view.encounter_pending().expect("pending");
            json!({
                "entry": entry,
                "name": e.name,
                "outcome": outcome.id(),
                "title": e.title,
                "text": e.text,
                "choices": (e.choices.iter())
                    .map(|(label, c)| json!({ "id": c.id(), "label": label }))
                    .collect::<Vec<_>>(),
            })
        }),
        "death": death,
        "messages": messages,
        "layers": layer_map,
    }) else {
        unreachable!("an object literal")
    };
    out.as_object_mut().expect("an object").extend(rest);
    out["leavings"] = json!(leavings);
    // The guilds they belong to and their ranks (left out before joining one).
    let guilds = view.own_guilds();
    if !guilds.is_empty() {
        out["guilds"] = (guilds.iter())
            .map(|g| json!({ "id": g.guild.id, "name": g.guild.name, "rank": g.title() }))
            .collect();
    }
    if include_state {
        out["state"] = serde_json::to_value(world.state()).expect("state serializes");
        // The current level's danger (spec §7 "Danger levels"): ground truth, never stored.
        let level = &world.state().current_level;
        out["danger"] = json!(world.content().floor_danger(level, world.player_level()));
        // The encounter tables matching the player's overworld cell (ground truth).
        out["encounter_tables"] = json!(world.encounter_tables_here());
        // The callings of the campaign and where the player stands with each.
        let callings: Vec<Value> = (world.content().callings_of(world.campaign()))
            .map(|c| {
                let status = match world.calling_status(c) {
                    CallingStatus::Open => "open",
                    CallingStatus::LevelNotMet => "level_not_met",
                    CallingStatus::NotFrom => "not_from",
                    CallingStatus::Called => "called",
                };
                json!({ "class": c.class, "level": c.level, "from": c.from, "status": status })
            })
            .collect();
        out["callings"] = json!(callings);
        // The priest's prayer: its strain, lock, face, and uses (ground truth).
        if let Some(p) = world.prayer_state() {
            out["prayer"] = json!({
                "strain": p.strain,
                "locked": p.locked,
                "face": p.face.id(),
                "uses": p.uses.iter().map(|u| u.id()).collect::<Vec<_>>(),
                "answers": p.answers,
            });
        }
        // The current level's zones (ground truth).
        out["zones"] = (world.zones().iter())
            .map(|z| {
                json!({
                    "kind": z.kind().id(),
                    "cells": z.cells().collect::<Vec<_>>(),
                    "until": z.until().seconds(),
                    "by": z.by().0,
                    "harm": z.harm(),
                })
            })
            .collect();
        // The current level's hallowed ground (Sanctuary; ground truth).
        out["hallows"] = (world.hallows().iter())
            .map(|h| {
                json!({
                    "pos": h.at(),
                    "radius": h.radius(),
                    "until": h.until().seconds(),
                    "dark": h.dark(),
                })
            })
            .collect();
    }
    out
}

/// The clock in seconds since the epoch, and as a calendar date and time of day.
pub(super) fn time_json(world: &World) -> Value {
    let t = world.time();
    json!({
        "seconds": t.seconds(),
        "text": t.to_string(),
        "year": t.year(),
        "month": t.month(),
        "month_name": world.content().calendar().month_name(t),
        "day": t.day(),
        "hour": t.hour(),
        "minute": t.minute(),
        "second": t.second(),
    })
}
