//! Action text: the syntax replays, tests, and `warlock-cli` write actions in (`move:E wait`).

use super::*;

/// Text syntax for scripting: `move:E wait open:E close:W kick:N pick:E ascend descend use_tool
/// rest:1h rest:dawn rest:dusk pickup pickup:ac drop:b sell:b pay pay:W equip:body:b remove:body
/// bash:E fire:12,5 throw:c:12,5 dig:E search search:10 disarm:E eat:b eat:floor:a read:b
/// read:b:12,5 read:b:c
/// chat:E ask:E:TOPIC steal:E throw_voice:12,5 vision:ID
/// skip:1h
/// weather:rain effect:fireball@12,5 effect:lightning_bolt@3,4>12,4 identify:a afflict:bleeding
/// drain:5 infect:filth_fever freeze:12,5,3 thaw:12,5,3` (durations: `30s`, `10m`, `2h`; LETTER names a
/// stack by its position, a letter for the first 26 (`a` = 0) or a number for any (`30`), and
/// LETTERS several, as letters (`ac`) or numbers (`0,2,30`); `pickup`, `drop`, `sell`, and
/// `identify` without letters take everything; `pay:DIR` pays and steps that way, out of the shop;
/// `equip:SLOT:LETTER` puts inventory stack LETTER on in slot SLOT, `remove:SLOT` takes off what
/// is there; `pick:DIR` picks the lock of the door that way; `bash:DIR` bashes with the held
/// shield; `dig:DIR` digs toward a wall; `search` searches once and `search:N` for up to N turns,
/// 1 to [`MAX_SEARCH`]; `disarm:DIR` tries to disarm the known trap that way; `fire:X,Y`
/// shoots at the level cell (X, Y), `throw:LETTER:X,Y` throws one of inventory stack LETTER
/// there; `eat:LETTER` eats one of inventory stack LETTER, `eat:floor:LETTER` one of the stack
/// LETTER of the pile underfoot; `read:LETTER:X,Y` reads inventory stack LETTER (a scroll) at
/// the level cell (X, Y), `read:LETTER:LETTER` on the second inventory stack; `chat:DIR` talks to the creature that way, `ask:DIR:TOPIC` asks
/// it about a topic by id, `steal:DIR` tries to steal from it; `use_tool` uses the light ready
/// in the Tool slot (a lantern: unlit, lit, hooded, unlit again); `throw_voice:X,Y` throws
/// the player's voice to the level cell (X, Y); weather: `clear`, `overcast`, `rain`, `heavy`; `effect:ID@X,Y[>X,Y]`
/// shows a spell effect at a cell, or from one cell toward another; the dev bursts
/// `freeze:X,Y,R` and `thaw:X,Y,R` freeze the water and melt the ice within radius R, 0 to
/// [`MAX_BURST`], of cell (X, Y)).
///
/// `Display` and `FromStr` round-trip; action names are case-insensitive when parsing (vision
/// profile, slot, and effect ids keep their case).
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Move(d) => write!(f, "move:{d}"),
            Action::Run(d) => write!(f, "run:{d}"),
            Action::Wait => f.write_str("wait"),
            Action::Open(d) => write!(f, "open:{d}"),
            Action::Close(d) => write!(f, "close:{d}"),
            Action::Kick(d) => write!(f, "kick:{d}"),
            Action::PickLock(d) => write!(f, "pick:{d}"),
            Action::Ascend => f.write_str("ascend"),
            Action::Descend => f.write_str("descend"),
            Action::UseTool => f.write_str("use_tool"),
            Action::PickUp(l) if l.is_empty() => f.write_str("pickup"),
            Action::PickUp(l) => write!(f, "pickup:{l}"),
            Action::Drop(l) if l.is_empty() => f.write_str("drop"),
            Action::Drop(l) => write!(f, "drop:{l}"),
            Action::Sell(l) if l.is_empty() => f.write_str("sell"),
            Action::Sell(l) => write!(f, "sell:{l}"),
            Action::Pay(None) => f.write_str("pay"),
            Action::Pay(Some(d)) => write!(f, "pay:{d}"),
            Action::Equip { slot, item } => write!(f, "equip:{slot}:{}", position_text(*item)),
            Action::Remove(slot) => write!(f, "remove:{slot}"),
            Action::Bash(d) => write!(f, "bash:{d}"),
            Action::Fire(at) => write!(f, "fire:{},{}", at.x, at.y),
            Action::Throw { item, at } => {
                write!(f, "throw:{}:{},{}", position_text(*item), at.x, at.y)
            }
            Action::Eat(Meal::Pack(item)) => write!(f, "eat:{}", position_text(*item)),
            Action::Eat(Meal::Floor(item)) => write!(f, "eat:floor:{}", position_text(*item)),
            Action::Read { item, target } => {
                write!(f, "read:{}", position_text(*item))?;
                match target {
                    None => Ok(()),
                    Some(ReadTarget::Cell(at)) => write!(f, ":{},{}", at.x, at.y),
                    Some(ReadTarget::Item(i)) => write!(f, ":{}", position_text(*i)),
                }
            }
            Action::Scribe { spell } => write!(f, "scribe:{spell}"),
            Action::Rest(Rest::For(d)) => write!(f, "rest:{d}"),
            Action::Rest(Rest::Dawn) => f.write_str("rest:dawn"),
            Action::Rest(Rest::Dusk) => f.write_str("rest:dusk"),
            Action::Encounter(choice) => write!(f, "encounter:{}", choice.id()),
            Action::Chat(d) => write!(f, "chat:{d}"),
            Action::Ask { dir, topic } => write!(f, "ask:{dir}:{topic}"),
            Action::Steal(d) => write!(f, "steal:{d}"),
            Action::Feint(d) => write!(f, "feint:{d}"),
            Action::Tame(d) => write!(f, "tame:{d}"),
            Action::Intimidate(at) => write!(f, "intimidate:{},{}", at.x, at.y),
            Action::ThrowVoice(at) => write!(f, "throw_voice:{},{}", at.x, at.y),
            Action::Order(Command::Heel) => f.write_str("order:follow"),
            Action::Order(Command::Hold) => f.write_str("order:stay"),
            Action::Order(Command::SetOn(at)) => write!(f, "order:hunt:{},{}", at.x, at.y),
            Action::SetStance(stance) => write!(f, "stance:{stance}"),
            Action::Dig(d) => write!(f, "dig:{d}"),
            Action::Search(1) => f.write_str("search"),
            Action::Search(turns) => write!(f, "search:{turns}"),
            Action::Disarm(d) => write!(f, "disarm:{d}"),
            Action::Listen(d) => write!(f, "listen:{d}"),
            Action::Wedge(d) => write!(f, "wedge:{d}"),
            Action::Heal { what: None, .. } => f.write_str("heal"),
            Action::Heal {
                what: Some(id),
                ally: None,
            } => write!(f, "heal:{id}"),
            Action::Heal {
                what: Some(id),
                ally: Some(d),
            } => write!(f, "heal:{id}:{d}"),
            Action::Pray => f.write_str("pray"),
            Action::Drink => f.write_str("drink"),
            Action::Forage => f.write_str("forage"),
            Action::Gather(d) => write!(f, "gather:{d}"),
            Action::Craft { recipe, picks } if picks.is_empty() => write!(f, "craft:{recipe}"),
            Action::Craft { recipe, picks } => {
                let picks: Vec<String> = (picks.iter())
                    .map(|p| p.map_or_else(|| "-".to_string(), position_text))
                    .collect();
                write!(f, "craft:{recipe}:{}", picks.join(","))
            }
            Action::Decipher { item } => write!(f, "decipher:{}", position_text(*item)),
            Action::Temper { item } => write!(f, "temper:{}", position_text(*item)),
            Action::Whittle { item } => write!(f, "whittle:{}", position_text(*item)),
            Action::Salvage { item } => write!(f, "salvage:{}", position_text(*item)),
            Action::Rig { rig, dir: None } => write!(f, "rig:{rig}"),
            Action::Rig { rig, dir: Some(d) } => write!(f, "rig:{rig}:{d}"),
            Action::Give { item, ally } => {
                write!(f, "give:{}", position_text(*item))?;
                match ally {
                    Some(dir) => write!(f, ":{dir}"),
                    None => Ok(()),
                }
            }
            Action::UseTalent(active) => write!(f, "use:{}", active.id()),
            Action::Use(active, None) => write!(f, "use:{}", active.id()),
            Action::Use(active, Some(d)) => write!(f, "use:{}:{d}", active.id()),
            Action::Cast(spell, None) => write!(f, "use:{spell}"),
            Action::Cast(spell, Some(at)) => write!(f, "use:{spell}:{},{}", at.x, at.y),
            Action::Act(id, dir) => write!(f, "use:{id}:{dir}"),
            Action::CastOn(spell, on) => write!(f, "use:{spell}:{}", position_text(*on)),
            Action::UseAt(active, at) => write!(f, "use:{}:{},{}", active.id(), at.x, at.y),
            Action::UseOn(active, on) => write!(f, "use:{}:{}", active.id(), position_text(*on)),
            Action::UseThrow(active, item, at) => write!(
                f,
                "use:{}:{}:{},{}",
                active.id(),
                position_text(*item),
                at.x,
                at.y
            ),
            Action::Shape(active, shape) => write!(f, "use:{}:{shape}", active.id()),
            Action::Gift(active, None) => write!(f, "use:{}", active.id()),
            Action::Gift(active, Some(at)) => write!(f, "use:{}:{},{}", active.id(), at.x, at.y),
            Action::Talent { id, choice: None } => write!(f, "talent:{id}"),
            Action::Talent {
                id,
                choice: Some(choice),
            } => write!(f, "talent:{id}:{choice}"),
            Action::Advance(skill) => write!(f, "advance:{skill}"),
            Action::LevelUpAuto => f.write_str("levelup:auto"),
            Action::GrantXp(xp) => write!(f, "xp:{xp}"),
            Action::Call(id) => write!(f, "call:{id}"),
            Action::SetRank { skill, rank } => write!(f, "rank:{skill}:{rank}"),
            Action::SetVision(id) => write!(f, "vision:{id}"),
            Action::SkipTime(d) => write!(f, "skip:{d}"),
            Action::SetWeather(w) => write!(f, "weather:{}", w.id()),
            Action::Spawn => f.write_str("spawn"),
            Action::Ally {
                creature,
                count,
                lasts: None,
            } if *count == 1 => write!(f, "ally:{creature}"),
            Action::Ally {
                creature,
                count,
                lasts: None,
            } => write!(f, "ally:{creature}:{count}"),
            Action::Ally {
                creature,
                count,
                lasts: Some(lasts),
            } => write!(f, "ally:{creature}:{count}:{lasts}"),
            Action::Meet { entry, outcome } => match outcome {
                Some(o) => write!(f, "meet:{entry}:{}", o.id()),
                None => write!(f, "meet:{entry}"),
            },
            Action::Effect { effect, at, to } => {
                write!(f, "effect:{effect}@{},{}", at.x, at.y)?;
                match to {
                    Some(to) => write!(f, ">{},{}", to.x, to.y),
                    None => Ok(()),
                }
            }
            Action::Identify(l) if l.is_empty() => f.write_str("identify"),
            Action::Identify(l) => write!(f, "identify:{l}"),
            Action::Afflict(id) => write!(f, "afflict:{id}"),
            Action::Drain(amount) => write!(f, "drain:{amount}"),
            Action::Resource(id) => write!(f, "resource:{id}"),
            Action::Infect(id) => write!(f, "infect:{id}"),
            Action::Reveal(id) => write!(f, "reveal:{id}"),
            Action::Lure(id) => write!(f, "lure:{id}"),
            Action::QuestRoll { quest, target } => write!(f, "quest_roll:{quest}:{target}"),
            Action::Freeze { at, radius } => write!(f, "freeze:{},{},{radius}", at.x, at.y),
            Action::Thaw { at, radius } => write!(f, "thaw:{},{},{radius}", at.x, at.y),
            Action::Zone { kind, at, radius } => {
                write!(f, "zone:{}@{},{},{radius}", kind.id(), at.x, at.y)
            }
        }
    }
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_ascii_lowercase();
        let (verb, arg) = match lower.split_once(':') {
            Some((verb, arg)) => (verb, Some(arg)),
            None => (lower.as_str(), None),
        };
        let dir = || -> Result<Dir, String> {
            arg.ok_or_else(|| format!("`{verb}` needs a direction, e.g. `{verb}:E`"))?
                .parse()
        };
        let bare = |action: Action| match arg {
            None => Ok(action),
            Some(_) => Err(format!("`{verb}` takes no direction")),
        };
        match verb {
            "move" => dir().map(Action::Move),
            "run" => dir().map(Action::Run),
            "open" => dir().map(Action::Open),
            "close" => dir().map(Action::Close),
            "chat" => dir().map(Action::Chat),
            "ask" => {
                let usage =
                    || "`ask` needs a direction and a topic, e.g. `ask:E:rooms`".to_string();
                let (dir, topic) = arg.and_then(|a| a.split_once(':')).ok_or_else(usage)?;
                if topic.is_empty() || topic.contains(':') {
                    return Err(usage());
                }
                Ok(Action::Ask {
                    dir: dir.parse().map_err(|_| usage())?,
                    topic: topic.to_string(),
                })
            }
            "steal" => dir().map(Action::Steal),
            "feint" => dir().map(Action::Feint),
            "tame" => dir().map(Action::Tame),
            "kick" => dir().map(Action::Kick),
            "pick" => dir().map(Action::PickLock),
            "bash" => dir().map(Action::Bash),
            "dig" => dir().map(Action::Dig),
            "disarm" => dir().map(Action::Disarm),
            "listen" => dir().map(Action::Listen),
            "gather" => dir().map(Action::Gather),
            "wedge" => dir().map(Action::Wedge),
            "heal" => {
                let usage = || {
                    "`heal:ID[:DIR]` names what to treat, e.g. `heal:bleeding` or \
                     `heal:bleeding:E` (an ally's)"
                        .to_string()
                };
                let Some(arg) = arg else {
                    return Ok(Action::Heal {
                        what: None,
                        ally: None,
                    });
                };
                let (id, ally) = match arg.split_once(':') {
                    Some((id, d)) => (id, Some(d.parse::<Dir>().map_err(|_| usage())?)),
                    None => (arg, None),
                };
                if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    return Err(usage());
                }
                Ok(Action::Heal {
                    what: Some(id.to_string()),
                    ally,
                })
            }
            "search" => match arg.map(str::parse::<u8>) {
                None => Ok(Action::Search(1)),
                Some(Ok(turns)) if (1..=MAX_SEARCH).contains(&turns) => Ok(Action::Search(turns)),
                Some(_) => Err(format!(
                    "`search:N` searches for 1 to {MAX_SEARCH} turns, e.g. `search:10`"
                )),
            },
            "wait" => bare(Action::Wait),
            "pray" => bare(Action::Pray),
            "drink" => bare(Action::Drink),
            "forage" => bare(Action::Forage),
            "craft" => {
                let usage = || {
                    "`craft` needs a recipe's id and its picks, e.g. `craft:long_sword:c,f` \
                     (`-` a skipped optional)"
                        .to_string()
                };
                let arg = arg.ok_or_else(usage)?;
                let (recipe, picks) = match arg.split_once(':') {
                    Some((recipe, picks)) => (recipe, Some(picks)),
                    None => (arg, None),
                };
                let id = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_';
                if recipe.is_empty() || !recipe.chars().all(id) {
                    return Err(usage());
                }
                let picks = match picks {
                    None => Vec::new(),
                    Some(picks) => (picks.split(','))
                        .map(|p| match p {
                            "-" => Ok(None),
                            p => parse_position(p).map(Some).ok_or_else(usage),
                        })
                        .collect::<Result<_, _>>()?,
                };
                Ok(Action::Craft {
                    recipe: recipe.to_string(),
                    picks,
                })
            }
            "decipher" => {
                let usage = || "`decipher` needs a letter, e.g. `decipher:b`".to_string();
                let item = arg.and_then(parse_position).ok_or_else(usage)?;
                Ok(Action::Decipher { item })
            }
            "temper" => {
                let usage = || "`temper` needs a letter, e.g. `temper:b`".to_string();
                let item = arg.and_then(parse_position).ok_or_else(usage)?;
                Ok(Action::Temper { item })
            }
            "whittle" => {
                let usage = || "`whittle` needs a letter, e.g. `whittle:b`".to_string();
                let item = arg.and_then(parse_position).ok_or_else(usage)?;
                Ok(Action::Whittle { item })
            }
            "salvage" => {
                let usage = || "`salvage` needs a letter, e.g. `salvage:b`".to_string();
                let item = arg.and_then(parse_position).ok_or_else(usage)?;
                Ok(Action::Salvage { item })
            }
            "rig" => {
                let usage =
                    || "`rig:ID:DIR` names a rig and a way, e.g. `rig:tripwire:E`".to_string();
                let arg = arg.ok_or_else(usage)?;
                let (rig, dir) = match arg.split_once(':') {
                    Some((rig, d)) => (rig, Some(d.parse::<Dir>().map_err(|_| usage())?)),
                    None => (arg, None),
                };
                let id = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_';
                if rig.is_empty() || !rig.chars().all(id) {
                    return Err(usage());
                }
                Ok(Action::Rig {
                    rig: rig.to_string(),
                    dir,
                })
            }
            "give" => {
                let usage = || "`give:LETTER[:DIR]` names a stack, e.g. `give:b`".to_string();
                let arg = arg.ok_or_else(usage)?;
                let (item, ally) = match arg.split_once(':') {
                    Some((item, d)) => (item, Some(d.parse::<Dir>().map_err(|_| usage())?)),
                    None => (arg, None),
                };
                let item = parse_position(item).ok_or_else(usage)?;
                Ok(Action::Give { item, ally })
            }
            "use" => {
                let usage = || {
                    "`use` needs an ability, e.g. `use:rage`, `use:smite:E`, `use:second_wind`, \
                     `use:blaze`, or `use:transfixing_stare:12,5`"
                };
                let (id, aim) = match arg.ok_or_else(usage)?.split_once(':') {
                    Some((id, aim)) => (id, Some(aim)),
                    None => (arg.unwrap_or_default(), None),
                };
                // A talent's own active asks for nothing, `use:second_wind`.
                if let Some(active) = TalentActive::from_id(id) {
                    return match aim {
                        None => Ok(Action::UseTalent(active)),
                        Some(_) => Err(format!("`use:{id}` takes no argument")),
                    };
                }
                // A birth sign's gift aims at a cell, `use:transfixing_stare:12,5`.
                if let Some(gift) = SignActive::from_id(id) {
                    let at = aim.map(parse_cell).transpose()?;
                    return Ok(Action::Gift(gift, at));
                }
                // Anything else that is no class active is a spell (its data's id), which may
                // aim at a cell too, `use:magic_missile:12,5`; the rules refuse an unknown one.
                let Some(active) = ClassActive::from_id(id) else {
                    if id.is_empty() || !id.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                        return Err(format!("`{id}` is no ability that is used"));
                    }
                    // A weapon active aimed in a direction, `use:lunge:E`: the direction in
                    // upper case (as typed), as a stack's letter is in lower case; NE, SE, SW,
                    // and NW, which are no letters, in either.
                    let typed = s.rsplit_once(':').map(|(_, a)| a);
                    let named = |a: &str| Dir::ALL.into_iter().find(|d| d.name() == a);
                    let dir = (typed.and_then(named))
                        .or_else(|| aim.and_then(|a| named(&a.to_ascii_uppercase())))
                        .filter(|d| typed.is_some_and(|t| t.len() == 2 || t == d.name()));
                    if let Some(dir) = dir {
                        return Ok(Action::Act(id.to_string(), dir));
                    }
                    // A spell on a stack the player carries names it by position,
                    // `use:identify:c`.
                    if let Some(on) = aim.filter(|a| !a.contains(',')) {
                        let on = parse_position(on).ok_or_else(|| {
                            format!("`use:{id}` needs a cell or a stack, e.g. `use:{id}:12,5`")
                        })?;
                        return Ok(Action::CastOn(id.to_string(), on));
                    }
                    let at = aim.map(parse_cell).transpose()?;
                    return Ok(Action::Cast(id.to_string(), at));
                };
                // A class active that aims at a cell, `use:volley:12,5`.
                if active.aims_cell() {
                    let at = aim
                        .ok_or_else(|| format!("`use:{id}` needs a cell, e.g. `use:{id}:12,5`"))?;
                    return Ok(Action::UseAt(active, parse_cell(at)?));
                }
                // One that throws a stack at a cell, `use:thrown_exploit:c:12,5` (bare, the rules
                // ask for both).
                if let Some(aim) = aim.filter(|_| active.aims_throw()) {
                    let usage =
                        || format!("`use:{id}` needs a stack and a cell, e.g. `use:{id}:c:12,5`");
                    let (item, at) = aim.split_once(':').ok_or_else(usage)?;
                    let item = parse_position(item).ok_or_else(usage)?;
                    return Ok(Action::UseThrow(active, item, parse_cell(at)?));
                }
                // One called in a weapon's shape, `use:spirit_weapon:long_sword` (bare, the rules
                // ask for the shape).
                if let Some(aim) = aim.filter(|_| active.aims_shape()) {
                    if aim.is_empty()
                        || !(aim.chars())
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                    {
                        return Err(format!(
                            "`use:{id}` needs a weapon's id, e.g. `use:{id}:long_sword`"
                        ));
                    }
                    return Ok(Action::Shape(active, aim.to_string()));
                }
                // One that works on a stack the player carries, `use:merchants_eye:c` (bare, the
                // rules ask for the stack).
                if let Some(aim) = aim.filter(|_| active.aims_item()) {
                    let on = parse_position(aim)
                        .ok_or_else(|| format!("`use:{id}` needs a stack, e.g. `use:{id}:c`"))?;
                    return Ok(Action::UseOn(active, on));
                }
                let dir = aim.map(str::parse::<Dir>).transpose()?;
                Ok(Action::Use(active, dir))
            }
            "ascend" => bare(Action::Ascend),
            "descend" => bare(Action::Descend),
            "use_tool" => bare(Action::UseTool),
            "pay" => match arg {
                None => Ok(Action::Pay(None)),
                Some(_) => dir().map(|d| Action::Pay(Some(d))),
            },
            "pickup" | "drop" | "sell" | "identify" => {
                let positions = match arg {
                    None => Positions::default(),
                    Some("") => return Err(format!("`{verb}:` needs letters, e.g. `{verb}:ab`")),
                    Some(text) => text.parse().map_err(|e| format!("`{s}`: {e}"))?,
                };
                Ok(match verb {
                    "pickup" => Action::PickUp(positions),
                    "drop" => Action::Drop(positions),
                    "sell" => Action::Sell(positions),
                    _ => Action::Identify(positions),
                })
            }
            "rest" => match arg {
                Some("dawn") => Ok(Action::Rest(Rest::Dawn)),
                Some("dusk") => Ok(Action::Rest(Rest::Dusk)),
                Some(d) => match d.parse::<Duration>()? {
                    d if d.is_zero() || d > MAX_REST => {
                        Err(format!("`rest:{d}`: rest from 1s up to 24h"))
                    }
                    d => Ok(Action::Rest(Rest::For(d))),
                },
                None => Err("`rest` needs a goal: `rest:1h`, `rest:dawn`, or `rest:dusk`".into()),
            },
            "skip" => match arg.map(str::parse::<Duration>).transpose()? {
                Some(d) if !d.is_zero() => Ok(Action::SkipTime(d)),
                _ => Err("`skip` needs a positive duration, e.g. `skip:1h`".into()),
            },
            "spawn" => bare(Action::Spawn),
            "ally" => {
                let usage = || {
                    "`ally` needs a creature, e.g. `ally:wolf`, `ally:wolf:2`, or `ally:wolf:2:10m`"
                        .to_string()
                };
                let mut parts = arg.ok_or_else(usage)?.split(':');
                let creature = parts.next().filter(|c| !c.is_empty()).ok_or_else(usage)?;
                let count = match parts.next() {
                    None => 1,
                    Some(n) => n
                        .parse::<u8>()
                        .ok()
                        .filter(|n| (1..=crate::world::MAX_DEV_ALLIES).contains(n))
                        .ok_or_else(|| {
                            format!("`ally`: the count is 1 to {}", crate::world::MAX_DEV_ALLIES)
                        })?,
                };
                let lasts = match parts.next() {
                    None => None,
                    Some(d) => Some(d.parse::<Duration>()?).filter(|d| !d.is_zero()),
                };
                if parts.next().is_some() {
                    return Err(usage());
                }
                Ok(Action::Ally {
                    creature: creature.to_string(),
                    count,
                    lasts,
                })
            }
            "meet" => {
                let usage = || {
                    "`meet` needs an encounter entry, e.g. `meet:kobolds` or \
                     `meet:kobolds:ambushed` (drop, met, ambushed, pass)"
                        .to_string()
                };
                let (entry, outcome) = match arg.ok_or_else(usage)?.split_once(':') {
                    Some((entry, o)) => (entry, Some(o)),
                    None => (arg.unwrap_or_default(), None),
                };
                let outcome = outcome
                    .map(|o| crate::world::EncounterForced::from_id(o).ok_or_else(usage))
                    .transpose()?;
                if entry.is_empty() {
                    return Err(usage());
                }
                Ok(Action::Meet {
                    entry: entry.to_string(),
                    outcome,
                })
            }
            "encounter" => arg
                .and_then(crate::world::EncounterChoice::from_id)
                .map(Action::Encounter)
                .ok_or_else(|| {
                    "`encounter` needs a choice: ambush, evade, fight, hail, hunt, or pass".into()
                }),
            "weather" => match arg.and_then(Weather::from_id) {
                Some(w) => Ok(Action::SetWeather(w)),
                None => Err("`weather` needs `clear`, `overcast`, `rain`, or `heavy`".into()),
            },
            "equip" => {
                // The slot id keeps its case; the letter is a letter (or a number).
                let usage = || "`equip` needs a slot and a letter, e.g. `equip:body:b`".to_string();
                let (slot, item) = s
                    .split_once(':')
                    .and_then(|(_, rest)| rest.rsplit_once(':'))
                    .ok_or_else(usage)?;
                match parse_position(item) {
                    Some(item) if !slot.is_empty() && !slot.contains(':') => Ok(Action::Equip {
                        slot: slot.to_string(),
                        item,
                    }),
                    _ => Err(usage()),
                }
            }
            "remove" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => {
                    Ok(Action::Remove(id.to_string()))
                }
                _ => Err("`remove` needs a slot, e.g. `remove:body`".into()),
            },
            "fire" => {
                let at = arg.ok_or_else(|| "`fire` needs a cell, e.g. `fire:12,5`".to_string())?;
                parse_cell(at).map(Action::Fire)
            }
            "intimidate" => {
                let usage = || "`intimidate` needs a cell, e.g. `intimidate:12,5`".to_string();
                parse_cell(arg.ok_or_else(usage)?).map(Action::Intimidate)
            }
            "throw_voice" => {
                let usage = || "`throw_voice` needs a cell, e.g. `throw_voice:12,5`".to_string();
                parse_cell(arg.ok_or_else(usage)?).map(Action::ThrowVoice)
            }
            "order" => {
                let usage = || {
                    "`order` needs an order: `order:follow`, `order:stay`, or `order:hunt:X,Y`"
                        .to_string()
                };
                match arg.map(|a| a.split_once(':')) {
                    Some(None) if arg == Some("follow") => Ok(Action::Order(Command::Heel)),
                    Some(None) if arg == Some("stay") => Ok(Action::Order(Command::Hold)),
                    Some(Some(("hunt", at))) => {
                        parse_cell(at).map(|at| Action::Order(Command::SetOn(at)))
                    }
                    _ => Err(usage()),
                }
            }
            "eat" => {
                let usage = || "`eat` needs a letter, e.g. `eat:b` or `eat:floor:a`".to_string();
                let (floor, item) = match arg.map(|a| a.split_once(':')) {
                    Some(Some(("floor", item))) => (true, item),
                    Some(None) => (false, arg.unwrap_or_default()),
                    _ => return Err(usage()),
                };
                let i = parse_position(item).ok_or_else(usage)?;
                Ok(Action::Eat(if floor {
                    Meal::Floor(i)
                } else {
                    Meal::Pack(i)
                }))
            }
            "read" => {
                let usage = || {
                    "`read` needs a letter, then a cell to aim at or a letter to read it on, e.g. \
                     `read:b:12,5` or `read:b:c`"
                        .to_string()
                };
                let arg = arg.ok_or_else(usage)?;
                let (item, target) = match arg.split_once(':') {
                    Some((item, target)) => (item, Some(target)),
                    None => (arg, None),
                };
                let target = match target {
                    None => None,
                    Some(cell) if cell.contains(',') => {
                        Some(ReadTarget::Cell(parse_cell(cell).map_err(|_| usage())?))
                    }
                    Some(i) => Some(ReadTarget::Item(parse_position(i).ok_or_else(usage)?)),
                };
                Ok(Action::Read {
                    item: parse_position(item).ok_or_else(usage)?,
                    target,
                })
            }
            "scribe" => {
                let usage = || "`scribe` needs a spell's id, e.g. `scribe:fireball`".to_string();
                let spell = arg.filter(|s| {
                    !s.is_empty()
                        && (s.chars())
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                });
                Ok(Action::Scribe {
                    spell: spell.ok_or_else(usage)?.to_string(),
                })
            }
            "throw" => {
                let usage = || "`throw` needs a letter and a cell, e.g. `throw:c:12,5`".to_string();
                let (item, at) = arg.and_then(|a| a.split_once(':')).ok_or_else(usage)?;
                Ok(Action::Throw {
                    item: parse_position(item).ok_or_else(usage)?,
                    at: parse_cell(at).map_err(|_| usage())?,
                })
            }
            "stance" => arg
                .ok_or_else(|| "`stance` needs a stance, e.g. `stance:coward`".to_string())?
                .parse()
                .map(Action::SetStance),
            "talent" => {
                // Ids keep their case.
                let usage = || {
                    "`talent` needs a talent id, e.g. `talent:hardy` or \
                     `talent:weapon_focus:long_blades`"
                        .to_string()
                };
                let rest = s.split_once(':').map(|(_, rest)| rest).ok_or_else(usage)?;
                let (id, choice) = match rest.split_once(':') {
                    Some((id, choice)) => (id, Some(choice)),
                    None => (rest, None),
                };
                let bad = |t: &str| {
                    t.is_empty() || t.contains([':', '@']) || t.contains(char::is_whitespace)
                };
                if bad(id) || choice.is_some_and(bad) {
                    return Err(usage());
                }
                Ok(Action::Talent {
                    id: id.to_string(),
                    choice: choice.map(str::to_string),
                })
            }
            "advance" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => {
                    Ok(Action::Advance(id.to_string()))
                }
                _ => Err("`advance` needs a skill id, e.g. `advance:stealth`".into()),
            },
            "levelup" => match arg {
                Some("auto") => Ok(Action::LevelUpAuto),
                _ => Err("`levelup` takes `auto`: `levelup:auto`".into()),
            },
            "rank" => {
                let rest = s.split_once(':').map_or("", |(_, rest)| rest);
                match rest.split_once(':') {
                    Some((skill, rank)) if !skill.is_empty() => match rank.parse::<u8>() {
                        Ok(rank) if rank <= crate::content::MAX_RANK => Ok(Action::SetRank {
                            skill: skill.to_string(),
                            rank,
                        }),
                        _ => Err("`rank` is 0 to 10, e.g. `rank:long_blades:5`".into()),
                    },
                    _ => Err(
                        "`rank` needs a weapon skill and a rank, e.g. `rank:long_blades:5`".into(),
                    ),
                }
            }
            "xp" => match arg.and_then(|a| a.parse::<u32>().ok()) {
                Some(xp) if xp > 0 => Ok(Action::GrantXp(xp)),
                _ => Err("`xp` needs a number of experience points, e.g. `xp:2500`".into()),
            },
            "call" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => Ok(Action::Call(id.to_string())),
                _ => Err("`call` needs a prestige class id, e.g. `call:hunter`".into()),
            },
            "vision" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() => Ok(Action::SetVision(id.to_string())),
                _ => Err("`vision` needs a profile id, e.g. `vision:darkvision`".into()),
            },
            "effect" => {
                // The effect id keeps its case.
                let usage = || {
                    "`effect` needs an effect and a cell, e.g. `effect:fireball@12,5` or \
                     `effect:lightning_bolt@3,4>12,4`"
                        .to_string()
                };
                let (effect, cells) = (s.split_once(':'))
                    .and_then(|(_, rest)| rest.split_once('@'))
                    .ok_or_else(usage)?;
                let bad = |c: char| matches!(c, ':' | '@' | '>') || c.is_whitespace();
                if effect.is_empty() || effect.contains(bad) {
                    return Err(usage());
                }
                let (at, to) = match cells.split_once('>') {
                    Some((at, to)) => (at, Some(to)),
                    None => (cells, None),
                };
                Ok(Action::Effect {
                    effect: effect.to_string(),
                    at: parse_cell(at).map_err(|_| usage())?,
                    to: to.map(parse_cell).transpose().map_err(|_| usage())?,
                })
            }
            "afflict" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => {
                    Ok(Action::Afflict(id.to_string()))
                }
                _ => Err("`afflict` needs a status id, e.g. `afflict:bleeding`".into()),
            },
            "drain" => match arg.map(str::parse::<u16>) {
                Some(Ok(amount)) if amount > 0 => Ok(Action::Drain(amount)),
                _ => Err("`drain` needs an amount of mana, 1 or more, e.g. `drain:5`".into()),
            },
            "resource" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => {
                    Ok(Action::Resource(id.to_string()))
                }
                _ => Err("`resource` needs a resource id, e.g. `resource:rage`".into()),
            },
            "infect" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => {
                    Ok(Action::Infect(id.to_string()))
                }
                _ => Err("`infect` needs a disease id, e.g. `infect:filth_fever`".into()),
            },
            "reveal" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => {
                    Ok(Action::Reveal(id.to_string()))
                }
                _ => Err("`reveal` needs a hidden location id, e.g. `reveal:fennmynster`".into()),
            },
            "lure" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => Ok(Action::Lure(id.to_string())),
                _ => Err("`lure` needs a lure quest's id, e.g. `lure:crystal_guardian`".into()),
            },
            // A tag's target keeps its own colon (`quest_roll:hunt:tag:goblin`).
            "quest_roll" => match arg.and_then(|a| a.split_once(':')) {
                Some((quest, target))
                    if !quest.is_empty()
                        && !target.is_empty()
                        && (!target.contains(':') || target.starts_with("tag:")) =>
                {
                    Ok(Action::QuestRoll {
                        quest: quest.to_string(),
                        target: target.to_string(),
                    })
                }
                _ => Err(
                    "`quest_roll` needs a quest and a target, e.g. `quest_roll:ballad:rat`".into(),
                ),
            },
            "freeze" | "thaw" => {
                let usage = || format!("`{verb}` needs a cell and a radius, e.g. `{verb}:12,5,3`");
                let (cell, radius) = arg.and_then(|a| a.rsplit_once(',')).ok_or_else(usage)?;
                let at = parse_cell(cell).map_err(|_| usage())?;
                let radius = match radius.trim().parse::<i32>() {
                    Ok(r) if (0..=MAX_BURST).contains(&r) => r,
                    _ => return Err(format!("`{s}`: the radius is 0 to {MAX_BURST}")),
                };
                Ok(if verb == "freeze" {
                    Action::Freeze { at, radius }
                } else {
                    Action::Thaw { at, radius }
                })
            }
            "zone" => {
                let usage = "`zone` needs a kind, a cell, and a radius, e.g. `zone:fire@12,5,3`";
                let (kind, place) = arg.and_then(|a| a.split_once('@')).ok_or(usage)?;
                let kind = (ZoneKind::ALL.iter())
                    .find(|k| k.id() == kind)
                    .copied()
                    .ok_or_else(|| format!("`{s}`: no zone kind `{kind}`"))?;
                let (cell, radius) = place.rsplit_once(',').ok_or(usage)?;
                let at = parse_cell(cell).map_err(|_| usage)?;
                let radius = match radius.trim().parse::<i32>() {
                    Ok(r) if (0..=MAX_BURST).contains(&r) => r,
                    _ => return Err(format!("`{s}`: the radius is 0 to {MAX_BURST}")),
                };
                Ok(Action::Zone { kind, at, radius })
            }
            _ => Err(format!(
                "unknown action `{s}` (expected move:DIR, wait, open:DIR, close:DIR, kick:DIR, \
                 pick:DIR, ascend, descend, use_tool, rest:1h|dawn|dusk, pickup[:LETTERS], \
                 drop[:LETTERS], \
                 sell[:LETTERS], pay[:DIR], equip:SLOT:LETTER, remove:SLOT, bash:DIR, fire:X,Y, \
                 throw:LETTER:X,Y, dig:DIR, eat:LETTER, eat:floor:LETTER, \
                 read:LETTER[:X,Y|:LETTER], chat:DIR, \
                 ask:DIR:TOPIC, steal:DIR, tame:DIR, throw_voice:X,Y, pray, stance:NAME, vision:ID, skip:DURATION, \
                 weather:clear|overcast|rain|heavy, effect:ID@X,Y[>X,Y], identify[:LETTERS], \
                 afflict:STATUS, drain:N, infect:DISEASE, reveal:ID, quest_roll:ID:TARGET, \
                 rank:SKILL:N, freeze:X,Y,R, thaw:X,Y,R, zone:KIND@X,Y,R)"
            )),
        }
    }
}

/// A level cell written `X,Y` (`12,5`).
fn parse_cell(text: &str) -> Result<Pos, String> {
    let bad = || format!("`{text}` is no cell; expected X,Y, e.g. `12,5`");
    let (x, y) = text.split_once(',').ok_or_else(bad)?;
    let coord = |v: &str| {
        (v.trim().parse::<i32>().ok())
            .filter(|c| c.abs() <= MOST_COORDINATE)
            .ok_or_else(bad)
    };
    Ok(Pos::new(coord(x)?, coord(y)?))
}

/// The farthest a cell of action text lies from 0 on either axis: far past any level, and near
/// enough that the cell arithmetic (`Pos::dist2`, lines, sight) never overflows (#1569).
pub const MOST_COORDINATE: i32 = 10_000;

/// The most times `ACTION*N` repeats an action ([`parse_actions`]).
pub const MOST_REPEATS: usize = 100_000;

/// Parses whitespace-separated actions, e.g. `"move:E move:SE wait"`; `ACTION*N` repeats an
/// action, e.g. `"move:N*3 wait*10"`, at most [`MOST_REPEATS`] times.
pub fn parse_actions(text: &str) -> Result<Vec<Action>, String> {
    let mut actions = Vec::new();
    for word in text.split_whitespace() {
        let (action, count) = match word.split_once('*') {
            Some((action, n)) => match n.parse::<usize>() {
                Ok(n) if n <= MOST_REPEATS => (action, n),
                Ok(_) => return Err(format!("`{word}`: repeats at most {MOST_REPEATS} times")),
                Err(e) => return Err(format!("`{word}`: {e}")),
            },
            None => (word, 1),
        };
        let action: Action = action.parse()?;
        actions.extend(std::iter::repeat_n(action, count));
    }
    Ok(actions)
}

/// Formats actions as whitespace-separated text; inverse of [`parse_actions`].
pub fn format_actions(actions: &[Action]) -> String {
    let words: Vec<String> = actions.iter().map(ToString::to_string).collect();
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_are_a_set_written_as_letters_or_numbers() {
        let mut l = Positions::of([3, 0, 3]);
        assert_eq!((l.len(), l.to_string()), (2, "ad".to_string()));
        assert!(l.contains(3) && !l.contains(1));
        l.remove(3);
        l.insert(25);
        assert_eq!(l.iter().collect::<Vec<_>>(), [0, 25]);
        assert_eq!("za".parse::<Positions>(), Ok(Positions::of([0, 25])));
        // Past `z` there are no letters: every position is a number then, and both forms (and a
        // mix of them) parse.
        l.insert(30);
        assert_eq!(l.to_string(), "0,25,30");
        for text in ["0,25,30", "az,30", "30,a,25", "za,30,30"] {
            assert_eq!(text.parse::<Positions>(), Ok(l.clone()), "{text}");
        }
        for bad in ["", "a,", ",b", "A", "-1", "+3", "3x", "a b"] {
            assert!(bad.parse::<Positions>().is_err(), "{bad}");
        }
        assert_eq!(letter(0), Some('a'));
        assert_eq!(letter(25), Some('z'));
        assert_eq!(letter(26), None);
        assert_eq!(
            (position_text(2), position_text(26)),
            ("c".into(), "26".into())
        );
        assert!(Positions::default().is_empty());
    }

    #[test]
    fn text_round_trips() {
        let mut all = vec![
            Action::Wait,
            Action::Ascend,
            Action::Descend,
            Action::UseTool,
            Action::ThrowVoice(Pos::new(12, 5)),
            Action::ThrowVoice(Pos::new(0, 40)),
            Action::Intimidate(Pos::new(12, 5)),
            Action::SetStance(CombatStance::Aggressive),
            Action::SetStance(CombatStance::Coward),
            Action::SetVision("low_light".into()),
            Action::SetVision("Mixed_Case".into()),
            Action::Rest(Rest::For(Duration::hours(1))),
            Action::Rest(Rest::For(Duration::minutes(90))),
            Action::Rest(Rest::Dawn),
            Action::Rest(Rest::Dusk),
            Action::SkipTime(Duration::hours(1)),
            Action::SkipTime(Duration::seconds(30)),
            Action::Spawn,
            Action::Order(Command::Heel),
            Action::Order(Command::Hold),
            Action::Order(Command::SetOn(Pos::new(12, 5))),
            Action::Ally {
                creature: "wolf".into(),
                count: 1,
                lasts: None,
            },
            Action::Ally {
                creature: "wolf".into(),
                count: 3,
                lasts: None,
            },
            Action::Ally {
                creature: "bear".into(),
                count: 2,
                lasts: Some(Duration::minutes(10)),
            },
            Action::PickUp(Positions::default()),
            Action::PickUp(Positions::of([0])),
            Action::PickUp(Positions::of([0, 2, 25])),
            Action::Drop(Positions::default()),
            Action::Drop(Positions::of([1])),
            Action::Drop(Positions::of(0..LETTERS)),
            Action::Drop(Positions::of([0, 26, 140])),
            Action::PickUp(Positions::of([27])),
            Action::Sell(Positions::default()),
            Action::Sell(Positions::of([0, 3])),
            Action::Pay(None),
            Action::Pay(Some(Dir::W)),
            Action::Equip {
                slot: "body".into(),
                item: 1,
            },
            Action::Equip {
                slot: "Left_Ring".into(),
                item: 25,
            },
            Action::Remove("main_hand".into()),
            Action::Fire(Pos::new(12, 5)),
            Action::Fire(Pos::new(0, 40)),
            Action::Throw {
                item: 2,
                at: Pos::new(7, 3),
            },
            Action::Effect {
                effect: "fireball".into(),
                at: Pos::new(12, 5),
                to: None,
            },
            Action::Effect {
                effect: "Lightning_Bolt".into(),
                at: Pos::new(-3, 4),
                to: Some(Pos::new(12, 0)),
            },
            Action::Identify(Positions::default()),
            Action::Identify(Positions::of([0, 3])),
            Action::Ask {
                dir: Dir::NW,
                topic: "the_rooms".into(),
            },
            Action::Eat(Meal::Pack(1)),
            Action::Eat(Meal::Floor(0)),
            Action::Eat(Meal::Floor(25)),
            Action::Eat(Meal::Pack(26)),
            Action::Eat(Meal::Floor(31)),
            Action::Read {
                item: 1,
                target: None,
            },
            Action::Read {
                item: 0,
                target: Some(ReadTarget::Cell(Pos::new(12, 5))),
            },
            Action::Read {
                item: 3,
                target: Some(ReadTarget::Item(2)),
            },
            Action::Read {
                item: 27,
                target: Some(ReadTarget::Item(30)),
            },
            Action::Scribe {
                spell: "fireball".into(),
            },
            Action::Scribe {
                spell: "magic_missile".into(),
            },
            Action::Throw {
                item: 40,
                at: Pos::new(1, 2),
            },
            Action::Equip {
                slot: "body".into(),
                item: 26,
            },
            Action::Identify(Positions::of([26])),
            Action::Sell(Positions::of([3, 99])),
            Action::Afflict("bleeding".into()),
            Action::Afflict("Odd_Case".into()),
            Action::Drain(1),
            Action::Drain(u16::MAX),
            Action::Resource("rage".into()),
            Action::Resource("Odd_Case".into()),
            Action::Infect("filth_fever".into()),
            Action::Infect("Odd_Case".into()),
            Action::Reveal("old_barrow".into()),
            Action::Lure("crystal_guardian".into()),
            Action::Talent {
                id: "hardy".into(),
                choice: None,
            },
            Action::Talent {
                id: "weapon_focus".into(),
                choice: Some("long_blades".into()),
            },
            Action::Talent {
                id: "Odd_Case".into(),
                choice: Some("Mixed".into()),
            },
            Action::Advance("stealth".into()),
            Action::Pray,
            Action::Forage,
            Action::Craft {
                recipe: "temper".into(),
                picks: Vec::new(),
            },
            Action::Craft {
                recipe: "long_sword".into(),
                picks: vec![Some(2), Some(5)],
            },
            Action::Craft {
                recipe: "dagger".into(),
                picks: vec![Some(0), None, Some(30)],
            },
            Action::Decipher { item: 1 },
            Action::Temper { item: 0 },
            Action::Temper { item: 27 },
            Action::Whittle { item: 3 },
            Action::Salvage { item: 2 },
            Action::Rig {
                rig: "tripwire".into(),
                dir: Some(Dir::E),
            },
            Action::Rig {
                rig: "pitfall".into(),
                dir: None,
            },
            Action::Give {
                item: 1,
                ally: None,
            },
            Action::Give {
                item: 27,
                ally: Some(Dir::E),
            },
            Action::Heal {
                what: None,
                ally: None,
            },
            Action::Heal {
                what: Some("bleeding".into()),
                ally: None,
            },
            Action::Heal {
                what: Some("bleeding".into()),
                ally: Some(Dir::E),
            },
            Action::Drink,
            Action::UseTalent(TalentActive::SecondWind),
            Action::UseTalent(TalentActive::BloodMagic),
            Action::Use(ClassActive::Rage, None),
            Action::Use(ClassActive::Smite, Some(Dir::SW)),
            Action::UseAt(ClassActive::Volley, Pos::new(12, 5)),
            Action::UseOn(ClassActive::MerchantsEye, 2),
            Action::UseThrow(ClassActive::ThrownExploit, 2, Pos::new(12, 5)),
            Action::UseThrow(ClassActive::ThrownExploit, 30, Pos::new(3, 4)),
            Action::UseOn(ClassActive::MerchantsEye, 30),
            Action::Shape(ClassActive::SpiritWeapon, "long_sword".into()),
            Action::Shape(ClassActive::SpiritWeapon, "great_axe".into()),
            Action::Gift(SignActive::Blaze, None),
            Action::Cast("blessing".into(), None),
            Action::Cast("arcane_bolt".into(), Some(Pos::new(12, 5))),
            Action::Act("lunge".into(), Dir::E),
            Action::Cast("shield_wall".into(), None),
            Action::CastOn("identify".into(), 2),
            Action::CastOn("identify".into(), 30),
            Action::LevelUpAuto,
            Action::GrantXp(2500),
            Action::Call("hunter".into()),
            Action::SetRank {
                skill: "twin_blades".into(),
                rank: 4,
            },
            Action::Freeze {
                at: Pos::new(40, 12),
                radius: 3,
            },
            Action::Thaw {
                at: Pos::new(0, 7),
                radius: 0,
            },
            Action::Zone {
                kind: ZoneKind::Fouled,
                at: Pos::new(40, 12),
                radius: 2,
            },
            Action::Ask {
                dir: Dir::S,
                topic: "quest-rats".into(),
            },
            Action::Search(1),
            Action::Search(10),
            Action::Search(MAX_SEARCH),
        ];
        all.extend(Weather::ALL.map(Action::SetWeather));
        for d in Dir::ALL {
            all.extend([
                Action::Move(d),
                Action::Run(d),
                Action::Open(d),
                Action::Close(d),
                Action::Chat(d),
                Action::Steal(d),
                Action::Feint(d),
                Action::Tame(d),
                Action::Kick(d),
                Action::PickLock(d),
                Action::Bash(d),
                Action::Dig(d),
                Action::Disarm(d),
                Action::Listen(d),
                Action::Gather(d),
                Action::Wedge(d),
            ]);
        }
        assert_eq!(parse_actions(&format_actions(&all)), Ok(all));
    }

    #[test]
    fn parses_case_insensitively_and_rejects_garbage() {
        assert_eq!(
            parse_actions("MOVE:ne  Wait"),
            Ok(vec![Action::Move(Dir::NE), Action::Wait])
        );
        assert_eq!(parse_actions(""), Ok(vec![]));
        assert!(parse_actions("move").is_err());
        assert!(parse_actions("move:up").is_err());
        assert_eq!(parse_actions("KICK:sw"), Ok(vec![Action::Kick(Dir::SW)]));
        assert!(parse_actions("kick").is_err());
        assert_eq!(parse_actions("PICK:e"), Ok(vec![Action::PickLock(Dir::E)]));
        assert!(parse_actions("pick").is_err());
        assert!(parse_actions("wait:E").is_err());
        assert_eq!(parse_actions("BASH:sw"), Ok(vec![Action::Bash(Dir::SW)]));
        assert!(parse_actions("bash").is_err());
        assert_eq!(parse_actions("Steal:N"), Ok(vec![Action::Steal(Dir::N)]));
        assert_eq!(parse_actions("feint:E"), Ok(vec![Action::Feint(Dir::E)]));
        assert!(parse_actions("feint").is_err());
        assert!(parse_actions("steal").is_err());
        assert_eq!(parse_actions("TAME:sw"), Ok(vec![Action::Tame(Dir::SW)]));
        assert!(parse_actions("tame").is_err());
        assert_eq!(
            parse_actions("DISARM:ne"),
            Ok(vec![Action::Disarm(Dir::NE)])
        );
        assert!(parse_actions("disarm").is_err());
        // A talent's active is used as every ability is (#644): no bare verb, no argument.
        assert_eq!(
            parse_actions("use:second_wind use:blood_magic"),
            Ok(vec![
                Action::UseTalent(TalentActive::SecondWind),
                Action::UseTalent(TalentActive::BloodMagic),
            ])
        );
        assert!(parse_actions("second_wind").is_err());
        assert!(parse_actions("blood_magic").is_err());
        assert!(parse_actions("use:second_wind:E").is_err());
        assert_eq!(
            parse_actions("SEARCH search:1 search:10"),
            Ok(vec![
                Action::Search(1),
                Action::Search(1),
                Action::Search(10)
            ])
        );
        for bad in ["search:0", "search:101", "search:E", "search:"] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
        assert!(parse_actions("dance").is_err());
        assert_eq!(
            parse_actions("move:N*3 wait*0 wait"),
            Ok(vec![
                Action::Move(Dir::N),
                Action::Move(Dir::N),
                Action::Move(Dir::N),
                Action::Wait
            ])
        );
        assert!(parse_actions("move:N*x").is_err());
        assert!(parse_actions("wait*").is_err());
        // A repeat count is capped: a huge one is an error, never an abort (#1598).
        assert_eq!(parse_actions("wait*100000").map(|a| a.len()), Ok(100_000));
        assert!(parse_actions("wait*100001").is_err());
        assert!(parse_actions("wait*99999999999").is_err());
        assert_eq!(
            parse_actions("USE_TOOL vision:uv_sight"),
            Ok(vec![Action::UseTool, Action::SetVision("uv_sight".into())])
        );
        assert!(parse_actions("vision").is_err());
        assert!(parse_actions("vision:").is_err());
        assert!(parse_actions("use_tool:E").is_err());
        assert_eq!(
            parse_actions("throw_voice:3,-4"),
            Ok(vec![Action::ThrowVoice(Pos::new(3, -4))])
        );
        assert_eq!(
            parse_actions("intimidate:3,4"),
            Ok(vec![Action::Intimidate(Pos::new(3, 4))])
        );
        assert!(parse_actions("intimidate").is_err());
        assert!(parse_actions("intimidate:E").is_err());
        for bad in [
            "lantern",
            "hood",
            "throw_voice",
            "throw_voice:E",
            "throw_voice:3",
        ] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
        assert_eq!(
            parse_actions("FREEZE:12,5,4 thaw:3,4,0"),
            Ok(vec![
                Action::Freeze {
                    at: Pos::new(12, 5),
                    radius: 4
                },
                Action::Thaw {
                    at: Pos::new(3, 4),
                    radius: 0
                },
            ])
        );
        for bad in [
            "freeze",
            "freeze:12,5",
            "freeze:12,5,21",
            "thaw:1,2,-1",
            "thaw:a,b,c",
        ] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
        assert_eq!(
            parse_actions("REST:Dawn rest:1h skip:2H"),
            Ok(vec![
                Action::Rest(Rest::Dawn),
                Action::Rest(Rest::For(Duration::hours(1))),
                Action::SkipTime(Duration::hours(2)),
            ])
        );
        assert_eq!(
            parse_actions("PICKUP:CA drop pickup drop:b"),
            Ok(vec![
                Action::PickUp(Positions::of([0, 2])),
                Action::Drop(Positions::default()),
                Action::PickUp(Positions::default()),
                Action::Drop(Positions::of([1])),
            ])
        );
        assert_eq!(
            Action::PickUp(Positions::of([2, 0])).to_string(),
            "pickup:ac"
        );
        // Any position as a number: the same stack as its letter, and past `z` too.
        assert_eq!(parse_actions("drop:2,0"), parse_actions("drop:ac"));
        assert_eq!(
            parse_actions("eat:1 equip:body:1"),
            parse_actions("eat:b equip:body:b")
        );
        assert_eq!(
            parse_actions("EAT:Floor:27 throw:30:4,5 drop:26"),
            Ok(vec![
                Action::Eat(Meal::Floor(27)),
                Action::Throw {
                    item: 30,
                    at: Pos::new(4, 5)
                },
                Action::Drop(Positions::of([26])),
            ])
        );
        for bad in [
            "eat:",
            "eat:ab",
            "throw:-1:4,5",
            "equip:body:",
            "drop:a,,b",
            "pickup:1x",
        ] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
        assert_eq!(
            parse_actions("WEATHER:Heavy weather:clear"),
            Ok(vec![
                Action::SetWeather(Weather::HeavyRain),
                Action::SetWeather(Weather::Clear)
            ])
        );
        assert_eq!(
            parse_actions("EQUIP:body:B remove:Missiles"),
            Ok(vec![
                Action::Equip {
                    slot: "body".into(),
                    item: 1
                },
                Action::Remove("Missiles".into()),
            ])
        );
        assert_eq!(
            Action::Equip {
                slot: "head".into(),
                item: 2
            }
            .to_string(),
            "equip:head:c"
        );
        assert_eq!(
            parse_actions("CHAT:ne ask:E:Rooms"),
            Ok(vec![
                Action::Chat(Dir::NE),
                Action::Ask {
                    dir: Dir::E,
                    topic: "rooms".into()
                }
            ])
        );
        assert_eq!(
            parse_actions("FIRE:3,4 throw:B:10,2"),
            Ok(vec![
                Action::Fire(Pos::new(3, 4)),
                Action::Throw {
                    item: 1,
                    at: Pos::new(10, 2)
                },
            ])
        );
        assert_eq!(
            parse_actions("EFFECT:Fireball@3,4 effect:lightning_bolt@1,2>9,2"),
            Ok(vec![
                Action::Effect {
                    effect: "Fireball".into(),
                    at: Pos::new(3, 4),
                    to: None
                },
                Action::Effect {
                    effect: "lightning_bolt".into(),
                    at: Pos::new(1, 2),
                    to: Some(Pos::new(9, 2))
                },
            ])
        );
        assert_eq!(
            Action::Effect {
                effect: "lightning_bolt".into(),
                at: Pos::new(1, 2),
                to: Some(Pos::new(9, 2)),
            }
            .to_string(),
            "effect:lightning_bolt@1,2>9,2"
        );
        for bad in [
            "effect",
            "effect:",
            "effect:fireball",
            "effect:fireball@",
            "effect:@3,4",
            "effect:fireball@3",
            "effect:fireball@3,4>",
            "effect:fireball@3,4>5",
            "effect:fire:ball@3,4",
            "effect:fireball@3,4>5,6>7,8",
        ] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
        assert_eq!(
            parse_actions("EAT:B eat:Floor:a"),
            Ok(vec![
                Action::Eat(Meal::Pack(1)),
                Action::Eat(Meal::Floor(0))
            ])
        );
        for bad in [
            "eat",
            "eat:",
            "eat:ab",
            "eat:floor",
            "eat:floor:",
            "eat:floor:ab",
            "eat:pack:a",
            "read",
            "read:",
            "read:ab",
            "read:a:",
            "read:a:bc",
            "read:a:1,b",
            "read:a:1,2,3",
            "scribe",
            "scribe:",
            "scribe:fire-ball",
            "scribe:a:b",
            "fire",
            "fire:",
            "fire:3",
            "fire:a,b",
            "throw",
            "throw:3,4",
            "throw:a",
            "throw:ab:3,4",
            "equip",
            "equip:body",
            "equip:body:",
            "equip::b",
            "equip:body:ab",
            "equip:a:b:c",
            "remove",
            "remove:",
            "remove:a:b",
            "weather",
            "weather:snow",
            "afflict",
            "afflict:",
            "afflict:a:b",
            "drain",
            "drain:",
            "drain:0",
            "drain:-1",
            "drain:x",
            "drain:65536",
            "resource",
            "resource:",
            "resource:a:b",
            "infect",
            "infect:",
            "infect:a:b",
            "pickup:",
            "pickup:a1",
            "pay:Q",
            "sell:",
            "sell:A1",
            "drop:A-",
            "drop:",
            "rest",
            "rest:noon",
            "rest:0m",
            "rest:25h",
            "skip",
            "skip:0s",
            "skip:x",
            "spawn:3",
            "chat",
            "chat:up",
            "ask",
            "ask:E",
            "ask:E:",
            "ask:up:rooms",
            "ask:E:a:b",
            "dig",
            "dig:",
            "dig:down",
        ] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
    }
}
