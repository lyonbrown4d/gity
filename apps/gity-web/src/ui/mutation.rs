use crate::api;
use leptos::prelude::*;
use serde_json::Value;

#[derive(Clone, Copy)]
pub struct Mutation {
    pub busy: RwSignal<bool>,
    pub error: RwSignal<String>,
    refresh: RwSignal<u64>,
}

impl Mutation {
    pub fn new(refresh: RwSignal<u64>) -> Self {
        Self {
            busy: RwSignal::new(false),
            error: RwSignal::new(String::new()),
            refresh,
        }
    }

    pub fn run(
        self,
        method: &'static str,
        path: String,
        body: Value,
        on_success: Option<Callback<()>>,
    ) {
        self.run_with_result(
            method,
            path,
            body,
            on_success.map(|callback| Callback::new(move |_: Value| callback.run(()))),
        );
    }

    pub fn run_with_result(
        self,
        method: &'static str,
        path: String,
        body: Value,
        on_success: Option<Callback<Value>>,
    ) {
        if self.busy.get_untracked() {
            return;
        }
        self.busy.set(true);
        self.error.set(String::new());
        wasm_bindgen_futures::spawn_local(async move {
            let result = api::request(method, &path, Some(body)).await;
            if self.busy.is_disposed() {
                return;
            }
            // Successful callbacks can close and dispose the form that owns us.
            self.busy.set(false);
            match result {
                Ok(value) => {
                    self.refresh.update(|n| *n += 1);
                    if let Some(callback) = on_success {
                        callback.run(value);
                    }
                }
                Err(cause) => self.error.set(cause),
            }
        });
    }
}
