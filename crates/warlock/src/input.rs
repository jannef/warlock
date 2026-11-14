//! Keyboard input: bindings, input modes, and the one router of key presses (spec §6).
//!
//! [`route_keys`] reads every key press exactly once. While a screen is open (the screen stack is
//! not empty) the press goes to the top screen, as a [`Press`] whose menu meaning comes from the
//! fixed menu keys (`settings::fixed`; Esc always backs out); otherwise it is a gameplay command,
//! read through the bindings of the current [`InputMode`]'s contexts. Prompts in the message line (the
//! direction for open/close/kick, for digging, for a chat, or for an ability, how long to rest,
//! look mode, aiming a shot or a throw, `--more--` when a turn says more than the message line
//! holds) are input modes, not screens.
//!
//! **Every key read in play is a [`KeyCommand`] in its [`Context`]** (#159), rebindable in
//! Controls and listed by the `?` help. The keys of menus, text fields, the dialog, the info
//! pages, the rest prompt, and aiming are fixed (#1533, `settings::fixed`), but for the
//! back-style commands, as are Esc (it always backs out and cancels, so no binding can lock the
//! player out) and the letters and digits that name the rows of a list.
//!
//! Letter and symbol commands bind by *logical* key (the character the keyboard layout produces),
//! so `<`, `>`, and `?` work on any layout. Arrows and the numpad bind by physical key code.

use crate::cli::Args;
use crate::game::{Game, PlayerAction, SimSet};
use crate::render::console::Console;
use crate::save::SaveCommand;
use crate::settings::Settings;
pub use crate::settings::bindings::{Bindings, Context, KeyBinding, KeyCommand};
use crate::settings::fixed::{self, AimKey};
use crate::settings::mods::{self, Mods};
use crate::ui::step_dialogs::{burst_dialog, sale_dialog, step_dialog, step_screen, treat_dialog};
use crate::ui::{
    Aim, AimKind, DevOverlays, Dialog, IdentifyBy, LastTarget, MenuChoice, Press, ScreenAction,
    ScreenName, Screens, more, quit_dialog,
};
use bevy::ecs::system::SystemParam;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use warlock_core::ability::{self, Ability};
use warlock_core::beside::{Beside, Way};
use warlock_core::content::{ClassActive, RigId, VisionProfile};
use warlock_core::perception::PerceptionView;
use warlock_core::time::Duration;
use warlock_core::weather::Weather;
use warlock_core::{Action, Dir, Pos, Positions};

/// Whether the developer keys work in this launch: always in debug builds, with `--dev` in
/// release builds (spec §9). Their bindings are in every table; only where this is false are
/// they never read, nor listed by the controls screen and the `?` help. It also gives New run
/// its seed field (`NewRunScreen`): elsewhere a new run's seed is no one's choice (#256).
pub fn dev_keys(args: &Args) -> bool {
    cfg!(debug_assertions) || args.dev
}

/// What keys currently mean.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    /// Waiting for a direction for `Open`, `Close`, `Kick`, `UseTool`, or `Chat`.
    Direction(KeyCommand),
    /// Waiting for a direction to use an ability in (one whose [`ability::Aim`] is a direction:
    /// a shield bash).
    Ability(Ability),
    /// Waiting for the way to the kept beast this stack goes on: Give, with more than one beside
    /// the player (the owner, #907 Q4; [`PerceptionView::kept_beside`]).
    GiveWay(usize),
    /// Waiting for the way to rig this trap on: Trapwright, after the rig list (#1423).
    RigWay(RigId),
    /// Asking how long to rest ([`REST_CHOICES`]).
    Rest,
    /// Look mode: a cursor at this level position.
    Look(Pos),
    /// Aiming a shot or a throw (`ui::target`).
    Aim(Aim),
    /// `--more--`: the message line shows page `n` of this turn's messages, and waits for
    /// [`KeyCommand::MoreNext`] (or [`KeyCommand::MoreSkip`], or Esc) before anything else
    /// (`ui::more`).
    More(usize),
}

/// Whether `action`, a step or a run's first step, would ask first from where the player stands
/// in `view`: a dialog ([`step_dialog`]) or a screen ([`step_screen`]). A step queued behind
/// another in one frame was asked about from the cell before (`game::apply_queued`, #1538).
pub(crate) fn asks_first(view: &PerceptionView<'_>, action: &Action) -> bool {
    match action {
        Action::Move(dir) | Action::Run(dir) => {
            step_dialog(view, *dir).is_some() || step_screen(view, *dir).is_some()
        }
        _ => false,
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // The settings file's bindings, if `main` loaded them.
        let bindings = app
            .world()
            .get_resource::<Bindings>()
            .cloned()
            .unwrap_or_default();
        app.insert_resource(bindings)
            .init_resource::<InputMode>()
            .init_resource::<LastTarget>()
            .add_systems(Startup, (aim_at_startup, press_keys_at_startup))
            .add_systems(Update, route_keys.before(SimSet));
    }
}

/// The one reader of key presses, so each press has exactly one consumer. While a screen is
/// open, every press goes to the top screen and gameplay sees none; otherwise presses are
/// gameplay commands. Screens open and close at once, so the next press in the same frame
/// already goes to the new top. A menu choice (starting or leaving a run) is applied after this
/// system, so the rest of that frame's presses are dropped, and so are they after a screen's
/// gameplay action while an object list stays open (#591). A step asked behind another in the
/// same frame is asked about again when it is applied (`game::apply_queued`, #1538).
pub fn route_keys(
    mut keys: MessageReader<KeyboardInput>,
    mut screens: Screens,
    mut settings: ResMut<Settings>,
    mut bindings: ResMut<Bindings>,
    mut play: Gameplay,
    mut choices: MessageWriter<MenuChoice>,
    console: Option<Res<Console>>,
) {
    // Item lists page by the console's size: the screens learn it before any press.
    let size = console.map(|c| (c.cols(), c.rows()));
    if let Some(size) = size
        && !keys.is_empty()
        && screens.stack.console() != Some(size)
    {
        screens.stack.set_console(size);
    }
    let mut chosen = false;
    for key in keys.read() {
        // A modifier alone means nothing anywhere (and never binds: Controls waits on for the
        // key it is held with).
        if chosen || key.state != ButtonState::Pressed || mods::is_modifier(key.key_code) {
            continue;
        }
        let action = if screens.stack.is_empty() {
            let last = screens.stack.last_ability();
            play.press(key, &bindings, last)
                .unwrap_or(ScreenAction::Stay)
        } else {
            // Screens edit copies, so the resources only change (and are saved) when a press
            // changed them.
            let (mut new_settings, mut new_bindings) = (settings.clone(), bindings.clone());
            let action = screens.stack.handle(
                &Press::from_input(key, play.mods(), &bindings),
                &mut new_settings,
                &mut new_bindings,
            );
            // A pick in the forging wizard narrows its later steps: read before the next press.
            if let Some(game) = play.game.as_ref().filter(|_| screens.stack.is_stale()) {
                screens.stack.reread(&game.world.perception());
            }
            if new_settings != *settings {
                *settings = new_settings;
            }
            if new_bindings.0 != bindings.0 {
                *bindings = new_bindings;
            }
            action
        };
        match action {
            ScreenAction::Stay => {}
            ScreenAction::Close => screens.stack.close(),
            ScreenAction::Act(action) => {
                screens.stack.close();
                if let Some(dialog) = play.act(action) {
                    screens.ask(dialog);
                }
                // A screen's action is applied after this system, and the screens left open
                // read the run again after that (`screens::refresh`): until then an object list
                // left open names stacks by positions the action may have changed (#591), so
                // while one is, the rest of this frame's presses are dropped.
                chosen |= screens.stack.shows_objects();
            }
            ScreenAction::Aim(kind) => {
                screens.stack.close();
                play.aim(kind);
            }
            ScreenAction::UseAbility(ability) => {
                // From the abilities menu, or from play (its key, the quick repeat).
                screens.stack.close();
                screens.stack.remember_ability(ability);
                if let Some(by) = play.identify_by(ability) {
                    screens.open_identify(by);
                } else {
                    match play.use_ability(ability) {
                        Some(ScreenAction::Open(name)) => screens.open(name),
                        Some(ScreenAction::Dialog(dialog)) => screens.ask(*dialog),
                        _ => {}
                    }
                }
            }
            ScreenAction::Open(name) => screens.open(name),
            ScreenAction::Replace(name) => screens.replace(name),
            ScreenAction::Return(name) => screens.stack.return_to(name),
            ScreenAction::Dialog(dialog) => screens.ask(*dialog),
            ScreenAction::Choose(choice) => {
                choices.write(choice);
                chosen = true;
            }
        }
    }
}

/// What gameplay input reads and writes.
#[derive(SystemParam)]
pub struct Gameplay<'w> {
    args: Res<'w, Args>,
    game: Option<Res<'w, Game>>,
    mode: ResMut<'w, InputMode>,
    last: ResMut<'w, LastTarget>,
    held: Res<'w, ButtonInput<KeyCode>>,
    overlays: ResMut<'w, DevOverlays>,
    actions: MessageWriter<'w, PlayerAction>,
    saves: MessageWriter<'w, SaveCommand>,
    /// The UI console, whose width lays out the message line's pages (`--more--`).
    console: Option<Res<'w, Console>>,
}

impl Gameplay<'_> {
    /// The modifiers held.
    fn mods(&self) -> Mods {
        Mods::held(&self.held)
    }

    /// Whether Shift is held.
    fn shift(&self) -> bool {
        self.mods().shift
    }

    /// Applies one key press in play, `last` the ability used last (for the quick repeat).
    /// Returns what it asks of the screens: one to open (`?` help, the pause menu on Esc when no
    /// prompt is open), a dialog, or an ability to use.
    fn press(
        &mut self,
        key: &KeyboardInput,
        bindings: &Bindings,
        last: Option<Ability>,
    ) -> Option<ScreenAction> {
        // No run, no play keys.
        self.game.as_ref()?;
        if !key.repeat
            && let Some(done) = self.fixed_key(key, bindings)
        {
            return done;
        }
        let game = self.game.as_ref()?;
        let mods = self.mods();
        let command = command_in(*self.mode, key, mods, bindings, dev_keys(&self.args))?;
        // Held keys repeat only for movement and waiting (searching), and the aiming cursor's
        // moves.
        let moves = matches!(command, KeyCommand::Move(_) | KeyCommand::Wait);
        if key.repeat && !moves {
            return None;
        }
        let mut open: Option<ScreenAction> = None;
        let next = match (*self.mode, command) {
            // `--more--`: the next page of this turn's messages, then play once the last shows;
            // skipping goes to play at once (the last page shows). Nothing else acts meanwhile.
            (InputMode::More(page), KeyCommand::MoreNext) => {
                let cols = self.console.as_ref().map_or(0, |c| c.cols());
                more::advance(page, more::stops(game, cols, bindings))
            }
            (InputMode::More(_), KeyCommand::MoreSkip | KeyCommand::Cancel) => InputMode::Normal,
            (InputMode::More(page), _) => InputMode::More(page),
            (InputMode::Aim(aim), KeyCommand::Move(dir)) => {
                InputMode::Aim(aim.step(dir, &game.world.perception()))
            }
            (InputMode::Aim(aim), KeyCommand::PrevTarget) => {
                InputMode::Aim(aim.cycle(&game.world.perception(), self.shift()))
            }
            // The key that started the aim confirms it, as Enter does (the owner, #1533).
            (InputMode::Aim(aim), _) if aim.confirmed_by(command) => return self.release(aim),
            (InputMode::Aim(_), KeyCommand::Cancel | KeyCommand::Look) => InputMode::Normal,
            (InputMode::Aim(aim), _) => InputMode::Aim(aim),
            (InputMode::Normal, KeyCommand::Missile) => {
                let view = game.world.perception();
                if let Some(item) = view.ready_wand().or(view.ready_rod()) {
                    InputMode::Aim(Aim::start(AimKind::Zap(item), &view, &self.last))
                } else if view.ready_to_fire().is_some() {
                    InputMode::Aim(Aim::start(AimKind::Fire, &view, &self.last))
                } else if let Some(item) = view.ready_to_throw() {
                    InputMode::Aim(Aim::start(AimKind::Throw(item), &view, &self.last))
                } else {
                    // Nothing ready (or no ammunition): the rules say so, and take no time.
                    let here = view.viewer();
                    self.actions.write(PlayerAction(Action::Fire(here)));
                    InputMode::Normal
                }
            }
            (InputMode::Normal, KeyCommand::Throw) => {
                open = Some(ScreenAction::Open(ScreenName::Throw));
                InputMode::Normal
            }
            (InputMode::Look(_), KeyCommand::Look | KeyCommand::Cancel) => InputMode::Normal,
            (InputMode::Look(at), KeyCommand::Move(dir)) => {
                let view = game.world.perception();
                let to = at.step(dir);
                InputMode::Look(Pos::new(
                    to.x.clamp(0, view.width() - 1),
                    to.y.clamp(0, view.height() - 1),
                ))
            }
            (InputMode::Look(at), _) => InputMode::Look(at),
            (InputMode::Normal, KeyCommand::Look) => InputMode::Look(game.world.player_pos()),
            // Heal with two kept beasts beside the player asked which way first (`tend_way`):
            // the one there, or, at the wait key, the player themselves.
            (InputMode::Ability(Ability::Heal), KeyCommand::Move(dir)) => {
                open = self.tend(Some(dir));
                InputMode::Normal
            }
            (InputMode::Ability(Ability::Heal), KeyCommand::Wait) => {
                open = self.tend(None);
                InputMode::Normal
            }
            (InputMode::Ability(ability), KeyCommand::Move(dir)) => {
                let content = game.world.content().clone();
                if let Some(action) = ability.action(&content, Some(dir)) {
                    self.actions.write(PlayerAction(action));
                }
                InputMode::Normal
            }
            (InputMode::Ability(_), _) => InputMode::Normal,
            (InputMode::GiveWay(item), KeyCommand::Move(dir)) => {
                self.actions.write(PlayerAction(Action::Give {
                    item,
                    ally: Some(dir),
                }));
                InputMode::Normal
            }
            (InputMode::GiveWay(_), _) => InputMode::Normal,
            (InputMode::RigWay(rig), KeyCommand::Move(dir)) => {
                let content = game.world.content();
                if let Some(rules) = content.rigs() {
                    self.actions.write(PlayerAction(Action::Rig {
                        rig: rules.rig(rig).id.clone(),
                        dir: Some(dir),
                    }));
                }
                InputMode::Normal
            }
            (InputMode::RigWay(_), _) => InputMode::Normal,
            (InputMode::Direction(cmd), KeyCommand::Move(dir)) => {
                let action = match cmd {
                    KeyCommand::Open => Action::Open(dir),
                    KeyCommand::Kick => Action::Kick(dir),
                    KeyCommand::UseTool => Action::Dig(dir),
                    KeyCommand::Chat => Action::Chat(dir),
                    _ => Action::Close(dir),
                };
                self.actions.write(PlayerAction(action));
                InputMode::Normal
            }
            (InputMode::Direction(_), _) => InputMode::Normal,
            // Any other command (Esc, or `R` again) cancels the rest prompt.
            (InputMode::Rest, _) => InputMode::Normal,
            (InputMode::Normal, KeyCommand::Rest) => InputMode::Rest,
            (InputMode::Normal, KeyCommand::Move(dir)) => {
                // Diving into deep water (spec §8 "Skills": Swim), leaving a shop owing anything
                // (spec §7 "Shops"), stepping onto a known trap (spec §8 "Traps"), setting out on
                // the overworld Fainting or Starving (spec §8 "Hunger"), or walking into something
                // that offers an action (spec §7 "Special rooms") asks first, in the dialog: one
                // dialog a step, the first that applies. The dive comes first because it always
                // asks (the owner's rule), and any other dialog's "go on" would step in unasked;
                // its own "Dive in" is a plain move, which the rules still judge (the keeper turns
                // back one who owes). A held key stops there (repeats never open the dialog).
                // Walking into an anvil opens Metalcraft's screen for the trained (#1387).
                let view = game.world.perception();
                let screen = step_dialog(&view, dir)
                    .map(ScreenAction::from)
                    .or_else(|| step_screen(&view, dir).map(ScreenAction::Open));
                match screen {
                    Some(screen) if !key.repeat => open = Some(screen),
                    Some(_) => {}
                    None => {
                        self.actions.write(PlayerAction(Action::Move(dir)));
                    }
                }
                InputMode::Normal
            }
            // A run's first step asks what a step would (#1325): the dialog's answer is a plain
            // step; with nothing to ask, the run.
            (InputMode::Normal, KeyCommand::Run(dir)) => {
                let view = game.world.perception();
                let screen = step_dialog(&view, dir)
                    .map(ScreenAction::from)
                    .or_else(|| step_screen(&view, dir).map(ScreenAction::Open));
                match screen {
                    Some(screen) => open = Some(screen),
                    None => {
                        self.actions.write(PlayerAction(Action::Run(dir)));
                    }
                }
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Pay) => {
                self.actions.write(PlayerAction(Action::Pay(None)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Wait) => {
                self.actions.write(PlayerAction(Action::Wait));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::SearchLong) => {
                let turns = game.world.content().search().turns;
                self.actions.write(PlayerAction(Action::Search(turns)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Descend) => {
                self.actions.write(PlayerAction(Action::Descend));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Ascend) => {
                self.actions.write(PlayerAction(Action::Ascend));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::PickUp) => {
                // One stack is picked up at once; several are chosen from on a screen. With
                // nothing here, the core says so.
                let view = game.world.perception();
                let letters = match view.objects(view.viewer()).len() {
                    0 => Some(Positions::default()),
                    1 => Some(Positions::of([0])),
                    _ => None,
                };
                match letters {
                    Some(letters) => {
                        self.actions.write(PlayerAction(Action::PickUp(letters)));
                    }
                    None => open = Some(ScreenAction::Open(ScreenName::PickUp)),
                }
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Drop) => {
                if game.world.perception().inventory().is_empty() {
                    self.actions
                        .write(PlayerAction(Action::Drop(Positions::default())));
                } else {
                    open = Some(ScreenAction::Open(ScreenName::Drop));
                }
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Inventory) => {
                open = Some(ScreenAction::Open(ScreenName::Inventory));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Equipment) => {
                open = Some(ScreenAction::Open(ScreenName::Equipment));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Character) => {
                open = Some(ScreenAction::Open(ScreenName::Character));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Skills) => {
                open = Some(ScreenAction::Open(ScreenName::Skills));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::MoreAggressive | KeyCommand::MoreDefensive) => {
                let now = game.world.perception().own_stance();
                let next = if command == KeyCommand::MoreAggressive {
                    now.more_aggressive()
                } else {
                    now.more_defensive()
                };
                self.actions.write(PlayerAction(Action::SetStance(next)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::WeaponSkills) => {
                open = Some(ScreenAction::Open(ScreenName::WeaponSkills));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Talents) => {
                open = Some(ScreenAction::Open(ScreenName::Talents));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Eat) => {
                open = Some(ScreenAction::Open(ScreenName::Eat));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Read) => {
                open = Some(ScreenAction::Open(ScreenName::Read));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Quests) => {
                open = Some(ScreenAction::Open(ScreenName::Quests));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Messages) => {
                open = Some(ScreenAction::Open(ScreenName::Messages));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::CycleVision) => {
                let view = game.world.perception();
                let next = next_vision(view.content().vision_profiles(), &view.vision().id);
                self.actions.write(PlayerAction(Action::SetVision(next)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::SkipHour) => {
                self.actions
                    .write(PlayerAction(Action::SkipTime(Duration::hours(1))));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::NextWeather) => {
                let now = game.world.perception().weather();
                self.actions
                    .write(PlayerAction(Action::SetWeather(next_weather(now))));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::IdentifyAll) => {
                self.actions
                    .write(PlayerAction(Action::Identify(Positions::default())));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::RevealMap) => {
                self.overlays.reveal = !self.overlays.reveal;
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::LightValues) => {
                self.overlays.light_values = !self.overlays.light_values;
                InputMode::Normal
            }
            // Kick asks always: any way may be kicked.
            (InputMode::Normal, KeyCommand::Kick) => InputMode::Direction(command),
            (InputMode::Normal, KeyCommand::Open) => {
                self.beside(Beside::Open, InputMode::Direction(command))
            }
            (InputMode::Normal, KeyCommand::Close) => {
                self.beside(Beside::Close, InputMode::Direction(command))
            }
            (InputMode::Normal, KeyCommand::Abilities) => {
                open = Some(ScreenAction::Open(ScreenName::Abilities));
                InputMode::Normal
            }
            // The ability used last, again; with none yet, the menu to choose one.
            (InputMode::Normal, KeyCommand::RepeatAbility) => {
                open = Some(last.map_or(
                    ScreenAction::Open(ScreenName::Abilities),
                    ScreenAction::UseAbility,
                ));
                InputMode::Normal
            }
            // An ability's own key (unbound by default) uses it as the menu does.
            (InputMode::Normal, KeyCommand::Bash) => {
                open = Some(ScreenAction::UseAbility(Ability::ShieldBash));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::ThrowVoice) => {
                open = Some(ScreenAction::UseAbility(Ability::ThrowVoice));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Disarm) => {
                open = Some(ScreenAction::UseAbility(Ability::Disarm));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::PickLock) => {
                open = Some(ScreenAction::UseAbility(Ability::PickLock));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Steal) => {
                open = Some(ScreenAction::UseAbility(Ability::Steal));
                InputMode::Normal
            }
            // The one creature in view next to the player, at once; among several, ask which way.
            (InputMode::Normal, KeyCommand::Chat) => {
                self.beside(Beside::Chat, InputMode::Direction(command))
            }
            (InputMode::Normal, KeyCommand::UseTool) => {
                let view = game.world.perception();
                if view.ready_light().is_some() {
                    // A light ready (a lantern): its next mode, at once.
                    self.actions.write(PlayerAction(Action::UseTool));
                    InputMode::Normal
                } else if view.dig_refusal().is_none() {
                    self.beside(Beside::Dig, InputMode::Direction(command))
                } else {
                    // Nothing to dig with, or untrained in Mining: the rules say so ("You have no
                    // digging tool ready.").
                    self.actions.write(PlayerAction(Action::Dig(Dir::N)));
                    InputMode::Normal
                }
            }
            (InputMode::Normal, KeyCommand::Help) => {
                open = Some(ScreenAction::Open(ScreenName::Help));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Cancel) => {
                open = Some(ScreenAction::Open(ScreenName::Pause));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Quit) => {
                open = Some(quit_dialog(true).into());
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Snapshot) => {
                self.saves.write(SaveCommand::Snapshot);
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::LoadSnapshot) => {
                self.saves.write(SaveCommand::LoadSnapshot);
                InputMode::Normal
            }
            // Other contexts' commands (aiming's, `--more--`'s, the screens'): `command_in`
            // never finds them here.
            (
                InputMode::Normal,
                KeyCommand::PrevTarget
                | KeyCommand::MoreNext
                | KeyCommand::MoreSkip
                | KeyCommand::Back
                | KeyCommand::TextBack
                | KeyCommand::PrevPage,
            ) => InputMode::Normal,
        };
        if *self.mode != next {
            *self.mode = next;
        }
        open
    }

    /// Asks the simulation to apply `action`, chosen on a screen. Dropping in a shop what it
    /// would buy asks first whether to sell (spec §7 "Shops"): the dialog is returned instead.
    fn act(&mut self, action: Action) -> Option<Dialog> {
        if let (Action::Drop(letters), Some(game)) = (&action, &self.game)
            && let Some(dialog) = sale_dialog(&game.world.perception(), letters)
        {
            return Some(dialog);
        }
        // The stack is chosen; two kept beasts beside the player: on which is asked next.
        if let (Action::Give { item, ally: None }, Some(game)) = (&action, &self.game)
            && game.world.perception().kept_beside().len() > 1
        {
            *self.mode = InputMode::GiveWay(*item);
            return None;
        }
        // The rig is chosen (Trapwright's list): the way is asked next.
        if let (Action::Rig { rig, dir: None }, Some(game)) = (&action, &self.game)
            && let Some(rig) = game.world.content().rigs().and_then(|r| r.id(rig))
        {
            *self.mode = InputMode::RigWay(rig);
            return None;
        }
        self.actions.write(PlayerAction(action));
        None
    }

    /// What `ability` identifies with on a stack the player carries, when they can use it now
    /// (the Identify spell, Merchant's eye): the identify list asks which stack.
    fn identify_by(&self, ability: Ability) -> Option<IdentifyBy> {
        let game = self.game.as_ref()?;
        let by = match ability {
            Ability::Spell(spell) => IdentifyBy::Spell(spell),
            // Make do and mend has its own list (`use_ability`).
            Ability::Class(ClassActive::MakeDo) => return None,
            Ability::Class(active) => IdentifyBy::Class(active),
            _ => return None,
        };
        let usable = (game.world.perception().abilities().iter())
            .any(|own| own.ability == ability && own.unavailable.is_none());
        (usable && ability.aim(game.world.content()) == ability::Aim::Item).then_some(by)
    }

    /// Uses `ability`: asks for what it needs (a direction in the message line, a cell in the
    /// aiming mode, an item from the list returned to open), then applies its action. One the
    /// player cannot use now is applied at once all the same, so the rules say why, taking no
    /// time (a shield bash without a shield). One that treats an ailment (Heal) asks which in
    /// the dialog, returned to open, when there are several.
    fn use_ability(&mut self, ability: Ability) -> Option<ScreenAction> {
        let game = self.game.as_ref()?;
        let usable = (game.world.perception().abilities().iter())
            .any(|own| own.ability == ability && own.unavailable.is_none());
        let content = game.world.content().clone();
        match (usable, ability.aim(&content)) {
            (true, ability::Aim::Direction) => {
                *self.mode = self.beside(Beside::Use(ability), InputMode::Ability(ability));
            }
            (true, ability::Aim::Cell | ability::Aim::Corpse) => {
                self.aim(AimKind::Ability(ability));
            }
            // Give's and Decipher's lists; the identify list asks which stack (`identify_by`,
            // before this).
            (true, ability::Aim::Item) => {
                if ability == Ability::Give {
                    return Some(ScreenAction::Open(ScreenName::Give));
                }
                if ability == Ability::Salvage {
                    return Some(ScreenAction::Open(ScreenName::Salvage));
                }
                if ability == Ability::Class(ClassActive::MakeDo) {
                    return Some(ScreenAction::Open(ScreenName::MakeDo));
                }
                if ability == Ability::Decipher {
                    return Some(ScreenAction::Open(ScreenName::Decipher));
                }
            }
            (true, ability::Aim::Nothing) => {
                if let Some(action) = ability.action(&content, None) {
                    self.actions.write(PlayerAction(action));
                }
            }
            // The throw list less flasks asks what, then the aiming mode where (Thrown exploit).
            (true, ability::Aim::Throw) => {
                return Some(ScreenAction::Open(ScreenName::ThrownExploit));
            }
            // The shape list asks which weapon (the Hunter's Spirit weapon).
            (true, ability::Aim::Shape) => return Some(ScreenAction::Open(ScreenName::Shape)),
            // The rig list asks which trap, then the way (the craftsperson's Trapwright).
            (true, ability::Aim::Rig) => return Some(ScreenAction::Open(ScreenName::Rig)),
            // The scribe list asks which spell (the wizard's Scribe).
            (true, ability::Aim::Spell) => return Some(ScreenAction::Open(ScreenName::Scribe)),
            // A profession's row opens its recipes.
            (true, ability::Aim::Recipes) => {
                if let Ability::Profession(skill) = ability {
                    return Some(ScreenAction::Open(ScreenName::Profession(skill)));
                }
            }
            (true, ability::Aim::Ailment) => {
                let view = game.world.perception();
                let ailments = view.own_ailments();
                // Two kept beasts beside the player (a companion and a familiar), one of them in
                // need: which is asked first, by the way to it (the owner, #907 Q4).
                if ability == Ability::Heal && tend_way(view.kept_beside(), &ailments) {
                    *self.mode = InputMode::Ability(Ability::Heal);
                    return None;
                }
                if ailments.len() > 1 {
                    return Some(treat_dialog(&ailments).into());
                }
                let one = ailments.into_iter().next().map(|a| a.action);
                if let Some(action) = one.or_else(|| ability.action(&content, None)) {
                    self.actions.write(PlayerAction(action));
                }
            }
            // Toward a known wall, if any, so nobody unseen is struck by mistake.
            (false, _) => {
                let view = game.world.perception();
                let here = view.viewer();
                let action = (ability.action(&content, Some(view.blank_way())))
                    .or_else(|| ability.action_at(&content, here))
                    .or_else(|| ability.action_on(&content, 0))
                    .or_else(|| ability.action_throw(0, here));
                if let Some(action) = action {
                    self.actions.write(PlayerAction(action));
                }
            }
        }
        None
    }

    /// A command or an ability that acts on something beside the player (#1327,
    /// [`warlock_core::beside`]): with nothing there, its action toward a blank way, so the
    /// rules say why, taking no time; with one thing, its action toward it at once; with
    /// several, `ask` (the mode to return). Kick never comes here.
    fn beside(&mut self, what: Beside, ask: InputMode) -> InputMode {
        let Some(game) = &self.game else {
            return InputMode::Normal;
        };
        let view = game.world.perception();
        let dir = match view.way(what) {
            Some(Way::Refuse(dir) | Way::Act(dir)) => dir,
            Some(Way::Ask) | None => return ask,
        };
        if let Some(action) = what.action(game.world.content(), dir) {
            self.actions.write(PlayerAction(action));
        }
        InputMode::Normal
    }

    /// Heal on the patient the way `dir` (the player with none), once the way is asked
    /// ([`tend_way`]): its one ailment at once, the choice of several in the dialog (returned to
    /// open), or, with none, the bare action, so the rules say why.
    fn tend(&mut self, dir: Option<Dir>) -> Option<ScreenAction> {
        let game = self.game.as_ref()?;
        let ailments: Vec<_> = (game.world.perception().own_ailments().into_iter())
            .filter(|a| matches!(a.action, Action::Heal { ally, .. } if ally == dir))
            .collect();
        if ailments.len() > 1 {
            return Some(treat_dialog(&ailments).into());
        }
        let action = (ailments.into_iter().next()).map_or(
            Action::Heal {
                what: None,
                ally: dir,
            },
            |a| a.action,
        );
        self.actions.write(PlayerAction(action));
        None
    }

    /// Starts aiming `kind` (a throw or a scroll chosen on a screen).
    fn aim(&mut self, kind: AimKind) {
        if let Some(game) = &self.game {
            let aim = Aim::start(kind, &game.world.perception(), &self.last);
            *self.mode = InputMode::Aim(aim);
        }
    }

    /// Looses what `aim` aims at its cursor; a burst that may catch the player (a scroll of
    /// fireball read too close) asks first, in the dialog, which is returned to open (spec §6
    /// "Targeting").
    fn release(&mut self, aim: Aim) -> Option<ScreenAction> {
        let dialog = (self.game.as_ref()).and_then(|g| burst_dialog(&g.world.perception(), aim));
        let Some(dialog) = dialog else {
            self.loose(aim);
            return None;
        };
        if let Some(game) = &self.game {
            self.last.remember(&game.world.perception(), aim);
        }
        *self.mode = InputMode::Normal;
        Some(dialog.into())
    }

    /// The fixed keys of the rest prompt and of aiming (`settings::fixed`), read before the
    /// bindings: what the press did, if it was one of them (a key aiming's own rebindable
    /// command has goes on to the bindings).
    fn fixed_key(
        &mut self,
        key: &KeyboardInput,
        bindings: &Bindings,
    ) -> Option<Option<ScreenAction>> {
        let mods = self.mods();
        match *self.mode {
            InputMode::Rest => {
                let rest = fixed::read(key, mods, |_| None, fixed::rest)?;
                self.actions.write(PlayerAction(Action::Rest(rest)));
                *self.mode = InputMode::Normal;
                Some(None)
            }
            InputMode::Aim(aim) => {
                let own = |k: &KeyBinding| bindings.owner_in(Context::Aim, k).map(|_| None);
                match fixed::read(key, mods, own, |k| fixed::aim(k).map(Some))?? {
                    AimKey::Loose => Some(self.release(aim)),
                    AimKey::NextTarget => {
                        let game = self.game.as_ref()?;
                        let next = aim.cycle(&game.world.perception(), !self.shift());
                        *self.mode = InputMode::Aim(next);
                        Some(None)
                    }
                }
            }
            _ => None,
        }
    }

    /// Looses the missile `aim` aims, remembering its target for the next shot.
    fn loose(&mut self, aim: Aim) {
        let Some(game) = &self.game else { return };
        self.last.remember(&game.world.perception(), aim);
        if let Some(action) = aim.action(game.world.content()) {
            self.actions.write(PlayerAction(action));
        }
        *self.mode = InputMode::Normal;
    }
}

/// Dev (`--fire`): starts the run aiming the readied missile weapon, as `f` would, so captures
/// show the targeting line; (`--look X,Y`) in look mode with the cursor there.
fn aim_at_startup(args: Res<Args>, game: Option<Res<Game>>, mut mode: ResMut<InputMode>) {
    if let (true, Some(game)) = (args.fire, &game) {
        let view = game.world.perception();
        *mode = InputMode::Aim(Aim::start(AimKind::Fire, &view, &LastTarget::default()));
    }
    if let (Some(at), Some(_)) = (args.look, &game) {
        *mode = InputMode::Look(at);
    }
}

/// Dev (`--keys`): presses the keys as if typed, once the run (and the screens) have started, so
/// captures show what they open.
fn press_keys_at_startup(args: Res<Args>, mut presses: MessageWriter<KeyboardInput>) {
    let Some(keys) = &args.keys else {
        return;
    };
    for key in &keys.0 {
        let (key_code, logical_key) = match key.as_str() {
            "Enter" => (KeyCode::Enter, Key::Enter),
            "Esc" => (KeyCode::Escape, Key::Escape),
            "Tab" => (KeyCode::Tab, Key::Tab),
            "Up" => (KeyCode::ArrowUp, Key::ArrowUp),
            "Down" => (KeyCode::ArrowDown, Key::ArrowDown),
            "Left" => (KeyCode::ArrowLeft, Key::ArrowLeft),
            "Right" => (KeyCode::ArrowRight, Key::ArrowRight),
            ch => (
                KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
                Key::Character(ch.into()),
            ),
        };
        // A character key types itself, as a keyboard's would (text fields read the text).
        let text = match &logical_key {
            Key::Character(ch) => Some(ch.clone()),
            _ => None,
        };
        presses.write(KeyboardInput {
            key_code,
            logical_key,
            state: ButtonState::Pressed,
            text,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }
}

/// The command a key press stands for in input mode `mode`: `--more--`'s keys and aiming's
/// before play's, which the other modes read alone; the developer keys last, in plain play,
/// where they are on (`dev`). The fixed keys of the rest prompt and of aiming are read before
/// this ([`Gameplay::press`]).
fn command_in(
    mode: InputMode,
    key: &KeyboardInput,
    mods: Mods,
    bindings: &Bindings,
    dev: bool,
) -> Option<KeyCommand> {
    let own = match mode {
        InputMode::Aim(_) => Some(Context::Aim),
        InputMode::More(_) => Some(Context::More),
        InputMode::Rest
        | InputMode::Normal
        | InputMode::Direction(_)
        | InputMode::Ability(_)
        | InputMode::GiveWay(_)
        | InputMode::RigWay(_)
        | InputMode::Look(_) => None,
    };
    (own.and_then(|context| bindings.lookup_in(context, key, mods)))
        .or_else(|| bindings.lookup(key, mods))
        .or_else(|| {
            let dev = dev && mode == InputMode::Normal;
            dev.then(|| bindings.lookup_in(Context::Dev, key, mods))
                .flatten()
        })
}

/// Whether Heal asks which way first: more than one kept beast beside the player (`ways`), and
/// something to treat on one of them among `ailments` (spec §8 "Allies"; the owner, #907 Q4:
/// "need to select direction if both adjacent"). One kept beast beside needs no asking.
pub(crate) fn tend_way(ways: &[Dir], ailments: &[warlock_core::heal::OwnAilment]) -> bool {
    ways.len() > 1
        && (ailments.iter())
            .any(|a| matches!(a.action, Action::Heal { ally: Some(d), .. } if ways.contains(&d)))
}

/// The id of the profile after `current` in `profiles`, wrapping around.
/// The weather after `now` (clear when it is not known, underground), wrapping around.
fn next_weather(now: Option<Weather>) -> Weather {
    let i = now.map_or(0, |w| (w.index() + 1) % Weather::ALL.len());
    Weather::ALL[i]
}

fn next_vision(profiles: &[VisionProfile], current: &str) -> String {
    let i = profiles.iter().position(|p| p.id == current).unwrap_or(0);
    profiles
        .get((i + 1) % profiles.len().max(1))
        .map_or_else(|| current.to_string(), |p| p.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::Rest;

    fn ailment(ally: Option<Dir>) -> warlock_core::heal::OwnAilment {
        warlock_core::heal::OwnAilment {
            name: "Bleeding".into(),
            action: Action::Heal { what: None, ally },
        }
    }

    #[test]
    fn heal_asks_the_way_only_with_two_kept_beasts_beside_and_something_to_treat_on_one() {
        let two = [Dir::E, Dir::S];
        // Two kept beasts, one hurt: asked.
        assert!(tend_way(&two, &[ailment(Some(Dir::E))]));
        // ... and with the player's own among the rows.
        assert!(tend_way(&two, &[ailment(None), ailment(Some(Dir::S))]));
        // One beside: no asking.
        assert!(!tend_way(&[Dir::E], &[ailment(Some(Dir::E))]));
        // Nothing on either (the player's own, a summon's the other way): no asking.
        assert!(!tend_way(&two, &[ailment(None)]));
        assert!(!tend_way(&two, &[ailment(Some(Dir::W))]));
        assert!(!tend_way(&[], &[]));
    }

    fn press(code: KeyCode, text: &str) -> KeyboardInput {
        KeyboardInput {
            key_code: code,
            logical_key: if text.is_empty() {
                Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified)
            } else {
                Key::Character(text.into())
            },
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        }
    }

    #[test]
    fn modes_read_their_own_context_first() {
        let b = Bindings::default();
        let (d, tab, west) = (
            press(KeyCode::KeyD, "d"),
            press(KeyCode::Tab, ""),
            press(KeyCode::Numpad4, "4"),
        );
        let enter = KeyboardInput {
            logical_key: Key::Enter,
            ..press(KeyCode::Enter, "")
        };
        let aim = InputMode::Aim(Aim::start(
            AimKind::Fire,
            &warlock_core::World::new(
                std::sync::Arc::new(warlock_core::content::Content::builtin().unwrap()),
                "showcase",
                None,
            )
            .unwrap()
            .perception(),
            &LastTarget::default(),
        ));
        let read = |mode, key: &KeyboardInput, dev| command_in(mode, key, Mods::NONE, &b, dev);
        assert_eq!(read(InputMode::Normal, &d, false), Some(KeyCommand::Drop));
        // The rest prompt's and aiming's fixed keys are read before this (`fixed_key`).
        let rest = fixed::read(&d, Mods::NONE, |_| None, fixed::rest);
        assert_eq!(rest, Some(Rest::Dawn));
        let next = fixed::read(&tab, Mods::NONE, |_| None, fixed::aim);
        assert_eq!(next, Some(AimKey::NextTarget));
        let loose = fixed::read(&enter, Mods::NONE, |_| None, fixed::aim);
        assert_eq!(loose, Some(AimKey::Loose));
        assert_eq!(read(aim, &west, false), Some(KeyCommand::Move(Dir::W)));
        assert_eq!(read(InputMode::Normal, &tab, false), None);
        assert_eq!(read(InputMode::Normal, &enter, false), None);
        // The developer keys only where they are on, and only in plain play.
        let f2 = press(KeyCode::F2, "");
        assert_eq!(read(InputMode::Normal, &f2, false), None);
        assert_eq!(
            read(InputMode::Normal, &f2, true),
            Some(KeyCommand::RevealMap)
        );
        assert_eq!(read(InputMode::Rest, &f2, true), None);
    }

    #[test]
    fn weather_cycles_from_clear_to_heavy_rain() {
        assert_eq!(next_weather(None), Weather::Clear);
        assert_eq!(next_weather(Some(Weather::Clear)), Weather::Overcast);
        assert_eq!(next_weather(Some(Weather::Rain)), Weather::HeavyRain);
        assert_eq!(next_weather(Some(Weather::HeavyRain)), Weather::Clear);
    }

    #[test]
    fn vision_cycles_through_profiles() {
        let content = warlock_core::content::Content::builtin().unwrap();
        let profiles = content.vision_profiles();
        assert_eq!(next_vision(profiles, "normal"), "low_light");
        let last = &profiles[profiles.len() - 1].id;
        assert_eq!(next_vision(profiles, last), "normal");
    }
}
