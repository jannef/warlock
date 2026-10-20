//! Where the game's persistent files live: a directory natively, `window.localStorage` on the web
//! (spec §7). Callers name files relative to the backend (`saves/tamsin-1.sav`,
//! `snapshots/tamsin-1.sav`, `settings.toml`); the backend decides what that means, and lists
//! the files in a folder. Errors are plain `io::Error`s with `NotFound` for a missing file, so
//! callers treat both backends alike.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// A place to keep named byte blobs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backend {
    /// Files in a directory (created on first write).
    Dir(PathBuf),
    /// Keys `warlock/<name>` in the browser's local storage, bytes base64-encoded.
    #[cfg(target_arch = "wasm32")]
    Browser,
}

impl Backend {
    /// Reads a file. A missing one is `ErrorKind::NotFound`.
    pub fn read(&self, name: &str) -> io::Result<Vec<u8>> {
        match self {
            Self::Dir(dir) => std::fs::read(dir.join(name)),
            #[cfg(target_arch = "wasm32")]
            Self::Browser => web::get(name)?
                .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
                .and_then(|text| {
                    decode_base64(&text)
                        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "bad base64"))
                }),
        }
    }

    /// Writes a file. Natively atomic (temp file, then rename); in the browser a single
    /// `setItem`, which either stores the whole value or fails (quota, storage disabled).
    pub fn write(&self, name: &str, bytes: &[u8]) -> io::Result<()> {
        match self {
            Self::Dir(dir) => write_atomic(&dir.join(name), bytes),
            #[cfg(target_arch = "wasm32")]
            Self::Browser => web::set(name, &encode_base64(bytes)),
        }
    }

    /// Moves a file, replacing the target. Never loses the source if the target cannot be written.
    pub fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        match self {
            Self::Dir(dir) => std::fs::rename(dir.join(from), dir.join(to)),
            #[cfg(target_arch = "wasm32")]
            Self::Browser => {
                let text = web::get(from)?.ok_or(io::ErrorKind::NotFound)?;
                web::set(to, &text)?;
                web::remove(from)
            }
        }
    }

    /// Deletes a file; one that is not there is no error.
    pub fn remove(&self, name: &str) -> io::Result<()> {
        match self {
            Self::Dir(dir) => match std::fs::remove_file(dir.join(name)) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
                other => other,
            },
            #[cfg(target_arch = "wasm32")]
            Self::Browser => web::remove(name),
        }
    }

    /// Whether the file exists.
    #[cfg(test)]
    pub fn exists(&self, name: &str) -> bool {
        match self {
            Self::Dir(dir) => dir.join(name).is_file(),
            #[cfg(target_arch = "wasm32")]
            Self::Browser => web::get(name).is_ok_and(|v| v.is_some()),
        }
    }

    /// The names of the files in folder `dir` (`saves`), sorted; not those in folders inside it.
    /// A folder that is not there holds none.
    pub fn list(&self, dir: &str) -> io::Result<Vec<String>> {
        let mut names = match self {
            Self::Dir(root) => match std::fs::read_dir(root.join(dir)) {
                Ok(entries) => {
                    let mut names = Vec::new();
                    for entry in entries {
                        let entry = entry?;
                        if entry.file_type()?.is_file()
                            && let Some(name) = entry.file_name().to_str()
                        {
                            names.push(name.to_string());
                        }
                    }
                    names
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
                Err(e) => return Err(e),
            },
            #[cfg(target_arch = "wasm32")]
            Self::Browser => files_in(web::keys()?, dir),
        };
        names.sort();
        Ok(names)
    }

    /// The file's location for messages.
    pub fn describe(&self, name: &str) -> String {
        match self {
            Self::Dir(dir) => dir.join(name).display().to_string(),
            #[cfg(target_arch = "wasm32")]
            Self::Browser => format!("browser storage ({BROWSER_PREFIX}{name})"),
        }
    }
}

/// Browser storage's key prefix: the keys are `warlock/settings.toml`, `warlock/saves/a.sav`,
/// and so on.
const BROWSER_PREFIX: &str = "warlock/";

/// The names of the files in folder `dir` among browser storage keys (`warlock/saves/a.sav`):
/// those under `warlock/<dir>/`, not in a folder inside it, without the prefix.
#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn files_in(keys: impl IntoIterator<Item = String>, dir: &str) -> Vec<String> {
    let prefix = format!("{BROWSER_PREFIX}{dir}/");
    (keys.into_iter())
        .filter_map(|key| key.strip_prefix(&prefix).map(str::to_string))
        .filter(|name| !name.is_empty() && !name.contains('/'))
        .collect()
}

/// Writes `bytes` to `path` via a temp file in the same directory and a rename, so a crash never
/// leaves a half-written file.
fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let result = (|| {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding. (Tiny on purpose: saves are a few KB.)
#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn encode_base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Decodes [`encode_base64`] output; `None` on any malformed input.
#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let text = text.trim_end_matches('=').as_bytes();
    if text.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    for chunk in text.chunks(4) {
        let mut n = 0u32;
        for (i, c) in chunk.iter().enumerate() {
            let v = B64.iter().position(|b| b == c)? as u32;
            n |= v << (18 - 6 * i);
        }
        for i in 0..chunk.len() - 1 {
            out.push((n >> (16 - 8 * i)) as u8);
        }
    }
    Some(out)
}

#[cfg(target_arch = "wasm32")]
pub mod web {
    //! `window.localStorage` access and the page-hide hook.

    use super::BROWSER_PREFIX as PREFIX;
    use std::cell::{Cell, RefCell};
    use std::io;
    use warlock_core::world::WorldState;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    fn storage() -> io::Result<web_sys::Storage> {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or_else(|| io::Error::other("browser storage is unavailable"))
    }

    fn js_err(e: wasm_bindgen::JsValue) -> io::Error {
        io::Error::other(format!("{e:?}"))
    }

    /// Whether local storage can be used at all (it throws or is absent in some privacy modes).
    pub fn available() -> bool {
        storage().is_ok()
    }

    pub fn get(name: &str) -> io::Result<Option<String>> {
        storage()?
            .get_item(&format!("{PREFIX}{name}"))
            .map_err(js_err)
    }

    pub fn set(name: &str, value: &str) -> io::Result<()> {
        storage()?
            .set_item(&format!("{PREFIX}{name}"), value)
            .map_err(js_err)
    }

    pub fn remove(name: &str) -> io::Result<()> {
        storage()?
            .remove_item(&format!("{PREFIX}{name}"))
            .map_err(js_err)
    }

    /// Every key in local storage (this site's, the game's and any other).
    pub fn keys() -> io::Result<Vec<String>> {
        let storage = storage()?;
        let len = storage.length().map_err(js_err)?;
        Ok((0..len)
            .filter_map(|i| storage.key(i).ok().flatten())
            .collect())
    }

    /// A run's state waiting for the page to be hidden, and where it goes.
    pub struct Pending {
        pub state: WorldState,
        /// The content fingerprint of the content it was played with.
        pub fingerprint: u64,
        /// The save's name (`saves/<id>.sav`).
        pub file: String,
        /// A file written beside it, by name, and its text: the save list's summary.
        pub beside: (String, String),
    }

    thread_local! {
        /// The run's state as of the last turn, waiting for the page to be hidden. Frames stop
        /// running in a hidden tab, so the listener cannot ask the world for a save; it encodes
        /// and writes this copy instead. Only the clone is made per turn: encoding costs
        /// milliseconds on big levels.
        static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) };
        static HOOKED: Cell<bool> = const { Cell::new(false) };
    }

    /// Replaces (or with `None` clears) the state saved when the page is hidden or closed.
    pub fn set_pending(pending: Option<Pending>) {
        PENDING.with(|p| *p.borrow_mut() = pending);
    }

    fn flush() {
        PENDING.with(|p| {
            let Some(pending) = p.borrow().as_ref().map(|pending| {
                let bytes = warlock_core::save::encode_state(&pending.state, pending.fingerprint);
                let (beside, text) = &pending.beside;
                set(&pending.file, &super::encode_base64(&bytes))
                    .and_then(|()| set(beside, &super::encode_base64(text.as_bytes())))
            }) else {
                return;
            };
            if let Err(e) = pending {
                bevy::log::warn!("could not save on page hide: {e}");
            }
        });
    }

    /// Reloads the page (a web page cannot quit; it comes back as a fresh main menu).
    pub fn reload() {
        if let Some(window) = web_sys::window() {
            let _ = window.location().reload();
        }
    }

    /// Installs the `visibilitychange` (to hidden) and `pagehide` listeners once.
    pub fn hook_page_hide() {
        if HOOKED.replace(true) {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let on_pagehide = Closure::<dyn FnMut()>::new(flush);
        let _ = window
            .add_event_listener_with_callback("pagehide", on_pagehide.as_ref().unchecked_ref());
        on_pagehide.forget();
        if let Some(document) = window.document() {
            let doc = document.clone();
            let on_visibility = Closure::<dyn FnMut()>::new(move || {
                if doc.hidden() {
                    flush();
                }
            });
            let _ = document.add_event_listener_with_callback(
                "visibilitychange",
                on_visibility.as_ref().unchecked_ref(),
            );
            on_visibility.forget();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips_every_length() {
        for len in 0..40u8 {
            let bytes: Vec<u8> = (0..len)
                .map(|i| i.wrapping_mul(37).wrapping_add(200))
                .collect();
            let text = encode_base64(&bytes);
            assert_eq!(decode_base64(&text).as_deref(), Some(&bytes[..]), "{len}");
        }
    }

    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(encode_base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(encode_base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(encode_base64(b"foob"), "Zm9vYg==");
        assert_eq!(decode_base64("Zm9vYg=="), Some(b"foob".to_vec()));
        assert_eq!(decode_base64("Zm9v!g=="), None);
        assert_eq!(decode_base64("Z"), None);
    }

    #[test]
    fn dir_backend_reads_writes_and_renames() {
        let dir = std::env::temp_dir().join(format!("warlock-storage-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let b = Backend::Dir(dir);
        assert!(!b.exists("a/x"));
        assert_eq!(
            b.read("a/x").expect_err("missing").kind(),
            io::ErrorKind::NotFound
        );
        b.write("a/x", b"hi").expect("write");
        assert!(b.exists("a/x"));
        b.rename("a/x", "a/y").expect("rename");
        assert!(!b.exists("a/x"));
        assert_eq!(b.read("a/y").expect("read"), b"hi");
    }

    #[test]
    fn dir_backend_lists_a_folders_files() {
        let dir = std::env::temp_dir().join(format!("warlock-list-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let b = Backend::Dir(dir.clone());
        assert_eq!(b.list("saves").expect("none"), Vec::<String>::new());
        b.write("saves/b.sav", b"b").expect("write");
        b.write("saves/a.sav", b"a").expect("write");
        b.write("saves/deeper/c.sav", b"c").expect("write");
        b.write("snapshots/a.sav", b"a").expect("write");
        assert_eq!(b.list("saves").expect("list"), ["a.sav", "b.sav"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The browser backend lists a folder from local storage's keys (the web build's half is
    /// `web::keys`, which needs a browser).
    #[test]
    fn browser_keys_list_a_folders_files() {
        let keys = [
            "warlock/saves/tamsin-1.sav",
            "warlock/saves/tamsin-1.toml",
            "warlock/saves/old/x.sav",
            "warlock/saves/",
            "warlock/snapshots/tamsin-1.sav",
            "warlock/settings.toml",
            "another-site/saves/y.sav",
        ]
        .map(String::from);
        assert_eq!(
            files_in(keys.clone(), "saves"),
            ["tamsin-1.sav", "tamsin-1.toml"]
        );
        assert_eq!(files_in(keys, "snapshots"), ["tamsin-1.sav"]);
    }
}
