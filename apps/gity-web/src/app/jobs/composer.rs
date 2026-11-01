use super::*;
#[component]
pub(super) fn Composer(context: Context) -> impl IntoView {
  let kind = RwSignal::new("noop".to_string());
  let payload = RwSignal::new(String::new());
  let attempts = RwSignal::new("3".to_string());
  let submit = move |event: leptos::ev::SubmitEvent| {
    event.prevent_default();
    if !context.can_write.get_untracked() || context.creation.busy.get_untracked() {
      return;
    }
    let body = match model::enqueue(
      &kind.get_untracked(),
      &payload.get_untracked(),
      &attempts.get_untracked(),
    ) {
      Ok(body) => body,
      Err(error) => {
        context.creation.error.set(t(error));
        return;
      }
    };
    context.creation.run(
      "POST",
      format!("{}/jobs", context.path.get_value()),
      body,
      Some(Callback::new(move |_| context.composer.set(false))),
    );
  };
  view! {
      <form class="jobs-composer" on:submit=submit>
          <div>
              <label for="job-kind">
                  {t("Job kind")}
                  <input
                      id="job-kind"
                      prop:value=move || kind.get()
                      on:input=move |event| kind.set(event_target_value(&event))
                      placeholder="noop"
                  />
              </label>
              <label for="job-max-attempts">
                  {t("Max attempts")}
                  <input
                      id="job-max-attempts"
                      type="number"
                      min="1"
                      prop:value=move || attempts.get()
                      on:input=move |event| attempts.set(event_target_value(&event))
                  />
              </label>
          </div>
          <label for="job-payload">
              {t("Payload JSON")}
              <textarea
                  id="job-payload"
                  prop:value=move || payload.get()
                  on:input=move |event| payload.set(event_target_value(&event))
                  placeholder="{\"reason\":\"manual\"}"
              ></textarea>
          </label>
          <Show when=move || !context.creation.error.get().is_empty()>
              <p class="error" role="alert">
                  {move || context.creation.error.get()}
              </p>
          </Show>
          <footer>
              <button
                  type="submit"
                  class="primary"
                  disabled=move || !context.can_write.get() || context.creation.busy.get()
              >
                  {move || t(
                      if context.creation.busy.get() {
                          "Enqueueing job..."
                      } else {
                          "Enqueue job"
                      },
                  )}
              </button>
          </footer>
      </form>
  }
}
