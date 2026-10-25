//! The keymap file, `warlock-keys.toml` (spec §6 "Rebinding", #159): every command's keys, by
//! context, with a format version, to back up, share, or carry between installs. It is plain
//! TOML, readable and editable:
//!
//! ```toml
//! version = 1
//!
//! [play] # In play
//! move_n = ["k", "Numpad8", "ArrowUp"]
//! bash = []
//!
//! [menu] # Menus and screens
//! menu_up = ["ArrowUp", "k"]
//! ```
//!
//! [`export`] writes every rebindable command (an empty list: no key). [`import`] changes only
//! the commands a file lists, as rebinding in Controls does: a key another command of the same
//! context has moves to the listed one, a key two listed commands claim goes to the first in the
//! controls screen's order, unknown commands and keys are skipped with a warning, and the
//! lockout rules hold (Esc cannot be bound; the vital commands keep a key). A file without a
//! `version` is version 0: command ids at the top level, or a `[bindings]` table as in
//! `settings.toml`.
//!
//! Where the file goes ([`KeymapFile`]): the config folder on the desktop (or the path given to
//! `--export-keys` / `--import-keys`, `main.rs`), and on the web a download and a file picker.

use crate::input::{Bindings, Context, KeyBinding, KeyCommand};
use crate::settings::keys::{REBINDABLE, VITAL};
use std::path::PathBuf;

/// The file's name in the config folder, and the web download's.
pub const FILE_NAME: &str = "warlock-keys.toml";

/// The format version [`export`] writes.
pub const VERSION: i64 = 1;

/// The keymap text for `bindings`: every rebindable command by context, in the controls
/// screen's order.
pub fn export(bindings: &Bindings) -> String {
    let mut out = String::from(
        "# Warlock keymap: every command's keys, by where they are read. A key is the character it\n\
         # types (\"k\", \">\", \"?\") or a key name (\"ArrowUp\", \"Numpad8\", \"Enter\", \"F5\"); an empty\n\
         # list leaves a command without a key. Import it in Controls or with\n\
         # `warlock --import-keys FILE`: commands it does not list keep their keys. Esc is fixed.\n",
    );
    out.push_str(&format!("version = {VERSION}\n"));
    for &context in Context::ALL {
        out.push_str(&format!("\n[{}] # {}\n", context.id(), context.title()));
        for &command in REBINDABLE.iter().filter(|c| c.context() == context) {
            let Some(id) = command.id() else { continue };
            let keys: Vec<String> = (bindings.keys_of(command).iter())
                .map(|k| toml::Value::String(k.to_text()).to_string())
                .collect();
            out.push_str(&format!("{id} = [{}]\n", keys.join(", ")));
        }
    }
    out
}

/// What an import did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// The commands whose keys changed, in the controls screen's order.
    pub changed: Vec<KeyCommand>,
    /// What was skipped or moved, a line each.
    pub warnings: Vec<String>,
}

impl Report {
    /// The report in a few words, for the controls screen and the command line: how many
    /// commands changed, then the problems, the first named.
    pub fn summary(&self, source: &str) -> Vec<String> {
        let changed = match self.changed.len() {
            0 => "no keys changed".to_string(),
            1 => format!("{} changed", self.changed[0].name()),
            n => format!("keys of {n} commands changed"),
        };
        let mut lines = vec![format!("Imported {source}: {changed}.")];
        match self.warnings.as_slice() {
            [] => {}
            [only] => lines.push(format!("! {only}")),
            [first, rest @ ..] => lines.push(format!("! {first} (and {} more)", rest.len())),
        }
        lines
    }
}

/// Reads a keymap into `bindings` (see the module docs). Fails only when the text is no TOML,
/// leaving `bindings` as they were.
pub fn import(bindings: &mut Bindings, text: &str) -> Result<Report, String> {
    let table: toml::Table =
        toml::from_str(text).map_err(|e| format!("not a keymap: {}", e.message()))?;
    let mut warnings = Vec::new();
    let version = match table.get("version") {
        None => 0,
        Some(v) => v.as_integer().unwrap_or_else(|| {
            warnings.push(format!(
                "version {v} is not a number; read as the current one"
            ));
            VERSION
        }),
    };
    if version > VERSION {
        warnings.push(format!(
            "written by a newer Warlock (version {version}); read as far as this one knows"
        ));
    }
    let listed = listed(&table, &mut warnings);
    let before = bindings.clone();
    apply(bindings, listed, &mut warnings);
    let changed = (REBINDABLE.iter().copied())
        .filter(|&c| before.keys_of(c) != bindings.keys_of(c))
        .collect();
    Ok(Report { changed, warnings })
}

/// The commands the file lists with their usable keys, in the controls screen's order; what
/// cannot be used is a warning.
fn listed(table: &toml::Table, warnings: &mut Vec<String>) -> Vec<(KeyCommand, Vec<KeyBinding>)> {
    // Context tables (and settings.toml's `[bindings]`) hold commands; so may the top level.
    let mut entries: Vec<(String, &str, &toml::Value)> = Vec::new();
    for (name, value) in table {
        let holds_commands =
            name == "bindings" || Context::ALL.iter().any(|c| c.id() == name.as_str());
        match value {
            _ if name == "version" => {}
            toml::Value::Table(inner) if holds_commands => {
                let place = |id: &String| format!("[{name}] {id}");
                entries.extend(inner.iter().map(|(id, v)| (place(id), id.as_str(), v)));
            }
            toml::Value::Array(_) => entries.push((name.clone(), name.as_str(), value)),
            _ => warnings.push(format!("`{name}` is not a context or a command; skipped")),
        }
    }
    let mut listed: Vec<(KeyCommand, Vec<KeyBinding>)> = Vec::new();
    for (place, id, value) in entries {
        let Some(command) = KeyCommand::from_id(id) else {
            warnings.push(format!("{place}: unknown command; skipped"));
            continue;
        };
        let Some(items) = value.as_array() else {
            warnings.push(format!("{place}: expected a list of keys; skipped"));
            continue;
        };
        let mut keys: Vec<KeyBinding> = Vec::new();
        for item in items {
            match item.as_str().and_then(KeyBinding::from_text) {
                Some(key) if keys.contains(&key) => {}
                Some(key) => keys.push(key),
                None => warnings.push(format!("{place}: {item} is no key that can be bound")),
            }
        }
        if keys.is_empty() && !items.is_empty() {
            warnings.push(format!("{place}: no usable keys; left as it was"));
        } else if keys.is_empty() && VITAL.contains(&command) {
            warnings.push(format!(
                "{place}: needed to reach the controls; left as it was"
            ));
        } else if listed.iter().any(|(c, _)| *c == command) {
            warnings.push(format!("{place}: listed twice; the first kept"));
        } else {
            listed.push((command, keys));
        }
    }
    listed.sort_by_key(|(c, _)| REBINDABLE.iter().position(|r| r == c));
    listed
}

/// Gives each listed command its keys, taking them from the commands of its context that have
/// them, as binding in Controls does.
fn apply(
    bindings: &mut Bindings,
    listed: Vec<(KeyCommand, Vec<KeyBinding>)>,
    warnings: &mut Vec<String>,
) {
    let pending: Vec<KeyCommand> = listed.iter().map(|(c, _)| *c).collect();
    for (i, (command, keys)) in listed.into_iter().enumerate() {
        let (done, later) = (&pending[..i], &pending[i + 1..]);
        let mut kept = Vec::new();
        for key in keys {
            let shown = key.display();
            match bindings.owner_in(command.context(), &key) {
                None => kept.push(key),
                Some(owner) if owner == command => kept.push(key),
                Some(owner) if done.contains(&owner) => warnings.push(format!(
                    "{shown} is listed for both {} and {}; kept for {}",
                    owner.name(),
                    command.name(),
                    owner.name()
                )),
                // A command listed later gets its own keys anyway.
                Some(owner) if !later.contains(&owner) && bindings.keeps(owner, &key) => {
                    warnings.push(format!(
                        "{shown} is the only key for {}; not moved to {}",
                        owner.name(),
                        command.name()
                    ));
                }
                Some(owner) => {
                    bindings.0.retain(|(k, c)| !(*c == owner && *k == key));
                    if !later.contains(&owner) && bindings.keys_of(owner).is_empty() {
                        warnings.push(format!(
                            "{} has no key now ({shown} went to {})",
                            owner.name(),
                            command.name()
                        ));
                    }
                    kept.push(key);
                }
            }
        }
        let same = bindings.keys_of(command).into_iter().eq(kept.iter());
        if same || (kept.is_empty() && VITAL.contains(&command)) {
            continue;
        }
        bindings.set_keys(command, kept);
    }
    bindings.ensure_vital();
}

/// Where Export keys and Import keys go in this launch.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum KeymapFile {
    /// Nowhere (captures and replays without `--config-dir`): both say so.
    #[default]
    Nowhere,
    /// This file (desktop: `warlock-keys.toml` in the config folder).
    Path(PathBuf),
    /// The browser: Export downloads the file, Import opens a file picker.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Browser,
}

impl KeymapFile {
    /// Writes `bindings` there. The message says where it went.
    pub fn export(&self, bindings: &Bindings) -> Result<String, String> {
        let text = export(bindings);
        match self {
            KeymapFile::Nowhere => Err("Keys are not stored in this launch.".into()),
            KeymapFile::Path(path) => {
                let write = || -> std::io::Result<()> {
                    if let Some(dir) = path.parent() {
                        std::fs::create_dir_all(dir)?;
                    }
                    std::fs::write(path, text)
                };
                write()
                    .map(|()| format!("Keys exported to {}.", path.display()))
                    .map_err(|e| format!("Could not write {}: {e}", path.display()))
            }
            KeymapFile::Browser => {
                web::download(FILE_NAME, &text).map(|()| format!("Keys saved as {FILE_NAME}."))
            }
        }
    }

    /// Reads the keymap there into `bindings`: the report's lines, or `Ok(None)` when the
    /// browser's file picker opened (its file arrives later: [`take_picked`]).
    pub fn import(&self, bindings: &mut Bindings) -> Result<Option<Vec<String>>, String> {
        match self {
            KeymapFile::Nowhere => Err("Keys are not stored in this launch.".into()),
            KeymapFile::Path(path) => {
                let text = std::fs::read_to_string(path).map_err(|e| match e.kind() {
                    std::io::ErrorKind::NotFound => format!(
                        "No keymap at {}: export one first, or put one there.",
                        path.display()
                    ),
                    _ => format!("Could not read {}: {e}", path.display()),
                })?;
                let report = import(bindings, &text)?;
                Ok(Some(report.summary(FILE_NAME)))
            }
            KeymapFile::Browser => web::pick().map(|()| None),
        }
    }
}

/// `--import-keys FILE` and `--export-keys FILE` (after `--reset-keys`, if given): FILE's keymap
/// into the settings file, which is saved, then the settings' keymap to FILE. The text to print.
#[cfg(not(target_arch = "wasm32"))]
pub fn command(args: &crate::cli::Args) -> Result<String, String> {
    let (store, settings, mut bindings) = super::load_for(args);
    let mut out = String::new();
    if let Some(path) = &args.import_keys {
        let shown = path.display();
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("cannot read {shown}: {e}"))?;
        let report = import(&mut bindings, &text).map_err(|e| format!("{shown}: {e}"))?;
        let mut summary = report.summary(&shown.to_string());
        summary.truncate(1);
        summary.extend(report.warnings.iter().map(|w| format!("  {w}")));
        store.save(&settings, &bindings)?;
        summary.push(match store.place() {
            Some(place) => format!("Saved in {place}."),
            None => "Not saved: settings are not stored in this launch.".into(),
        });
        out.extend(summary.into_iter().map(|line| line + "\n"));
    }
    if let Some(path) = &args.export_keys {
        let shown = path.display();
        std::fs::write(path, export(&bindings))
            .map_err(|e| format!("cannot write {shown}: {e}"))?;
        out.push_str(&format!("Wrote the keymap to {shown}.\n"));
    }
    Ok(out)
}

/// The text of a keymap file the browser's picker delivered since the last call, if any (never
/// on the desktop).
pub fn take_picked() -> Option<Result<String, String>> {
    web::take_picked()
}

#[cfg(not(target_arch = "wasm32"))]
mod web {
    //! No browser on the desktop.

    pub fn download(_name: &str, _text: &str) -> Result<(), String> {
        Err("There is no browser to download with.".into())
    }

    pub fn pick() -> Result<(), String> {
        Err("There is no browser to pick a file with.".into())
    }

    pub fn take_picked() -> Option<Result<String, String>> {
        None
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    //! The download (a blob URL on a clicked link) and the file picker (a clicked file input,
    //! whose file is read as text and kept for [`take_picked`]). Both run in the frame after a
    //! key press, within the browser's user-activation window.

    use std::cell::RefCell;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::js_sys;

    thread_local! {
        /// The picked file's text (or why it could not be read), until a frame takes it.
        static PICKED: RefCell<Option<Result<String, String>>> = const { RefCell::new(None) };
        /// The last download's blob URL, revoked at the next one.
        static LAST_URL: RefCell<Option<String>> = const { RefCell::new(None) };
    }

    fn js(e: JsValue) -> String {
        format!("{e:?}")
    }

    fn document() -> Result<web_sys::Document, String> {
        (web_sys::window().and_then(|w| w.document())).ok_or_else(|| "no page".to_string())
    }

    pub fn download(name: &str, text: &str) -> Result<(), String> {
        let parts = js_sys::Array::of1(&JsValue::from_str(text));
        let options = web_sys::BlobPropertyBag::new();
        options.set_type("text/plain;charset=utf-8");
        let blob =
            web_sys::Blob::new_with_str_sequence_and_options(&parts, &options).map_err(js)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(js)?;
        let link: web_sys::HtmlAnchorElement = (document()?.create_element("a").map_err(js)?)
            .dyn_into()
            .map_err(|_| "no link".to_string())?;
        link.set_href(&url);
        link.set_download(name);
        link.click();
        if let Some(old) = LAST_URL.replace(Some(url)) {
            let _ = web_sys::Url::revoke_object_url(&old);
        }
        Ok(())
    }

    pub fn pick() -> Result<(), String> {
        let input: web_sys::HtmlInputElement = (document()?.create_element("input").map_err(js)?)
            .dyn_into()
            .map_err(|_| "no file input".to_string())?;
        input.set_type("file");
        input.set_accept(".toml,text/plain");
        let chosen = input.clone();
        let on_change = Closure::once(move || {
            let Some(file) = chosen.files().and_then(|files| files.get(0)) else {
                return;
            };
            let on_text = Closure::once(move |text: JsValue| {
                let text = text.as_string().ok_or_else(|| "not text".to_string());
                PICKED.set(Some(text));
            });
            let on_error = Closure::once(move |e: JsValue| PICKED.set(Some(Err(js(e)))));
            let _ = file.text().then2(&on_text, &on_error);
            on_text.forget();
            on_error.forget();
        });
        input.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        on_change.forget();
        input.click();
        Ok(())
    }

    pub fn take_picked() -> Option<Result<String, String>> {
        PICKED.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::KeyCode;
    use warlock_core::Dir;

    fn ch(s: &str) -> KeyBinding {
        KeyBinding::Char(s.into())
    }

    /// Bindings with a few keys changed in different contexts.
    fn customized() -> Bindings {
        let mut b = Bindings::default();
        b.bind_primary(KeyCommand::Move(Dir::N), ch("w")).unwrap();
        b.bind_primary(KeyCommand::MenuUp, ch("w")).unwrap();
        b.bind_primary(KeyCommand::Loose, ch("f")).unwrap();
        b.bind_primary(KeyCommand::Quit, ch("Q")).unwrap();
        b.bind_primary(KeyCommand::Wait, ch("k")).unwrap(); // Kick is left without a key.
        b
    }

    #[test]
    fn export_then_import_is_identical() {
        let original = customized();
        let text = export(&original);
        // Every rebindable command is in the file, each in its context's table.
        let table: toml::Table = toml::from_str(&text).unwrap();
        assert_eq!(table["version"].as_integer(), Some(VERSION));
        for &command in REBINDABLE {
            let context = table[command.context().id()].as_table().unwrap();
            assert!(context.contains_key(command.id().unwrap()), "{command:?}");
        }
        // Into the defaults: everything as exported, nothing to say.
        let mut fresh = Bindings::default();
        let report = import(&mut fresh, &text).unwrap();
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(fresh.to_map(), original.to_map());
        assert!(fresh.keys_of(KeyCommand::Kick).is_empty());
        assert!(report.changed.contains(&KeyCommand::Kick));
        // Into itself: nothing changes.
        let mut same = original.clone();
        let report = import(&mut same, &text).unwrap();
        assert_eq!(report, Report::default());
        assert_eq!(same.0, original.0);
        // Into scrambled keys: back to the exported ones.
        let mut scrambled = Bindings::default();
        for (command, key) in [
            (KeyCommand::Wait, "k"),
            (KeyCommand::Descend, "w"),
            (KeyCommand::MenuDown, "w"),
            (KeyCommand::Yes, "Q"),
        ] {
            scrambled.bind_primary(command, ch(key)).unwrap();
        }
        let report = import(&mut scrambled, &text).unwrap();
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(scrambled.to_map(), original.to_map());
    }

    #[test]
    fn a_partial_file_changes_only_what_it_lists() {
        let mut b = customized();
        let before = b.clone();
        let report = import(&mut b, "version = 1\n[play]\ndescend = [\"D\"]\n").unwrap();
        assert_eq!(report.changed, [KeyCommand::Descend]);
        assert!(report.warnings.is_empty());
        assert_eq!(b.keys_of(KeyCommand::Descend), vec![&ch("D")]);
        for &command in REBINDABLE.iter().filter(|&&c| c != KeyCommand::Descend) {
            assert_eq!(b.keys_of(command), before.keys_of(command), "{command:?}");
        }
    }

    #[test]
    fn unknown_commands_and_keys_are_skipped_with_a_warning() {
        let mut b = Bindings::default();
        let text = r#"
            version = 1
            colour = "blue"
            [play]
            ghost = ["x"]
            open = ["Banana", "O"]
            close = ["Banana"]
            kick = "K"
            [menu]
            confirm = ["Escape"]
            menu_up = []
        "#;
        let report = import(&mut b, text).unwrap();
        assert_eq!(report.changed, [KeyCommand::Open]);
        assert_eq!(b.keys_of(KeyCommand::Open), vec![&ch("O")]);
        assert_eq!(b.keys_of(KeyCommand::Close), vec![&ch("c")]);
        for needle in [
            "colour",
            "[play] ghost: unknown command",
            "[play] open: \"Banana\"",
            "[play] close: no usable keys",
            "[play] kick: expected a list",
            "[menu] confirm: no usable keys",
            "[menu] menu_up: needed to reach the controls",
        ] {
            assert!(
                report.warnings.iter().any(|w| w.contains(needle)),
                "{needle}: {:?}",
                report.warnings
            );
        }
        // Esc stays Cancel's, and Choose keeps its keys.
        assert_eq!(
            b.keys_of(KeyCommand::Cancel),
            vec![&KeyBinding::Code(KeyCode::Escape)]
        );
        assert_eq!(b.keys_of(KeyCommand::Confirm).len(), 2);
    }

    #[test]
    fn conflicts_are_reported_and_keys_move_within_a_context_only() {
        let mut b = Bindings::default();
        let text = r#"
            version = 1
            [play]
            wait = ["x"]
            look = ["x", "Y"]
            open = ["k"]
            [menu]
            mark = ["ArrowUp"]
            [aim]
            loose = ["o"]
        "#;
        let report = import(&mut b, text).unwrap();
        // Two listed commands claim `x`: the first in the controls screen's order keeps it.
        assert_eq!(b.owner(&ch("x")), Some(KeyCommand::Wait));
        assert_eq!(b.keys_of(KeyCommand::Look), vec![&ch("Y")]);
        // `k` moves from Kick, which has no key left; `o` while aiming leaves Open alone.
        assert!(b.keys_of(KeyCommand::Kick).is_empty());
        assert_eq!(b.owner(&ch("o")), None, "Open took k instead");
        assert_eq!(b.owner_in(Context::Aim, &ch("o")), Some(KeyCommand::Loose));
        // ↑ moves from Menu up, which keeps `k`; never from Move north in play.
        let up = KeyBinding::Code(KeyCode::ArrowUp);
        assert_eq!(b.owner_in(Context::Menu, &up), Some(KeyCommand::Mark));
        assert_eq!(b.keys_of(KeyCommand::MenuUp), vec![&ch("k")]);
        assert_eq!(b.owner(&up), Some(KeyCommand::Move(Dir::N)));
        assert!(b.conflicts().is_empty());
        for needle in [
            "x is listed for both Wait a turn and Look around",
            "Kick has no key now",
        ] {
            assert!(
                report.warnings.iter().any(|w| w.contains(needle)),
                "{needle}: {:?}",
                report.warnings
            );
        }
    }

    #[test]
    fn the_last_key_of_a_vital_command_stays_unless_it_is_listed_too() {
        let mut b = Bindings::default();
        b.bind_primary(KeyCommand::Erase, KeyBinding::Code(KeyCode::Enter))
            .unwrap();
        // Choose has only NumEnter left: Mark may not take it.
        let numpad = KeyBinding::Code(KeyCode::NumpadEnter);
        let report = import(&mut b, "[menu]\nmark = [\"NumpadEnter\"]\n").unwrap();
        assert_eq!(b.keys_of(KeyCommand::Confirm), vec![&numpad]);
        assert!(report.warnings[0].contains("only key for Choose"));
        // Swapping it with a command listed too is fine.
        let text = "[menu]\nmark = [\"NumpadEnter\"]\nconfirm = [\"Space\"]\n";
        let report = import(&mut b, text).unwrap();
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(b.keys_of(KeyCommand::Mark), vec![&numpad]);
        assert_eq!(
            b.keys_of(KeyCommand::Confirm),
            vec![&KeyBinding::Code(KeyCode::Space)]
        );
    }

    #[test]
    fn older_and_newer_versions_are_read() {
        // Version 0: ids at the top level, or settings.toml's `[bindings]` table.
        let mut b = Bindings::default();
        let report = import(&mut b, "descend = [\"D\"]\n").unwrap();
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(b.keys_of(KeyCommand::Descend), vec![&ch("D")]);
        let settings =
            crate::settings::to_toml(&crate::settings::Settings::default(), &customized());
        let mut b = Bindings::default();
        let report = import(&mut b, &settings).unwrap();
        assert_eq!(b.to_map(), customized().to_map());
        // The settings' other values are no commands: each is said once.
        assert!(report.warnings.iter().any(|w| w.contains("font_px")));
        // A newer version: read as far as this one knows, and said.
        let mut b = Bindings::default();
        let report = import(&mut b, "version = 9\n[play]\ndescend = [\"D\"]\n").unwrap();
        assert_eq!(b.keys_of(KeyCommand::Descend), vec![&ch("D")]);
        assert!(report.warnings[0].contains("newer"));
        // Not TOML at all: an error, and nothing changes.
        let mut b = Bindings::default();
        assert!(import(&mut b, "version = = 1").is_err());
        assert_eq!(b.0, Bindings::default().0);
    }

    #[test]
    fn the_report_is_short() {
        let mut report = Report {
            changed: vec![KeyCommand::Wait],
            warnings: Vec::new(),
        };
        assert_eq!(
            report.summary("x.toml"),
            ["Imported x.toml: Wait a turn changed."]
        );
        report.changed.push(KeyCommand::Open);
        report.warnings = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(
            report.summary("x.toml"),
            [
                "Imported x.toml: keys of 2 commands changed.",
                "! a (and 2 more)"
            ]
        );
    }

    #[test]
    fn the_file_on_disk_round_trips() {
        let dir = std::env::temp_dir().join(format!("warlock-keymap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = KeymapFile::Path(dir.join(FILE_NAME));
        let mut b = Bindings::default();
        // Nothing there yet: said, nothing changes.
        let missing = file.import(&mut b).unwrap_err();
        assert!(missing.starts_with("No keymap at"), "{missing}");
        let said = file.export(&customized()).unwrap();
        assert!(said.starts_with("Keys exported to"), "{said}");
        let lines = file.import(&mut b).unwrap().expect("read at once");
        assert!(
            lines[0].starts_with("Imported warlock-keys.toml"),
            "{lines:?}"
        );
        assert_eq!(b.to_map(), customized().to_map());
        assert!(KeymapFile::Nowhere.export(&b).is_err());
    }

    #[test]
    fn the_shipped_default_keymap_is_the_defaults() {
        // `assets/keymaps/default.toml`: regenerate with
        // `WARLOCK_UPDATE_KEYMAPS=1 cargo test -p warlock keymap` after changing a default key.
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/keymaps/default.toml"
        );
        let defaults = export(&Bindings::default());
        if std::env::var_os("WARLOCK_UPDATE_KEYMAPS").is_some() {
            std::fs::write(path, &defaults).unwrap();
        }
        let shipped = std::fs::read_to_string(path).expect("assets/keymaps/default.toml");
        // A command added since is not listed (an import keeps its keys); a changed default is.
        let mut b = Bindings::default();
        let report = import(&mut b, &shipped).unwrap();
        assert_eq!(report, Report::default(), "regenerate {path}");
    }
}
