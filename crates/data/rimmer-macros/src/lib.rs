//! rimmer 的过程宏入口。

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Error, Fields, Ident, LitStr, Result, Type, parse_macro_input,
    spanned::Spanned,
};

/// 为实体结构体生成 Jimmer 风格元模型 API。
#[proc_macro_derive(Entity, attributes(rimmer))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_entity(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_entity(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let entity_ident = input.ident;
    let type_name = entity_ident.to_string();
    let table_name = parse_entity_table_name(&input.attrs, &type_name)?;
    let fields = parse_named_fields(input.data)?;
    let static_ident = format_ident!("__RIMMER_{}_FIELDS", to_upper_ident(&type_name));

    let field_metadata = fields.iter().map(|field| {
        let rust_name = field.ident.to_string();
        let column_name = &field.column_name;
        let kind = field.kind.tokens();
        quote! {
            ::rimmer::FieldMetadata::new(#rust_name, #column_name, #kind)
        }
    });

    let field_methods = fields.iter().map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;
        let rust_name = field.ident.to_string();
        let column_name = &field.column_name;
        let kind = field.kind.tokens();
        quote! {
            #[doc = concat!("返回字段 `", stringify!(#ident), "` 的强类型表达式。")]
            pub fn #ident() -> ::rimmer::Field<Self, #ty> {
                ::rimmer::Field::new(Self::entity(), #rust_name, #column_name, #kind)
            }
        }
    });

    Ok(quote! {
        static #static_ident: &[::rimmer::FieldMetadata] = &[
            #(#field_metadata,)*
        ];

        impl #entity_ident {
            /// 返回当前实体的元模型定义。
            pub fn entity() -> ::rimmer::EntityDef<Self> {
                ::rimmer::EntityDef::new(#type_name, #table_name, #static_ident)
            }

            /// 返回当前实体的表对象。
            pub fn table() -> ::rimmer::Table<Self> {
                ::rimmer::Table::new(Self::entity())
            }

            /// 返回当前实体的 Fetcher 创建器。
            pub fn fetcher() -> ::rimmer::FetcherCreator<Self> {
                ::rimmer::new_fetcher(Self::entity())
            }

            /// 使用闭包创建当前实体的部分对象 Draft。
            pub fn draft<F>(block: F) -> ::rimmer::Draft<Self>
            where
                F: FnOnce(::rimmer::Draft<Self>) -> ::rimmer::Draft<Self>,
            {
                ::rimmer::new_draft(Self::entity()).by(block)
            }

            #(#field_methods)*
        }

        impl ::rimmer::Entity for #entity_ident {
            fn entity() -> ::rimmer::EntityDef<Self> {
                Self::entity()
            }
        }
    })
}

fn parse_entity_table_name(attrs: &[syn::Attribute], type_name: &str) -> Result<String> {
    let mut table_name = None;
    for attr in attrs {
        if !attr.path().is_ident("rimmer") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table") {
                let value = meta.value()?;
                let literal: LitStr = value.parse()?;
                table_name = Some(literal.value());
                return Ok(());
            }
            Err(meta.error("unsupported rimmer entity attribute"))
        })?;
    }
    Ok(table_name.unwrap_or_else(|| to_upper_snake(type_name)))
}

fn parse_named_fields(data: Data) -> Result<Vec<EntityField>> {
    let Data::Struct(data) = data else {
        return Err(Error::new(
            Span::call_site(),
            "rimmer Entity can only be derived for structs",
        ));
    };
    let Fields::Named(fields) = data.fields else {
        return Err(Error::new(
            data.struct_token.span(),
            "rimmer Entity requires named struct fields",
        ));
    };

    fields
        .named
        .into_iter()
        .map(|field| {
            let span = field.span();
            let ident = field
                .ident
                .ok_or_else(|| Error::new(span, "rimmer Entity requires named fields"))?;
            let rust_name = ident.to_string();
            let (kind, column_name) = parse_field_attrs(&field.attrs, &rust_name)?;
            Ok(EntityField {
                ident,
                ty: field.ty,
                column_name,
                kind,
            })
        })
        .collect()
}

fn parse_field_attrs(
    attrs: &[syn::Attribute],
    rust_name: &str,
) -> Result<(GeneratedFieldKind, String)> {
    let mut kind = GeneratedFieldKind::Scalar;
    let mut column_name = None;
    for attr in attrs {
        if !attr.path().is_ident("rimmer") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if let Some(parsed_kind) = GeneratedFieldKind::from_path(&meta.path) {
                kind = parsed_kind;
                return Ok(());
            }
            if meta.path.is_ident("column") {
                let value = meta.value()?;
                let literal: LitStr = value.parse()?;
                column_name = Some(literal.value());
                return Ok(());
            }
            Err(meta.error("unsupported rimmer field attribute"))
        })?;
    }
    Ok((
        kind,
        column_name.unwrap_or_else(|| to_upper_snake(rust_name)),
    ))
}

struct EntityField {
    ident: Ident,
    ty: Type,
    column_name: String,
    kind: GeneratedFieldKind,
}

#[derive(Clone, Copy)]
enum GeneratedFieldKind {
    Id,
    Key,
    Scalar,
    ManyToOne,
    OneToMany,
    ManyToMany,
    Transient,
    IdView,
}

impl GeneratedFieldKind {
    fn from_path(path: &syn::Path) -> Option<Self> {
        if path.is_ident("id") {
            return Some(Self::Id);
        }
        if path.is_ident("key") {
            return Some(Self::Key);
        }
        if path.is_ident("scalar") {
            return Some(Self::Scalar);
        }
        if path.is_ident("many_to_one") {
            return Some(Self::ManyToOne);
        }
        if path.is_ident("one_to_many") {
            return Some(Self::OneToMany);
        }
        if path.is_ident("many_to_many") {
            return Some(Self::ManyToMany);
        }
        if path.is_ident("transient") {
            return Some(Self::Transient);
        }
        if path.is_ident("id_view") {
            return Some(Self::IdView);
        }
        None
    }

    fn tokens(self) -> proc_macro2::TokenStream {
        match self {
            Self::Id => quote!(::rimmer::FieldKind::Id),
            Self::Key => quote!(::rimmer::FieldKind::Key),
            Self::Scalar => quote!(::rimmer::FieldKind::Scalar),
            Self::ManyToOne => quote!(::rimmer::FieldKind::ManyToOne),
            Self::OneToMany => quote!(::rimmer::FieldKind::OneToMany),
            Self::ManyToMany => quote!(::rimmer::FieldKind::ManyToMany),
            Self::Transient => quote!(::rimmer::FieldKind::Transient),
            Self::IdView => quote!(::rimmer::FieldKind::IdView),
        }
    }
}

fn to_upper_ident(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn to_upper_snake(value: &str) -> String {
    let mut out = String::new();
    let mut previous_is_lower_or_digit = false;
    for ch in value.chars() {
        if ch.is_ascii_uppercase() {
            if previous_is_lower_or_digit {
                out.push('_');
            }
            out.push(ch);
            previous_is_lower_or_digit = false;
            continue;
        }
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
            previous_is_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
            continue;
        }
        if !out.ends_with('_') {
            out.push('_');
        }
        previous_is_lower_or_digit = false;
    }
    out
}
