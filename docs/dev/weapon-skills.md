# Weapon skills and fighting styles

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08i-character.md. Keep it current with the code, as CLAUDE.md says.

Weapon skills (spec §8 "Weapon skills"; M22, #134): data is `data/weapon_skills.toml`
(`content/weapon_skill.rs`: `WeaponSkillTable` with `MarkRules`, named need curves, skills
(`WeaponSkillDef`: `id`, `name`, `phrase`, `Role`, curve, `[Benefit; 11]` per rank) and the
symmetric defaults; `SkillId` is an index in file order, in the content fingerprint). Weapons
name theirs (`category` → `ObjectDef::skill`, its role checked against what the weapon is),
shields bash (`bash` → `ObjectDef::bash`), races and classes give starting ranks
(`weapon_skills`, dice; classes `weapon_skill_choice`), creatures fixed ranks. The runtime side
is `weapon_skills.rs`: `WeaponSkills` (a `Training { rank, marks }` per skill on
`Actor::weapon_skills`, saved and hashed), `train`
(marks → a rank, then
`raise`: the defaults rise, their marks reset; no chaining), `from_ranks` (starting ranks,
order-free), `progress`. The rules hook in at one place each: `Actor::attack_skill(kind, with)`
picks the skill of an attack (a blow: the wielded weapon's, else unarmed; `Melee` `with` a
shield: shield bashing; a shot: the launcher's; a throw: thrown; a kick, spec §8 "Doors": unarmed
whatever is wielded, `World::player_kicks`), and `fight.rs::resolve` takes the skills its caller
picked (`AttackSkills`: the weapon skill and the fighting style); `Profile::with(&Benefit)` and
`Actor::melee`/`ranged`/`thrown`/`bash`/`kick` apply to-hit, damage, and threat; `Actor::defence` adds
parry and shield blocking; `World::attack_price` takes `speed` off the cost and
`World::reload_price` `reload` (`World::attack_cost`: with the weapon's `attack_cost` and magic
items' speed; a shield bash costs a blow with the shield, never the wielded weapon's cost);
`World::resolve` (`fight.rs`) earns the player's marks (`practise`; a Shield-credited miss on
the player trains shield blocking; `World::level_of` weighs the opponent by its creature type's
`level`, spec §8 "Experience") and pushes "You feel more skilled ..." into the unsaved
`World::news`, said after the attack's own messages (`say_news`, also at the end of
`perceive` for a block in a creature's turn). Starting ranks roll from
their own stream (`character/weapon-skills/...`), so stat rolls never move. Presentation reads
`PerceptionView::own_weapon_skills` only: ranks and a bar, **never a benefit**. The rank tables (#554, docs/design/combat-progression.md §3, §5): ranks 1 to 5 keep the first
tables' values, ranks 6 to 10 grow by family (light blades, precise, heavy, middling, launchers
and thrown, shield blocking; a pattern for writing the arrays, not a data concept);
`rank_ten_holds_the_family_tables` holds every skill's and style's rank 10 to the design's table
and every array to never less at a higher rank. `MAX_SAVING` is 60 (crossbows' `reload` at rank
10); a saving past 100 % in all (crossbows 10 with Marksman 10) prices the reload at the floor,
`Benefit::saving`'s 1. Tests:
`content/weapon_skill.rs`, `weapon_skills.rs`, `world/weapon_skill_tests.rs`; the wiki's tables
are `xtask/src/wiki/weapon_skills.rs`. What ranks teach (weapon abilities, #561) is
`docs/dev/weapon-abilities.md`: a rank risen says it (`World::ranks_news`, called by `practise`).

Fighting styles (spec §8 "Fighting styles"; M22, #135) are weapon skills of `Role::Style`: rows
after the weapon skills in `data/weapon_skills.toml` with a `grip` (`content/weapon_skill.rs`:
`Grip`, `Style { grip, weapons }`; a style naming no `weapons` takes the rest of its grip,
`WeaponSkillTable::style_for`, one configuration one style), the same ranks, marks, curves, and
`Benefit`s (plus `crit`, `ease_main`/`ease_off`, `bash`; which a grip may use is checked), and
defaults only among styles. **Every rule asks the attack, not a skill**:
`Actor::attack_skills(kind, with)` is the `AttackSkills` of an attack (its weapon skill and its
style, `Actor::style(kind, with)`: blows by `Actor::grip`, the hands' hold; shots `launcher`;
throws `thrown`), and `Actor::attack_benefit(kind, with)` / `benefit_of(skills)` the weapon
skill's benefit `plus` the style's, read by `Actor::melee`/`off_hand_blow`/`ranged`/
`thrown(item)`/`bash`, `Actor::defence` (the hands' style's parry and shield), and
`World::attack_price(id, action, with, skills)` (`World::attack_cost` of what the attack is made
`with`, then the speed of the weapon skill and the style together, then the stance and the
burden; priced as the attack begins, `docs/dev/time.md`); `World::resolve` practises the style
after the weapon skill. A profile adds the style's benefit right after the weapon skill's, before
the affixes and the stance. A kick (`World::player_kicks`, `door.rs`) is made whatever the hands
hold: unarmed fighting alone (`AttackSkills::only`), in no style, so no style speeds it either. Two weapons: a
`light_weapon` object fits every hand slot (`Content::fits`); `Actor::off_hand` is the second of
two weapons in hand; `World::second_blow` (`fight.rs`) is the one place a melee attack's second
blow is decided, on `Cue::First` (its other cues are the free blows, `docs/dev/combat.md`): the
off hand's, `[two_weapons]` penalties eased by `TwoWeapons::eased`; else
Sword and Board's chance of a shield bash after a hit, one gameplay draw only when the chance is
above 0; for the player (`player_attacks`) and creatures (`World::attack`) alike; it is an
attack `with` the off-hand weapon or the shield (`Actor::attack_skill` tells them apart by
`ObjectDef::bash`), silent, and resolved like any blow: its own weapon's skill and affixes
(`Actor::off_hand` gives its inventory position), its own blood, bleeding, and big hits; hunger's
`exert` stays once per action. Classes give `styles`/`style_choice`, rolled from
`character/styles/...` after everything else. Presentation: `PerceptionView::own_styles_in_use`
(the screen's `•`, the dump's `, in use`), never a benefit or penalty. Tests:
`content/weapon_skill.rs` (`styles_are_checked`), `world/style_tests.rs`; wiki tables
`styles.md`, `style-defaults.md`, `style-benefits.md` and the classes' column.

Caster styles (#571, combat progression family): spells that roll to hit
(`SpellData::rolls_to_hit`: a damage spell, a ray, a bolt, or a touch, resolved by the attack
roll) draw on the one skill of `Role::Spell` (`spell_attack`, its own `Section::Spells` on the
screen) and a style of a caster grip (`Grip::RangedSpell`, `TouchSpell`, `ArmouredSpell`;
`Grip::casts`; their styles name no weapons). `Actor::spell_skills(content, touch)` picks them:
armoured first (`Actor::worn_spell_cost` above 0: anything worn with a `spell_cost_percent`),
else touch or ranged; `Actor::attack_skill(Spell, _)` is the spell skill, `Actor::style(Spell,
_)` stays `None` (`styles_in_use` marks no caster style). The one reader is
`world/cast.rs::cast_single` (own casts paid in mana only; a scroll or a wand keeps the bare
roll): `benefit_of(skills).at_distance(distance, table.spell_attacks())` (`[spell_attacks] far`,
default 5: Sniper's `far_to_hit` and `far_threat` from there) goes into `Profile::with` after the
spell's to-hit scales; `World::spell_practised` then practises both against the target's level
and calls `resolve_triggers` with `AttackKind::Spell` (Pinning ray, Staggering touch, Withering
touch, Spell guard; `struck` fires for the one struck as for a blow; `kill` and `big_hit` are not raised for spells, decided). Battle
caster's `armour` (percent off the armour's percent) is read by `Actor::armour_spell_cost`
whatever is cast, and `World::practise_in_armour` (from `pay_cast` on `Pay::Mana`) trains it on
every other spell cast in such armour: `attempt` marks as against the player's own level. No
`speed` for the role or the caster grips (the loader refuses it: a cast's time is the spell's).
New `Benefit` fields: `far_to_hit`, `far_threat`, `armour` (checked per grip like the others).
Creatures cast no spell that rolls to hit (their spells are intents), so no creature data gives
ranks in them yet. Classes: wizard Spell attacks 2 and Sniper 1, druid 1 and Sniper 1, witch and
priest 1 and Toucher 1, paladin 1 and Battle caster 1, bard Spell attacks 1. Talents: none new; Stylist (a
chosen style) may choose a caster style, raising it a rank. Tests:
`content/weapon_skill.rs` (`the_caster_ladder_loads_and_is_checked`),
`world/caster_style_tests.rs`.
