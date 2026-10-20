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
`PerceptionView::own_weapon_skills` only: ranks and a bar, **never a benefit**. Tests:
`content/weapon_skill.rs`, `weapon_skills.rs`, `world/weapon_skill_tests.rs`; the wiki's tables
are `xtask/src/wiki/weapon_skills.rs`.

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
