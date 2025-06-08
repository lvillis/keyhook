# keyhook-model

[![Crates.io](https://img.shields.io/crates/v/keyhook-model.svg)](https://crates.io/crates/keyhook-model)
[![Docs.rs](https://docs.rs/keyhook-model/badge.svg)](https://docs.rs/keyhook-model)

Shared data structures used by the **KeyHook** family of crates  
(back-end Tauri service, front-end Yew SPA, and any third-party
integration).

## ✨ What’s inside?

| Type         | Purpose                                              |
|--------------|------------------------------------------------------|
| `HttpMethod` | Enum of supported HTTP verbs (`GET`, …).             |
| `HookRule`   | Mapping from a global hot-key to a webhook endpoint. |
