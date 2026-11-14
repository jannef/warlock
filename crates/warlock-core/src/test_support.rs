//! Helpers shared by the core's tests (#1509): the built-in content, loaded once per test binary.

use crate::content::Content;
use std::sync::{Arc, LazyLock};

/// The built-in content (`assets/`), loaded once for every test that shares it.
pub(crate) static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("built-in content")));

/// The shared built-in content ([`CONTENT`]).
pub(crate) fn builtin() -> Arc<Content> {
    CONTENT.clone()
}
