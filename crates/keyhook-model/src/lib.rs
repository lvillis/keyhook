//! **Shared data models** consumed by both the Yew (frontend) and
//! Tauri (backend) crates.

use serde::{Deserialize, Serialize};

/// HTTP verbs supported by KeyHook.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

/// A mapping from a *global shortcut* to a *webhook endpoint*.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HookRule {
    /// `None` when created on the client; a `Uuid` is assigned server-side.
    pub id: Option<String>,
    /// Example: `"Ctrl+Shift+F"`.
    pub hotkey: String,
    pub method: HttpMethod,
    pub url: String,
    /// Optional JSON request body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Whether the rule is active. Default: `true`.
    #[serde(default = "enabled_true")]
    pub enabled: bool,
}

#[inline]
fn enabled_true() -> bool {
    true
}
