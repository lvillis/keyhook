//! **KeyHook backend** — Tauri 2.5.1
//! *Global shortcut · tray · hide-on-close · persistent config*
//! Logging is powered by `tracing`.

use std::time::Duration;

use anyhow::{Context, Result as AnyResult};
use keyhook_model::{HookRule, HttpMethod};
use once_cell::sync::Lazy;
use reqwest::{Client, Method, header::CONTENT_TYPE};
use tauri::{
    AppHandle, Builder, Emitter, Manager, RunEvent, WindowEvent, Wry, async_runtime,
    generate_context,
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

mod config;
mod logger;

/* ──────────────── HTTP ──────────────── */

static HTTP: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .expect("HTTP client must build at start-up")
});

fn to_method(m: &HttpMethod) -> Method {
    match m {
        HttpMethod::GET => Method::GET,
        HttpMethod::POST => Method::POST,
        HttpMethod::PUT => Method::PUT,
        HttpMethod::DELETE => Method::DELETE,
        HttpMethod::PATCH => Method::PATCH,
    }
}

/// Fire a webhook and return `Ok(true)` on any **2xx** response.
async fn fire(rule: &HookRule) -> AnyResult<bool> {
    let mut req = HTTP.request(to_method(&rule.method), &rule.url);

    if let Some(b) = &rule.body {
        req = req
            // Explicit JSON content type when request has a body
            .header(CONTENT_TYPE, "application/json")
            .body(b.clone());
    }

    tracing::debug!(
        method = ?rule.method,
        url    = %rule.url,
        body   = %rule.body.as_deref().unwrap_or(""),
        "sending request",
    );

    let resp = req
        .send()
        .await
        .with_context(|| format!("sending {:?} {}", rule.method, rule.url))?;

    Ok(resp.status().is_success())
}

/* ──────────────── Tauri IPC Commands ──────────────── */

#[tauri::command]
fn list_rules(app: AppHandle<Wry>) -> Vec<HookRule> {
    tracing::debug!("IPC list_rules");
    config::load_rules(&app)
}

#[tauri::command]
fn upsert_rule(app: AppHandle<Wry>, mut rule: HookRule) -> tauri::Result<()> {
    tracing::debug!(?rule, "IPC upsert_rule");

    let mut rules = config::load_rules(&app);

    // Assign ID on first insert
    if rule.id.is_none() {
        rule.id = Some(uuid::Uuid::new_v4().to_string());
    }

    match &rule.id {
        Some(id) if rules.iter().any(|r| r.id.as_deref() == Some(id)) => {
            for r in &mut rules {
                if r.id.as_deref() == Some(id) {
                    *r = rule.clone();
                }
            }
        }
        _ => rules.push(rule),
    }

    if let Err(e) = config::save_rules(&app, &rules).and_then(|_| register_shortcuts(&app, &rules))
    {
        tracing::error!(error = %e, "upsert_rule failed");
    }
    Ok(())
}

#[tauri::command]
fn delete_rule(app: AppHandle<Wry>, id: String) -> tauri::Result<()> {
    tracing::debug!(id = %id, "IPC delete_rule");
    let mut rules = config::load_rules(&app);
    rules.retain(|r| r.id.as_deref() != Some(&id));

    if let Err(e) = config::save_rules(&app, &rules).and_then(|_| register_shortcuts(&app, &rules))
    {
        tracing::error!(error = %e, "delete_rule failed");
    }
    Ok(())
}

/* ──────────────── Global Shortcuts ──────────────── */

/// (Re)register all enabled shortcuts; called on every rules update.
fn register_shortcuts(app: &AppHandle<Wry>, rules: &[HookRule]) -> AnyResult<()> {
    tracing::debug!("re-registering all shortcuts ({} rules)", rules.len());
    let gsc = app.global_shortcut();
    let _ = gsc.unregister_all();

    for rule in rules.iter().filter(|r| r.enabled) {
        let shortcut: Shortcut = rule
            .hotkey
            .parse()
            .with_context(|| format!("invalid hotkey {}", rule.hotkey))?;

        tracing::debug!(hotkey = %rule.hotkey, "register shortcut");

        let rule_clone = rule.clone();
        let app_clone = app.clone();

        if let Err(e) = gsc.on_shortcut(shortcut, move |_, _, ev: ShortcutEvent| {
            if ev.state != ShortcutState::Pressed {
                return;
            }
            tracing::debug!(hotkey = %rule_clone.hotkey, "shortcut pressed");

            let rule = rule_clone.clone();
            let ah = app_clone.clone();

            async_runtime::spawn(async move {
                let ok = match fire(&rule).await {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!(error = %e, "webhook error");
                        false
                    }
                };
                tracing::debug!(hotkey = %rule.hotkey, ok, "webhook fired");
                let _ = ah.emit("hook:fired", (rule.id.clone(), ok));
            });
        }) {
            tracing::warn!(error = %e, hotkey = %rule.hotkey, "register failed");
        }
    }
    Ok(())
}

/* ──────────────── System Tray ──────────────── */

fn init_tray(app: &AppHandle<Wry>) -> tauri::Result<()> {
    let icon = Image::from_bytes(include_bytes!("../../keyhook-ui/public/logo.png"))?;

    let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let tray_menu = {
        let menu = Menu::new(app)?;
        menu.append(&show_item)?;
        menu.append(&quit_item)?;
        menu
    };

    TrayIconBuilder::with_id("keyhook-tray")
        .icon(icon)
        .tooltip("KeyHook")
        .menu(&tray_menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, evt| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = evt
            {
                if let Some(w) = tray.app_handle().get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

/* ──────────────── Application Entry ──────────────── */

pub fn run() -> tauri::Result<()> {
    logger::init();
    tracing::info!("🚀 KeyHook starting");

    Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_rules,
            upsert_rule,
            delete_rule
        ])
        .setup(|app| {
            let rules = config::load_rules(app.app_handle());
            if let Err(e) = register_shortcuts(app.app_handle(), &rules) {
                tracing::error!(error = %e, "register_shortcuts failed");
            }

            init_tray(app.app_handle())?;

            // Hide instead of quit when user clicks window close
            if let Some(w) = app.get_webview_window("main") {
                let w_clone = w.clone();
                w.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w_clone.hide();
                    }
                });
            }
            Ok(())
        })
        .build(generate_context!())?
        .run(|app, event| {
            if let RunEvent::MenuEvent(me) = event {
                match me.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                }
            }
        });

    Ok(())
}
