//! Helpers for reading and writing `$APP_CONFIG/keyhook.json`.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use keyhook_model::HookRule;
use tauri::{AppHandle, Manager};

/// Return the absolute path of the persistent config file.
///
/// The file lives under Tauri's *app-config* directory:
/// ```text
/// <platform-config-root>/keyhook.json
/// ```
pub fn config_path(app: &AppHandle) -> Result<PathBuf> {
    app.path()
        .app_config_dir()
        .context("failed to get app_config_dir from Tauri")
        .map(|p| p.join("keyhook.json"))
}

/// Load all stored rules.  
/// On any error an **empty list** is returned (non-fatal).
pub fn load_rules(app: &AppHandle) -> Vec<HookRule> {
    let path = match config_path(app) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("config path error: {e:#}");
            return Vec::new();
        }
    };

    fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))
        .and_then(|s| {
            serde_json::from_str::<Vec<HookRule>>(&s)
                .context("parsing keyhook.json to Vec<HookRule>")
        })
        .unwrap_or_else(|e| {
            eprintln!("load_rules error: {e:#}");
            Vec::new()
        })
}

/// Persist the rule list as pretty-printed JSON.
pub fn save_rules(app: &AppHandle, rules: &[HookRule]) -> Result<()> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating dir {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(rules).context("serialising HookRule list to JSON")?;
    fs::write(&path, json).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}
