use crate::{api, editors::MarkdownEditor, ui::*};
use leptos::prelude::*;
use serde_json::json;
use wasm_bindgen::JsCast;
#[component]
pub(super) fn IssueComposer(path: String, base: String, users: Query) -> impl IntoView {
  let busy = RwSignal::new(false);
  let error = RwSignal::new(String::new());
  let submit = move |event: leptos::ev::SubmitEvent| {
    event.prevent_default();
    if busy.get_untracked() {
      return;
    }
    let form: web_sys::HtmlFormElement = event.target().unwrap().unchecked_into();
    let Ok(data) = web_sys::FormData::new_with_form(&form) else {
      return;
    };
    let title = data.get("title").as_string().unwrap_or_default();
    let description = data.get("description").as_string().unwrap_or_default();
    let assignee = data.get("assignee").as_string().unwrap_or_default();
    let path = path.clone();
    let base = base.clone();
    busy.set(true);
    error.set(String::new());
    wasm_bindgen_futures::spawn_local(async move {
      let result = async {
        let issue = api::request(
          "POST",
          &path,
          Some(json!({"title":title,"description":description})),
        )
        .await?;
        let number = api::text(&issue, "number");
        if !assignee.is_empty() {
          api::request(
            "PATCH",
            &format!("{path}/{number}/assignees"),
            Some(json!({"user_ids":[assignee]})),
          )
          .await?;
        }
        api::navigate(&format!("{base}/issues/{number}"));
        Ok::<_, String>(())
      }
      .await;
      if busy.is_disposed() {
        return;
      }
      if let Err(e) = result {
        error.set(e);
      }
      busy.set(false);
    });
  };
  view! {
      <form class="issue-composer" on:submit=submit>
          <input name="title" placeholder="Issue title" aria-label="Issue title" required />
          <MarkdownEditor
              id="new-issue-description".into()
              name="description".into()
              content=String::new()
          />
          <select name="assignee" aria-label="Assign to user (optional)">
              <option value="">{crate::i18n::t("Assign to user (optional)")}</option>
              {move || {
                  api::list(&users.data.get())
                      .into_iter()
                      .map(|u| {
                          view! {
                              <option value=api::text(
                                  &u,
                                  "id",
                              )>{api::text(&u, "username")}</option>
                          }
                      })
                      .collect_view()
              }}
          </select>
          <Show when=move || !error.get().is_empty()>
              <p class="error" role="alert">
                  {move || error.get()}
              </p>
          </Show>
          <div>
              <thaw::Button
                  appearance=thaw::ButtonAppearance::Primary
                  button_type=thaw::ButtonType::Submit
                  loading=Signal::derive(move || busy.get())
                  disabled=Signal::derive(move || busy.get())
              >
                  {crate::i18n::t("Create issue")}
              </thaw::Button>
          </div>
      </form>
  }
}
