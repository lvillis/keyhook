//! Yew SPA for KeyHook.
//! Features:
//! * Method selector (GET / POST / PUT / DELETE / PATCH)
//! * Optional JSON body
//! * Toast feedback on shortcut trigger

use js_sys::{Function, Reflect};
use keyhook_model::{HookRule, HttpMethod};
use serde::Serialize;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use yew::prelude::*;
use yew_hooks::use_effect_once;

/* ---------- Tauri IPC ---------- */
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    fn listen(event: &str, cb: &Function) -> js_sys::Promise;
}

/* ---------- IPC Arg Structs ---------- */
#[derive(Serialize)]
struct UpsertArgs {
    rule: HookRule,
}
#[derive(Serialize)]
struct IdArg<'a> {
    id: &'a str,
}

/* ---------- Promise → Result ---------- */
async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue> {
    JsFuture::from(invoke(cmd, args)).await
}

/* ---------- Toast Helper ---------- */
fn push_toast(msg: &str) {
    let win = web_sys::window().unwrap();
    let doc = win.document().unwrap();

    let root = doc.get_element_by_id("toasts").unwrap_or_else(|| {
        let div = doc.create_element("div").unwrap();
        div.set_id("toasts");
        doc.body().unwrap().append_child(&div).unwrap();
        div
    });

    let div = doc.create_element("div").unwrap();
    div.set_inner_html(msg);
    div.set_class_name("toast");
    root.append_child(&div).unwrap();

    let cb = Closure::<dyn FnOnce()>::once({
        let div = div.clone();
        move || {
            let _ = div.remove();
        }
    });
    win.set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 3000)
        .unwrap();
    cb.forget();
}

/* ====================== Component ====================== */
#[function_component(App)]
pub fn app() -> Html {
    /* ---- Reactive State ---- */
    let rules = use_state_eq(Vec::<HookRule>::new);
    let hotkey = use_state(String::new);
    let method = use_state(|| HttpMethod::GET);
    let url = use_state(String::new);
    let body = use_state(String::new);

    /* ---- Initial Load ---- */
    {
        let rules = rules.clone();
        use_effect_once(move || {
            spawn_local(async move {
                if let Ok(list) = tauri_invoke("list_rules", JsValue::NULL).await {
                    let v: Vec<HookRule> = serde_wasm_bindgen::from_value(list).unwrap_or_default();
                    rules.set(v);
                }
            });
            || ()
        });
    }

    /* ---- Listen for Fires ---- */
    {
        use_effect_once(|| {
            spawn_local(async {
                let cb = Closure::<dyn Fn(JsValue)>::new(move |evt| {
                    if let Ok(payload) = Reflect::get(&evt, &JsValue::from_str("payload")) {
                        if let Ok((id, ok)) =
                            serde_wasm_bindgen::from_value::<(Option<String>, bool)>(payload)
                        {
                            let msg = match id {
                                Some(i) => format!("Hotkey {i} fired: {ok}"),
                                None => format!("Hotkey fired: {ok}"),
                            };
                            push_toast(&msg);
                        }
                    }
                });
                let _ = JsFuture::from(listen("hook:fired", cb.as_ref().unchecked_ref())).await;
                cb.forget();
            });
            || ()
        });
    }

    /* ---- Submit Logic ---- */
    let on_submit = {
        let hotkey = hotkey.clone();
        let url = url.clone();
        let method = method.clone();
        let body = body.clone();
        let rules = rules.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let rule = HookRule {
                id: None,
                hotkey: (*hotkey).clone(),
                method: (*method).clone(),
                url: (*url).clone(),
                body: if body.is_empty() {
                    None
                } else {
                    Some((*body).clone())
                },
                enabled: true,
            };

            // Reset form
            hotkey.set(String::new());
            method.set(HttpMethod::GET);
            url.set(String::new());
            body.set(String::new());

            let rules = rules.clone();
            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&UpsertArgs { rule }).unwrap();
                if tauri_invoke("upsert_rule", args).await.is_ok() {
                    if let Ok(list) = tauri_invoke("list_rules", JsValue::NULL).await {
                        let v: Vec<HookRule> =
                            serde_wasm_bindgen::from_value(list).unwrap_or_default();
                        rules.set(v);
                        push_toast("Rule added/updated ✔");
                    }
                }
            });
        })
    };

    /* ---- Delete Logic ---- */
    let on_delete = {
        let rules = rules.clone();
        Callback::from(move |id: String| {
            let rules = rules.clone();
            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&IdArg { id: &id }).unwrap();
                if tauri_invoke("delete_rule", args).await.is_ok() {
                    if let Ok(list) = tauri_invoke("list_rules", JsValue::NULL).await {
                        let v: Vec<HookRule> =
                            serde_wasm_bindgen::from_value(list).unwrap_or_default();
                        rules.set(v);
                        push_toast("Rule removed 🗑");
                    }
                }
            });
        })
    };

    /* ---- Hotkey Capture ---- */
    let hotkey_onkeydown = {
        let hotkey = hotkey.clone();
        Callback::from(move |e: KeyboardEvent| {
            e.prevent_default();

            let mut parts = Vec::new();
            if e.ctrl_key() {
                parts.push("Ctrl");
            }
            if e.shift_key() {
                parts.push("Shift");
            }
            if e.alt_key() {
                parts.push("Alt");
            }
            if e.meta_key() {
                parts.push("Meta");
            }

            let code = e.code();
            if !code.is_empty()
                && !matches!(
                    code.as_str(),
                    "ControlLeft"
                        | "ControlRight"
                        | "ShiftLeft"
                        | "ShiftRight"
                        | "AltLeft"
                        | "AltRight"
                        | "MetaLeft"
                        | "MetaRight"
                )
            {
                parts.push(code.trim_start_matches("Key").into());
            }

            if !parts.is_empty() {
                hotkey.set(parts.join("+"));
            }
        })
    };

    /* ---- View ---- */
    html! {
        <main class="container">
            <div class="card">
                <h2>{"Add / Update Rule"}</h2>
                <form class="col" autocomplete="off" onsubmit={on_submit}>
                    <div class="row">
                        <input
                            placeholder="Hotkey"
                            value={(*hotkey).clone()}
                            onkeydown={hotkey_onkeydown}
                            oninput={Callback::from(move |e: InputEvent| {
                                hotkey.set(
                                    e.target_unchecked_into::<web_sys::HtmlInputElement>()
                                     .value()
                                )
                            })}
                        />

                        {{
                            // Current method needs an owned value
                            let current = (*method).clone();
                            let on_change = {
                                let handle = method.clone();
                                Callback::from(move |e: Event| {
                                    let v = e.target_unchecked_into::<web_sys::HtmlSelectElement>()
                                             .value();
                                    let m = match v.as_str() {
                                        "GET"    => HttpMethod::GET,
                                        "POST"   => HttpMethod::POST,
                                        "PUT"    => HttpMethod::PUT,
                                        "DELETE" => HttpMethod::DELETE,
                                        "PATCH"  => HttpMethod::PATCH,
                                        _        => HttpMethod::GET,
                                    };
                                    handle.set(m);
                                })
                            };

                            html!{
                                <select value={format!("{:?}", current)} onchange={on_change}>
                                    { for [HttpMethod::GET, HttpMethod::POST, HttpMethod::PUT,
                                           HttpMethod::DELETE, HttpMethod::PATCH]
                                        .iter()
                                        .map(|m| html!{
                                            <option value={format!("{:?}", m)}
                                                    selected={*m == current}>
                                                { format!("{:?}", m) }
                                            </option>
                                        })
                                    }
                                </select>
                            }
                        }}
                    </div>

                    <input
                        placeholder="Webhook URL"
                        value={(*url).clone()}
                        oninput={Callback::from(move |e: InputEvent| {
                            url.set(
                                e.target_unchecked_into::<web_sys::HtmlInputElement>()
                                 .value()
                            )
                        })}
                    />

                    {
                        if matches!(*method, HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH) {
                            html! {
                                <textarea
                                    placeholder="JSON body"
                                    value={(*body).clone()}
                                    oninput={Callback::from(move |e: InputEvent| {
                                        body.set(
                                            e.target_unchecked_into::<web_sys::HtmlTextAreaElement>()
                                             .value()
                                        )
                                    })}
                                />
                            }
                        } else { html!{} }
                    }

                    <button type="submit">{"Save"}</button>
                </form>
            </div>

            <div class="card">
                <h2>{"Registered Rules"}</h2>
                <table>
                    <thead>
                        <tr>
                            <th>{"Hotkey"}</th>
                            <th>{"Method"}</th>
                            <th>{"URL"}</th>
                            <th>{"Action"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for (*rules).iter().map(|r| {
                            let id = r.id.clone().unwrap_or_default();
                            let on_delete = on_delete.clone();
                            html!{
                                <tr>
                                    <td>{ &r.hotkey }</td>
                                    <td>{ format!("{:?}", r.method) }</td>
                                    <td>{ &r.url }</td>
                                    <td>
                                        <button class="btn-del"
                                            onclick={Callback::from(move |_| on_delete.emit(id.clone()))}>
                                            {"Delete"}
                                        </button>
                                    </td>
                                </tr>
                            }
                        }) }
                    </tbody>
                </table>
            </div>
        </main>
    }
}
