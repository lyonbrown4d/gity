use super::{detail::Review, *};
use serde_json::json;

#[component]
pub(super) fn Comments(review: Review) -> impl IntoView {
  let content = review.context.comment_draft;
  let generation = RwSignal::new(0u64);
  let mutation = Mutation::new(review.comments.refresh);
  view! {
      <div class="mr-box mr-stack mr-discussion">
          <div class="mr-between">
              <div>
                  <div class="mr-tags">
                      <thaw::Icon icon=icondata_lu::LuMessageSquare width="16px" height="16px" />
                      <strong>{t("Discussion")}</strong>
                  </div>
                  <p class="mr-muted mr-small">
                      {t("Review conversation for this merge request.")}
                  </p>
              </div>
              <Reload q=review.comments />
          </div>
          <Status q=review.comments />
          <Show when=move || {
              !review.comments.loading.get()
                  && model::list(&review.comments.data.get(), "comments").is_empty()
          }>
              <p class="mr-box mr-muted">{t("No comments yet.")}</p>
          </Show>
          {move || {
              model::list(&review.comments.data.get(), "comments")
                  .into_iter()
                  .map(|comment| {
                      view! {
                          <article class="mr-box mr-comment">
                              <p class="mr-small mr-muted">
                                  {format!(
                                      "{} · {}",
                                      model::user(
                                          &review.context.users.data.get(),
                                          &api::text(&comment, "author_user_id"),
                                      ),
                                      relative_time(&api::text(&comment, "created_at")),
                                  )}
                              </p>
                              <crate::markdown::Markdown content=api::text(&comment, "body") />
                          </article>
                      }
                  })
                  .collect_view()
          }}
          <form
              class="mr-stack"
              on:submit=move |e| {
                  e.prevent_default();
                  if !review.context.can("comment", 20)
                      || content.get_untracked().trim().is_empty()
                  {
                      return;
                  }
                  mutation
                      .run(
                          "POST",
                          format!("{}/comments", review.path.get_value()),
                          json!({"content":content.get_untracked().trim()}),
                          Some(
                              Callback::new(move |_| {
                                  content.set(String::new());
                                  generation.update(|n| *n += 1);
                              }),
                          ),
                      );
              }
          >
              {move || {
                  generation.get();
                  if review.context.can("comment", 20) {
                      view! {
                          <crate::editors::MarkdownEditor
                              id=format!("mr-comment-{}", generation.get_untracked())
                              name="content".into()
                              content=content.get_untracked()
                              on_change=Callback::new(move |value| content.set(value))
                          />
                      }
                          .into_any()
                  } else {
                      view! {
                          <textarea
                              aria-label=t("Add a merge request comment")
                              placeholder=t("Add a merge request comment")
                              disabled
                              rows="4"
                          ></textarea>
                      }
                          .into_any()
                  }
              }}
              <MutationError mutation=mutation />
              <div class="mr-end">
                  <button
                      type="submit"
                      class="primary"
                      disabled=move || {
                          !review.context.can("comment", 20) || mutation.busy.get()
                              || content.get().trim().is_empty()
                      }
                  >
                      {move || t(if mutation.busy.get() { "Commenting..." } else { "Comment" })}
                  </button>
              </div>
          </form>
      </div>
  }
}
