//! Movement and travel (spec §3, §8 "Time"): the player's step ([`crate::action::Action::Move`]),
//! stairs and the ways between levels ([`crate::action::Action::Descend`], `Ascend`), the edge
//! of a local area, arriving on a level (generated on the first visit) and spawning what lives
//! there, special rooms entered, and terrain changing under the light. `world/mod.rs` has only
//! the dispatch lines.

use super::spawn::Spawner;
use super::{
    Actor, ActorId, ActorKind, FreeBlows, LevelState, Mind, Patrol, Shop, Spent, Talk, World,
    WorldError, WorldState,
};
use crate::action::{AttackKind, Event};
use crate::burden;
use crate::content::{
    ActionKind, CombatStance, Content, CreatureSpawn, EdgeExit, LevelKind, LevelTemplate,
    StairsDir, Stance, TerrainId, local_area_id,
};
use crate::equipment::Equipment;
use crate::geom::{Dir, Pos};
use crate::magic::loot::Luck;
use crate::mapgen::local::arrival;
use crate::object::Stack;
use crate::skills::Use;
use std::borrow::Cow;

impl World {
    /// Whether the player can move under their load; if not, says so (spec §8 "Burden").
    pub(super) fn can_move(&mut self) -> bool {
        let movable = self.load_of(self.state.player).penalty().can_move();
        if !movable {
            self.say(burden::MOVE_REFUSED);
        }
        movable
    }

    /// Whether the player can step onto `to` under their load as it would be there: a heavy
    /// hall halves the limit, so one may be unable to go in, yet always able to step out
    /// (spec §7 "Special rooms"). If not, says so.
    fn can_step_to(&mut self, to: Pos) -> bool {
        let player = &self.state.actors[&self.state.player];
        let there = self.state.load_at(&self.content, player, to).penalty();
        if there.can_move() {
            return true;
        }
        match self.load_of(self.state.player).penalty().can_move() {
            true => self.say(burden::HEAVY_REFUSED),
            false => self.say(burden::MOVE_REFUSED),
        }
        false
    }

    /// Changes terrain on the current level and invalidates light masks it may affect. A lock
    /// stays only on a closed door or a shut chest, a door's trap goes with the door (kicked in), and a secret
    /// door is gone with the wall it looked like.
    pub(super) fn set_terrain(&mut self, p: Pos, terrain: TerrainId) {
        let def = self.content.terrain(terrain);
        // A lock stays on a closed door or a shut chest (spec §8 "Chests").
        let (closed_door, door) = (def.is_closed_door() || def.is_shut_chest(), def.is_door());
        let was_door = self.state.current().is_door(&self.content, p);
        self.reshaped += 1;
        let level = self.state.current_mut();
        level.terrain.set(p, terrain);
        if !closed_door {
            level.locks.remove(&p);
        }
        // A trap on a door is part of it (spec §8 "Trapped doors"): it opens and shuts with the
        // door, and is gone when the door is.
        if was_door && !door {
            level.traps.remove(&p);
        }
        // A secret door is the wall it looks like until it is found: whatever becomes of that
        // wall (dug through, blasted, found), the secret is gone.
        level.secrets.remove(&p);
        let level = self.state.current();
        self.lights.terrain_changed(level, &self.content, p);
    }

    /// The player strikes at `target`, which they `seen` or not, in melee, moving against it.
    fn strike_at(&mut self, target: ActorId, seen: bool, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let me = self.state.player();
        let melee = me.attack_skills(&self.content, AttackKind::Melee, None);
        let with = me.weapon_item(&self.content);
        // Priced as the attack begins (every attack is), before the blows and the exertion,
        // which may burn enough to leave the player hungrier.
        let price = self.attack_price(player, ActionKind::Melee, with, melee);
        self.player_attacks(target, seen, events);
        self.exert(ActionKind::Melee);
        Spent::Turn(price)
    }

    pub(super) fn player_moves(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let from = self.player_pos();
        let mut to = from.step(dir);
        let level = self.state.current();
        let Some(&terrain) = level.terrain.get(to) else {
            if let Some(exit) = level.exit.clone() {
                if !self.can_move() {
                    return Spent::Nothing;
                }
                return self.leave_by_edge(exit, events);
            }
            self.say("You can't go that way.");
            return Spent::Nothing;
        };
        let def = self.content.terrain(terrain);
        // A hostile incorporeal creature the player sees inside solid terrain there
        // (`incorporeal.rs`): they strike at it. Unseen, the terrain is in the way as ever.
        if !def.walkable
            && let Some(other) = self.state.actor_at(&level.id, to)
            && self.perception.sees_actors_at(to)
            && self.stance(self.state.player, other.id) == Stance::Hostile
        {
            return self.strike_at(other.id, true, events);
        }
        if def.is_closed_door() || def.is_shut_chest() {
            return self.open_door(to, events);
        }
        if !def.walkable {
            let name = def.name.clone();
            self.say(format!("The {name} blocks your way."));
            return Spent::Nothing;
        }
        // Someone there: the player attacks them if they are hostile (by their stance toward the
        // player, which the player knows from what they are), else swaps places with them
        // unless they will not make way. Only what the player sees of them is said.
        let mut swap = None;
        if let Some(other) = self.state.actor_at(&level.id, to) {
            let target = other.id;
            let seen = self.perception.sees_actors_at(to);
            let player = self.state.player;
            if self.stance(player, target) == Stance::Hostile {
                return self.strike_at(target, seen, events);
            }
            if let Some(why) = self.refusal(target) {
                self.refused(target, why, seen);
                return Spent::Nothing;
            }
            swap = Some((target, seen));
        } else if let Some(price) = self.meets_glass(to) {
            // An image in a hall of mirrors (`mirrors.rs`).
            return Spent::Turn(price);
        }
        if !self.can_step_to(to) {
            return Spent::Nothing;
        }
        // Out of a shop's door owing anything: the keeper turns the player back (spec §7
        // "Shops"), a step the other way, or none at all if the shop floor is full; whoever
        // stood in the door stays there.
        // Turned back, the player did not choose the cell they step onto: no sidestepping a
        // trap there (`sidestep.rs`).
        let mut willing = true;
        if let Some(back) = self.turned_back(from, to) {
            to = back;
            swap = None;
            willing = false;
        }
        let player = self.state.player;
        let fleeing = self.flees(player, to);
        // Stepping away from beside a hostile opportunist, they are struck as they turn to go
        // (`free_blow.rs`), and may not live to take the step.
        if !self.step_away(player, to, events) {
            let onto = self.state.current().terrain[to];
            return Spent::Turn(self.step_price(player, self.content.terrain(onto), fleeing));
        }
        // Ice there may give way (then it is a step into the water); climbing out onto it from
        // the water takes longer.
        let footing = self.tread(player, from, to);
        let onto = self.state.current().terrain[to];
        let step = self.step_price(player, self.content.terrain(onto), fleeing);
        // Slip away (spec §8 "Birth sign"): a step out of reach, cheaper.
        let step = self.slipped(player, to, step) + self.tread_price(player, footing);
        let overworld = self.state.current().kind == LevelKind::Overworld;
        self.state.overworld_step = overworld.then_some(dir);
        if to == from {
            // Turned back with nowhere to go: the step's time, spent at the door.
            return Spent::Turn(step);
        }
        if let Some(actor) = self.state.actors.get_mut(&player) {
            actor.pos = to;
        }
        // Moving, they let go of whatever they held (Drowning grip, `lucky.rs`).
        self.let_go(player, None);
        events.push(Event::ActorMoved {
            actor: player,
            from,
            to,
        });
        self.after_tread(player, to, footing, events);
        self.make_step_noise(player, to, fleeing);
        // Wading through a marsh may give a disease, by how long the step takes.
        let speed = self.state.player().speed(&self.content);
        self.wade(
            onto,
            u64::try_from(step).unwrap_or(0) / u64::from(speed.max(1)),
        );
        if let Some((other, seen)) = swap {
            self.swap_places(other, to, from, seen, events);
        }
        // A journey on the overworld trains survival.
        if overworld && self.trains(Use::Travel) {
            self.train(Use::Travel);
        }
        // Entering a shop the keeper greets the player; leaving it owing is theft.
        self.shop_step(from, to);
        let level = self.state.current();
        if let Some(stairs) = level.stairs_at(to) {
            // A location goes by its name, even while it leads nowhere yet.
            let named =
                (stairs.name.clone()).or_else(|| Some(self.place_name(stairs.to.as_ref()?)));
            let text = match (level.kind, stairs.dir, named) {
                (LevelKind::Overworld, StairsDir::Down, Some(name)) => {
                    format!("The way into {name} is here.")
                }
                (_, StairsDir::Down, _) => "There is a staircase down here.".into(),
                (_, StairsDir::Up, _) => "There is a staircase up here.".into(),
            };
            self.say(text);
        }
        // A feature of the ground there ("There is an altar here."), before what lies on it.
        self.announce_feature();
        // Goods for sale say their price (spec §7 "Shops").
        let text = match self.state.current().objects_at(to) {
            [] => None,
            [one] => Some(match self.asking_price(to, one) {
                Some(price) => format!(
                    "You see here {} (for sale, {price} gold).",
                    self.lore().describe(one)
                ),
                None => format!("You see here {}.", self.lore().describe(one)),
            }),
            _ => Some("There are several objects here.".to_string()),
        };
        if let Some(text) = text {
            self.say(text);
        }
        // A trap there goes off (spec §8 "Traps"), unless the player knew of it and stepped
        // past it of their own will (`sidestep.rs`); a pit costs the next action too.
        let climb = if willing {
            self.step_willingly(to, events)
        } else {
            self.step_onto_trap(to, player, events)
        };
        // A collapsed floor may give way under them (`collapse.rs`).
        self.collapse_step(to, events);
        // Stepping up beside a hostile that braces or shoots point-blank (weapon abilities,
        // `weapon_ability.rs`), they are met as they come, as a creature is (`free_blow.rs`).
        if self.state.death.is_none() {
            self.step_up(player, from, events);
        }
        Spent::Turn(step + climb)
    }

    /// What a level is called in messages; a dungeon branch goes by the branch's name.
    pub(super) fn place_name(&self, id: &str) -> String {
        if let Some((branch, _)) = self.content.branch_level(id) {
            return branch.name.clone();
        }
        if let Some(level) = self.state.levels.get(id) {
            return level.name.clone();
        }
        self.content
            .level(id)
            .map_or_else(|| id.to_string(), |t| t.name.clone())
    }

    /// Walks off the edge of an outdoor area (a local area, or a hand-made one an overworld
    /// location leads into), back to its overworld cell: like taking stairs. The quick leave
    /// (`quick_leave.rs`) goes the same way.
    pub(super) fn leave_by_edge(&mut self, exit: EdgeExit, events: &mut Vec<Event>) -> Spent {
        let from = self.state.current_level.clone();
        if let Err(err) = self.travel(&exit.to, Some(exit.pos)) {
            self.say(format!("You cannot leave this way ({err})."));
            return Spent::Nothing;
        }
        events.push(Event::LevelChanged { from, to: exit.to });
        let name = self.state.current().name.clone();
        self.say(format!("You return to {name}."));
        self.took(true, ActionKind::Stairs)
    }

    /// `>` on an overworld cell that is not a location: enters the cell's local area, arriving
    /// on the side the player walked onto the cell from (stepping east, on its west side), or at
    /// its start (the middle) if they did not walk there.
    pub(super) fn enter_local_area(&mut self, events: &mut Vec<Event>) -> bool {
        let id = local_area_id(self.player_pos());
        let Some(area) = self.content.local_area(&self.state.campaign, &id) else {
            self.say("There is nothing to explore here.");
            return false;
        };
        let what = self.content.terrain(area.terrain).name.clone();
        if !self.can_move() {
            return false;
        }
        let from = self.state.current_level.clone();
        let side = self.state.overworld_step.map(Dir::opposite);
        let content = self.content.clone();
        let arrival = match self.visit(&id, Some(&from)) {
            Ok(level) => side.and_then(|s| arrival(&level.terrain, &content, s)),
            Err(err) => {
                self.say(format!("You cannot go there ({err})."));
                return false;
            }
        };
        if let Err(err) = self.travel(&id, arrival) {
            self.say(format!("You cannot go there ({err})."));
            return false;
        }
        events.push(Event::LevelChanged { from, to: id });
        self.say(format!("You explore the {what}."));
        true
    }

    pub(super) fn take_stairs(&mut self, dir: StairsDir, events: &mut Vec<Event>) -> bool {
        let here = self.player_pos();
        let link = self
            .state
            .current()
            .stairs_at(here)
            .filter(|s| s.dir == dir)
            .cloned();
        let way = match dir {
            StairsDir::Down => "down",
            StairsDir::Up => "up",
        };
        let kind = self.state.current().kind;
        let Some(link) = link else {
            if kind == LevelKind::Overworld && dir == StairsDir::Down {
                return self.enter_local_area(events);
            }
            if dir == StairsDir::Up
                && let Some(done) = self.quick_leave(events)
            {
                return done;
            }
            self.say(format!("You see no way {way} here."));
            return false;
        };
        let Some(to) = link.to else {
            self.say("These stairs lead nowhere yet.");
            return false;
        };
        if !self.can_move() {
            return false;
        }
        let from = self.state.current_level.clone();
        if let Err(err) = self.travel(&to, None) {
            self.say(format!("The stairs are blocked ({err})."));
            return false;
        }
        events.push(Event::LevelChanged { from, to });
        let level = self.state.current();
        // A revealed location goes by its own name (spec §7 "Hidden locations").
        let name = link.name.unwrap_or_else(|| level.name.clone());
        let text = match (kind, level.kind, dir) {
            (LevelKind::Overworld, _, _) => format!("You enter {name}."),
            (_, LevelKind::Overworld, _) => format!("You climb out to {name}."),
            (_, _, StairsDir::Down) => format!("You descend the stairs to {name}."),
            (_, _, StairsDir::Up) => format!("You climb the stairs to {name}."),
        };
        self.say(text);
        true
    }

    /// Moves the player to level `to` (generating it on the first visit), arriving at `at` if
    /// given, else on the stairs that lead back, else at the level's start, else on its first
    /// staircase.
    pub(super) fn travel(&mut self, to: &str, at: Option<Pos>) -> Result<(), WorldError> {
        let from = self.state.current_level.clone();
        let new = !self.state.levels.contains_key(to);
        self.visit(to, Some(&from))?;
        let target = &self.state.levels[to];
        let arrival = at
            .or_else(|| {
                target
                    .stairs
                    .iter()
                    .find(|s| s.to.as_deref() == Some(from.as_str()))
                    .map(|s| s.pos)
            })
            .or(target.start)
            .or_else(|| target.stairs.first().map(|s| s.pos))
            .ok_or_else(|| WorldError::NoPlayerStart(to.to_string()))?;
        let arrival = self.free_cell_near(to, arrival);
        let player = self.state.player;
        if let Some(actor) = self.state.actors.get_mut(&player) {
            actor.level = to.to_string();
            actor.pos = arrival;
        }
        self.state.current_level = to.to_string();
        self.state.overworld_step = None;
        self.let_go(player, None);
        self.reach(to);
        if new {
            self.discover_level(to);
        }
        // Floors the player was not on stay frozen: what comes, comes from now on; and Sixth
        // sense says how the place feels (spec §7 "Danger levels").
        self.arrive_spawner(to);
        self.sense_danger(to);
        self.sense_quarry(to);
        // Sounds remembered on the level left say nothing about this one.
        self.state.recent_sounds.clear();
        if let Some(line) = self.content.arrival(to) {
            self.say(line.to_string());
        }
        if let Some(line) = super::rooms::pervade_line(&self.state, &self.content) {
            self.say(line);
        }
        let _ = self.settle_in();
        Ok(())
    }

    /// Level `id` as it is now, generating it on the first visit (entered from `from`).
    pub(super) fn visit(
        &mut self,
        id: &str,
        from: Option<&str>,
    ) -> Result<&LevelState, WorldError> {
        if !self.state.levels.contains_key(id) {
            let visit = crate::mapgen::Visit {
                luck: Luck::of(&self.state.player().talent_mods(&self.content)),
                player_level: self.player_level(),
                uniques: Some(&self.state.progress.uniques),
            };
            let template = level_template(
                &self.content,
                (&self.state.campaign, self.state.seed),
                id,
                (from, &visit),
            )?;
            instantiate_level(&mut self.state, &self.content, &template);
        }
        Ok(&self.state.levels[id])
    }

    /// `p` if no actor stands there, else the nearest walkable unoccupied cell (ring search).
    pub(super) fn free_cell_near(&self, level_id: &str, p: Pos) -> Pos {
        let level = &self.state.levels[level_id];
        let free = |q: Pos| {
            level.is_walkable(&self.content, q) && self.state.actor_at(level_id, q).is_none()
        };
        if self.state.actor_at(level_id, p).is_none() {
            return p;
        }
        for r in 1..level.width().max(level.height()) {
            for dy in -r..=r {
                for dx in -r..=r {
                    let q = Pos::new(p.x + dx, p.y + dy);
                    if q.chebyshev(p) == r && free(q) {
                        return q;
                    }
                }
            }
        }
        p
    }
}

/// The first time the player is inside a special room (spec §7 "Special rooms"): marks it
/// entered and returns what they are told; from then on they know it (look mode names it).
pub(super) fn enter_room(state: &mut WorldState, content: &Content) -> Option<String> {
    let here = state.player().pos;
    let room = (state.current_mut().rooms.iter_mut()).find(|r| r.contains(here))?;
    if room.entered {
        return None;
    }
    room.entered = true;
    room.glimpsed = true;
    let room = room.clone();
    Some(super::shrine::enter_line(&room, state.player(), content))
}

/// The template of level `id`: a hand-made level, or one generated from the run seed: a level of
/// a dungeon branch (`entered_from` is where the up staircase of a branch's first level leads)
/// or a local area of the overworld, its loot rolled with the player's `luck` as they first come
/// (Treasure hunter, Hoard sense).
pub(super) fn level_template<'a>(
    content: &'a Content,
    (campaign, seed): (&str, u64),
    id: &str,
    (entered_from, visit): (Option<&str>, &crate::mapgen::Visit),
) -> Result<Cow<'a, LevelTemplate>, WorldError> {
    if let Some(template) = content.level(id) {
        return Ok(Cow::Borrowed(template));
    }
    if !content.has_level(campaign, id) {
        return Err(WorldError::UnknownLevel(id.to_string()));
    }
    crate::mapgen::generate_for(content, (campaign, seed), id, entered_from, visit)
        .map(Cow::Owned)
        .map_err(|e| WorldError::Generation(id.to_string(), e.to_string()))
}

/// Creates the runtime state for a level and spawns its creatures, each with its type's stats,
/// carrying, wearing, and wielding what the level gives it; its shops are kept by the keepers
/// spawned at their posts. The food lying there and carried is made now (it rots from now on).
pub(super) fn instantiate_level(
    state: &mut WorldState,
    content: &Content,
    template: &LevelTemplate,
) {
    let id = template.id.as_str();
    let mut level = LevelState::from_template(template, content, state.time);
    crate::rot::made_at(level.objects.values_mut().flatten(), content, state.time);
    // What the chests hold is made now too (spec §8 "Chests").
    let held = level.chests.values_mut().flat_map(|c| c.holds.iter_mut());
    crate::rot::made_at(held, content, state.time);
    // Jeweller (spec §8 "Class abilities"): the loose gems of a level made now are cut finer.
    let finer = (state.actors.get(&state.player)).map_or(0, |p| p.talent_mods(content).gem_cut);
    if finer > 0 {
        let held = level.chests.values_mut().flat_map(|c| c.holds.iter_mut());
        for stack in level.objects.values_mut().flatten().chain(held) {
            crate::gem::cut_finer(content, stack, finer);
        }
    }
    let first = state.next_actor_id;
    for plan in &template.shops {
        let spawned = (template.creatures.iter()).position(|c| c.pos == plan.post);
        if let Some(k) = spawned {
            let keeper = ActorId(first + u32::try_from(k).unwrap_or(u32::MAX));
            let shop = Shop::from_plan(plan, keeper, &template.terrain, content);
            level.shops.push(shop);
        }
    }
    // A level with a danger rule keeps what comes to it (spec §7 "Danger levels").
    if let Some((_, _, spawning)) = content.spawning(id) {
        level.spawner = Some(Spawner::new(state.seed, id, state.time, spawning.every));
    }
    state.levels.insert(id.to_string(), level);
    // The hidden locations revealed before the overworld was made (spec §7 "Hidden locations").
    super::reveal::apply_revealed(state, id);
    // Under the open sky its water has frozen and thawed with the weather of the last weeks.
    state.pre_roll_ice(content, id);
    for spawn in &template.creatures {
        make_creature(state, content, id, spawn);
    }
}

/// Makes the creature `spawn` places on level `id` (it must exist): one of its type, with the
/// type's stats, carrying, wearing, and wielding what the spawn gives it, its food made now; a unique type is remembered as met.
pub(super) fn make_creature(
    state: &mut WorldState,
    content: &Content,
    id: &str,
    spawn: &CreatureSpawn,
) -> ActorId {
    let actor_id = ActorId(state.next_actor_id);
    state.next_actor_id += 1;
    let def = content.creature(spawn.creature);
    // A unique type comes once a run, however it came (spec §7 "Danger levels").
    if def.unique {
        state.progress.uniques.insert(def.id.clone());
    }
    let (carry, equipment) = placed_gear(content, state.seed, id, actor_id, spawn);
    // An animated thing is made of its body or its pieces (spec §8 "Animated objects").
    let mut rng = crate::animated::stream(state.seed, &actor_id.0.to_string());
    let (carry, equipment) =
        crate::animated::kit(content, &mut rng, spawn.creature, (carry, equipment));
    let weapon_skills = crate::animated::weapon_skills(content, spawn.creature, &carry, &equipment)
        .unwrap_or_else(|| def.weapon_skills.clone());
    let mut actor = Actor {
        id: actor_id,
        kind: ActorKind::Creature,
        creature: spawn.creature,
        level: id.to_string(),
        pos: spawn.pos,
        patrol: (!spawn.route.is_empty()).then(|| Patrol {
            route: spawn.route.clone(),
            step: 0,
        }),
        energy: 0,
        heard: Vec::new(),
        mind: Mind::default(),
        inventory: carry,
        equipment,
        stats: def.stats,
        hp: 0,
        hp_base: def.base_hp,
        regen: 0,
        mana: 0,
        mana_base: def.base_mana,
        mana_regen: 0,
        resources: Default::default(),
        skills: def.skills.clone(),
        effects: Default::default(),
        diseases: Default::default(),
        damage: Default::default(),
        drained: Default::default(),
        weapon_skills,
        faction_override: (spawn.faction)
            .or_else(|| super::court::join(content, state.seed, actor_id, spawn.creature)),
        summoner: None,
        alignment: None,
        alignment_override: spawn.alignment,
        npc: spawn.npc.clone(),
        talk: Talk::default(),
        spells: Default::default(),
        streak: None,
        satiation: None,
        stance: CombatStance::Neutral,
        free_blows: FreeBlows::default(),
        talents: Default::default(),
        boons: Vec::new(),
        stat_marks: Default::default(),
    };
    actor.hp = actor.max_hp(content);
    actor.mana = actor.max_mana(content);
    // A teeming den's sleeper (`den.rs`).
    if let Some(asleep) = content.sleep_status().filter(|_| spawn.asleep) {
        actor.effects.add(asleep, content.status(asleep), 1, None);
    }
    crate::rot::made_at(&mut actor.inventory, content, state.time);
    state.actors.insert(actor_id, actor);
    actor_id
}

/// What creature `spawn`, placed on level `level` as actor `id`, carries and of that wears and
/// wields: its fixed kit (or the gear its generated level rolled), or else its type's gear,
/// rolled now from `gear/<actor id>` at the level's item level (spec §8 "Creature gear").
fn placed_gear(
    content: &Content,
    seed: u64,
    level: &str,
    id: ActorId,
    spawn: &CreatureSpawn,
) -> (Vec<Stack>, Equipment) {
    if !spawn.rolls_gear {
        return (spawn.carry.clone(), spawn.equipment.clone());
    }
    let mut rng = crate::rng::Rng::gear(seed, &id.0.to_string());
    let pack = crate::gear::roll(content, &mut rng, spawn.creature, content.item_level(level));
    (pack.items, pack.equipment)
}
