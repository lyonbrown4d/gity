use super::*;
#[derive(Clone)]
pub struct Field {
  pub key: &'static str,
  pub label: &'static str,
  pub kind: &'static str,
  pub required: bool,
  pub default: &'static str,
}
pub fn field(
  key: &'static str,
  label: &'static str,
  kind: &'static str,
  required: bool,
  default: &'static str,
) -> Field {
  Field {
    key,
    label,
    kind,
    required,
    default,
  }
}

#[component]
pub(super) fn FormField(f: Field, initial: Value, title: String) -> impl IntoView {
  let raw = if f.kind == "array" {
    initial
      .get(f.key)
      .and_then(Value::as_array)
      .map(|items| {
        items
          .iter()
          .filter_map(Value::as_str)
          .collect::<Vec<_>>()
          .join(",")
      })
      .unwrap_or_else(|| f.default.into())
  } else if initial.get(f.key).is_some() {
    api::text(&initial, f.key)
  } else {
    f.default.into()
  };
  let id = format!("form-{}-{}", title.replace(' ', "-"), f.key);
  let input = if f.kind == "code" {
    view! { <crate::editors::CodeEditor id=id.clone() name=f.key.to_string() content=raw /> }
      .into_any()
  } else if f.kind == "org" || f.kind == "user" {
    view! {
        <Lookup
            id=id.clone()
            name=f.key.to_string()
            kind=f.kind
            initial=raw
            required=f.required
        />
    }
    .into_any()
  } else if f.kind == "markdown" {
    view! { <crate::editors::MarkdownEditor id=id.clone() name=f.key.to_string() content=raw /> }
      .into_any()
  } else if f.kind == "textarea" || f.kind == "json" {
    view! { <textarea id=id.clone() name=f.key required=f.required rows="6" prop:value=raw></textarea> }
                                    .into_any()
  } else if f.kind == "checkbox" {
    view! { <input id=id.clone() name=f.key type="checkbox" prop:checked=raw == "true" /> }
      .into_any()
  } else if f.kind.starts_with("select:") {
    let selected = raw.clone();
    view! {
        <select id=id.clone() name=f.key>
            {f
                .kind[7..]
                .split('|')
                .map(|o| {
                    view! {
                        <option value=o.to_string() selected=o == selected>
                            {o.to_string()}
                        </option>
                    }
                })
                .collect_view()}
        </select>
    }
    .into_any()
  } else {
    view! {
        <input
            id=id.clone()
            name=f.key
            type=if f.kind == "password" {
                "password"
            } else if f.kind == "number" {
                "number"
            } else if f.kind == "email" {
                "email"
            } else {
                "text"
            }
            required=f.required
            prop:value=raw
        />
    }
    .into_any()
  };
  view! {
      <label
          for=id
          class=if matches!(f.kind, "textarea" | "markdown" | "json" | "code") {
              "wide"
          } else {
              ""
          }
      >
          <span>{crate::i18n::t(f.label)}</span>
          {input}
      </label>
  }
}
#[component]
fn Lookup(
  id: String,
  name: String,
  kind: &'static str,
  initial: String,
  required: bool,
) -> impl IntoView {
  let q = if kind == "org" {
    query("/orgs".into())
  } else {
    query_all_pages("/users".into())
  };
  view! {
      <select id=id name=name required=required>
          <option value="">{crate::i18n::t("Select…")}</option>
          {move || {
              api::list(&q.data.get())
                  .into_iter()
                  .map(|v| {
                      let key = api::text(&v, "id");
                      let label = api::text(&v, if kind == "org" { "name" } else { "username" });
                      view! {
                          <option value=key.clone() selected=key == initial>
                              {label}
                              " ("
                              {key.clone()}
                              ")"
                          </option>
                      }
                  })
                  .collect_view()
          }}
      </select>
      <Status q=q />
  }
}
