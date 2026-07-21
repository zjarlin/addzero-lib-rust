//! Directory-aware module collection macro for addzero Rust crates.
//!
//! `az-automod` keeps the familiar `automod::dir!` call shape while adding
//! support for directories that do not already have same-name `.rs` entry
//! files.
//!
//! ```ignore
//! automod::dir!(pub "src");
//! ```
//!
//! The path is relative to `CARGO_MANIFEST_DIR`.
//!
//! Source files become modules, excluding `mod.rs`, `lib.rs`, and `main.rs`.
//! Directories without a same-name `.rs` entry file become inline modules
//! recursively. Directories with a same-name `.rs` entry file are left to that
//! entry file. `src/bin` keeps its Cargo meaning and is not collected.

#![cfg_attr(az_automod_nightly_tracked_path, feature(proc_macro_tracked_path))]
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

extern crate proc_macro;

mod error;

use crate::error::{Error, Result};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Visibility, parse_macro_input};

struct Arg {
    vis: Visibility,
    path: LitStr,
}

impl Parse for Arg {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            vis: input.parse()?,
            path: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn dir(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Arg);
    let vis = &input.vis;
    let rel_path = input.path.value();

    let dir = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(manifest_dir) => PathBuf::from(manifest_dir).join(rel_path),
        None => PathBuf::from(rel_path),
    };

    let expanded = match source_modules(dir) {
        Ok(modules) => modules
            .into_iter()
            .map(|module| mod_item(vis, module))
            .collect(),
        Err(err) => syn::Error::new(Span::call_site(), err).to_compile_error(),
    };

    TokenStream::from(expanded)
}

fn mod_item(vis: &Visibility, module: Module) -> TokenStream2 {
    let module_name = normalize_module_name(&module.source_name);
    let ident = Ident::new(&module_name, Span::call_site());

    match module.kind {
        ModuleKind::File => {
            let path = Option::into_iter(if module.source_name == module_name {
                None
            } else {
                Some(format!("{}.rs", module.source_name))
            });

            quote! {
                #(#[path = #path])*
                #vis mod #ident;
            }
        }
        ModuleKind::Directory { items } => {
            let path = Option::into_iter(if module.source_name == module_name {
                None
            } else {
                Some(module.source_name)
            });
            let items = items.into_iter().map(|module| mod_item(vis, module));

            quote! {
                #(#[path = #path])*
                #vis mod #ident {
                    #(#items)*
                }
            }
        }
    }
}

fn normalize_module_name(name: &str) -> String {
    let mut module_name = name.replace('-', "_");
    if module_name.starts_with(|ch: char| ch.is_ascii_digit()) {
        module_name.insert(0, '_');
    }
    module_name
}

struct Module {
    source_name: String,
    kind: ModuleKind,
}

enum ModuleKind {
    File,
    Directory { items: Vec<Module> },
}

fn source_modules<P: AsRef<Path>>(dir: P) -> Result<Vec<Module>> {
    let dir = dir.as_ref();
    track_path(dir);

    let mut files = BTreeMap::new();
    let mut dirs = BTreeMap::new();
    let mut failures = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let file_name = entry.file_name();

        if file_type.is_file() {
            collect_file_name(file_name, &mut files, &mut failures);
        } else if file_type.is_dir() && should_collect_dir(entry.path().as_path()) {
            collect_dir_name(file_name, entry.path(), &mut dirs, &mut failures);
        }
    }

    failures.sort();
    if let Some(failure) = failures.into_iter().next() {
        return Err(Error::Utf8(failure));
    }

    let file_module_names: BTreeSet<_> = files.keys().cloned().collect();
    let mut modules = Vec::new();

    for (_, source_name) in files {
        modules.push(Module {
            source_name,
            kind: ModuleKind::File,
        });
    }

    for (module_name, (source_name, path)) in dirs {
        if file_module_names.contains(&module_name) {
            continue;
        }

        let items = source_modules(path)?;
        modules.push(Module {
            source_name,
            kind: ModuleKind::Directory { items },
        });
    }

    modules.sort_by(|left, right| {
        normalize_module_name(&left.source_name).cmp(&normalize_module_name(&right.source_name))
    });

    if modules.is_empty() {
        return Err(Error::Empty);
    }

    Ok(modules)
}

#[cfg(az_automod_nightly_tracked_path)]
fn track_path(path: &Path) {
    proc_macro::tracked::path(path);
}

#[cfg(not(az_automod_nightly_tracked_path))]
fn track_path(_: &Path) {}

fn should_collect_dir(path: &Path) -> bool {
    !(path.file_name() == Some(OsStr::new("bin"))
        && path.parent().and_then(Path::file_name) == Some(OsStr::new("src")))
}

fn collect_file_name(
    file_name: OsString,
    files: &mut BTreeMap<String, String>,
    failures: &mut Vec<OsString>,
) {
    if file_name == "mod.rs" || file_name == "lib.rs" || file_name == "main.rs" {
        return;
    }

    let path = Path::new(&file_name);
    if path.extension() != Some(OsStr::new("rs")) {
        return;
    }

    match file_name.into_string() {
        Ok(mut utf8) => {
            utf8.truncate(utf8.len() - ".rs".len());
            files.insert(normalize_module_name(&utf8), utf8);
        }
        Err(non_utf8) => {
            failures.push(non_utf8);
        }
    }
}

fn collect_dir_name(
    file_name: OsString,
    path: PathBuf,
    dirs: &mut BTreeMap<String, (String, PathBuf)>,
    failures: &mut Vec<OsString>,
) {
    match file_name.into_string() {
        Ok(utf8) => {
            dirs.insert(normalize_module_name(&utf8), (utf8, path));
        }
        Err(non_utf8) => {
            failures.push(non_utf8);
        }
    }
}
