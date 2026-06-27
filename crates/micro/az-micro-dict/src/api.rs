//! Dictionary contribution SPI and build-time enum source generation.
//!
//! Use this crate from `build.rs` when dictionary metadata is owned by a database
//! or an admin system instead of checked-in Rust source. Providers implement
//! [`DictionaryContributor`], and [`DictBuildGenerator`] writes validated JSON
//! specs plus an `enums.rs` file under the requested output directory.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use az_dict_spec::api::{DictionaryItemSpec, DictionarySpec, RawValueKind};
use az_str::api::{is_blank, to_pascal_case};
use az_str::sanitize::sanitize_file_stem as sanitize_ascii_file_stem;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Source-neutral dictionary metadata contributed before enum code generation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryContribution {
    pub enum_name: String,
    #[serde(flatten)]
    pub spec: DictionarySpec,
}

impl DictionaryContribution {
    pub fn new(enum_name: impl Into<String>, spec: DictionarySpec) -> Self {
        Self {
            enum_name: enum_name.into(),
            spec,
        }
    }

    fn validate_shape(&self) -> Result<()> {
        if is_blank(Some(&self.enum_name)) {
            bail!("invalid dictionary contribution: enum_name cannot be empty");
        }
        if to_pascal_case(&self.enum_name, "", "").is_empty() {
            bail!(
                "invalid dictionary contribution: enum_name {} is not a Rust type identifier",
                self.enum_name
            );
        }
        Ok(())
    }
}

/// Dictionary metadata provider SPI.
///
/// Implement this trait for database readers, RuoYi table adapters, checked-in
/// fixtures, or remote metadata clients. The trait is synchronous so it can be
/// called from ordinary Cargo build scripts without requiring a runtime.
pub trait DictionaryContributor {
    fn contribute(&self) -> Result<Vec<DictionaryContribution>>;
}

/// In-memory contributor for tests, fixtures, and static build scripts.
#[derive(Clone, Debug)]
pub struct StaticDictionaryContributor {
    dictionaries: Vec<DictionaryContribution>,
}

impl StaticDictionaryContributor {
    pub fn new(dictionaries: Vec<DictionaryContribution>) -> Self {
        Self { dictionaries }
    }
}

impl DictionaryContributor for StaticDictionaryContributor {
    fn contribute(&self) -> Result<Vec<DictionaryContribution>> {
        Ok(self.dictionaries.clone())
    }
}

/// One row from common RuoYi-style `sys_dict_type` + `sys_dict_data` queries.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuoyiDictRow {
    pub dict_type: String,
    pub dict_name: String,
    pub dict_label: String,
    pub dict_value: String,
    pub dict_sort: i64,
    pub status: String,
    pub remark: Option<String>,
    pub css_class: Option<String>,
    pub list_class: Option<String>,
}

/// Converts RuoYi-style dictionary rows into a single string-valued contribution.
#[derive(Clone, Debug)]
pub struct RuoyiDictionaryContributor {
    rows: Vec<RuoyiDictRow>,
    scope: String,
}

impl RuoyiDictionaryContributor {
    pub fn new(rows: Vec<RuoyiDictRow>, scope: impl Into<String>) -> Self {
        Self {
            rows,
            scope: scope.into(),
        }
    }
}

impl DictionaryContributor for RuoyiDictionaryContributor {
    fn contribute(&self) -> Result<Vec<DictionaryContribution>> {
        let mut dict_codes = BTreeSet::new();
        for row in &self.rows {
            dict_codes.insert(row.dict_type.clone());
        }

        let mut contributions = Vec::new();
        for dict_code in dict_codes {
            let mut rows = self
                .rows
                .iter()
                .filter(|row| row.dict_type == dict_code && row.status == "0")
                .cloned()
                .collect::<Vec<_>>();
            rows.sort_by_key(|row| row.dict_sort);
            let Some(first) = rows.first() else {
                continue;
            };
            let dict_name = first.dict_name.clone();

            let items = rows
                .into_iter()
                .map(|row| {
                    let meta = ruoyi_meta_json(&row);
                    DictionaryItemSpec {
                        code: row.dict_value.to_case(Case::Snake),
                        label: row.dict_label,
                        description: row.remark,
                        raw_int_value: None,
                        raw_text_value: Some(row.dict_value),
                        sort_index: row.dict_sort,
                        enabled: true,
                        meta,
                    }
                })
                .collect();

            contributions.push(DictionaryContribution::new(
                dict_code.to_case(Case::Pascal),
                DictionarySpec {
                    code: dict_code.clone(),
                    scope: self.scope.clone(),
                    name: dict_name,
                    description: None,
                    raw_value_kind: RawValueKind::String,
                    open_enum: false,
                    unknown_variant: None,
                    sort_index: 0,
                    items,
                },
            ));
        }

        Ok(contributions)
    }
}

/// Files produced by [`DictBuildGenerator`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedDictFiles {
    pub root_dir: PathBuf,
    pub specs_dir: PathBuf,
    pub enums_file: PathBuf,
    pub spec_files: Vec<PathBuf>,
}

/// Build-time generator that writes dictionary specs and enum source to disk.
#[derive(Default)]
pub struct DictBuildGenerator {
    contributors: Vec<Box<dyn DictionaryContributor>>,
}

impl DictBuildGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_contributor(mut self, contributor: impl DictionaryContributor + 'static) -> Self {
        self.contributors.push(Box::new(contributor));
        self
    }

    /// Collects dictionaries from all contributors and writes generated files.
    ///
    /// The output layout is:
    ///
    /// - `<output_dir>/az_micro_dict/specs/<dict_code>.json`
    /// - `<output_dir>/az_micro_dict/enums.rs`
    pub fn generate_to(self, output_dir: impl AsRef<Path>) -> Result<GeneratedDictFiles> {
        let output_dir = output_dir.as_ref();
        let root_dir = output_dir.join("az_micro_dict");
        let specs_dir = root_dir.join("specs");
        fs::create_dir_all(&specs_dir)
            .with_context(|| format!("failed to create {}", specs_dir.display()))?;

        let mut contributions = Vec::new();
        for contributor in &self.contributors {
            contributions.extend(contributor.contribute()?);
        }
        contributions.sort_by(|left, right| {
            left.spec
                .sort_index
                .cmp(&right.spec.sort_index)
                .then_with(|| left.spec.code.cmp(&right.spec.code))
        });

        let mut seen_codes = BTreeSet::new();
        let mut seen_names = BTreeSet::new();
        let mut generated_specs = Vec::new();
        let mut generated_enums = Vec::new();

        for contribution in contributions {
            contribution.validate_shape()?;
            if !seen_codes.insert(contribution.spec.code.clone()) {
                bail!(
                    "invalid dictionary contribution: duplicate dictionary code {}",
                    contribution.spec.code
                );
            }
            if !seen_names.insert(contribution.enum_name.clone()) {
                bail!(
                    "invalid dictionary contribution: duplicate enum name {}",
                    contribution.enum_name
                );
            }

            let enum_name = contribution.enum_name.clone();
            let spec = contribution.spec;
            spec.validate()?;

            let spec_file_name = format!("{}.json", sanitize_file_stem(&spec.code)?);
            let spec_file = specs_dir.join(spec_file_name);
            let spec_json = spec.to_pretty_json_string()?;
            fs::write(&spec_file, spec_json)
                .with_context(|| format!("failed to write {}", spec_file.display()))?;

            generated_enums.push(render_dict_enum_invocation(&enum_name, &spec, &spec_file)?);
            generated_specs.push(spec_file);
        }

        let enums_file = root_dir.join("enums.rs");
        let enums_source = render_enums_file(generated_enums)?;
        fs::write(&enums_file, enums_source)
            .with_context(|| format!("failed to write {}", enums_file.display()))?;

        Ok(GeneratedDictFiles {
            root_dir,
            specs_dir,
            enums_file,
            spec_files: generated_specs,
        })
    }
}

fn render_dict_enum_invocation(
    enum_name: &str,
    spec: &DictionarySpec,
    spec_file: &Path,
) -> Result<TokenStream2> {
    let enum_ident = format_ident!("{enum_name}");
    let dict_code = &spec.code;
    let spec_path = spec_file.display().to_string();
    let raw_type = match spec.raw_value_kind {
        RawValueKind::Int => quote! { i64 },
        RawValueKind::String => quote! { &'static str },
    };

    Ok(quote! {
        ::az_dict_macros::dict_enum!(
            name = #enum_ident,
            dict = #dict_code,
            spec = include_str!(#spec_path),
            raw_type = #raw_type
        );
    })
}

fn render_enums_file(generated_enums: Vec<TokenStream2>) -> Result<String> {
    let source = quote! {
        #(#generated_enums)*
    };
    normalize_rust_source(source)
}

fn normalize_rust_source(source: TokenStream2) -> Result<String> {
    let syntax_tree = syn::parse2(source).context("failed to parse generated dict enum source")?;
    Ok(prettyplease::unparse(&syntax_tree))
}

fn ruoyi_meta_json(row: &RuoyiDictRow) -> Option<Value> {
    let mut map = serde_json::Map::new();
    if let Some(css_class) = row.css_class.as_deref().filter(|value| !value.is_empty()) {
        map.insert("cssClass".to_string(), Value::String(css_class.to_string()));
    }
    if let Some(list_class) = row.list_class.as_deref().filter(|value| !value.is_empty()) {
        map.insert(
            "listClass".to_string(),
            Value::String(list_class.to_string()),
        );
    }
    if map.is_empty() {
        None
    } else {
        Some(Value::Object(map))
    }
}

fn sanitize_file_stem(value: &str) -> Result<String> {
    if is_blank(Some(value)) {
        bail!("invalid dictionary contribution: dictionary code cannot be empty");
    }
    let stem = sanitize_ascii_file_stem(value);
    if stem == "." || stem == ".." {
        bail!("invalid dictionary contribution: dictionary code {value} is not a valid file stem");
    }
    Ok(stem)
}
