<!-- ─── Language Switch & ToC (top-right) ────────────────────────── -->
<div align="right">

<span style="color:#999;">🇺🇸 English</span> ·
<a href="README.zh-CN.md">🇨🇳 中文</a> &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; Table of Contents ↗️

</div>

<h1 align="center"><code>keyhook</code></h1>

<p align="center">
  ⌨️ <strong>Global Hotkeys → Webhooks</strong> — one desktop app to trigger any HTTP request.
</p>

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/keyhook.svg)](https://crates.io/crates/keyhook)
[![Repo Size](https://img.shields.io/github/repo-size/lvillis/keyhook?color=328657)](https://github.com/lvillis/keyhook)
[![CI](https://github.com/lvillis/keyhook/actions/workflows/ci.yaml/badge.svg)](https://github.com/lvillis/keyhook/actions)
[![Say Thanks](https://img.shields.io/badge/Say%20Thanks-!-1EAEDB.svg)](mailto:lvillis@outlook.com?subject=Thanks%20for%20keyhook!)

</div>

---

## ✨ Features

| Capability               | Details                                                                                           |
|--------------------------|---------------------------------------------------------------------------------------------------|
| 🔑 **Global shortcuts**  | System-wide hotkeys registered via `tauri-plugin-global-shortcut`.                                |
| 🌐 **Webhook actions**   | Fire `GET`, `POST`, `PUT`, `DELETE`, or `PATCH` requests &nbsp;(optional JSON body, 8 s timeout). |
| 🎛 **Live GUI**          | Yew + Trunk single-page app for adding, editing, deleting rules.                                  |
| 💾 **Persistent config** | Rules saved to a pretty-printed `keyhook.json` in the user-specific *app-config* directory.       |
| 🪟 **Tray mode**         | Runs in the system tray, auto-hides the main window, quit & show options.                         |
| 📜 **Structured logs**   | `tracing` output to console with UTC timestamps and log-level filtering (`KEYHOOK_LOG`).          |
| 📦 **Portable build**    | One binary per OS (`tauri bundle`), no runtime dependencies apart from system webview.            |

## 📸 Screenshots

![img](docs/assets/img.png)

## 🕸 Architecture

```mermaid
graph LR
  subgraph "KeyHook (Desktop App)"
    GSL[Global Shortcut\nListener]
    RE[Rule Engine]
    HTTP[[HTTP Client]]
  end
  U[User] -->|Hotkey| GSL --> RE --> HTTP
  HTTP --> REST[(REST API)]
  HTTP --> N8n[(n8n Workflow)]
  HTTP --> Zapier[(Zapier)]
  HTTP --> Script[(Custom Script)]
```
