use super::{
    markdown::IssueMarkdown,
    model::{IssueQueries, relative_time, user_label},
};
use crate::{api, editors::MarkdownEditor, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::json;

#[component]
pub(super) fn Comments(
    q: IssueQueries,
    path: String,
    base: String,
    can_comment: Signal<bool>,
    attachment_path: String,
) -> impl IntoView {
    let comment = RwSignal::new(String::new());
    let generation = RwSignal::new(0u64);
    let mutation = Mutation::new(q.comments.refresh);
    let comment_path = format!("{path}/comments");
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if !can_comment.get_untracked() {
            return;
        }
        let content = comment.get_untracked().trim().to_string();
        if content.is_empty() {
            mutation.error.set(t("Comment content is required"));
            return;
        }
        mutation.run(
            "POST",
            comment_path.clone(),
            json!({"content":content}),
            Some(Callback::new(move |_| {
                comment.set(String::new());
                generation.update(|value| *value += 1);
            })),
        );
    };
    view! {
        <section class="issue-comments">
            <header>
                <p>{t("Discussion")}</p>
                <span class="badge outline">
                    {move || api::list(&q.comments.data.get()).len()}" "{t("comment(s)")}
                </span>
            </header>
            <Show when=move || q.comments.loading.get()>
                <p class="muted" role="status">
                    {t("Loading comments...")}
                </p>
            </Show>
            <Show when=move || !q.comments.error.get().is_empty()>
                <p class="error" role="alert">
                    {move || q.comments.error.get()}
                </p>
            </Show>
            <Show when=move || {
                !q.comments.loading.get() && q.comments.error.get().is_empty()
                    && api::list(&q.comments.data.get()).is_empty()
            }>
                <p class="issue-comments-empty">{t("No comments yet.")}</p>
            </Show>
            {move || {
                api::list(&q.comments.data.get())
                    .into_iter()
                    .map(|row| {
                        let row = normalized(&row);
                        view! {
                            <article class="issue-comment">
                                <p>
                                    {user_label(
                                        &q.users.data.get(),
                                        &api::text(&row, "author_user_id"),
                                    )}" · "{relative_time(&api::text(&row, "created_at"))}
                                </p>
                                <IssueMarkdown
                                    content=api::text(&row, "content")
                                    base=base.clone()
                                />
                            </article>
                        }
                    })
                    .collect_view()
            }}
        </section>
        <Show
            when=move || can_comment.get()
            fallback=move || {
                view! {
                    <p class="branch-notice">
                        {t(
                            "Your current project role can inspect issues, but cannot comment on them.",
                        )}
                    </p>
                }
            }
        >
            <div class="issue-comment-composer">
                <form on:submit=submit.clone()>
                    <p>{t("Add a comment")}</p>
                    {move || {
                        generation.get();
                        view! {
                            <MarkdownEditor
                                id="issue-comment-content".into()
                                name="content".into()
                                content=comment.get_untracked()
                                on_change=Callback::new(move |value| comment.set(value))
                            />
                        }
                    }}
                    <Show when=move || !mutation.error.get().is_empty()>
                        <p class="error" role="alert">
                            {move || mutation.error.get()}
                        </p>
                    </Show>
                    <footer>
                        <button
                            class="primary"
                            disabled=move || mutation.busy.get() || comment.get().trim().is_empty()
                        >
                            {move || t(
                                if mutation.busy.get() { "Commenting..." } else { "Comment" },
                            )}
                        </button>
                    </footer>
                </form>
                <crate::files::IssueAttachment
                    path=attachment_path.clone()
                    on_uploaded=Callback::new(move |markdown: String| {
                        comment
                            .update(|content| {
                                if !content.trim().is_empty() {
                                    content.push_str("\n\n");
                                }
                                content.push_str(&markdown);
                            });
                        generation.update(|value| *value += 1);
                    })
                />
            </div>
        </Show>
    }
}
