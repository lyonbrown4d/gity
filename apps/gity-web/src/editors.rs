//! Rust bindings for the vendored Monaco and Milkdown distributions.
use js_sys::{Array, Function, Object, Reflect};
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};

#[wasm_bindgen(raw_module = "/vendor/editors.js")]
extern "C" {
    #[wasm_bindgen(thread_local_v2,js_name=monaco)]
    static MONACO: JsValue;
    #[wasm_bindgen(thread_local_v2,js_name=EditorWorker)]
    static EDITOR_WORKER: JsValue;
    #[wasm_bindgen(thread_local_v2,js_name=Editor)]
    static EDITOR: JsValue;
    #[wasm_bindgen(thread_local_v2,js_name=rootCtx)]
    static ROOT_CTX: JsValue;
    #[wasm_bindgen(thread_local_v2,js_name=defaultValueCtx)]
    static DEFAULT_CTX: JsValue;
    #[wasm_bindgen(thread_local_v2,js_name=listener)]
    static LISTENER: JsValue;
    #[wasm_bindgen(thread_local_v2,js_name=listenerCtx)]
    static LISTENER_CTX: JsValue;
    #[wasm_bindgen(thread_local_v2,js_name=commonmark)]
    static COMMONMARK: JsValue;
    #[wasm_bindgen(thread_local_v2,js_name=gfm)]
    static GFM: JsValue;
    #[wasm_bindgen(thread_local_v2,js_name=nord)]
    static NORD: JsValue;
}
fn get(object: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    Reflect::get(object, &JsValue::from_str(key))
}
fn set(object: &JsValue, key: &str, value: &JsValue) {
    let _ = Reflect::set(object, &JsValue::from_str(key), value);
}
fn call(object: &JsValue, key: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    let f = get(object, key)?.dyn_into::<Function>()?;
    let a = Array::new();
    for v in args {
        a.push(v);
    }
    f.apply(object, &a)
}
fn element(id: &str) -> Option<web_sys::Element> {
    web_sys::window()?.document()?.get_element_by_id(id)
}
type CodeInstance = (JsValue, Closure<dyn FnMut()>);
fn code_editor(
    host: &str,
    input: &str,
    content: &str,
    language: &str,
    readonly: bool,
) -> Result<CodeInstance, JsValue> {
    let host = element(host).ok_or(JsValue::NULL)?;
    let input: web_sys::HtmlTextAreaElement = element(input).ok_or(JsValue::NULL)?.unchecked_into();
    if get(&js_sys::global(), "MonacoEnvironment")?.is_undefined() {
        let env = Object::new();
        let worker = Closure::<dyn FnMut(JsValue, JsValue) -> JsValue>::new(move |_, _| {
            EDITOR_WORKER.with(|v| {
                Reflect::construct(v.unchecked_ref::<Function>(), &Array::new())
                    .unwrap_or(JsValue::NULL)
            })
        });
        set(&env, "getWorker", &worker.into_js_value());
        set(&js_sys::global(), "MonacoEnvironment", &env);
    }
    let options = Object::new();
    for (key, value) in [
        ("value", JsValue::from_str(content)),
        ("language", JsValue::from_str(language)),
        ("readOnly", JsValue::from_bool(readonly)),
        (
            "theme",
            JsValue::from_str(if crate::api::storage("gity.theme") == "dark" {
                "vs-dark"
            } else {
                "vs"
            }),
        ),
        ("automaticLayout", JsValue::TRUE),
        ("fontSize", JsValue::from_f64(12.)),
        ("scrollBeyondLastLine", JsValue::FALSE),
        ("wordWrap", JsValue::from_str("on")),
    ] {
        set(&options, key, &value);
    }
    let minimap = Object::new();
    set(&minimap, "enabled", &JsValue::FALSE);
    set(&options, "minimap", &minimap);
    let editor =
        MONACO.with(|m| call(&get(m, "editor")?, "create", &[host.into(), options.into()]))?;
    let current = editor.clone();
    let change = Closure::<dyn FnMut()>::new(move || {
        if let Ok(v) = call(&current, "getValue", &[]) {
            input.set_value(&v.as_string().unwrap_or_default());
            if let Ok(event) = web_sys::Event::new("input") {
                let _ = input.dispatch_event(&event);
            }
        }
    });
    call(
        &editor,
        "onDidChangeModelContent",
        &[change.as_ref().clone()],
    )?;
    Ok((editor, change))
}
#[component]
pub fn CodeEditor(
    id: String,
    name: String,
    content: String,
    #[prop(default = false)] readonly: bool,
    #[prop(default="plaintext".into())] language: String,
    #[prop(default = 1)] line: u32,
    #[prop(default = 1)] column: u32,
    #[prop(default = 0)] match_length: u32,
    #[prop(optional)] on_change: Option<Callback<String>>,
) -> impl IntoView {
    let host = format!("{id}-host");
    let h = host.clone();
    let input = id.clone();
    let value = content.clone();
    Effect::new(move |_| {
        if let Ok((editor, callback)) = code_editor(&h, &input, &value, &language, readonly) {
            jump_to(&editor, line, column, match_length);
            let managed = SendWrapper::new((editor, callback));
            on_cleanup(move || {
                let (editor, _callback) = managed.take();
                if let Ok(model) = call(&editor, "getModel", &[]) {
                    let _ = call(&model, "dispose", &[]);
                }
                let _ = call(&editor, "dispose", &[]);
            });
        }
    });
    view! {
        <div class="code-editor">
            <div id=host></div>
            <textarea
                id=id
                name=name
                rows="18"
                readonly=readonly
                prop:value=content
                aria-label="File content"
                on:input=move |event| {
                    if let Some(callback) = on_change {
                        callback.run(event_target_value(&event));
                    }
                }
            ></textarea>
        </div>
    }
}
#[component]
pub fn MarkdownEditor(
    id: String,
    name: String,
    content: String,
    #[prop(optional)] on_change: Option<Callback<String>>,
) -> impl IntoView {
    let host = format!("{id}-markdown");
    let h = host.clone();
    let input = id.clone();
    let value = content.clone();
    Effect::new(move |_| {
        let Some(host) = element(&h) else {
            return;
        };
        let Some(input) =
            element(&input).map(|e| e.unchecked_into::<web_sys::HtmlTextAreaElement>())
        else {
            return;
        };
        let input_for_listener = input.clone();
        let updated =
            Closure::<dyn FnMut(JsValue, String)>::new(move |_: JsValue, markdown: String| {
                input_for_listener.set_value(&markdown);
                if let Ok(e) = web_sys::Event::new("input") {
                    let _ = input_for_listener.dispatch_event(&e);
                }
            });
        let updated_fn = updated.as_ref().clone();
        let initial = value.clone();
        let target = host.clone();
        let configure = Closure::<dyn FnMut(JsValue)>::new(move |ctx| {
            ROOT_CTX.with(|key| {
                let _ = call(&ctx, "set", &[key.clone(), target.clone().into()]);
            });
            DEFAULT_CTX.with(|key| {
                let _ = call(&ctx, "set", &[key.clone(), JsValue::from_str(&initial)]);
            });
            LISTENER_CTX.with(|key| {
                if let Ok(manager) = call(&ctx, "get", std::slice::from_ref(key)) {
                    let _ = call(
                        &manager,
                        "markdownUpdated",
                        std::slice::from_ref(&updated_fn),
                    );
                }
            });
        });
        let result = EDITOR.with(|editor| {
            let instance = call(editor, "make", &[])?;
            let instance = call(&instance, "config", &[NORD.with(Clone::clone)])?;
            let instance = call(&instance, "config", &[configure.as_ref().clone()])?;
            for plugin in [
                COMMONMARK.with(Clone::clone),
                GFM.with(Clone::clone),
                LISTENER.with(Clone::clone),
            ] {
                call(&instance, "use", &[plugin])?;
            }
            call(&instance, "create", &[])
        });
        let Ok(promise) = result.and_then(|v| v.dyn_into::<js_sys::Promise>()) else {
            return;
        };
        let saved = Rc::new(RefCell::new(None::<JsValue>));
        let cleanup = SendWrapper::new((saved.clone(), configure, updated));
        on_cleanup(move || {
            let (instance, _config, _updated) = cleanup.take();
            if let Some(instance) = instance.borrow_mut().take() {
                let _ = call(&instance, "destroy", &[]);
            }
        });
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(instance) = wasm_bindgen_futures::JsFuture::from(promise).await {
                if host.is_connected() {
                    input.set_hidden(true);
                    *saved.borrow_mut() = Some(instance);
                } else {
                    let _ = call(&instance, "destroy", &[]);
                }
            }
        });
    });
    view! {
        <div class="markdown-editor">
            <div id=host></div>
            <textarea
                id=id
                name=name
                rows="8"
                prop:value=content
                aria-label="Markdown content"
                on:input=move |event| {
                    if let Some(callback) = on_change {
                        callback.run(event_target_value(&event));
                    }
                }
            ></textarea>
        </div>
    }
}

pub fn set_theme(dark: bool) {
    MONACO.with(|m| {
        if let Ok(editor) = get(m, "editor") {
            let _ = call(
                &editor,
                "setTheme",
                &[JsValue::from_str(if dark { "vs-dark" } else { "vs" })],
            );
        }
    });
}

mod navigation;
use navigation::jump_to;
