//! 当前客户机上的 Rust 文件生成服务。

use std::{
    collections::BTreeSet,
    env, fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use az_str::api::to_snake_case;

use crate::contract::{
    CodegenExecutionTarget, GenerateRustFileRequest, GeneratedRustFile, RustTypeDefinition,
};

/// 在运行 AIO native backend 的当前客户机上写入 Rust 文件。
#[derive(Clone, Debug)]
pub struct ClientRustCodegen {
    base_directory: PathBuf,
}

impl ClientRustCodegen {
    /// 创建以指定目录解析相对路径的客户机生成服务。
    pub fn new(base_directory: impl Into<PathBuf>) -> Self {
        Self {
            base_directory: base_directory.into(),
        }
    }

    /// 返回网页未填写绝对路径时使用的客户机基准目录。
    pub fn base_directory(&self) -> &Path {
        &self.base_directory
    }

    /// 校验定义、格式化源码并写入客户机目录。
    pub fn generate(&self, request: GenerateRustFileRequest) -> Result<GeneratedRustFile> {
        let source = render_rust_source(&request.definition)?;
        let allowed_root = fs::canonicalize(&self.base_directory).with_context(|| {
            format!(
                "解析客户机代码生成授权根目录失败: {}",
                self.base_directory.display()
            )
        })?;
        let target_directory =
            self.resolve_target_directory(&request.target_directory, &allowed_root)?;
        let target_directory = authorize_target_directory(&target_directory, &allowed_root)?;
        fs::create_dir_all(&target_directory)
            .with_context(|| format!("创建客户机目录失败: {}", target_directory.display()))?;
        let target_directory = fs::canonicalize(&target_directory)
            .with_context(|| format!("解析客户机目录失败: {}", target_directory.display()))?;
        ensure_path_within_root(&target_directory, &allowed_root)?;
        let file_name = resolve_file_name(request.file_name.as_deref(), &request.definition)?;
        let file_path = target_directory.join(file_name);
        write_source_file(&file_path, &source, request.overwrite)?;

        Ok(GeneratedRustFile {
            execution_target: CodegenExecutionTarget::CurrentClient,
            file_path: file_path.display().to_string(),
            byte_length: source.len(),
            source,
        })
    }

    fn resolve_target_directory(&self, value: &str, allowed_root: &Path) -> Result<PathBuf> {
        let value = value.trim();
        if value.is_empty() {
            bail!("invalid target directory: 目标目录不能为空");
        }
        let expanded = expand_home_directory(value)?;
        if expanded.is_absolute() {
            return Ok(expanded);
        }
        Ok(allowed_root.join(expanded))
    }
}

fn render_rust_source(definition: &RustTypeDefinition) -> Result<String> {
    let raw_source = match definition {
        RustTypeDefinition::Enum {
            type_name,
            variants,
        } => render_enum(type_name, variants)?,
        RustTypeDefinition::Struct { type_name, fields } => render_struct(type_name, fields)?,
    };
    let syntax = syn::parse_file(&raw_source).context("invalid generated Rust source")?;
    Ok(prettyplease::unparse(&syntax))
}

fn render_enum(type_name: &str, variants: &[crate::contract::RustEnumVariant]) -> Result<String> {
    let type_name = parse_identifier("enum type", type_name)?;
    if variants.is_empty() {
        bail!("invalid enum definition: 枚举至少需要一个变体");
    }
    let mut seen_names = BTreeSet::new();
    let mut entries = Vec::with_capacity(variants.len());
    for variant in variants {
        let name = parse_identifier("enum variant", &variant.name)?;
        if !seen_names.insert(name.clone()) {
            bail!("duplicate enum variant: {name}");
        }
        let entry = match variant.discriminant {
            Some(value) => format!("    {name} = {value},"),
            None => format!("    {name},"),
        };
        entries.push(entry);
    }
    Ok(format!(
        "#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]\npub enum {type_name} {{\n{}\n}}\n",
        entries.join("\n")
    ))
}

fn render_struct(type_name: &str, fields: &[crate::contract::RustStructField]) -> Result<String> {
    let type_name = parse_identifier("struct type", type_name)?;
    if fields.is_empty() {
        bail!("invalid struct definition: 结构体至少需要一个字段");
    }
    let mut seen_names = BTreeSet::new();
    let mut entries = Vec::with_capacity(fields.len());
    for field in fields {
        let name = parse_identifier("struct field", &field.name)?;
        if !seen_names.insert(name.clone()) {
            bail!("duplicate struct field: {name}");
        }
        let rust_type = field.rust_type.trim();
        if rust_type.is_empty() {
            bail!("invalid struct field type: 字段 {name} 的 Rust 类型不能为空");
        }
        syn::parse_str::<syn::Type>(rust_type)
            .with_context(|| format!("invalid Rust type for field {name}: {rust_type}"))?;
        entries.push(format!("    pub {name}: {rust_type},"));
    }
    Ok(format!(
        "#[derive(Clone, Debug, PartialEq)]\npub struct {type_name} {{\n{}\n}}\n",
        entries.join("\n")
    ))
}

fn parse_identifier(role: &str, value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("invalid {role}: 标识符不能为空");
    }
    let identifier = syn::parse_str::<syn::Ident>(value)
        .with_context(|| format!("invalid {role} identifier: {value}"))?;
    Ok(identifier.to_string())
}

fn resolve_file_name(requested: Option<&str>, definition: &RustTypeDefinition) -> Result<String> {
    let file_name = requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{}.rs", to_snake_case(definition.type_name(), "", "")));
    let path = Path::new(&file_name);
    let mut components = path.components();
    let valid_component = matches!(components.next(), Some(Component::Normal(_)));
    if !valid_component || components.next().is_some() {
        bail!("invalid Rust file name: 文件名不能包含目录: {file_name}");
    }
    if path.extension().and_then(|value| value.to_str()) != Some("rs") {
        bail!("invalid Rust file name: 文件扩展名必须是 .rs: {file_name}");
    }
    Ok(file_name)
}

fn expand_home_directory(value: &str) -> Result<PathBuf> {
    if value == "~" {
        return home_directory();
    }
    let Some(relative) = value.strip_prefix("~/") else {
        return Ok(PathBuf::from(value));
    };
    Ok(home_directory()?.join(relative))
}

fn ensure_path_within_root(path: &Path, allowed_root: &Path) -> Result<()> {
    if path.starts_with(allowed_root) {
        return Ok(());
    }
    bail!(
        "forbidden: 客户机目标目录必须位于授权根目录 {} 内，实际为 {}",
        allowed_root.display(),
        path.display()
    )
}

fn authorize_target_directory(path: &Path, allowed_root: &Path) -> Result<PathBuf> {
    if path
        .components()
        .any(|component| component == Component::ParentDir)
    {
        bail!("forbidden: 客户机目标目录不能包含 .. 路径段");
    }
    let existing_ancestor = path
        .ancestors()
        .find(|ancestor| ancestor.exists())
        .ok_or_else(|| anyhow::anyhow!("客户机目标目录没有可解析的父目录: {}", path.display()))?;
    let missing_suffix = path
        .strip_prefix(existing_ancestor)
        .with_context(|| format!("解析客户机目标目录相对路径失败: {}", path.display()))?;
    let canonical_ancestor = fs::canonicalize(existing_ancestor).with_context(|| {
        format!(
            "解析客户机目标目录父级失败: {}",
            existing_ancestor.display()
        )
    })?;
    let canonical_target = canonical_ancestor.join(missing_suffix);
    ensure_path_within_root(&canonical_target, allowed_root)?;
    Ok(canonical_target)
}

fn home_directory() -> Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("invalid target directory: 当前客户机没有 HOME 环境变量"))
}

fn write_source_file(path: &Path, source: &str, overwrite: bool) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true);
    if overwrite {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    let mut file = options.open(path).with_context(|| {
        if path.exists() && !overwrite {
            format!("conflict: 客户机文件已存在，未启用覆盖: {}", path.display())
        } else {
            format!("打开客户机文件失败: {}", path.display())
        }
    })?;
    file.write_all(source.as_bytes())
        .with_context(|| format!("写入客户机文件失败: {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("同步客户机文件失败: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::contract::{RustEnumVariant, RustStructField};

    #[test]
    fn generates_struct_on_current_client() -> Result<()> {
        let temp = TempDir::new()?;
        let generator = ClientRustCodegen::new(temp.path());
        let generated = generator.generate(GenerateRustFileRequest {
            target_directory: "generated".to_string(),
            file_name: None,
            overwrite: false,
            definition: RustTypeDefinition::Struct {
                type_name: "DeviceState".to_string(),
                fields: vec![
                    RustStructField {
                        name: "device_id".to_string(),
                        rust_type: "String".to_string(),
                    },
                    RustStructField {
                        name: "online".to_string(),
                        rust_type: "bool".to_string(),
                    },
                ],
            },
        })?;

        // 关键断言：相对目录必须落在运行 AIO 的当前客户机基准目录中。
        assert!(generated.file_path.ends_with("generated/device_state.rs"));
        assert!(generated.source.contains("pub struct DeviceState"));
        assert!(generated.source.contains("pub device_id: String"));
        Ok(())
    }

    #[test]
    fn refuses_to_overwrite_existing_file_by_default() -> Result<()> {
        let temp = TempDir::new()?;
        let generator = ClientRustCodegen::new(temp.path());
        let request = GenerateRustFileRequest {
            target_directory: temp.path().display().to_string(),
            file_name: Some("status.rs".to_string()),
            overwrite: false,
            definition: RustTypeDefinition::Enum {
                type_name: "Status".to_string(),
                variants: vec![RustEnumVariant {
                    name: "Ready".to_string(),
                    discriminant: Some(1),
                }],
            },
        };
        generator.generate(request.clone())?;
        let error = generator
            .generate(request)
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default();

        // 关键断言：网页未明确允许覆盖时不能改写客户机已有源码。
        assert!(error.starts_with("conflict:"));
        Ok(())
    }

    #[test]
    fn rejects_directory_components_in_file_name() {
        let definition = RustTypeDefinition::Enum {
            type_name: "Status".to_string(),
            variants: vec![RustEnumVariant {
                name: "Ready".to_string(),
                discriminant: None,
            }],
        };
        let error = resolve_file_name(Some("../status.rs"), &definition)
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default();

        // 关键断言：文件名不能绕过网页选择的目标目录。
        assert!(error.contains("不能包含目录"));
    }

    #[test]
    fn rejects_target_directory_outside_authorized_root() -> Result<()> {
        let allowed_root = TempDir::new()?;
        let outside = TempDir::new()?;
        let generator = ClientRustCodegen::new(allowed_root.path());
        let error = generator
            .generate(GenerateRustFileRequest {
                target_directory: outside.path().display().to_string(),
                file_name: None,
                overwrite: false,
                definition: RustTypeDefinition::Enum {
                    type_name: "Status".to_string(),
                    variants: vec![RustEnumVariant {
                        name: "Ready".to_string(),
                        discriminant: None,
                    }],
                },
            })
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default();

        // 关键断言：网页不能越过客户机配置的代码生成授权根目录。
        assert!(error.starts_with("forbidden:"));
        Ok(())
    }
}
