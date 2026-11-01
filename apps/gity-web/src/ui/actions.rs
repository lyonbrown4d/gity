use super::*;
use serde_json::json;
#[component]
pub fn Action(
    label: String,
    path: String,
    #[prop(default="POST".into())] method: String,
    #[prop(default=json!({}))] body: Value,
    refresh: RwSignal<u64>,
    #[prop(default = false)] confirm: bool,
    #[prop(default=Signal::stored(false))] disabled: Signal<bool>,
    #[prop(default="Confirm action".into())] confirmation_title: String,
    #[prop(default="Confirm this action to continue.".into())] confirmation_description: String,
    #[prop(default="Confirm".into())] confirmation_label: String,
    #[prop(optional)] children: Option<Children>,
    #[prop(optional)] busy_state: Option<RwSignal<bool>>,
) -> impl IntoView {
    let busy = busy_state.unwrap_or_else(|| RwSignal::new(false));
    let error = RwSignal::new(String::new());
    let open = RwSignal::new(false);
    let verification = RwSignal::new(String::new());
    let expected = body
        .get("confirmation")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let expected_label = expected.clone();
    let accessible_label = crate::i18n::t(&label);
    let content = children
        .map(|children| children().into_any())
        .unwrap_or_else(|| accessible_label.clone().into_any());
    let execute = Callback::new(move |_: leptos::ev::MouseEvent| {
        if busy.get_untracked() || disabled.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(String::new());
        let path = path.clone();
        let method = method.clone();
        let body = body.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api::request(&method, &path, Some(body)).await;
            if busy.is_disposed() {
                return;
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    if !refresh.is_disposed() {
                        refresh.update(|v| *v += 1);
                    }
                }
                Err(e) => {
                    let _ = error.try_set(e);
                }
            }
        });
    });
    view! {
        <span class="action">
            <thaw::Button
                button_type=thaw::ButtonType::Button
                attr:aria-label=accessible_label
                loading=Signal::derive(move || busy.get())
                disabled=Signal::derive(move || busy.get() || disabled.get())
                on_click=move |ev| {
                    if confirm {
                        open.set(true);
                    } else {
                        execute.run(ev);
                    }
                }
            >
                {content}
            </thaw::Button>
            <Show when=move || !error.get().is_empty()>
                <span class="error" role="alert">
                    {move || error.get()}
                </span>
            </Show>
        </span>
        <thaw::Dialog open=open>
            <thaw::DialogSurface>
                <thaw::DialogBody>
                    <thaw::DialogTitle>{crate::i18n::t(&confirmation_title)}</thaw::DialogTitle>
                    <thaw::DialogContent>
                        <p>{crate::i18n::t(&confirmation_description)}</p>
                        {if expected_label.is_empty() {
                            None
                        } else {
                            Some(
                                view! {
                                    <label>
                                        {crate::i18n::t("Type ")}<strong>{expected_label}</strong>
                                        {crate::i18n::t(" to confirm")}
                                        <input
                                            aria-label="Confirmation"
                                            on:input=move |e| verification.set(event_target_value(&e))
                                        />
                                    </label>
                                },
                            )
                        }}
                    </thaw::DialogContent>
                    <thaw::DialogActions>
                        <thaw::Button
                            button_type=thaw::ButtonType::Button
                            on_click=move |_| { open.set(false) }
                        >
                            {crate::i18n::t("Cancel")}
                        </thaw::Button>
                        <thaw::Button
                            button_type=thaw::ButtonType::Button
                            appearance=thaw::ButtonAppearance::Primary
                            disabled=Signal::derive(move || {
                                busy.get() || disabled.get()
                                    || (!expected.is_empty() && verification.get() != expected)
                            })
                            on_click=move |ev| {
                                open.set(false);
                                execute.run(ev);
                            }
                        >
                            {crate::i18n::t(&confirmation_label)}
                        </thaw::Button>
                    </thaw::DialogActions>
                </thaw::DialogBody>
            </thaw::DialogSurface>
        </thaw::Dialog>
    }
}
