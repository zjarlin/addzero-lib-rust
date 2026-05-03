use addzero_dict_spec::{DictionarySpec, RawValueKind};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, LitInt, LitStr, Result, Token, Type, parse_macro_input};

#[proc_macro]
pub fn dict_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DictEnumInput);
    expand_dict_enum(input).unwrap_or_else(|error| error).into()
}

fn expand_dict_enum(
    input: DictEnumInput,
) -> std::result::Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let spec_text = load_spec_text(&input.spec).map_err(compile_error)?;
    let spec = DictionarySpec::from_json_str(&spec_text)
        .map_err(|error| compile_error(error.to_string()))?;
    if spec.code != input.dict.value() {
        return Err(compile_error(format!(
            "dict code mismatch: expected {}, got {}",
            input.dict.value(),
            spec.code
        )));
    }

    let raw_type: Type = input.raw_type.unwrap_or_else(|| match spec.raw_value_kind {
        RawValueKind::Int => syn::parse_quote!(i64),
        RawValueKind::String => syn::parse_quote!(&'static str),
    });
    let enum_name = input.name;
    let dict_code = input.dict;

    let mut variant_names = BTreeSet::new();
    let mut unit_variants = Vec::new();
    let mut code_arms = Vec::new();
    let mut label_arms = Vec::new();
    let mut description_arms = Vec::new();
    let mut meta_arms = Vec::new();
    let mut raw_arms = Vec::new();
    let mut from_raw_arms = Vec::new();
    let mut try_from_raw_arms = Vec::new();
    let mut item_entries = Vec::new();

    for item in &spec.items {
        let variant_name = to_pascal_case(&item.code);
        if !variant_names.insert(variant_name.clone()) {
            return Err(compile_error(format!(
                "duplicate normalized variant name for item code {}",
                item.code
            )));
        }
        let variant_ident = format_ident!("{variant_name}");
        let code = LitStr::new(&item.code, proc_macro2::Span::call_site());
        let label = LitStr::new(&item.label, proc_macro2::Span::call_site());
        let description = LitStr::new(item.description_text(), proc_macro2::Span::call_site());
        let meta_json: Option<String> = item
            .meta
            .as_ref()
            .map(|value| serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()));

        let (raw_expr, raw_pattern) = match spec.raw_value_kind {
            RawValueKind::Int => {
                let value = item.raw_int_value.expect("validated raw int");
                let lit = LitInt::new(&value.to_string(), proc_macro2::Span::call_site());
                (quote! { #lit as #raw_type }, quote! { #lit })
            }
            RawValueKind::String => {
                let value = item.raw_text_value.as_deref().expect("validated raw text");
                let lit = LitStr::new(value, proc_macro2::Span::call_site());
                (quote! { #lit }, quote! { #lit })
            }
        };

        unit_variants.push(quote! { #variant_ident });
        code_arms.push(quote! { Self::#variant_ident => #code });
        label_arms.push(quote! { Self::#variant_ident => #label });
        description_arms.push(quote! { Self::#variant_ident => #description });
        if let Some(meta_json) = meta_json {
            let meta_lit = LitStr::new(&meta_json, proc_macro2::Span::call_site());
            meta_arms.push(quote! { Self::#variant_ident => Some(#meta_lit) });
            item_entries.push(quote! {
                ::addzero_dict_spec::DictEnumItem {
                    code: #code,
                    label: #label,
                    description: #description,
                    raw_value: #raw_expr,
                    meta_json: Some(#meta_lit),
                }
            });
        } else {
            meta_arms.push(quote! { Self::#variant_ident => None });
            item_entries.push(quote! {
                ::addzero_dict_spec::DictEnumItem {
                    code: #code,
                    label: #label,
                    description: #description,
                    raw_value: #raw_expr,
                    meta_json: None,
                }
            });
        }
        raw_arms.push(quote! { Self::#variant_ident => #raw_expr });
        from_raw_arms.push(quote! { #raw_pattern => Self::#variant_ident });
        try_from_raw_arms.push(quote! { #raw_pattern => Some(Self::#variant_ident) });
    }

    let open_impl = if spec.open_enum {
        let unknown_ident = format_ident!("{}", to_pascal_case(spec.normalized_unknown_variant()));
        if !variant_names.insert(unknown_ident.to_string()) {
            return Err(compile_error("unknown variant collides with item variant"));
        }
        let from_raw_expr = quote! {
            pub fn from_raw(value: #raw_type) -> Self {
                match value {
                    #(#from_raw_arms,)*
                    other => Self::#unknown_ident(other),
                }
            }
        };
        let try_from_raw_expr = quote! {
            pub fn try_from_raw(value: #raw_type) -> Option<Self> {
                match value {
                    #(#try_from_raw_arms,)*
                    _ => None,
                }
            }
        };
        let code_unknown = LitStr::new(
            spec.normalized_unknown_variant(),
            proc_macro2::Span::call_site(),
        );
        unit_variants.push(quote! { #unknown_ident(#raw_type) });
        code_arms.push(quote! { Self::#unknown_ident(_) => #code_unknown });
        label_arms.push(quote! { Self::#unknown_ident(_) => #code_unknown });
        description_arms.push(quote! { Self::#unknown_ident(_) => "" });
        meta_arms.push(quote! { Self::#unknown_ident(_) => None });
        raw_arms.push(quote! { Self::#unknown_ident(value) => *value });
        quote! {
            #from_raw_expr
            #try_from_raw_expr
        }
    } else {
        quote! {
            pub fn from_raw(value: #raw_type) -> Option<Self> {
                match value {
                    #(#try_from_raw_arms,)*
                    _ => None,
                }
            }
        }
    };

    Ok(quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum #enum_name {
            #(#unit_variants,)*
        }

        impl #enum_name {
            pub const DICT_CODE: &'static str = #dict_code;

            #open_impl

            pub fn code(&self) -> &'static str {
                match self {
                    #(#code_arms,)*
                }
            }

            pub fn label(&self) -> &'static str {
                match self {
                    #(#label_arms,)*
                }
            }

            pub fn description(&self) -> &'static str {
                match self {
                    #(#description_arms,)*
                }
            }

            pub fn meta_json(&self) -> Option<&'static str> {
                match self {
                    #(#meta_arms,)*
                }
            }

            pub fn raw_value(&self) -> #raw_type {
                match self {
                    #(#raw_arms,)*
                }
            }

            pub fn items() -> &'static [::addzero_dict_spec::DictEnumItem<#raw_type>] {
                const ITEMS: &[::addzero_dict_spec::DictEnumItem<#raw_type>] = &[
                    #(#item_entries,)*
                ];
                ITEMS
            }
        }
    })
}

fn compile_error(message: impl Into<String>) -> proc_macro2::TokenStream {
    let message = message.into();
    quote! { compile_error!(#message); }
}

fn load_spec_text(expr: &Expr) -> std::result::Result<String, String> {
    match expr {
        Expr::Macro(expr_macro) if expr_macro.mac.path.is_ident("include_str") => {
            let include_expr = syn::parse2::<Expr>(expr_macro.mac.tokens.clone())
                .map_err(|_| "spec must be include_str!(\"...\")".to_string())?;
            let include_path = evaluate_string_expr(&include_expr)?;
            let spec_path = resolve_include_path(&include_path)?;
            fs::read_to_string(&spec_path).map_err(|error| {
                format!("failed to read spec file {}: {error}", spec_path.display())
            })
        }
        Expr::Lit(expr_lit) => {
            if let syn::Lit::Str(text) = &expr_lit.lit {
                Ok(text.value())
            } else {
                Err("spec must be a string literal or include_str!(...)".to_string())
            }
        }
        _ => Err("spec must be a string literal or include_str!(...)".to_string()),
    }
}

fn evaluate_string_expr(expr: &Expr) -> std::result::Result<String, String> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            syn::Lit::Str(value) => Ok(value.value()),
            _ => Err("string expression must be a string literal".to_string()),
        },
        Expr::Macro(expr_macro) if expr_macro.mac.path.is_ident("concat") => {
            let args = Punctuated::<Expr, Token![,]>::parse_terminated
                .parse2(expr_macro.mac.tokens.clone())
                .map_err(|_| "concat! arguments must be string expressions".to_string())?;
            let mut combined = String::new();
            for arg in args {
                combined.push_str(&evaluate_string_expr(&arg)?);
            }
            Ok(combined)
        }
        Expr::Macro(expr_macro) if expr_macro.mac.path.is_ident("env") => {
            let name = syn::parse2::<LitStr>(expr_macro.mac.tokens.clone())
                .map_err(|_| "env! expects a single string literal variable name".to_string())?;
            std::env::var(name.value()).map_err(|error| {
                format!(
                    "failed to resolve env!({:?}) while loading dict spec: {error}",
                    name.value()
                )
            })
        }
        _ => Err(
            "string expression must be a string literal, concat!(...), or env!(...)".to_string(),
        ),
    }
}

fn resolve_include_path(path_text: &str) -> std::result::Result<PathBuf, String> {
    let candidate = PathBuf::from(path_text);
    if candidate.is_absolute() {
        return Ok(normalize_path(candidate));
    }

    let source_file = resolve_call_site_file()?;
    let base_dir = source_file
        .parent()
        .ok_or_else(|| "failed to resolve macro call site directory".to_string())?;
    Ok(normalize_path(base_dir.join(candidate)))
}

// Resolve the source file path at the macro call site.
//
// Uses `proc_macro::Span::local_file()` and `Span::file()`, both stabilized in
// Rust 1.88.  Workspace MSRV is now 1.88 (see workspace Cargo.toml).
fn resolve_call_site_file() -> std::result::Result<PathBuf, String> {
    let span = proc_macro::Span::call_site();
    if let Some(path) = span.local_file() {
        return Ok(path);
    }
    let display_path = span.file();
    let path = PathBuf::from(&display_path);
    if path.as_os_str().is_empty() || display_path.starts_with('<') {
        return Err("failed to resolve macro call site source file".to_string());
    }
    Ok(path)
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn to_pascal_case(input: &str) -> String {
    let mut output = String::new();
    let mut capitalize = true;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if output.is_empty() && ch.is_ascii_digit() {
                output.push('V');
            }
            if capitalize {
                output.push(ch.to_ascii_uppercase());
                capitalize = false;
            } else {
                output.push(ch.to_ascii_lowercase());
            }
        } else {
            capitalize = true;
        }
    }
    if output.is_empty() {
        "Value".to_string()
    } else {
        output
    }
}

struct DictEnumInput {
    name: Ident,
    dict: LitStr,
    spec: Expr,
    raw_type: Option<Type>,
}

impl Parse for DictEnumInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut name = None;
        let mut dict = None;
        let mut spec = None;
        let mut raw_type = None;

        while !input.is_empty() {
            let field: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match field.to_string().as_str() {
                "name" => name = Some(input.parse()?),
                "dict" => dict = Some(input.parse()?),
                "spec" => spec = Some(input.parse()?),
                "raw_type" => raw_type = Some(input.parse()?),
                other => {
                    return Err(syn::Error::new(
                        field.span(),
                        format!("unsupported field: {other}"),
                    ));
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(Self {
            name: name.ok_or_else(|| input.error("missing name"))?,
            dict: dict.ok_or_else(|| input.error("missing dict"))?,
            spec: spec.ok_or_else(|| input.error("missing spec"))?,
            raw_type,
        })
    }
}
