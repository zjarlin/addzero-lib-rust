//! 插件系统的过程宏集合，为插件开发者提供声明式的属性宏。
//!
//! 提供以下过程宏：
//! - `#[az_plugin]` - 标记插件入口函数（占位宏，当前透传输入）
//! - `#[az_page]` - 标记页面处理函数（占位宏，当前透传输入）
//! - `#[az_starter]` - 标记启动器函数，自动通过 `inventory` 机制注册到插件注册表
//!
//! `#[az_starter]` 是唯一具有实际功能的宏：它将被标注的函数包装为
//! [`az_plugin_registry::StarterRegistration`] 并提交到 `inventory`，
//! 使得插件内核在启动时能自动发现并调用所有注册的启动器。

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn az_plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn az_page(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn az_starter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let ident = function.sig.ident.clone();

    quote! {
        #function

        ::az_plugin_registry::inventory::submit! {
            ::az_plugin_registry::StarterRegistration {
                constructor: #ident,
            }
        }
    }
    .into()
}
