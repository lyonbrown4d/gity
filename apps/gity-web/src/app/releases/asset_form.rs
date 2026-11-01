use super::*;
use serde_json::json;

#[component]
pub(super) fn AssetForm(context: Context) -> impl IntoView {
  let name = RwSignal::new(String::new());
  let url = RwSignal::new(String::new());
  let kind = RwSignal::new("package".to_string());
  let mutation = Mutation::new(context.tags.refresh);
  view! {
      <form
          class="releases-box releases-asset-form"
          on:submit=move |e| {
              e.prevent_default();
              if !context.can_admin() {
                  return;
              }
              let release = context.active();
              let id = api::text(&release["release"], "id");
              if id.is_empty() {
                  mutation.error.set(t("Select a release first."));
                  return;
              }
              if name.get_untracked().trim().is_empty()
                  || model::asset_url(url.get_untracked().trim()).is_none()
              {
                  mutation.error.set(t("Release link name and URL are required."));
                  return;
              }
              mutation
                  .run(
                      "POST",
                      format!("{}/releases/{id}/links", context.path.get_value()),
                      json!(
                          {"name":name.get_untracked().trim(),"url":url.get_untracked().trim(),"link_type":kind.get_untracked()}
                      ),
                      Some(Callback::new(move |_| context.link_composer.set(false))),
                  );
          }
      >
          <label>
              {t("Link name")}
              <input
                  aria-label=t("Link name")
                  placeholder="linux-amd64 tarball"
                  required
                  prop:value=move || name.get()
                  on:input=move |e| name.set(event_target_value(&e))
              />
          </label>
          <label>
              {t("URL")}
              <input
                  aria-label=t("URL")
                  type="url"
                  placeholder="https://example.com/artifact.tar.gz"
                  required
                  prop:value=move || url.get()
                  on:input=move |e| url.set(event_target_value(&e))
              />
          </label>
          <label>
              {t("Link type")}
              <select
                  aria-label=t("Link type")
                  prop:value=move || kind.get()
                  on:change=move |e| kind.set(event_target_value(&e))
              >
                  {["package", "image", "runbook", "other"]
                      .into_iter()
                      .map(|value| view! { <option value=value>{value}</option> })
                      .collect_view()}
              </select>
          </label>
          <div class="releases-end">
              <button
                  type="submit"
                  class="primary"
                  disabled=move || !context.can_admin() || mutation.busy.get()
              >
                  {t("Add link")}
              </button>
          </div>
          <Show when=move || !mutation.error.get().is_empty()>
              <p class="error" role="alert">
                  {move || mutation.error.get()}
              </p>
          </Show>
      </form>
  }
}
