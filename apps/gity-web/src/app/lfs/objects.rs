use super::*;
#[component]
pub(super) fn Objects(objects: Query) -> impl IntoView {
  view! {
      <section class="lfs-box">
          <h3>
              <thaw::Icon icon=icondata_lu::LuDatabase width="16px" height="16px" />
              {t("Stored objects")}
          </h3>
          <div class="table-scroll">
              <table>
                  <thead>
                      <tr>
                          <th>{t("OID")}</th>
                          <th>{t("Size")}</th>
                          <th>{t("Updated")}</th>
                      </tr>
                  </thead>
                  <tbody>
                      <Show when=move || objects.loading.get()>
                          <tr>
                              <td colspan="3">{t("Loading LFS objects...")}</td>
                          </tr>
                      </Show>
                      <Show when=move || {
                          !objects.loading.get() && objects.error.get().is_empty()
                              && rows(objects, "objects").is_empty()
                      }>
                          <tr>
                              <td colspan="3">
                                  <div class="lfs-object-empty">
                                      <strong>{t("No LFS objects stored yet.")}</strong>
                                      <p>
                                          {t(
                                              "Tracked large files will appear here after the first LFS push.",
                                          )}
                                      </p>
                                  </div>
                              </td>
                          </tr>
                      </Show>
                      {move || {
                          rows(objects, "objects")
                              .into_iter()
                              .map(|row| {
                                  let date = api::text(&row, "updated_at");
                                  view! {
                                      <tr>
                                          <td class="lfs-oid" title=api::text(&row, "oid")>
                                              {api::text(&row, "oid")}
                                          </td>
                                          <td>
                                              {bytes(row["byte_size"].as_u64().unwrap_or_default())}
                                          </td>
                                          <td>
                                              {if date.is_empty() {
                                                  "--".into()
                                              } else {
                                                  relative_time(&date)
                                              }}
                                          </td>
                                      </tr>
                                  }
                              })
                              .collect_view()
                      }}
                  </tbody>
              </table>
          </div>
      </section>
  }
}
