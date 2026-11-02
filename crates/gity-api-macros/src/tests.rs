use super::expand;
use quote::quote;
use syn::ItemFn;

fn expand_source(input: proc_macro2::TokenStream) -> Result<String, String> {
  let function = syn::parse2::<ItemFn>(input).map_err(|error| error.to_string())?;
  expand(quote!(prefix = "/api/v1/projects"), function)
    .map(|tokens| tokens.to_string().split_whitespace().collect())
    .map_err(|error| error.to_string())
}

#[test]
fn tuple_path_is_unwrapped_and_reconstructed_for_rest_wrapper() {
  let generated = expand_source(quote! {
    #[utoipa::path(get, path = "/{project_id}/{issue_id}", responses((status = 200)))]
    pub async fn show(
      State(app): State<AppState>,
      Path((project_id, issue_id)): Path<(i64, i64)>,
    ) -> Result<Json<Value>, AppError> {
      Ok(Json(json!([project_id, issue_id])))
    }
  })
  .unwrap();
  assert!(generated.contains("pubasyncfnshow"));
  assert!(generated.contains("Path((project_id,issue_id))"));
  assert!(generated.contains("__application_show"));
  assert!(generated.contains("(i64,i64)"));
  assert!(generated.contains("args.path::<(i64,i64)>()?"));
  assert!(generated.contains("IntoApiOutput::into_api_output"));
}

#[test]
fn headerless_json_operation_uses_typed_body_without_fabricated_auth() {
  let generated = expand_source(quote! {
    #[utoipa::path(post, path = "/{id}/issues", responses((status = 201)))]
    async fn create(Path(id): Path<i64>, Json(input): Json<CreateIssue>) -> Json<Value> {
      Json(json!({ "id": id, "title": input.title }))
    }
  })
  .unwrap();

  assert!(generated.contains("let__operation_body_1:CreateIssue=args.body::<CreateIssue>()?"));
  assert!(generated.contains("args.path::<i64>()?"));
  assert!(!generated.contains("api_application::context"));
  assert!(generated.contains(r#"prefix:"/api/v1/projects""#));
  assert!(generated.contains(r#"method:"POST""#));
}

#[test]
fn header_context_is_bridged_in_rest_and_shared_operation() {
  let generated = expand_source(quote! {
    #[utoipa::path(get, path = "/{id}/issues", responses((status = 200)))]
    async fn list(Path(id): Path<i64>, headers: HeaderMap) -> Json<Value> {
      Json(json!({ "id": id, "header": headers.get("x-test") }))
    }
  })
  .unwrap();
  assert!(generated.contains("context(&headers,crate::api_application::ApiTransport::Rest"));
  assert!(generated.contains("letheaders=crate::api_application::headers(&headers)"));
  assert!(generated.contains("letheaders=__operation_context.clone()"));
}

#[test]
fn unsupported_request_extractor_is_rejected() {
  let error = expand_source(quote! {
    #[utoipa::path(post, path = "/upload", responses((status = 200)))]
    async fn upload(request: Request) -> Json<Value> { Json(json!({})) }
  })
  .unwrap_err();

  assert!(error.contains("unsupported application operation extractor"));
}
