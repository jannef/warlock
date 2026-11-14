//! Modifier keys in play bindings (#1323): Ctrl, Alt, and Super held with a key, `Ctrl+r` or
//! `Cmd+s`. Super is Command on macOS and the Windows/Super key elsewhere; Alt is Option on
//! macOS. Shift counts only with a physical key (`Shift+ArrowUp`): a character key's character
//! already carries it (`K` against `k`).
//!
//! A modified binding is [`KeyBinding::Chord`]. Its text form puts the modifiers before the key,
//! `"Ctrl+r"`, `"Alt+ArrowUp"`, `"Super+s"`, in that order (Ctrl, Alt, Shift, Super); reading
//! also takes `Control`, `Option`, `Cmd`, `Command`, `Win`, and `Meta`. Controls, `?`, and the
//! hint lines show the platform's names: `Cmd` on macOS (Alt stays `Alt`, #1385).
//!
//! Matching is exact ([`press_candidates`]): `Ctrl+r` never fires `r`, nor `r` while Ctrl is
//! held. Alt changes the character a key types (Option on macOS, AltGr elsewhere, which some
//! systems report as Ctrl+Alt), so a modified character key is read from the physical key when
//! Alt is held (`Option+e` is `Alt+e`, never `Alt+´`); a character Alt typed that is no letter or
//! digit (`@` on many layouts) still fires a plain binding of that character, when no modified
//! binding takes the press first.
//!
//! Only play commands take modifiers (`Context::Play`; menus and prompts are fixed), and
//! shortcuts the system or the browser keep (Cmd+Q, Alt+F4, Ctrl+W on the web) are refused
//! ([`refusal`]).

use crate::input::{Context, KeyBinding, KeyCommand};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

/// The modifiers held with a key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// Command on macOS, the Windows/Super key elsewhere.
    pub sup: bool,
}

/// Where the game runs, for the modifiers' names and the shortcuts it keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    Mac,
    /// Windows and Linux.
    Desktop,
    /// The web build (in any browser, on any system).
    Web,
}

impl Platform {
    /// The platform this build runs on.
    pub fn current() -> Self {
        if cfg!(target_arch = "wasm32") {
            Platform::Web
        } else if cfg!(target_os = "macos") {
            Platform::Mac
        } else {
            Platform::Desktop
        }
    }
}

impl Mods {
    pub const NONE: Mods = Mods {
        ctrl: false,
        alt: false,
        shift: false,
        sup: false,
    };
    pub const CTRL: Mods = Mods {
        ctrl: true,
        ..Mods::NONE
    };
    pub const ALT: Mods = Mods {
        alt: true,
        ..Mods::NONE
    };

    /// The modifiers held now.
    pub fn held(keys: &ButtonInput<KeyCode>) -> Self {
        let any = |a, b| keys.pressed(a) || keys.pressed(b);
        Self {
            ctrl: any(KeyCode::ControlLeft, KeyCode::ControlRight),
            alt: any(KeyCode::AltLeft, KeyCode::AltRight),
            shift: any(KeyCode::ShiftLeft, KeyCode::ShiftRight),
            sup: any(KeyCode::SuperLeft, KeyCode::SuperRight),
        }
    }

    pub fn is_empty(self) -> bool {
        self == Mods::NONE
    }

    /// These modifiers without Shift.
    pub fn without_shift(self) -> Self {
        Self {
            shift: false,
            ..self
        }
    }

    /// Both sets' modifiers.
    pub fn union(self, other: Mods) -> Self {
        Self {
            ctrl: self.ctrl || other.ctrl,
            alt: self.alt || other.alt,
            shift: self.shift || other.shift,
            sup: self.sup || other.sup,
        }
    }

    /// The text form's prefix, `"Ctrl+Alt+"`.
    pub fn to_text(self) -> String {
        self.names(["Ctrl", "Alt", "Shift", "Super"])
    }

    /// The prefix as `platform` names the keys: `Cmd+` on macOS. Alt keeps its name there (the
    /// Option key, labelled Alt on many Mac keyboards; #1385), so the default keys, which hold
    /// Alt, read the same on every platform.
    pub fn display_on(self, platform: Platform) -> String {
        match platform {
            Platform::Mac => self.names(["Ctrl", "Alt", "Shift", "Cmd"]),
            Platform::Desktop | Platform::Web => self.to_text(),
        }
    }

    fn names(self, names: [&str; 4]) -> String {
        let held = [self.ctrl, self.alt, self.shift, self.sup];
        (held.iter().zip(names))
            .filter(|(held, _)| **held)
            .map(|(_, name)| format!("{name}+"))
            .collect()
    }

    /// Splits the modifier names off the front of a text form, `"Ctrl+Alt+r"` → (Ctrl+Alt,
    /// `"r"`). A name is only one when a key follows it, so `"+"` and `"Ctrl++"` read as the
    /// `+` key.
    pub fn split(text: &str) -> (Mods, &str) {
        let mut mods = Mods::NONE;
        let mut rest = text;
        while let Some((name, after)) = rest.split_once('+') {
            if after.is_empty() {
                break;
            }
            match name.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => mods.ctrl = true,
                "alt" | "option" | "opt" => mods.alt = true,
                "shift" => mods.shift = true,
                "super" | "cmd" | "command" | "win" | "meta" => mods.sup = true,
                _ => break,
            }
            rest = after;
        }
        (mods, rest)
    }
}

/// Whether `code` is a modifier key, which never binds alone.
pub fn is_modifier(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::ControlLeft
            | KeyCode::ControlRight
            | KeyCode::AltLeft
            | KeyCode::AltRight
            | KeyCode::ShiftLeft
            | KeyCode::ShiftRight
            | KeyCode::SuperLeft
            | KeyCode::SuperRight
            | KeyCode::Fn
            | KeyCode::FnLock
            | KeyCode::CapsLock
    )
}

/// The letter or digit a physical key types on a US layout: what a modified character binding
/// is read from when the character typed says nothing (Alt changed it, or Ctrl made a control
/// character of it).
fn physical_char(code: KeyCode) -> Option<char> {
    const LETTERS: [KeyCode; 26] = [
        KeyCode::KeyA,
        KeyCode::KeyB,
        KeyCode::KeyC,
        KeyCode::KeyD,
        KeyCode::KeyE,
        KeyCode::KeyF,
        KeyCode::KeyG,
        KeyCode::KeyH,
        KeyCode::KeyI,
        KeyCode::KeyJ,
        KeyCode::KeyK,
        KeyCode::KeyL,
        KeyCode::KeyM,
        KeyCode::KeyN,
        KeyCode::KeyO,
        KeyCode::KeyP,
        KeyCode::KeyQ,
        KeyCode::KeyR,
        KeyCode::KeyS,
        KeyCode::KeyT,
        KeyCode::KeyU,
        KeyCode::KeyV,
        KeyCode::KeyW,
        KeyCode::KeyX,
        KeyCode::KeyY,
        KeyCode::KeyZ,
    ];
    const DIGITS: [KeyCode; 10] = [
        KeyCode::Digit0,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];
    if let Some(i) = LETTERS.iter().position(|c| *c == code) {
        return Some(char::from(b'a' + i as u8));
    }
    (DIGITS.iter().position(|c| *c == code)).map(|i| char::from(b'0' + i as u8))
}

/// The one printable character `key` stands for, if it does.
fn printable(key: &Key) -> Option<char> {
    let Key::Character(text) = key else {
        return None;
    };
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if !c.is_control() && !c.is_whitespace() => Some(c),
        _ => None,
    }
}

/// The key of a press held with Ctrl, Alt, or Super: a physical key the game binds by its code,
/// else the character it types, read from the physical key when Alt changed it. A letter takes
/// its case from Shift.
fn chord_key(input: &KeyboardInput, mods: Mods, codes: &[KeyCode]) -> Option<KeyBinding> {
    if codes.contains(&input.key_code) {
        return Some(KeyBinding::Code(input.key_code));
    }
    let typed = printable(&input.logical_key);
    let ch = match (mods.alt, typed) {
        (false, Some(c)) => Some(c),
        _ => physical_char(input.key_code).or(typed),
    }?;
    let ch = match (ch.is_ascii_alphabetic(), mods.shift) {
        (true, true) => ch.to_ascii_uppercase(),
        (true, false) => ch.to_ascii_lowercase(),
        (false, _) => ch,
    };
    Some(KeyBinding::Char(ch.to_string()))
}

/// The binding a press stands for, `codes` the physical keys the game binds by code: without
/// Ctrl, Alt, or Super the plain key, with them a [`KeyBinding::Chord`]. Shift is never part of
/// it (screens read it as `Press::shift`; Controls adds it to a physical key bound in play). A
/// modifier alone is none.
pub fn press_binding(input: &KeyboardInput, mods: Mods, codes: &[KeyCode]) -> Option<KeyBinding> {
    if is_modifier(input.key_code) {
        return None;
    }
    let chord = mods.without_shift();
    if chord.is_empty() {
        let numpad = format!("{:?}", input.key_code).starts_with("Numpad");
        if !numpad && let Some(c) = printable(&input.logical_key) {
            return Some(KeyBinding::Char(c.to_string()));
        }
        return codes
            .contains(&input.key_code)
            .then_some(KeyBinding::Code(input.key_code));
    }
    Some(KeyBinding::chord(chord, chord_key(input, mods, codes)?))
}

/// `key` with Shift added, if it is a physical key (a character carries Shift itself).
pub fn with_shift(key: KeyBinding) -> KeyBinding {
    match key.base() {
        KeyBinding::Code(_) => {
            let mods = key.mods().union(Mods {
                shift: true,
                ..Mods::NONE
            });
            KeyBinding::chord(mods, key.base().clone())
        }
        _ => key,
    }
}

/// The bindings a press may stand for, in the order they are tried: the physical key, held with
/// Ctrl, Alt, or Super the chord ([`press_binding`]), with Shift held first with Shift
/// (`Shift+ArrowUp`), then without (so Shift+Tab is a Tab when no command has Shift+Tab, which
/// its command reads backwards); then the plain character the press typed, if Ctrl and Super
/// are not held (or Ctrl comes with Alt, AltGr on Windows) and the character is no letter or
/// digit when Alt is held: so `@` typed with Option or AltGr fires `@`, and a numpad digit a
/// digit's binding when no numpad binding has it.
pub fn press_candidates(input: &KeyboardInput, mods: Mods, codes: &[KeyCode]) -> Vec<KeyBinding> {
    let mut found = Vec::new();
    if is_modifier(input.key_code) {
        return found;
    }
    let chord = mods.without_shift();
    let own = match chord.is_empty() {
        true => Some(KeyBinding::Code(input.key_code)),
        false => chord_key(input, mods, codes).map(|key| KeyBinding::chord(chord, key)),
    };
    if let Some(own) = own {
        if mods.shift && matches!(own.base(), KeyBinding::Code(_)) {
            found.push(with_shift(own.clone()));
        }
        found.push(own);
    }
    let altgr = mods.ctrl && mods.alt;
    let plain = !mods.sup && (!mods.ctrl || altgr);
    if plain && let Some(c) = printable(&input.logical_key) {
        let typed_by_alt = mods.alt && c.is_ascii_alphanumeric();
        let typed = KeyBinding::Char(c.to_string());
        if !typed_by_alt && !found.contains(&typed) {
            found.push(typed);
        }
    }
    found
}

/// Why `key` may not be a key of `command` on `platform`, if it may not: only play commands
/// take Ctrl, Alt, Super, or Shift with a key, and a shortcut the system or the browser keeps
/// would never reach the game.
pub fn refusal(key: &KeyBinding, command: KeyCommand, platform: Platform) -> Option<String> {
    let KeyBinding::Chord(mods, base) = key else {
        return None;
    };
    if command.context() != Context::Play {
        return Some(format!(
            "{} takes plain keys only: Ctrl, Alt, and Super are for keys in play",
            command.name()
        ));
    }
    let kept = reserved(*mods, base, platform)?;
    Some(format!("{} is kept by {kept}", key.display_on(platform)))
}

/// What keeps the shortcut `mods` + `base` on `platform` (the system, the browser), if anything.
fn reserved(mods: Mods, base: &KeyBinding, platform: Platform) -> Option<&'static str> {
    let ch = match base {
        KeyBinding::Char(s) => s.to_ascii_lowercase(),
        _ => String::new(),
    };
    let code = match base {
        KeyBinding::Code(code) => Some(*code),
        _ => None,
    };
    let only = |want: Mods| mods == want;
    let cmd = Mods {
        sup: true,
        ..Mods::NONE
    };
    let alt = Mods {
        alt: true,
        ..Mods::NONE
    };
    let ctrl_alt = Mods {
        ctrl: true,
        alt: true,
        ..Mods::NONE
    };
    match platform {
        Platform::Mac => {
            let keys = ["q", "w", "h", "m"];
            let cmd_key = only(cmd) && keys.contains(&ch.as_str());
            let switch = mods.sup && matches!(code, Some(KeyCode::Tab | KeyCode::Space));
            (cmd_key || switch).then_some("macOS")
        }
        Platform::Desktop => {
            let alt_key = only(alt) && matches!(code, Some(KeyCode::F4 | KeyCode::Tab));
            let ctrl_alt_del = only(ctrl_alt) && code == Some(KeyCode::Delete);
            let win = only(cmd)
                && (["d", "e", "l", "r"].contains(&ch.as_str()) || code == Some(KeyCode::Tab));
            (alt_key || ctrl_alt_del || win).then_some("the system")
        }
        Platform::Web => {
            let browser =
                ["n", "q", "t", "w"].contains(&ch.as_str()) && (only(Mods::CTRL) || only(cmd));
            let tab = (mods.ctrl || mods.sup) && code == Some(KeyCode::Tab);
            let alt_key = only(alt) && code == Some(KeyCode::F4);
            (browser || tab || alt_key).then_some("the browser")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Bindings;
    use crate::settings::keys::CODES;
    use warlock_core::Dir;

    fn key(text: &str) -> KeyBinding {
        KeyBinding::from_text(text).expect("a key")
    }

    /// A press of `code` that stands for `logical`.
    fn input(code: KeyCode, logical: Key) -> KeyboardInput {
        KeyboardInput {
            key_code: code,
            logical_key: logical,
            state: bevy::input::ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        }
    }

    fn typed(code: KeyCode, ch: &str) -> KeyboardInput {
        input(code, Key::Character(ch.into()))
    }

    /// The modifiers a text form's prefix names, `"Ctrl+Alt+"`.
    fn mods(prefix: &str) -> Mods {
        Mods::split(&format!("{prefix}x")).0
    }

    #[test]
    fn the_text_form_round_trips() {
        for text in [
            "r",
            "+",
            "Ctrl++",
            "Ctrl+r",
            "Ctrl+R",
            "Alt+ArrowUp",
            "Shift+ArrowUp",
            "Super+s",
            "Ctrl+Alt+Shift+Super+F5",
            "Ctrl+Numpad5",
        ] {
            assert_eq!(key(text).to_text(), text);
        }
        // The other names of the modifiers read as the same keys, written the one way.
        assert_eq!(key("Cmd+s").to_text(), "Super+s");
        assert_eq!(key("command+s").to_text(), "Super+s");
        assert_eq!(key("Option+e").to_text(), "Alt+e");
        assert_eq!(key("Control+Alt+x").to_text(), "Ctrl+Alt+x");
        assert_eq!(key("Super+Ctrl+x").to_text(), "Ctrl+Super+x");
        // A plain key stays plain; Shift is no part of a character's binding.
        assert_eq!(key("Ctrl+r"), KeyBinding::chord(Mods::CTRL, key("r")));
        assert_eq!(key("Shift+r"), key("r"));
        assert_eq!(key("ArrowUp"), KeyBinding::Code(KeyCode::ArrowUp));
        // Esc cannot be bound with modifiers either, nor a modifier without a key.
        assert_eq!(KeyBinding::from_text("Ctrl+Escape"), None);
        assert_eq!(KeyBinding::from_text("Ctrl+"), None);
        assert_eq!(KeyBinding::from_text("Ctrl+Foo"), None);
    }

    #[test]
    fn the_platform_names_the_modifiers() {
        let all = key("Ctrl+Alt+Super+s");
        assert_eq!(all.display_on(Platform::Mac), "Ctrl+Alt+Cmd+s");
        assert_eq!(all.display_on(Platform::Desktop), "Ctrl+Alt+Super+s");
        assert_eq!(all.display_on(Platform::Web), "Ctrl+Alt+Super+s");
        assert_eq!(key("Shift+ArrowUp").display_on(Platform::Mac), "Shift+↑");
        assert_eq!(
            key("Ctrl+Numpad5").display_on(Platform::Desktop),
            "Ctrl+Num5"
        );
    }

    #[test]
    fn matching_is_exact() {
        let mut b = Bindings::default();
        b.bind_primary(KeyCommand::Look, key("Ctrl+r")).unwrap();
        let r = typed(KeyCode::KeyR, "r");
        assert_eq!(b.lookup(&r, Mods::NONE), Some(KeyCommand::Read));
        assert_eq!(b.lookup(&r, Mods::CTRL), Some(KeyCommand::Look));
        // `r` never fires while Ctrl or Super is held, nor Ctrl+r with more held.
        b.reset_command(KeyCommand::Look);
        assert_eq!(b.owner(&key("Ctrl+r")), None);
        assert_eq!(b.lookup(&r, Mods::CTRL), None);
        assert_eq!(b.lookup(&r, mods("Super+")), None);
        b.bind_primary(KeyCommand::Look, key("Ctrl+r")).unwrap();
        assert_eq!(b.lookup(&r, mods("Ctrl+Alt+")), None);
        // Alt+r is no `r` either (Windows and Linux type `r` with it).
        assert_eq!(b.lookup(&r, mods("Alt+")), None);
    }

    #[test]
    fn a_physical_key_reads_shift() {
        let mut b = Bindings::default();
        b.bind_primary(KeyCommand::Look, key("Shift+ArrowUp"))
            .unwrap();
        let up = input(KeyCode::ArrowUp, Key::ArrowUp);
        assert_eq!(b.lookup(&up, Mods::NONE), Some(KeyCommand::Move(Dir::N)));
        assert_eq!(b.lookup(&up, mods("Shift+")), Some(KeyCommand::Look));
        // A physical key with no Shift binding of its own still answers with Shift held: Tab
        // is Next target while aiming (fixed), which Shift reverses.
        let tab = input(KeyCode::Tab, Key::Tab);
        let next = crate::settings::fixed::read(
            &tab,
            mods("Shift+"),
            |_| None,
            crate::settings::fixed::aim,
        );
        assert_eq!(next, Some(crate::settings::fixed::AimKey::NextTarget));
        // Alt with an arrow is another key: Run (#1385).
        assert_eq!(b.lookup(&up, mods("Alt+")), Some(KeyCommand::Run(Dir::N)));
        assert_eq!(b.lookup(&up, mods("Ctrl+Alt+")), None);
    }

    #[test]
    fn alt_is_read_from_the_physical_key() {
        // Option+e on macOS types a dead key or `´`: the binding is Alt+e all the same.
        let option_e = typed(KeyCode::KeyE, "´");
        assert_eq!(
            press_binding(&option_e, mods("Alt+"), CODES),
            Some(key("Alt+e"))
        );
        let dead = input(KeyCode::KeyE, Key::Dead(Some('´')));
        assert_eq!(
            press_binding(&dead, mods("Alt+"), CODES),
            Some(key("Alt+e"))
        );
        // Ctrl's control characters say nothing either; Shift gives a letter its case.
        let ctrl_r = typed(KeyCode::KeyR, "\u{12}");
        assert_eq!(
            press_binding(&ctrl_r, Mods::CTRL, CODES),
            Some(key("Ctrl+r"))
        );
        let ctrl_shift_r = typed(KeyCode::KeyR, "r");
        assert_eq!(
            press_binding(&ctrl_shift_r, mods("Ctrl+Shift+"), CODES),
            Some(key("Ctrl+R"))
        );
        // A numpad key is its code, held with Ctrl too.
        let num5 = typed(KeyCode::Numpad5, "5");
        assert_eq!(
            press_binding(&num5, Mods::CTRL, CODES),
            Some(key("Ctrl+Numpad5"))
        );
    }

    #[test]
    fn a_character_alt_types_fires_its_plain_binding() {
        // `@` is Option+l on some Mac layouts, AltGr+q (Ctrl+Alt on Windows) on others: the
        // Character screen's `@` answers, unless a modified binding takes the press first.
        let mut b = Bindings::default();
        let at = typed(KeyCode::KeyL, "@");
        assert_eq!(b.lookup(&at, mods("Alt+")), Some(KeyCommand::Character));
        let altgr = typed(KeyCode::KeyQ, "@");
        assert_eq!(
            b.lookup(&altgr, mods("Ctrl+Alt+")),
            Some(KeyCommand::Character)
        );
        assert_eq!(b.lookup(&at, mods("Super+")), None);
        b.bind_primary(KeyCommand::Look, key("Alt+l")).unwrap();
        assert_eq!(b.lookup(&at, mods("Alt+")), Some(KeyCommand::Look));
    }

    #[test]
    fn a_modifier_alone_never_binds() {
        let keys = [
            KeyCode::ControlLeft,
            KeyCode::AltRight,
            KeyCode::SuperLeft,
            KeyCode::ShiftLeft,
        ];
        for code in keys {
            let press = input(code, Key::Control);
            assert_eq!(press_binding(&press, Mods::CTRL, CODES), None);
            assert!(press_candidates(&press, Mods::CTRL, CODES).is_empty());
        }
    }

    #[test]
    fn kept_shortcuts_are_refused() {
        let refused = |text: &str, platform| refusal(&key(text), KeyCommand::Read, platform);
        for text in ["Super+q", "Super+w", "Super+Tab", "Super+h"] {
            assert!(refused(text, Platform::Mac).is_some(), "{text}");
        }
        assert_eq!(
            refused("Super+q", Platform::Mac).as_deref(),
            Some("Cmd+q is kept by macOS")
        );
        for text in ["Alt+F4", "Alt+Tab", "Ctrl+Alt+Delete", "Super+l"] {
            assert!(refused(text, Platform::Desktop).is_some(), "{text}");
        }
        for text in ["Ctrl+w", "Ctrl+t", "Super+w", "Ctrl+Tab", "Alt+F4"] {
            assert!(refused(text, Platform::Web).is_some(), "{text}");
        }
        // Others bind, and so does a kept one's key on another system.
        for platform in [Platform::Mac, Platform::Desktop, Platform::Web] {
            for text in ["Ctrl+r", "Ctrl+s", "Ctrl+Numpad5", "Alt+ArrowUp", "r"] {
                assert_eq!(refused(text, platform), None, "{text}");
            }
        }
        assert_eq!(refused("Ctrl+w", Platform::Desktop), None);
        assert_eq!(refused("Alt+F4", Platform::Mac), None);
        // Only play commands take modifiers.
        let menu = refusal(&key("Ctrl+q"), KeyCommand::Back, Platform::Desktop);
        assert!(menu.is_some_and(|why| why.starts_with("Back (Esc always works) takes plain")));
        assert_eq!(
            refusal(&key("q"), KeyCommand::Back, Platform::Desktop),
            None
        );
    }

    #[test]
    fn default_runs_are_never_refused() {
        // Alt with every movement key (#1325, #1385), on every platform; no default key is refused.
        let b = Bindings::default();
        for dir in Dir::ALL {
            assert!(!b.keys_of(KeyCommand::Run(dir)).is_empty(), "{dir:?}");
        }
        let up = input(KeyCode::ArrowUp, Key::ArrowUp);
        assert_eq!(b.lookup(&up, Mods::ALT), Some(KeyCommand::Run(Dir::N)));
        assert_eq!(b.lookup(&up, Mods::NONE), Some(KeyCommand::Move(Dir::N)));
        let num3 = input(KeyCode::Numpad3, Key::Character("3".into()));
        assert_eq!(b.lookup(&num3, Mods::ALT), Some(KeyCommand::Run(Dir::SE)));
        // Ctrl is no default modifier any more: macOS keeps Ctrl+arrows for Spaces (#1385).
        assert_eq!(b.lookup(&up, Mods::CTRL), None);
        assert!(
            b.0.iter().all(|(key, _)| !key.mods().ctrl),
            "no Ctrl default"
        );
        // Option+s on a Mac types `ß` (a dead key on some layouts), Windows' Alt+s an `s`:
        // both are Search for a while, read from the physical key.
        let search = Some(KeyCommand::SearchLong);
        assert_eq!(b.lookup(&typed(KeyCode::KeyS, "ß"), Mods::ALT), search);
        assert_eq!(b.lookup(&typed(KeyCode::KeyS, "s"), Mods::ALT), search);
        let dead = input(KeyCode::KeyS, Key::Dead(None));
        assert_eq!(b.lookup(&dead, Mods::ALT), search);
        for platform in [Platform::Mac, Platform::Desktop, Platform::Web] {
            for (key, command) in &b.0 {
                assert_eq!(refusal(key, *command, platform), None, "{key:?}");
            }
        }
    }
}
