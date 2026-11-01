use crate::{api, editors::CodeEditor, markdown::Markdown, ui::*};
use leptos::prelude::*;
use serde_json::{Value, json};
mod intelligence;
mod search;
mod tree;
use intelligence::Intelligence;
use search::SourceSearch;
use tree::TreeNode;
#[derive(Clone)]
pub struct FileTarget {
    pub path: String,
    pub line: u32,
    pub column: u32,
    pub match_length: u32,
}
#[component]
pub fn Source(
    project_path: String,
    #[prop(default="main".into())] default_branch: String,
    #[prop(default=Value::Null)] project: Value,
) -> impl IntoView {
    let branch = RwSignal::new(default_branch);
    let blob = RwSignal::new(Value::Null);
    let error = RwSignal::new(String::new());
    let target = RwSignal::new(None::<FileTarget>);
    let create = RwSignal::new(false);
    let branches = query(format!("{project_path}/repository/branches"));
    let tree_path = project_path.clone();
    let readme_path = project_path.clone();
    let tree = query_fn(
        move || {
            format!(
                "{tree_path}/repository/tree?branch_name={}",
                api::enc(&branch.get())
            )
        },
        true,
    );
    let readme = query_fn(
        move || {
            format!(
                "{readme_path}/repository/readme?branch_name={}",
                api::enc(&branch.get())
            )
        },
        true,
    );
    Effect::new(move |_| {
        branch.get();
        blob.set(Value::Null);
        target.set(None);
    });
    let open_path = project_path.clone();
    let open_file = Callback::new(move |file: FileTarget| {
        let path = open_path.clone();
        let selected_branch = branch.get_untracked();
        target.set(Some(file.clone()));
        error.set(String::new());
        wasm_bindgen_futures::spawn_local(async move {
            match api::request(
                "GET",
                &format!(
                    "{path}/repository/blob?branch_name={}&path={}",
                    api::enc(&selected_branch),
                    api::enc(&file.path)
                ),
                None,
            )
            .await
            {
                Ok(v) => {
                    if !blob.is_disposed()
                        && branch.get_untracked() == selected_branch
                        && target.get_untracked().is_some_and(|t| t.path == file.path)
                    {
                        blob.set(v);
                    }
                }
                Err(e) => {
                    if !error.is_disposed() {
                        error.set(e);
                    }
                }
            }
        });
    });
    let root_path = project_path.clone();
    let clone_url = api::text(&project, "clone_http_url");
    view! {
        <section class="source-workspace">
            <div class="source-toolbar">
                <div class="source-title">
                    <span>{crate::i18n::t("‹›")}</span>
                    <div>
                        <h2>{crate::i18n::t("Source workspace")}</h2>
                        <p>{clone_url}</p>
                    </div>
                </div>
                <div class="toolbar">
                    <span class="badge outline mono">
                        {move || api::list(&tree.data.get()).len()}" root item(s)"
                    </span>
                    <select
                        aria-label="Branch"
                        on:change=move |e| branch.set(event_target_value(&e))
                    >
                        {move || {
                            api::list(&branches.data.get())
                                .into_iter()
                                .map(|b| {
                                    let name = api::text(&b, "name");
                                    let selected = name == branch.get();
                                    view! {
                                        <option value=name.clone() selected=selected>
                                            {name.clone()}
                                        </option>
                                    }
                                })
                                .collect_view()
                        }}
                    </select>
                    <thaw::Button
                        appearance=thaw::ButtonAppearance::Primary
                        on_click=move |_| create.set(true)
                    >
                        {crate::i18n::t("＋ Create file")}
                    </thaw::Button>
                </div>
            </div>
            <Show when=move || !error.get().is_empty()>
                <p class="error" role="alert">
                    {move || error.get()}
                </p>
            </Show>
            <div class="source-columns">
                <aside class="source-navigation">
                    <div class="source-strip">
                        <strong>{crate::i18n::t("▤ Project files")}</strong>
                        <small>{move || api::list(&tree.data.get()).len()}</small>
                    </div>
                    <SourceSearch
                        project_path=project_path.clone()
                        branch=branch
                        on_open=open_file
                    />
                    <Status q=tree />
                    <div class="source-tree">
                        {move || {
                            api::list(&tree.data.get())
                                .into_iter()
                                .map(|entry| {
                                    view! {
                                        <TreeNode
                                            entry=entry
                                            project_path=root_path.clone()
                                            branch=branch
                                            selected=Signal::derive(move || api::text(
                                                &blob.get(),
                                                "path",
                                            ))
                                            on_open=open_file
                                            depth=0
                                        />
                                    }
                                })
                                .collect_view()
                        }}
                        <Show when=move || {
                            !tree.loading.get() && api::list(&tree.data.get()).is_empty()
                        }>
                            <p class="muted">{crate::i18n::t("No files found.")}</p>
                        </Show>
                    </div>
                </aside>
                <main class="source-canvas">
                    <div class="source-strip">
                        <strong>
                            {move || {
                                let p = api::text(&blob.get(), "path");
                                if p.is_empty() {
                                    let p = api::text(&readme.data.get(), "path");
                                    if p.is_empty() { "README Preview".into() } else { p }
                                } else {
                                    p
                                }
                            }}
                        </strong>
                        <small>{move || api::text(&blob.get(), "size")}</small>
                    </div>
                    {move || {
                        let data = blob.get();
                        if !data.is_null() {
                            if data["is_binary"] == true {
                                return view! {
                                    <div class="source-empty">
                                        {crate::i18n::t(
                                            "This file is binary cannot be shown in Monaco.",
                                        )}
                                    </div>
                                }
                                    .into_any();
                            }
                            let file = api::text(&data, "path");
                            let position = target.get();
                            view! {
                                <CodeEditor
                                    id=format!("source-{}", api::enc(&file))
                                    name="content".into()
                                    content=api::text(&data, "content")
                                    readonly=true
                                    language=language(&file)
                                    line=position.as_ref().map(|v| v.line).unwrap_or(1)
                                    column=position.as_ref().map(|v| v.column).unwrap_or(1)
                                    match_length=position
                                        .as_ref()
                                        .map(|v| v.match_length)
                                        .unwrap_or(0)
                                />
                            }
                                .into_any()
                        } else {
                            let content = api::text(&readme.data.get(), "content");
                            if content.is_empty() {
                                view! {
                                    <div class="source-empty">
                                        <strong>{crate::i18n::t("README not found.")}</strong>
                                        <p>
                                            {crate::i18n::t(
                                                "Select a source file from the left to open in Monaco.",
                                            )}
                                        </p>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="source-readme">
                                        <p class="muted">{crate::i18n::t("README Preview")}</p>
                                        <Markdown content=content />
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    }}
                </main>
                <Intelligence project_path=project_path.clone() branch=branch />
            </div>
            <thaw::Dialog open=create>
                <thaw::DialogSurface>
                    <thaw::DialogBody>
                        <thaw::DialogTitle>{crate::i18n::t("Create file")}</thaw::DialogTitle>
                        <thaw::DialogContent>
                            {move || {
                                view! {
                                    <EditorForm
                                        title="Commit new file".into()
                                        path=format!("{project_path}/repository/files")
                                        fields=vec![
                                            field("branch_name", "Branch", "text", true, "main"),
                                            field("path", "File path", "text", true, ""),
                                            field(
                                                "message",
                                                "Commit message",
                                                "text",
                                                true,
                                                "Add new file",
                                            ),
                                            field("content", "Content", "code", false, ""),
                                        ]
                                        initial=json!({"branch_name":branch.get()})
                                        embedded=true
                                        refresh=tree.refresh
                                        on_saved=Callback::new(move |_| create.set(false))
                                    />
                                }
                            }}
                        </thaw::DialogContent>
                    </thaw::DialogBody>
                </thaw::DialogSurface>
            </thaw::Dialog>
        </section>
    }
}
mod language;
use language::language;
