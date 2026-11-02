//! Generate REST wrappers and protocol-neutral application operation adapters.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Attribute, FnArg, ItemFn, Pat, Type, parse_macro_input, parse_quote};

mod syntax;
use syntax::{
  Extractor, OperationArgs, extractor, extractor_pattern, is_utoipa_path, operation_route,
  pattern_expression, strip_mutability,
};

#[proc_macro_attribute]
pub fn application_operation(attr: TokenStream, item: TokenStream) -> TokenStream {
  let item = parse_macro_input!(item as ItemFn);
  let attr = TokenStream2::from(attr);
  match expand(attr, item) {
    Ok(tokens) => tokens.into(),
    Err(error) => error.into_compile_error().into(),
  }
}

fn expand(attr: TokenStream2, mut item: ItemFn) -> syn::Result<TokenStream2> {
  let args = syn::parse2::<OperationArgs>(attr)?;
  let (method, path) = operation_route(&item.attrs)?;
  let name = item.sig.ident.clone();
  let shared_name = format_ident!("__application_{}", name);
  let invoke_name = format_ident!("__invoke_{}", name);
  let original_body = item.block.clone();
  let mut shared_sig = item.sig.clone();
  shared_sig.ident = shared_name.clone();
  let mut invoke_params = Vec::new();
  let mut wrapper_args = Vec::new();
  let mut shared_args = Vec::new();
  let mut has_headers = false;

  for input in &mut shared_sig.inputs {
    let FnArg::Typed(arg) = input else {
      return Err(syn::Error::new_spanned(
        input,
        "`self` receivers are unsupported in application operations",
      ));
    };
    let pattern = (*arg.pat).clone();
    let extractor = extractor(&arg.ty)?;
    match extractor {
      Extractor::State(inner) => {
        let pattern = extractor_pattern(&pattern, "State")?;
        *arg.pat = pattern.clone();
        *arg.ty = inner.clone();
        let expr = pattern_expression(&pattern)?;
        wrapper_args.push(quote!(#expr));
        shared_args.push(quote!(__operation_state.clone()));
      }
      Extractor::Path(inner) => {
        let pattern = extractor_pattern(&pattern, "Path")?;
        *arg.pat = pattern.clone();
        *arg.ty = inner.clone();
        let expr = pattern_expression(&pattern)?;
        wrapper_args.push(quote!(#expr));
        let binding = format_ident!("__operation_path_{}", invoke_params.len());
        shared_args.push(quote!(#binding));
        invoke_params.push(quote! {
          let #binding: #inner = args.path::<#inner>()?;
        });
      }
      Extractor::Query(inner) => {
        let pattern = extractor_pattern(&pattern, "Query")?;
        *arg.pat = pattern.clone();
        *arg.ty = inner.clone();
        let expr = pattern_expression(&pattern)?;
        wrapper_args.push(quote!(#expr));
        let binding = format_ident!("__operation_query_{}", invoke_params.len());
        shared_args.push(quote!(#binding));
        invoke_params.push(quote! {
          let #binding: #inner = args.query::<#inner>()?;
        });
      }
      Extractor::Json(inner) => {
        let pattern = extractor_pattern(&pattern, "Json")?;
        *arg.pat = pattern.clone();
        *arg.ty = inner.clone();
        let expr = pattern_expression(&pattern)?;
        wrapper_args.push(quote!(#expr));
        let binding = format_ident!("__operation_body_{}", invoke_params.len());
        shared_args.push(quote!(#binding));
        invoke_params.push(quote! {
          let #binding: #inner = args.body::<#inner>()?;
        });
      }
      Extractor::Headers => {
        let Pat::Ident(ident) = &pattern else {
          return Err(syn::Error::new_spanned(
            pattern,
            "HeaderMap extractor must use a simple identifier pattern",
          ));
        };
        has_headers = true;
        *arg.ty = parse_quote!(gity_api_contracts::ApiContext);
        let ident = ident.ident.clone();
        shared_args.push(quote!(#ident));
        wrapper_args.push(quote!(crate::api_application::context(
          &#ident,
          crate::api_application::ApiTransport::Rest,
        )));
        invoke_params.push(quote! {
          let #ident = __operation_context.clone();
        });
      }
    }
  }

  let original_attrs = std::mem::take(&mut item.attrs);
  for input in &mut item.sig.inputs {
    if let FnArg::Typed(argument) = input {
      strip_mutability(&mut argument.pat);
    }
  }
  let shared_attrs: Vec<Attribute> = original_attrs
    .iter()
    .filter(|attr| !is_utoipa_path(attr))
    .cloned()
    .collect();
  item.attrs = original_attrs;
  let wrapper_body = quote!({ #shared_name(#(#wrapper_args),*).await });
  *item.block = syn::parse2(wrapper_body)?;

  let result_conversion = quote!(crate::api_application::IntoApiOutput::into_api_output(
    #shared_name(#(#shared_args),*).await
  ));
  let return_type: Type = parse_quote!(crate::api_application::InvocationFuture);
  let prefix = args.prefix;
  let shared_body = if has_headers {
    let header_ident = shared_sig.inputs.iter().find_map(|input| match input {
      FnArg::Typed(arg) if matches!(&*arg.ty, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "ApiContext")) => match &*arg.pat {
        Pat::Ident(ident) => Some(ident.ident.clone()),
        _ => None,
      },
      _ => None,
    }).expect("validated HeaderMap argument");
    quote!({
      let #header_ident = crate::api_application::headers(&#header_ident);
      #original_body
    })
  } else {
    quote!(#original_body)
  };
  let registration = quote! {
    inventory::submit! {
      crate::api_application::OperationRegistration {
        prefix: #prefix,
        method: #method,
        path: #path,
        handler: concat!(module_path!(), "::", stringify!(#name)),
        invoke: #invoke_name,
      }
    }
  };

  Ok(quote! {
    #item

    #(#shared_attrs)*
    #shared_sig #shared_body

    pub(crate) fn #invoke_name(
      __operation_state: crate::api_application::AppState,
      __operation_context: gity_api_contracts::ApiContext,
      args: crate::api_application::CallArguments,
    ) -> #return_type {
      Box::pin(async move {
        #(#invoke_params)*
        #result_conversion
      })
    }

    #registration
  })
}

#[cfg(test)]
mod tests;
