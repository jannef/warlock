//! The controls screen and rebinding UI (spec §6): select a command, press a key to bind it.
//!
//! Commands are listed by context (`input::Context`: in play, aiming, `--more--`, menus and
//! screens, text fields, info pages, and in builds that have them the developer keys), then, at
//! the very bottom, the abilities' own keys (play commands, unbound by default: the abilities
//! menu has them all; #644); a key is unique within its context only. Enter on a command opens
//! its keys as a sub-list under it (#1324): each key, the first the one hints name, then "Add a
//! key". Enter on a key waits for one to replace it, on "Add a key" for one to add after the
//! last; Delete removes the selected key; Esc closes the sub-list. A play command's key keeps
//! the modifiers held with it (`Ctrl+r`; a modifier alone never binds, and kept shortcuts and
//! modifiers on other contexts' keys are refused with the reason: `settings::mods`). A key
//! another command of the same context uses moves to this one, and the message says so; Esc
//! (fixed) is refused, taken or removed, and so is a key fixed where the command is read (the
//! menu keys among them, `settings::fixed`, #1533), so the player cannot lock themselves out. Esc cancels, Delete on a command resets
//! it, and "Reset all keys" restores the defaults. "Export keys" and "Import keys" write and read
//! the keymap file (`settings::keymap`): in the config folder on the desktop, a download and a
//! file picker on the web (whose file arrives a few frames later: [`receive_keymap`]).
//!
//! The `?` help is the same screen, read-only ([`ControlsScreen::help`]): it lists every
//! command by context, including Esc, and closes on `?` or Esc. The list is drawn by one piece
//! of code for both, so help always matches the bindings.

use super::cursor;
use super::menu::{HINTS, Nav, Panel, Press, truncated, wrap};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, Context, KeyBinding, KeyCommand};
use crate::render::console::Console;
use crate::settings::keymap::{self, KeymapFile};
use crate::settings::keys::{ABILITY_KEYS, Bound, Conflict, REBINDABLE};
use crate::settings::mods;
use bevy::prelude::*;

/// What the screen is doing.
#[derive(Debug, Default, PartialEq, Eq)]
enum Mode {
    #[default]
    Browse,
    /// Waiting for a key for the command at the selected row.
    Capture,
    /// The `?` help: nothing can be changed; `selected` is the first row shown.
    ReadOnly,
}

/// One row of the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    /// A line of text about keys in general.
    Note(&'static str),
    /// The space before a group.
    Blank,
    /// The heading of a context's group.
    Header(Context),
    /// The heading of the abilities' own keys, the last group ([`ABILITY_KEYS`]).
    AbilityKeys,
    Command(KeyCommand),
    /// Key number `.1` (from 0, the one hints name) of the open command.
    Key(KeyCommand, usize),
    /// The open command's last row: capture a key to add.
    AddKey(KeyCommand),
    Reset,
    Export,
    Import,
    Back,
}

impl Row {
    /// Whether the row can be selected: with a command's keys open, only its sub-list's rows.
    fn selectable(self, open: bool) -> bool {
        match open {
            true => matches!(self, Row::Key(..) | Row::AddKey(_)),
            false => matches!(
                self,
                Row::Command(_) | Row::Reset | Row::Export | Row::Import | Row::Back
            ),
        }
    }

    /// The command a row is or belongs to.
    fn command(self) -> Option<KeyCommand> {
        match self {
            Row::Command(command) | Row::Key(command, _) | Row::AddKey(command) => Some(command),
            _ => None,
        }
    }

    /// The label of an action row.
    fn action(self) -> Option<&'static str> {
        match self {
            Row::Reset => Some("Reset all keys"),
            Row::Export => Some("Export keys…"),
            Row::Import => Some("Import keys…"),
            Row::Back => Some("Back"),
            Row::Note(_)
            | Row::Blank
            | Row::Header(_)
            | Row::AbilityKeys
            | Row::Command(_)
            | Row::Key(..)
            | Row::AddKey(_) => None,
        }
    }
}

/// The heading of the abilities' own keys, below every other group.
const ABILITY_KEYS_TITLE: &str = "Ability keys (all in the abilities menu too)";

/// What the list says first: the keys no binding changes.
const NOTES: [&str; 4] = [
    "Esc always backs out and cancels; it cannot be rebound.",
    "Menus, dialogs, text fields, and info pages have fixed keys,",
    "as do the rest prompt and aiming: hint lines name them.",
    "On screens the numpad moves as the arrows do (fixed).",
];

/// The command whose keys are open as a sub-list, and how many keys it has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Open {
    command: KeyCommand,
    keys: usize,
}

/// The widest panel a long key list widens the screen to; longer lists are cut short with `…`
/// (each key shows in full in the command's sub-list).
const WIDEST: i32 = 80;

/// Selection state of the controls screen. Rows are the notes, then each context's commands
/// under its heading, then the abilities' own keys under theirs, then "Reset all keys",
/// "Export keys…", "Import keys…", and "Back"; in help, the notes and every command.
#[derive(Debug)]
pub struct ControlsScreen {
    /// The selected row; in help, the first row shown.
    selected: usize,
    mode: Mode,
    /// The command whose keys are open under it, if one is.
    open: Option<Open>,
    /// Whether the developer keys are listed (they work in this launch).
    dev: bool,
    /// Result of the last action, a line or a few; a leading `!` marks a refusal (never color
    /// alone).
    message: Vec<String>,
    /// Where Export keys and Import keys go.
    keymap: KeymapFile,
}

impl ControlsScreen {
    /// The rebinding screen; `dev`: list the developer keys too; `keymap`: where Export keys and
    /// Import keys go.
    pub fn new(dev: bool, keymap: KeymapFile) -> Self {
        let mut screen = Self {
            selected: 0,
            mode: Mode::Browse,
            open: None,
            dev,
            message: Vec::new(),
            keymap,
        };
        screen.selected = screen.next_selectable(0, 1);
        screen
    }

    /// The `?` help: every command, read-only.
    pub fn help(dev: bool) -> Self {
        Self {
            selected: 0,
            mode: Mode::ReadOnly,
            open: None,
            dev,
            message: Vec::new(),
            keymap: KeymapFile::Nowhere,
        }
    }

    pub fn is_help(&self) -> bool {
        self.mode == Mode::ReadOnly
    }

    /// The rows shown: the notes, then every context's commands in [`REBINDABLE`] order under
    /// its heading (help adds Esc's Cancel to play's), then, at the very bottom, the abilities'
    /// own keys ([`ABILITY_KEYS`]: play commands, niche, unbound by default; #644); the
    /// rebinding screen ends with its actions.
    fn rows(&self) -> Vec<Row> {
        let read_only = self.mode == Mode::ReadOnly;
        let mut rows: Vec<Row> = NOTES.iter().map(|note| Row::Note(note)).collect();
        for &context in Context::ALL {
            if context == Context::Dev && !self.dev {
                continue;
            }
            rows.extend([Row::Blank, Row::Header(context)]);
            let commands =
                (REBINDABLE.iter()).filter(|c| c.context() == context && !ABILITY_KEYS.contains(c));
            commands.for_each(|&c| self.push_command(&mut rows, c));
            if read_only && context == Context::Play {
                rows.push(Row::Command(KeyCommand::Cancel));
            }
        }
        rows.extend([Row::Blank, Row::AbilityKeys]);
        (ABILITY_KEYS.iter()).for_each(|&c| self.push_command(&mut rows, c));
        if !read_only {
            rows.extend([Row::Blank, Row::Reset, Row::Export, Row::Import, Row::Back]);
        }
        rows
    }

    /// `command`'s row, and, when it is open, its keys and "Add a key" under it.
    fn push_command(&self, rows: &mut Vec<Row>, command: KeyCommand) {
        rows.push(Row::Command(command));
        if let Some(open) = self.open.filter(|o| o.command == command) {
            rows.extend((0..open.keys).map(|i| Row::Key(command, i)));
            rows.push(Row::AddKey(command));
        }
    }

    /// The first selectable row from `from` on in direction `step` (wrapping around).
    fn next_selectable(&self, from: usize, step: isize) -> usize {
        let rows = self.rows();
        let n = rows.len() as isize;
        let open = self.open.is_some();
        (0..n)
            .map(|i| (from as isize + step * i).rem_euclid(n) as usize)
            .find(|&i| rows[i].selectable(open))
            .unwrap_or(0)
    }

    /// The command at the selected row, or whose key or "Add a key" it is.
    fn selected_command(&self) -> Option<KeyCommand> {
        self.rows().get(self.selected).and_then(|r| r.command())
    }

    /// Keeps the open sub-list's rows to the keys its command has now.
    fn sync(&mut self, bindings: &Bindings) {
        if let Some(open) = &mut self.open {
            open.keys = bindings.keys_of(open.command).len();
            let last = self.rows().len().saturating_sub(1);
            self.selected = self.selected.min(last);
        }
    }

    /// Opens the keys of the command at the selected row under it, the first key selected.
    fn open_keys(&mut self, command: KeyCommand, bindings: &Bindings) {
        let keys = bindings.keys_of(command).len();
        self.open = Some(Open { command, keys });
        self.selected += 1;
        self.message = vec![format!(
            "The keys of {}. The first is the one hints name.",
            command.name()
        )];
    }

    /// Closes the open sub-list, its command selected.
    fn close_keys(&mut self) {
        let Some(open) = self.open.take() else {
            return;
        };
        self.selected = (self.rows().iter())
            .position(|r| *r == Row::Command(open.command))
            .unwrap_or(0);
        self.message.clear();
    }

    /// Applies a key press to `bindings`. Returns true when the player leaves the screen.
    pub fn handle(&mut self, press: &Press, bindings: &mut Bindings) -> bool {
        self.sync(bindings);
        let leave = self.apply(press, bindings);
        self.sync(bindings);
        leave
    }

    fn apply(&mut self, press: &Press, bindings: &mut Bindings) -> bool {
        match self.mode {
            Mode::Capture => {
                self.capture(press, bindings);
                return false;
            }
            Mode::ReadOnly => return self.read_only(press, bindings),
            Mode::Browse => {}
        }
        let row = self.rows().get(self.selected).copied();
        if self.open.is_some() {
            self.keys_list(press, row, bindings);
            return false;
        }
        match press.nav {
            Nav::Back => return true,
            Nav::Up => {
                self.selected =
                    self.next_selectable(cursor::up(self.selected, self.rows().len()), -1);
                self.message.clear();
            }
            Nav::Down => {
                self.selected =
                    self.next_selectable(cursor::down(self.selected, self.rows().len()), 1);
                self.message.clear();
            }
            Nav::Enter if !press.repeat => match row {
                Some(Row::Reset) => {
                    bindings.reset_all();
                    self.message = vec!["All keys are back to their defaults.".into()];
                }
                Some(Row::Export) => {
                    self.message = vec![match self.keymap.export(bindings) {
                        Ok(said) => said,
                        Err(why) => format!("! {why}"),
                    }];
                }
                Some(Row::Import) => {
                    self.message = match self.keymap.import(bindings) {
                        Ok(Some(report)) => report,
                        Ok(None) => vec!["Choose a keymap file to import.".into()],
                        Err(why) => vec![format!("! {why}")],
                    };
                }
                Some(Row::Back) => return true,
                Some(Row::Command(command)) => self.open_keys(command, bindings),
                _ => {}
            },
            Nav::Erase if !press.repeat => {
                if let Some(command) = self.selected_command() {
                    bindings.reset_command(command);
                    let name = command.name();
                    self.message = vec![format!("{name} is back to its default keys.")];
                }
            }
            _ => {}
        }
        false
    }

    /// The open sub-list: the menu keys move within it, Enter captures a key for the selected
    /// row, Delete removes the selected key, and Esc closes it.
    fn keys_list(&mut self, press: &Press, row: Option<Row>, bindings: &mut Bindings) {
        match press.nav {
            Nav::Back => self.close_keys(),
            Nav::Up => {
                self.selected =
                    self.next_selectable(cursor::up(self.selected, self.rows().len()), -1);
                self.message.clear();
            }
            Nav::Down => {
                self.selected =
                    self.next_selectable(cursor::down(self.selected, self.rows().len()), 1);
                self.message.clear();
            }
            Nav::Enter if !press.repeat => {
                self.mode = Mode::Capture;
                self.message.clear();
            }
            Nav::Erase if !press.repeat => {
                if let Some(Row::Key(command, at)) = row {
                    let removed = bindings.remove_key(command, at);
                    self.message = vec![remove_message(command, removed, bindings)];
                }
            }
            _ => {}
        }
    }

    /// Help: the menu keys scroll; Esc or the help key (`?`) closes.
    fn read_only(&mut self, press: &Press, bindings: &Bindings) -> bool {
        let help_key = press
            .binding
            .as_ref()
            .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::Help));
        match press.nav {
            _ if help_key => return true,
            Nav::Back => return true,
            Nav::Up => self.selected = self.selected.saturating_sub(1),
            Nav::Down => {
                let last = self.rows().len().saturating_sub(1);
                self.selected = (self.selected + 1).min(last);
            }
            _ => {}
        }
        false
    }

    fn capture(&mut self, press: &Press, bindings: &mut Bindings) {
        if press.repeat {
            return;
        }
        if press.nav == Nav::Back {
            self.mode = Mode::Browse;
            self.message = vec!["Cancelled.".into()];
            return;
        }
        let Some(key) = press.binding.clone() else {
            let refusal = "! That key cannot be bound. Try another, or Esc to cancel.";
            self.message = vec![refusal.into()];
            return;
        };
        let (command, at) = match self.rows().get(self.selected) {
            Some(Row::Key(command, at)) => (*command, Some(*at)),
            Some(Row::AddKey(command)) => (*command, None),
            _ => {
                self.mode = Mode::Browse;
                return;
            }
        };
        // Shift counts with a physical key bound in play (Shift+ArrowUp); a character carries
        // it, and elsewhere Shift reverses a key (Shift+Tab), so it is left out there.
        let key = match press.shift && command.context() == Context::Play {
            true => mods::with_shift(key),
            false => key,
        };
        if let Some(why) = key.refused_for(command) {
            self.message = vec![format!("! {why}. Try another, or Esc to cancel.")];
            return;
        }
        let shown = key.display();
        let name = command.name();
        self.mode = Mode::Browse;
        let keys: Vec<KeyBinding> = bindings.keys_of(command).into_iter().cloned().collect();
        let had = keys.iter().position(|k| *k == key);
        let old = at.and_then(|at| keys.get(at)).map(|k| k.display());
        self.message = vec![match bindings.bind_at(command, key, at) {
            Ok(Bound::Free) => match (had, old) {
                (Some(had), _) if at.is_none_or(|at| at == had) => {
                    format!("{name} has {shown} already.")
                }
                (Some(_), Some(old)) => format!("{name}: {shown} and {old} traded places."),
                (None, Some(old)) => format!("{name}: {old} is now {shown}."),
                _ => format!("{name}: {shown} added."),
            },
            Ok(Bound::Moved {
                from,
                keyless: true,
                ..
            }) => format!(
                "{shown} moved to {}; {} has no key.",
                command.name(),
                from.name()
            ),
            Ok(Bound::Moved { from, .. }) => {
                format!("{shown} moved to {} (was {}).", command.name(), from.name())
            }
            Err(conflict) => format!(
                "! {shown} is fixed for {} and cannot be taken.",
                conflict.owner.name()
            ),
        }];
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        let rows = self.rows();
        let count = rows.len();
        let read_only = self.mode == Mode::ReadOnly;
        // Room below the list for the message (up to four lines) and the footer; help has no
        // messages.
        let spare = if read_only { 6 } else { 12 };
        let label_w = rows
            .iter()
            .filter_map(|row| match row {
                Row::Command(c) => Some(c.name().chars().count()),
                _ => None,
            })
            .max()
            .unwrap_or(0) as i32;
        // Wide enough for the longest key list (besides the label column: the marker, two
        // spaces each side of the value, and the scroll arrow's column), within reason.
        let longest = (rows.iter())
            .filter_map(|row| match row {
                Row::Command(c) => Some(key_list(bindings, *c).chars().count()),
                _ => None,
            })
            .max()
            .unwrap_or(0) as i32;
        let width = (label_w + longest + 11).clamp(60, WIDEST);
        let panel = Panel::new(console, " Controls ", width, count as i32 + spare);
        let value_w = (panel.w - label_w - 11).max(1) as usize;
        let message_rows = if read_only { 0 } else { 6 };
        let visible = (panel.body_rows() - message_rows).max(1) as usize;
        let first = if read_only {
            self.selected
        } else {
            self.selected.saturating_sub(visible - 1)
        }
        .min(count.saturating_sub(visible));
        let capturing = self.mode == Mode::Capture;
        for (i, row) in rows.iter().enumerate().skip(first).take(visible) {
            let y = panel.y + 2 + (i - first) as i32;
            // Help has nothing to select.
            let selected = !read_only && i == self.selected;
            match row {
                Row::Note(note) => {
                    console.print(panel.inner_x(), y, note, TEXT_DIM);
                }
                Row::Blank => {}
                Row::Header(context) => {
                    console.print(panel.inner_x(), y, context.title(), ACCENT);
                }
                Row::AbilityKeys => {
                    console.print(panel.inner_x(), y, ABILITY_KEYS_TITLE, ACCENT);
                }
                Row::Command(command) => {
                    let value = truncated(&key_list(bindings, *command), value_w);
                    panel.row(console, y, selected, &command.name(), &value, label_w);
                }
                Row::Key(command, at) => {
                    let value = match bindings.keys_of(*command).get(*at) {
                        _ if selected && capturing => "press a key...".to_string(),
                        Some(key) => truncated(&key.display(), value_w),
                        None => String::new(),
                    };
                    let label = format!("  Key {}", at + 1);
                    panel.row(console, y, selected, &label, &value, label_w);
                }
                Row::AddKey(_) => {
                    let value = match selected && capturing {
                        true => "press a key...",
                        false => "",
                    };
                    panel.row(console, y, selected, "  Add a key", value, label_w);
                }
                Row::Reset | Row::Export | Row::Import | Row::Back => {
                    let label = row.action().unwrap_or_default();
                    panel.row(console, y, selected, label, "", label_w);
                }
            }
        }
        if first > 0 {
            console.print(panel.x + panel.w - 3, panel.y + 1, "↑", TEXT_DIM);
        }
        let more = first + visible < count;
        if more {
            console.print(panel.x + panel.w - 3, panel.y + panel.h - 2, "↓", TEXT_DIM);
        }
        if !read_only {
            self.draw_message(console, &panel, bindings);
        }
        let keys = HINTS;
        let footer = if read_only {
            let close = match bindings.key_name(KeyCommand::Help) {
                Some(key) => format!("{key} or Esc close"),
                None => "Esc close".to_string(),
            };
            let scroll = if first > 0 || more {
                format!("{} scroll   ", keys.up_down())
            } else {
                String::new()
            };
            format!("{scroll}{close}")
        } else if self.open.is_some() {
            format!(
                "{} replace or add   {} remove   Esc done",
                keys.confirm, keys.erase
            )
        } else {
            format!("{} keys   {} reset   Esc back", keys.confirm, keys.erase)
        };
        panel.footer(console, &footer);
    }

    /// The lines under the list: the capture prompt, the last action's message, or a warning
    /// that a context holds a key twice (which loading resolves, so never for long).
    fn draw_message(&self, console: &mut Console, panel: &Panel, bindings: &Bindings) {
        let lines: Vec<String> = if self.mode == Mode::Capture {
            let name = self
                .selected_command()
                .map_or_else(String::new, |c| c.name());
            vec![format!("Press a key for \"{name}\". Esc cancels.")]
        } else if !self.message.is_empty() {
            self.message.clone()
        } else if let Some((key, first, second)) = bindings.conflicts().first() {
            vec![format!(
                "! {} is bound to both {} and {} ({}).",
                key.display(),
                first.name(),
                second.name(),
                first.context().title()
            )]
        } else {
            Vec::new()
        };
        // Each line in its own color: an import's summary, then its first problem.
        let capture = self.mode == Mode::Capture;
        let wrapped = lines.iter().flat_map(|l| {
            let color = if capture || l.starts_with('!') {
                ACCENT
            } else {
                TEXT
            };
            wrap_hard(l, panel.inner_w())
                .into_iter()
                .map(move |w| (w, color))
        });
        let top = panel.y + panel.h - 7;
        for (i, (line, color)) in wrapped.take(4).enumerate() {
            console.print(panel.inner_x(), top + i as i32, &line, color);
        }
    }

    /// Shows what an import from the browser's file picker did.
    pub fn imported(&mut self, lines: Vec<String>) {
        self.message = lines;
    }
}

/// Applies a keymap file the browser's picker delivered (Import keys on the web) and shows the
/// result on the controls screen, if it is still open. A no-op on the desktop, which reads the
/// file at once.
pub(super) fn receive_keymap(
    mut bindings: ResMut<Bindings>,
    mut stack: ResMut<super::screens::ScreenStack>,
) {
    let Some(picked) = keymap::take_picked() else {
        return;
    };
    let mut changed = bindings.clone();
    let lines = match picked.and_then(|text| keymap::import(&mut changed, &text)) {
        Ok(report) => {
            for warning in &report.warnings {
                warn!("keymap: {warning}");
            }
            report.summary("the keymap")
        }
        Err(why) => vec![format!("! {why}")],
    };
    if changed.0 != bindings.0 {
        *bindings = changed;
    }
    stack.keymap_imported(lines);
}

/// `text` wrapped to `width` cells, breaking words longer than that (a file's path).
fn wrap_hard(text: &str, width: i32) -> Vec<String> {
    let width = width.max(1) as usize;
    let chunks = |line: String| {
        let chars: Vec<char> = line.chars().collect();
        let pieces: Vec<String> = chars.chunks(width).map(|c| c.iter().collect()).collect();
        pieces
    };
    wrap(text, width as i32)
        .into_iter()
        .flat_map(chunks)
        .collect()
}

/// What removing a key from `command` did, in words (`bindings` as it is after).
fn remove_message(
    command: KeyCommand,
    removed: Option<Result<KeyBinding, Conflict>>,
    bindings: &Bindings,
) -> String {
    let name = command.name();
    match removed {
        Some(Ok(key)) if bindings.keys_of(command).is_empty() => {
            format!("{name}: {} removed; it has no key now.", key.display())
        }
        Some(Ok(key)) => format!("{name}: {} removed.", key.display()),
        Some(Err(conflict)) => format!(
            "! {} is fixed for {name} and cannot be removed.",
            conflict.key.display()
        ),
        None => String::new(),
    }
}

/// `text` cut to `width` cells, ending in `…` when cut.
/// A command's keys, primary first, separated by spaces.
fn key_list(bindings: &Bindings, command: KeyCommand) -> String {
    let keys: Vec<String> = bindings
        .keys_of(command)
        .iter()
        .map(|k| k.display())
        .collect();
    if keys.is_empty() {
        "(none)".into()
    } else {
        keys.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::Dir;

    fn nav(screen: &mut ControlsScreen, b: &mut Bindings, nav: Nav) -> bool {
        screen.handle(&Press::nav(nav), b)
    }

    fn key(screen: &mut ControlsScreen, b: &mut Bindings, text: &str) {
        let press = Press {
            nav: Nav::Other,
            edit: None,
            binding: KeyBinding::from_text(text),
            repeat: false,
            text: None,
            shift: false,
        };
        screen.handle(&press, b);
    }

    /// Selects the row of `row` (it must be on the screen).
    fn select(screen: &mut ControlsScreen, row: Row) {
        screen.selected = screen.rows().iter().position(|r| *r == row).expect("a row");
    }

    fn message(screen: &ControlsScreen) -> String {
        screen.message.join(" ")
    }

    /// Closes any open keys, then opens `command`'s: its first key (or "Add a key") selected.
    fn open(screen: &mut ControlsScreen, b: &mut Bindings, command: KeyCommand) {
        if screen.open.is_some() {
            nav(screen, b, Nav::Back);
        }
        select(screen, Row::Command(command));
        nav(screen, b, Nav::Enter);
    }

    /// Waits for a key to replace the selected one (or to add) and presses `text`.
    fn capture(screen: &mut ControlsScreen, b: &mut Bindings, text: &str) {
        nav(screen, b, Nav::Enter);
        assert_eq!(screen.mode, Mode::Capture);
        key(screen, b, text);
    }

    #[test]
    fn rebinding_replaces_the_primary_key() {
        let (mut screen, mut b) = (
            ControlsScreen::new(false, KeymapFile::Nowhere),
            Bindings::default(),
        );
        // The first row selected is the first command, under its heading; Enter opens its keys
        // with the first selected.
        assert_eq!(screen.selected_command(), Some(KeyCommand::Move(Dir::N)));
        nav(&mut screen, &mut b, Nav::Enter);
        assert_eq!(
            screen.rows()[screen.selected],
            Row::Key(KeyCommand::Move(Dir::N), 0)
        );
        capture(&mut screen, &mut b, "w");
        assert_eq!(screen.mode, Mode::Browse);
        assert_eq!(key_list(&b, KeyCommand::Move(Dir::N)), "w ↑");
        assert_eq!(message(&screen), "Move north: Num8 is now w.");
    }

    #[test]
    fn a_key_is_added_replaced_and_removed_in_the_sub_list() {
        let (mut screen, mut b) = (
            ControlsScreen::new(false, KeymapFile::Nowhere),
            Bindings::default(),
        );
        let north = KeyCommand::Move(Dir::N);
        open(&mut screen, &mut b, north);
        // Up wraps to "Add a key", the sub-list's last row; Down stays inside it.
        nav(&mut screen, &mut b, Nav::Up);
        assert_eq!(screen.rows()[screen.selected], Row::AddKey(north));
        capture(&mut screen, &mut b, "w");
        assert_eq!(key_list(&b, north), "Num8 ↑ w");
        assert_eq!(message(&screen), "Move north: w added.");
        // The new key's row is selected, "Add a key" below it.
        assert_eq!(screen.rows()[screen.selected], Row::Key(north, 2));
        assert_eq!(screen.rows()[screen.selected + 1], Row::AddKey(north));
        nav(&mut screen, &mut b, Nav::Down);
        nav(&mut screen, &mut b, Nav::Down);
        assert_eq!(screen.rows()[screen.selected], Row::Key(north, 0));
        // The second key replaced, leaving the others where they are.
        nav(&mut screen, &mut b, Nav::Down);
        capture(&mut screen, &mut b, "Ctrl+e");
        assert_eq!(key_list(&b, north), "Num8 Ctrl+e w");
        assert_eq!(message(&screen), "Move north: ↑ is now Ctrl+e.");
        // A key it has already: in another place, the two trade; added, nothing changes.
        capture(&mut screen, &mut b, "w");
        assert_eq!(key_list(&b, north), "Num8 w Ctrl+e");
        assert_eq!(message(&screen), "Move north: w and Ctrl+e traded places.");
        nav(&mut screen, &mut b, Nav::Up);
        nav(&mut screen, &mut b, Nav::Up);
        assert_eq!(screen.rows()[screen.selected], Row::AddKey(north));
        capture(&mut screen, &mut b, "Numpad8");
        assert_eq!(message(&screen), "Move north has Num8 already.");
        // Another command's key moves, and the message says what moved.
        capture(&mut screen, &mut b, "s");
        assert_eq!(key_list(&b, north), "Num8 w Ctrl+e s");
        assert_eq!(message(&screen), "s moved to Move north (was Wait/search).");
        // Delete removes the selected key (the one just added), and the rest stay.
        assert_eq!(screen.rows()[screen.selected], Row::Key(north, 3));
        nav(&mut screen, &mut b, Nav::Erase);
        assert_eq!(key_list(&b, north), "Num8 w Ctrl+e");
        assert_eq!(message(&screen), "Move north: s removed.");
        for _ in 0..3 {
            select(&mut screen, Row::Key(north, 0));
            nav(&mut screen, &mut b, Nav::Erase);
        }
        assert_eq!(key_list(&b, north), "(none)");
        assert_eq!(
            message(&screen),
            "Move north: Ctrl+e removed; it has no key now."
        );
        assert_eq!(screen.rows()[screen.selected], Row::AddKey(north));
        // Esc closes the sub-list, the command selected, and the screen stays.
        assert!(!nav(&mut screen, &mut b, Nav::Back));
        assert_eq!(screen.open, None);
        assert_eq!(screen.rows()[screen.selected], Row::Command(north));
        assert!(!screen.rows().iter().any(|r| matches!(r, Row::AddKey(_))));
    }

    #[test]
    fn a_used_key_moves_and_the_message_says_what_moved() {
        let (mut screen, mut b) = (
            ControlsScreen::new(false, KeymapFile::Nowhere),
            Bindings::default(),
        );
        open(&mut screen, &mut b, KeyCommand::Move(Dir::N));
        capture(&mut screen, &mut b, "Numpad2");
        assert_eq!(
            message(&screen),
            "Num2 moved to Move north (was Move south)."
        );
        let j = KeyBinding::from_text("Numpad2").unwrap();
        assert_eq!(b.owner(&j), Some(KeyCommand::Move(Dir::N)));
        // Kick has one key: it is left with none, and the table shows that.
        let old = b.keys_of(KeyCommand::Kick)[0].display();
        open(&mut screen, &mut b, KeyCommand::Wait);
        capture(&mut screen, &mut b, &old);
        assert_eq!(
            message(&screen),
            format!("{old} moved to Wait/search; Kick has no key.")
        );
        assert_eq!(key_list(&b, KeyCommand::Kick), "(none)");
    }

    #[test]
    fn a_key_moves_only_within_its_context() {
        let (mut screen, mut b) = (
            ControlsScreen::new(false, KeymapFile::Nowhere),
            Bindings::default(),
        );
        // `q` for Back on the menus leaves play's keys alone: it stays the quest log in play.
        open(&mut screen, &mut b, KeyCommand::Back);
        capture(&mut screen, &mut b, "q");
        assert_eq!(message(&screen), "Back (Esc always works): q added.");
        let q = KeyBinding::from_text("q").unwrap();
        assert_eq!(b.owner_in(Context::Menu, &q), Some(KeyCommand::Back));
        assert_eq!(b.owner(&q), Some(KeyCommand::Quests));
        // A key fixed on the menus is refused, and the screen waits on for another key.
        open(&mut screen, &mut b, KeyCommand::Back);
        capture(&mut screen, &mut b, "j");
        assert_eq!(screen.mode, Mode::Capture);
        assert_eq!(
            message(&screen),
            "! j is fixed in menus and screens. Try another, or Esc to cancel."
        );
    }

    #[test]
    fn a_modified_key_binds_in_play_and_is_refused_elsewhere() {
        let (mut screen, mut b) = (
            ControlsScreen::new(false, KeymapFile::Nowhere),
            Bindings::default(),
        );
        open(&mut screen, &mut b, KeyCommand::Read);
        capture(&mut screen, &mut b, "Ctrl+r");
        let name = KeyCommand::Read.name();
        assert_eq!(message(&screen), format!("{name}: r is now Ctrl+r."));
        let ctrl_r = KeyBinding::from_text("Ctrl+r").unwrap();
        assert_eq!(b.owner(&ctrl_r), Some(KeyCommand::Read));
        assert_eq!(key_list(&b, KeyCommand::Read), "Ctrl+r");
        // A menu key takes no modifiers: refused, and the screen waits on for another key.
        open(&mut screen, &mut b, KeyCommand::Back);
        let before = b.0.clone();
        capture(&mut screen, &mut b, "Ctrl+w");
        assert_eq!(b.0, before);
        assert_eq!(screen.mode, Mode::Capture);
        assert!(message(&screen).starts_with("! Back (Esc always works) takes plain keys only"));
    }

    #[test]
    fn shift_counts_with_a_physical_key_in_play() {
        let (mut screen, mut b) = (
            ControlsScreen::new(false, KeymapFile::Nowhere),
            Bindings::default(),
        );
        open(&mut screen, &mut b, KeyCommand::Look);
        nav(&mut screen, &mut b, Nav::Enter);
        let press = Press {
            shift: true,
            binding: KeyBinding::from_text("ArrowUp"),
            ..Press::nav(Nav::Other)
        };
        screen.handle(&press, &mut b);
        assert_eq!(key_list(&b, KeyCommand::Look), "Shift+↑");
        // The plain arrow still moves north.
        let up = KeyBinding::from_text("ArrowUp").unwrap();
        assert_eq!(b.owner(&up), Some(KeyCommand::Move(Dir::N)));
    }

    #[test]
    fn a_fixed_commands_key_is_refused() {
        let (mut screen, mut b) = (
            ControlsScreen::new(false, KeymapFile::Nowhere),
            Bindings::default(),
        );
        let before = b.0.clone();
        nav(&mut screen, &mut b, Nav::Enter);
        nav(&mut screen, &mut b, Nav::Enter);
        let press = Press {
            nav: Nav::Other,
            edit: None,
            binding: Some(KeyBinding::Code(bevy::prelude::KeyCode::Escape)),
            repeat: false,
            text: None,
            shift: false,
        };
        screen.handle(&press, &mut b);
        assert_eq!(b.0, before);
        let message = message(&screen);
        assert!(
            message.starts_with('!') && message.contains("Cancel"),
            "{message}"
        );
    }

    #[test]
    fn escape_cancels_a_capture_without_leaving() {
        let (mut screen, mut b) = (
            ControlsScreen::new(false, KeymapFile::Nowhere),
            Bindings::default(),
        );
        // Esc backs out a step at a time: the capture, the command's keys, then the screen.
        nav(&mut screen, &mut b, Nav::Enter);
        nav(&mut screen, &mut b, Nav::Enter);
        assert!(!nav(&mut screen, &mut b, Nav::Back));
        assert_eq!(screen.mode, Mode::Browse);
        assert!(screen.open.is_some());
        assert!(!nav(&mut screen, &mut b, Nav::Back));
        assert_eq!(screen.open, None);
        assert!(nav(&mut screen, &mut b, Nav::Back));
    }

    #[test]
    fn delete_resets_one_command_and_reset_row_resets_all() {
        let (mut screen, mut b) = (
            ControlsScreen::new(false, KeymapFile::Nowhere),
            Bindings::default(),
        );
        let before = b.to_map();
        open(&mut screen, &mut b, KeyCommand::Move(Dir::N));
        capture(&mut screen, &mut b, "w");
        nav(&mut screen, &mut b, Nav::Back);
        nav(&mut screen, &mut b, Nav::Erase);
        assert_eq!(b.to_map(), before);
        open(&mut screen, &mut b, KeyCommand::Move(Dir::N));
        capture(&mut screen, &mut b, "w");
        nav(&mut screen, &mut b, Nav::Back);
        select(&mut screen, Row::Reset);
        nav(&mut screen, &mut b, Nav::Enter);
        assert_eq!(b.to_map(), before);
        select(&mut screen, Row::Back);
        assert!(nav(&mut screen, &mut b, Nav::Enter));
    }

    #[test]
    fn export_and_import_write_and_read_the_keymap() {
        let dir = std::env::temp_dir().join(format!("warlock-controls-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join(keymap::FILE_NAME);
        let mut screen = ControlsScreen::new(false, KeymapFile::Path(path.clone()));
        let mut b = Bindings::default();
        // Nothing to import yet: said, nothing changes.
        select(&mut screen, Row::Import);
        nav(&mut screen, &mut b, Nav::Enter);
        assert!(message(&screen).starts_with("! No keymap at"), "{screen:?}");
        // Rebind, export, reset, import: the rebinding is back.
        open(&mut screen, &mut b, KeyCommand::Descend);
        capture(&mut screen, &mut b, "D");
        nav(&mut screen, &mut b, Nav::Back);
        select(&mut screen, Row::Export);
        nav(&mut screen, &mut b, Nav::Enter);
        assert_eq!(
            message(&screen),
            format!("Keys exported to {}.", path.display())
        );
        let customized = b.to_map();
        select(&mut screen, Row::Reset);
        nav(&mut screen, &mut b, Nav::Enter);
        assert_ne!(b.to_map(), customized);
        select(&mut screen, Row::Import);
        nav(&mut screen, &mut b, Nav::Enter);
        assert_eq!(b.to_map(), customized);
        assert_eq!(
            message(&screen),
            "Imported warlock-keys.toml: Go down stairs changed."
        );
        // Where keys are not stored, both say so.
        let mut nowhere = ControlsScreen::new(false, KeymapFile::Nowhere);
        select(&mut nowhere, Row::Export);
        nav(&mut nowhere, &mut b, Nav::Enter);
        assert!(message(&nowhere).starts_with('!'));
    }

    #[test]
    fn the_selection_skips_headings_and_wraps_around() {
        let (mut screen, mut b) = (
            ControlsScreen::new(false, KeymapFile::Nowhere),
            Bindings::default(),
        );
        let first = screen.selected;
        // Up from the first command wraps to Back, the last row; down again comes back.
        nav(&mut screen, &mut b, Nav::Up);
        assert_eq!(screen.rows()[screen.selected], Row::Back);
        nav(&mut screen, &mut b, Nav::Up);
        assert_eq!(screen.rows()[screen.selected], Row::Import);
        nav(&mut screen, &mut b, Nav::Down);
        nav(&mut screen, &mut b, Nav::Down);
        assert_eq!(screen.selected, first);
        // Down through a whole context lands on the next one's first command.
        select(&mut screen, Row::Command(KeyCommand::Quit));
        nav(&mut screen, &mut b, Nav::Down);
        assert_eq!(screen.selected_command(), Some(KeyCommand::PrevTarget));
        for _ in 0..500 {
            nav(&mut screen, &mut b, Nav::Down);
            assert!(screen.rows()[screen.selected].selectable(false));
        }
    }

    #[test]
    fn commands_are_grouped_by_context_and_dev_keys_only_where_they_work() {
        let plain = ControlsScreen::new(false, KeymapFile::Nowhere).rows();
        let headers: Vec<Context> = (plain.iter())
            .filter_map(|r| match r {
                Row::Header(c) => Some(*c),
                _ => None,
            })
            .collect();
        assert_eq!(headers, Context::ALL[..Context::ALL.len() - 1]);
        assert!(!plain.contains(&Row::Command(KeyCommand::RevealMap)));
        // Every rebindable command but the developer keys, each under its context's heading,
        // and the abilities' own keys (play commands) last, under theirs.
        let mut context = None;
        for row in &plain {
            match row {
                Row::Header(c) => context = Some(*c),
                Row::AbilityKeys => context = None,
                Row::Command(command) if context.is_none() => {
                    assert!(ABILITY_KEYS.contains(command), "{command:?}");
                }
                Row::Command(command) => {
                    assert_eq!(Some(command.context()), context);
                    assert!(!ABILITY_KEYS.contains(command), "{command:?}");
                }
                _ => {}
            }
        }
        let last_group = plain.iter().rposition(|r| *r == Row::AbilityKeys);
        let last_header = plain.iter().rposition(|r| matches!(r, Row::Header(_)));
        assert!(last_group > last_header, "the ability keys come last");
        for command in REBINDABLE.iter().filter(|c| c.context() != Context::Dev) {
            assert!(plain.contains(&Row::Command(*command)), "{command:?}");
        }
        let dev = ControlsScreen::new(true, KeymapFile::Nowhere).rows();
        assert!(dev.contains(&Row::Header(Context::Dev)));
        assert!(dev.contains(&Row::Command(KeyCommand::RevealMap)));
        let last_group = dev.iter().rposition(|r| *r == Row::AbilityKeys);
        let dev_header = dev.iter().position(|r| *r == Row::Header(Context::Dev));
        assert!(last_group > dev_header, "below the developer keys too");
    }

    #[test]
    fn draws_in_tiny_consoles_with_scrolling() {
        let b = Bindings::default();
        for rows in [40, 14, 6] {
            let mut console = Console::default();
            console.resize(70, rows);
            for mut screen in [
                ControlsScreen::new(true, KeymapFile::Nowhere),
                ControlsScreen::help(true),
            ] {
                let last = screen.rows().len() - 1;
                for selected in [0, 7, 40, last] {
                    screen.selected = selected;
                    screen.draw(&mut console, &b);
                }
            }
        }
    }

    /// The text of console row `y`.
    fn line(console: &Console, y: i32) -> String {
        (0..console.cols() as i32)
            .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
            .collect()
    }

    #[test]
    fn a_long_key_list_widens_the_panel_then_is_cut_short() {
        let mut b = Bindings::default();
        let north = KeyCommand::Move(Dir::N);
        let draw = |b: &Bindings, screen: &ControlsScreen| {
            let mut console = Console::default();
            console.resize(120, 44);
            screen.draw(&mut console, b);
            let text: Vec<String> = (0..44).map(|y| line(&console, y)).collect();
            text.into_iter().find(|l| l.contains("Move north")).unwrap()
        };
        let screen = ControlsScreen::new(false, KeymapFile::Nowhere);
        let narrow = draw(&b, &screen);
        for text in ["Ctrl+e", "Ctrl+f", "Ctrl+g", "Ctrl+h"] {
            b.bind_at(north, KeyBinding::from_text(text).unwrap(), None)
                .unwrap();
        }
        let wide = draw(&b, &screen);
        assert!(wide.trim().len() > narrow.trim().len(), "{wide}");
        assert!(wide.contains("Ctrl+h"), "{wide}");
        for text in ["Ctrl+j", "Ctrl+u", "Ctrl+y", "Ctrl+z"] {
            b.bind_at(north, KeyBinding::from_text(text).unwrap(), None)
                .unwrap();
        }
        let cut = draw(&b, &screen);
        assert!(cut.contains('…') && !cut.contains("Ctrl+z"), "{cut}");
        assert!(cut.trim().chars().count() <= WIDEST as usize, "{cut}");
    }

    #[test]
    fn help_lists_every_command_by_context_and_changes_nothing() {
        let mut b = Bindings::default();
        let mut help = ControlsScreen::help(true);
        let rows = help.rows();
        for extra in [
            KeyCommand::Cancel,
            KeyCommand::RevealMap,
            KeyCommand::Snapshot,
            KeyCommand::Back,
            KeyCommand::PrevTarget,
        ] {
            assert!(rows.contains(&Row::Command(extra)), "{extra:?}");
        }
        assert!(!rows.contains(&Row::Reset) && !rows.contains(&Row::Back));

        let before = b.0.clone();
        for nav in [Nav::Enter, Nav::Erase, Nav::Down, Nav::Enter] {
            assert!(!help.handle(&Press::nav(nav), &mut b));
        }
        assert_eq!(b.0, before);
        assert_eq!(help.mode, Mode::ReadOnly);

        let mut console = Console::default();
        console.resize(80, 130);
        help.selected = 0;
        help.draw(&mut console, &b);
        let text: Vec<String> = (0..130).map(|y| line(&console, y)).collect();
        let has = |needle: &str| text.iter().any(|l| l.contains(needle));
        assert!(
            text.iter()
                .any(|l| l.contains("Move north") && l.contains("Num8 ↑"))
        );
        assert!(
            text.iter()
                .any(|l| l.contains("Cancel") && l.contains("Esc"))
        );
        for needle in [
            "? or Esc close",
            "Esc always backs out",
            "have fixed keys",
            "Menus and screens",
            "Aiming",
        ] {
            assert!(has(needle), "{needle}");
        }
        assert!(!has("│ >"), "nothing is selected");
    }

    #[test]
    fn help_closes_on_the_help_key_or_escape() {
        let mut b = Bindings::default();
        let question = Press {
            nav: Nav::Other,
            edit: None,
            binding: KeyBinding::from_text("?"),
            repeat: false,
            text: Some('?'),
            shift: false,
        };
        assert!(ControlsScreen::help(false).handle(&question, &mut b));
        assert!(ControlsScreen::help(false).handle(&Press::nav(Nav::Back), &mut b));
        // The rebinding screen takes `?` as an ordinary key.
        assert!(!ControlsScreen::new(false, KeymapFile::Nowhere).handle(&question, &mut b));
    }

    #[test]
    fn help_scrolls_within_the_list() {
        let mut b = Bindings::default();
        let mut help = ControlsScreen::help(false);
        nav(&mut help, &mut b, Nav::Up);
        assert_eq!(help.selected, 0);
        for _ in 0..200 {
            nav(&mut help, &mut b, Nav::Down);
        }
        assert_eq!(help.selected, help.rows().len() - 1);
    }
}
