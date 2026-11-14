//! The craftsperson's patron (spec §8 "Class abilities": Patron; #1421, the craftsperson design
//! §7.4).
//!
//! A character with the Patron class ability (`FortuneMods::patron`, the craftsperson's from
//! level 12) has a patron in town: a town's class helper for `patron` (`content/town.rs`),
//! placed with the aristocrat's servant's code (`stipend.rs`: as the town is made, or as the
//! ability is given if it is made already), of the player's race and speaking its register,
//! whose dialogue has `[dialogue.patron]` words. Messages call them "your patron".
//!
//! - **A commission** ([`World::patron_chat`], one line in `talk.rs::chat`): chatting while none
//!   is open draws one, the **work**, from the **pool** ([`pool`]): every generic recipe (no
//!   `use`) making one thing of every profession the player is trained in, whose `from` their
//!   trained value reaches, and whose result has an aspect to better (arms and armour: the graded
//!   things), in recipe order; one `index` on the stream `patron/<n>`, `n` the commissions ever
//!   taken. The patron says `offer` and hands over the recipe's stock at its base ([`advance`])
//!   through `World::receive` (what would overload the player is set down at their feet). An
//!   empty pool: `idle`.
//! - **The log**: the open commission shows among the active quests
//!   ([`PerceptionView::commission`], `QuestKind::Commission`), counted as no quest: it has no
//!   record, so no experience, guild rank, Invest bonus, or goodwill comes of it.
//! - **Hand-in** (the same chat while one is open): the first stack carried of the work's object
//!   that is Fine or better (one improvement at least; one not worn first) is taken and paid for,
//!   `data/rules/patron.toml`'s `pays` percent of its worth (`Stack::value`, the grade's multiplier
//!   counted), `bonus` percent more for one bearing a maker's mark (`Worked::marked`, Maker's
//!   mark), with `delivered`; Negotiation is marked (`Use::Deal`). Whose work it is, the
//!   patron does not ask. Only plain ones carried: `plain`; none: a one-liner (the rotation),
//!   then `waiting`.
//! - **One at a time**: the next is offered by the next chat; no cap, no wait.
//! - **The patron's death** ([`World::patron_dies`], one line in `fight.rs::dies`): the open
//!   commission is dropped; no one takes their place.
//!
//! Saved and hashed: [`WorldState::patron`](crate::world::WorldState), left out while no commission
//! was ever taken, so other runs hash as before.

use crate::action::Event;
use crate::content::{
    ActionKind, Content, DialogueId, InputKind, MaterialId, ObjectId, PartSource, PatronWords,
    QuestKind, RecipeCount, RecipeDef, RecipeId,
};
use crate::dice::Dice;
use crate::magic::worked;
use crate::object::Stack;
use crate::perception::PerceptionView;
use crate::rng::Rng;
use crate::skills::Use;
use crate::text::article;
use crate::world::people::quest::QuestView;
use crate::world::{ActorId, Spent, World};
use serde::{Deserialize, Serialize};

/// The player's commissions from their patron.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Patron {
    /// How many commissions were ever given: the next one's stream.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub taken: u32,
    /// The one open now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<Commission>,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl Patron {
    pub fn is_empty(&self) -> bool {
        self.taken == 0 && self.open.is_none()
    }
}

/// An open commission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Commission {
    /// The recipe of the work.
    pub recipe: String,
    /// The patron as messages call them ("your patron"): the log's giver.
    pub giver: String,
    /// What the patron said as they gave it: the log's offer.
    pub offer: String,
}

/// The recipes a patron may commission of `player` (the module docs), in recipe order.
pub(crate) fn pool(content: &Content, player: &crate::world::Actor) -> Vec<RecipeId> {
    let one = RecipeCount::Dice(Dice::fixed(1));
    (content.recipes().list.iter().enumerate())
        .filter(|(_, r)| r.uses.is_none())
        .filter(|(_, r)| (player.skills.value(r.skill)).is_some_and(|v| v >= r.from))
        .filter(|(_, r)| {
            (r.result.as_ref()).is_some_and(|res| {
                res.count == one && !worked::aspects(content.object(res.object)).is_empty()
            })
        })
        .filter_map(|(i, _)| u16::try_from(i).ok().map(RecipeId))
        .collect()
}

/// The material stock of `materials` is advanced in for a part whose base is `base`: the base
/// material, else the piece's own, else the cheapest (as `craft.rs::stock_words` prices it).
fn priced_in(content: &Content, materials: &[MaterialId], base: MaterialId) -> Option<MaterialId> {
    let piece = content.object(content.object_id(crate::piece::ID)?);
    let own = piece.parts.first().map(|p| p.material);
    (materials.iter().copied().find(|&m| m == base))
        .or_else(|| (materials.iter().copied()).find(|&m| Some(m) == own))
        .or_else(|| (materials.iter().copied()).min_by_key(|&m| content.material(m).value))
}

/// What a patron advances for `recipe`: the stock of each `"stock"` part and stock table, in
/// pieces of the material the row prices it in, as much as one takes; each object and stock
/// input; never a gem (the module docs).
pub(crate) fn advance(content: &Content, recipe: &RecipeDef) -> Vec<Stack> {
    let Some(result) = &recipe.result else {
        return Vec::new();
    };
    let object = result.object;
    let mut given: Vec<Stack> = Vec::new();
    let mut sourced: Vec<ObjectId> = Vec::new();
    let mut add = |stack: Option<Stack>| {
        if let Some(stack) = stack {
            crate::object::add_to_pile(&mut given, &[stack], content);
        }
    };
    for part in &recipe.parts {
        let stock = match &part.source {
            PartSource::Gem => continue,
            PartSource::Stock => super::craft::worked_materials(content, recipe, object, part.part),
            PartSource::Sources { input, stock, .. } => {
                if stock.is_empty() {
                    continue;
                }
                sourced.extend(*input);
                stock.clone()
            }
        };
        let Some(base) = content.object(object).parts.get(part.part) else {
            continue;
        };
        let Some(material) = priced_in(content, &stock, base.material) else {
            continue;
        };
        let count = super::craft_make::stock_need(content, recipe, part.part, material, 1);
        add(count.and_then(|n| crate::piece::make(content, material, n)));
    }
    for input in &recipe.inputs {
        let stack = match &input.kind {
            InputKind::Object(o) if sourced.contains(o) => continue,
            InputKind::Object(o) => Some(Stack::new(*o, input.count)),
            InputKind::Material(m) => crate::piece::make(content, *m, input.count),
            InputKind::Group(group) => {
                let of_group: Vec<MaterialId> = (content.materials().iter().enumerate())
                    .filter(|(_, m)| m.group.as_deref() == Some(group.as_str()))
                    .filter_map(|(i, _)| u16::try_from(i).ok().map(MaterialId))
                    .collect();
                let first = of_group.first().copied();
                (first.and_then(|f| priced_in(content, &of_group, f)))
                    .and_then(|m| crate::piece::make(content, m, input.count))
            }
        };
        add(stack);
    }
    given
}

/// The work of `recipe` with its article: "a long sword".
fn work_words(content: &Content, recipe: &RecipeDef) -> String {
    let name = recipe
        .result
        .as_ref()
        .map_or(recipe.name.as_str(), |r| &content.object(r.object).name);
    format!("{} {name}", article(name))
}

impl PerceptionView<'_> {
    /// The patron's open commission as the quest log shows it, if one is open.
    pub(crate) fn commission(&self) -> Option<QuestView> {
        let open = self.patron.open.as_ref()?;
        let content = self.content;
        let recipe = content.recipe(content.recipes().id(&open.recipe)?);
        let object = recipe.result.as_ref()?.object;
        let name = &content.object(object).name;
        let fine = (self.player.inventory.iter()).any(|s| fine(s, object));
        let status = if fine {
            format!("Return to {}.", open.giver)
        } else {
            "Not made yet.".to_string()
        };
        let mut giver = open.giver.clone();
        if let Some(first) = giver.get_mut(..1) {
            first.make_ascii_uppercase();
        }
        Some(QuestView {
            id: "commission".into(),
            name: format!("Commission: {}", work_words(content, recipe)),
            kind: QuestKind::Commission,
            active: true,
            failed: false,
            completed: 0,
            giver: open.giver.clone(),
            offer: open.offer.clone(),
            objective: format!("{giver} wants you to bring them a Fine {name} or better."),
            target: Some(content.object(object).id.clone()),
            status,
        })
    }
}

/// Whether `stack` is a Fine (or better) `object`: what a commission of it takes.
fn fine(stack: &Stack, object: ObjectId) -> bool {
    stack.object == object && stack.worked().is_some_and(|w| w.improvements() > 0)
}

impl World {
    /// The words of a patron, if dialogue `d` is one's.
    fn patron_words(&self, d: DialogueId) -> Option<&PatronWords> {
        self.content.dialogue(d).patron.as_ref()
    }

    /// `line` with `{address}` made the player's word in `words`.
    fn to_player(&self, words: &PatronWords, line: &str) -> String {
        let Some(character) = &self.state.character else {
            return line.to_string();
        };
        line.replace("{address}", words.address.of(character.spec.gender))
    }

    /// Chats with creature `id`, who speaks `dialogue`, if it is a patron: offers a commission,
    /// or takes the work and pays for it (the module docs). `None` for anyone else.
    pub(in crate::world) fn patron_chat(
        &mut self,
        id: ActorId,
        dialogue: DialogueId,
        events: &mut Vec<Event>,
    ) -> Option<Spent> {
        let words = self.patron_words(dialogue)?.clone();
        let rules = *self.content.patron()?;
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.talk.met = true;
        }
        let mut spent = Spent::Turn(self.price_of(self.state.player, ActionKind::Chat));
        match self.state.patron.open.clone() {
            None => self.offer_commission(id, &words, events),
            Some(open) => {
                let content = self.content.clone();
                let recipe = (content.recipes().id(&open.recipe)).map(|r| content.recipe(r));
                let work = recipe.map_or_else(String::new, |r| work_words(&content, r));
                let object = recipe.and_then(|r| r.result.as_ref()).map(|r| r.object);
                let inventory = &self.state.player().inventory;
                let equipment = &self.state.player().equipment;
                // One not worn first; a spirit weapon is nothing to hand over.
                let found = object.and_then(|o| {
                    let ok = |(_, s): &(usize, &Stack)| fine(s, o) && !s.is_spirit();
                    let all = || inventory.iter().enumerate().filter(ok);
                    (all().find(|(i, _)| !equipment.holds(*i)))
                        .or_else(|| all().next())
                        .map(|(i, _)| i)
                });
                let plain = object.is_some_and(|o| inventory.iter().any(|s| s.object == o));
                match found {
                    Some(item) => {
                        let (taken, stack) = self.hand_over_one(item, ActionKind::Chat);
                        spent = taken;
                        // A maker's mark is in the worth, paid for once (#1489).
                        let gold = rules.paid(stack.value(&content));
                        let gold = u32::try_from(gold).unwrap_or(u32::MAX);
                        let what = self.lore().describe(&stack);
                        self.say(format!("You hand over {what}."));
                        let line = words.delivered.replace("{gold}", &gold.to_string());
                        let line = self.to_player(&words, &line);
                        self.answer(id, line, None, events);
                        self.state.patron.open = None;
                        if let Some(coin) = crate::chest::currency(&content).filter(|_| gold > 0) {
                            self.receive(vec![Stack::new(coin, gold)]);
                        }
                        self.train(Use::Deal);
                    }
                    None if plain => {
                        let line = self.to_player(&words, &words.plain.replace("{work}", &work));
                        self.answer(id, line, None, events);
                    }
                    // Waiting: a one-liner (the rotation), then the reminder.
                    None => {
                        let said = self.state.actors[&id].talk.said;
                        let d = content.dialogue(dialogue);
                        let one = self.one_liner(id, d, said);
                        if let Some(actor) = self.state.actors.get_mut(&id) {
                            actor.talk.said += 1;
                        }
                        let waiting = words.waiting.replace("{work}", &work);
                        let line = self.to_player(&words, &format!("{one} {waiting}"));
                        self.answer(id, line, None, events);
                    }
                }
            }
        }
        self.train_stat(crate::content::StatUse::Chat);
        Some(spent)
    }

    /// Draws a commission from the pool and hands over its stock, or says there is nothing to
    /// commission (the module docs).
    fn offer_commission(&mut self, id: ActorId, words: &PatronWords, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let pool = pool(&content, self.state.player());
        if pool.is_empty() {
            let line = self.to_player(words, &words.idle);
            self.answer(id, line, None, events);
            return;
        }
        let taken = self.state.patron.taken;
        let mut rng = Rng::stream(self.state.seed, &format!("patron/{taken}"));
        let recipe = content.recipe(pool[rng.index(pool.len())]);
        let stock = advance(&content, recipe);
        let line = (words.offer.replace("{work}", &work_words(&content, recipe)))
            .replace("{stock}", &self.listed(&stock));
        let line = self.to_player(words, &line);
        let giver = self.called(&self.state.actors[&id]);
        self.state.patron.taken += 1;
        self.state.patron.open = Some(Commission {
            recipe: recipe.id.clone(),
            giver,
            offer: line.clone(),
        });
        self.answer(id, line, None, events);
        self.receive(stock);
    }

    /// Creature `id` dies: a patron's open commission is dropped (the module docs).
    pub(in crate::world) fn patron_dies(&mut self, id: ActorId) {
        let patron = (self.state.actors.get(&id))
            .and_then(|a| self.dialogue_of(a))
            .is_some_and(|d| self.patron_words(d).is_some());
        if patron && self.state.patron.open.take().is_some() {
            self.say("With your patron dead, the commission is off.");
        }
    }

    /// The `progress` dump's row (dev-only): the commissions taken and the one open,
    /// `patron 3 taken, open long_sword`; `None` while none was ever taken.
    pub fn patron_row(&self) -> Option<String> {
        let patron = &self.state.patron;
        (!patron.is_empty()).then(|| {
            let open = (patron.open.as_ref()).map_or("none", |c| c.recipe.as_str());
            format!("patron {} taken, open {open}", patron.taken)
        })
    }
}
