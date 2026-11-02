use proc_macro2::TokenStream;
use quote::quote;
use syn::{
  Attribute, GenericArgument, LitStr, Meta, Pat, PathArguments, Token, Type,
  parse::{Parse, ParseStream, Parser},
  punctuated::Punctuated,
};

pub(super) struct OperationArgs {
  pub(super) prefix: LitStr,
}

impl Parse for OperationArgs {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    let key: syn::Ident = input.parse()?;
    if key != "prefix" {
      return Err(syn::Error::new(key.span(), "expected `prefix = \"...\"`"));
    }
    input.parse::<Token![=]>()?;
    let prefix = input.parse()?;
    if !input.is_empty() {
      return Err(input.error("unexpected tokens after operation prefix"));
    }
    Ok(Self { prefix })
  }
}

#[derive(Clone)]
pub(super) enum Extractor {
  State(Type),
  Path(Type),
  Query(Type),
  Json(Type),
  Headers,
}

pub(super) fn operation_route(attrs: &[Attribute]) -> syn::Result<(LitStr, LitStr)> {
  for attr in attrs {
    if !is_utoipa_path(attr) {
      continue;
    }
    let Meta::List(list) = &attr.meta else {
      continue;
    };
    let metas = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(list.tokens.clone())?;
    let mut method = None;
    let mut path = None;
    for meta in metas {
      match meta {
        Meta::Path(value) if value.segments.len() == 1 => {
          let candidate = value.segments[0].ident.to_string().to_ascii_uppercase();
          if ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"]
            .contains(&candidate.as_str())
          {
            method = Some(LitStr::new(&candidate, value.segments[0].ident.span()));
          }
        }
        Meta::NameValue(value) if value.path.is_ident("path") => {
          if let syn::Expr::Lit(expr) = value.value
            && let syn::Lit::Str(value) = expr.lit
          {
            path = Some(value);
          }
        }
        _ => {}
      }
    }
    return match (method, path) {
      (Some(method), Some(path)) => Ok((method, path)),
      _ => Err(syn::Error::new_spanned(
        attr,
        "expected an existing `#[utoipa::path(method, path = \"...\")]` attribute",
      )),
    };
  }
  Err(syn::Error::new(
    proc_macro2::Span::call_site(),
    "application operation requires an existing `#[utoipa::path(...)]` attribute",
  ))
}

pub(super) fn is_utoipa_path(attr: &Attribute) -> bool {
  let segments: Vec<_> = attr
    .path()
    .segments
    .iter()
    .map(|segment| segment.ident.to_string())
    .collect();
  segments == ["utoipa", "path"]
}

pub(super) fn extractor(ty: &Type) -> syn::Result<Extractor> {
  if is_header_type(ty) {
    return Ok(Extractor::Headers);
  }
  let Type::Path(path) = ty else {
    return Err(unsupported_extractor(ty));
  };
  let Some(segment) = path.path.segments.last() else {
    return Err(syn::Error::new_spanned(
      ty,
      "unsupported empty extractor type",
    ));
  };
  if !["State", "Path", "Query", "Json"].contains(&segment.ident.to_string().as_str()) {
    return Err(unsupported_extractor(ty));
  }
  let inner = match &segment.arguments {
    PathArguments::AngleBracketed(args) => args.args.iter().find_map(|arg| match arg {
      GenericArgument::Type(ty) => Some(ty.clone()),
      _ => None,
    }),
    _ => None,
  };
  let Some(inner) = inner else {
    return Err(syn::Error::new_spanned(
      ty,
      "extractor must have a concrete type parameter",
    ));
  };
  match segment.ident.to_string().as_str() {
    "State" => Ok(Extractor::State(inner)),
    "Path" => Ok(Extractor::Path(inner)),
    "Query" => Ok(Extractor::Query(inner)),
    "Json" => Ok(Extractor::Json(inner)),
    _ => Err(unsupported_extractor(ty)),
  }
}

fn unsupported_extractor(ty: &Type) -> syn::Error {
  syn::Error::new_spanned(
    ty,
    "unsupported application operation extractor; use State, Path, Query, Json, or HeaderMap",
  )
}

pub(super) fn extractor_pattern(pattern: &Pat, name: &str) -> syn::Result<Pat> {
  let Pat::TupleStruct(tuple) = pattern else {
    return Err(syn::Error::new_spanned(
      pattern,
      format!("{name} extractor must use its standard tuple pattern"),
    ));
  };
  if tuple
    .path
    .segments
    .last()
    .is_none_or(|segment| segment.ident != name)
    || tuple.elems.len() != 1
  {
    return Err(syn::Error::new_spanned(
      pattern,
      format!("{name} extractor must bind exactly one value"),
    ));
  }
  Ok(tuple.elems.first().expect("one pattern checked").clone())
}

fn is_header_type(ty: &Type) -> bool {
  matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "HeaderMap"))
}

pub(super) fn pattern_expression(pattern: &Pat) -> syn::Result<TokenStream> {
  match pattern {
    Pat::Ident(ident) => {
      let name = &ident.ident;
      Ok(quote!(#name))
    }
    Pat::Tuple(tuple) => {
      let values = tuple
        .elems
        .iter()
        .map(pattern_expression)
        .collect::<syn::Result<Vec<_>>>()?;
      Ok(quote!((#(#values),*)))
    }
    Pat::Paren(paren) => pattern_expression(&paren.pat),
    Pat::Reference(reference) => {
      let value = pattern_expression(&reference.pat)?;
      if reference.mutability.is_some() {
        Ok(quote!(&mut #value))
      } else {
        Ok(quote!(&#value))
      }
    }
    _ => Err(syn::Error::new_spanned(
      pattern,
      "extractor patterns must be identifiers or tuples",
    )),
  }
}

pub(super) fn strip_mutability(pattern: &mut Pat) {
  match pattern {
    Pat::Ident(pattern) => {
      pattern.mutability = None;
      if let Some((_, subpattern)) = &mut pattern.subpat {
        strip_mutability(subpattern);
      }
    }
    Pat::Tuple(pattern) => {
      for element in &mut pattern.elems {
        strip_mutability(element);
      }
    }
    Pat::TupleStruct(pattern) => {
      for element in &mut pattern.elems {
        strip_mutability(element);
      }
    }
    Pat::Struct(pattern) => {
      for field in &mut pattern.fields {
        strip_mutability(&mut field.pat);
      }
    }
    Pat::Reference(pattern) => strip_mutability(&mut pattern.pat),
    Pat::Paren(pattern) => strip_mutability(&mut pattern.pat),
    Pat::Slice(pattern) => {
      for element in &mut pattern.elems {
        strip_mutability(element);
      }
    }
    Pat::Type(pattern) => strip_mutability(&mut pattern.pat),
    Pat::Or(pattern) => {
      for case in &mut pattern.cases {
        strip_mutability(case);
      }
    }
    _ => {}
  }
}
