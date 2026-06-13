//! CLI REPL 引擎框架。
//!
//! 提供一套可扩展的命令行交互式解释器（REPL）基础设施，用于构建
//! 带有参数解析、帮助系统和命令注册能力的 CLI 工具。
//!
//! # 核心类型
//!
//! - [`ReplEngine`] — REPL 主引擎，管理命令注册、输入解析与执行循环。
//! - [`Command`] — 命令 trait，由使用者实现以定义具体的 CLI 命令。
//! - [`ParamDef`] / [`ParamValue`] — 参数定义与值类型，支持 String、Int、Float、Bool 四种基本类型。
//! - [`ParsedParams`] — 解析后的参数集合，按位置索引访问。
//!
//! # 主要功能
//!
//! - 支持按命令名称或数字索引执行命令
//! - 内置帮助系统（`h`）和退出命令（`q`）
//! - 自动参数解析，支持默认值与必填校验
//! - 通过 [`Command`] trait 实现命令扩展

use anyhow::{anyhow, bail};
use az_derive_aliases::{
    apply, from_display, plain_code_display_no_default_enum, plain_eq, plain_partial_eq,
};

/// 默认退出命令。
pub const EXIT_COMMAND: &str = "q";
/// 默认帮助命令。
pub const HELP_COMMAND: &str = "h";

/// REPL 支持的参数类型。
///
/// `code()` 产出稳定的小写机器值，`Display` 产出帮助文本中的展示名。
#[apply(plain_code_display_no_default_enum)]
pub enum ParamType {
    /// UTF-8 字符串参数。
    #[display("String")]
    String,
    /// 64 位有符号整数参数。
    #[display("Int")]
    Int,
    /// 64 位浮点数参数。
    #[display("Float")]
    Float,
    /// 布尔参数，解析时接受 `y/yes/true` 和 `n/no/false`。
    #[display("Boolean")]
    Bool,
}

/// 已解析出的 REPL 参数值。
#[apply(from_display)]
pub enum ParamValue {
    /// 字符串值。
    #[from(String, &str)]
    #[display("{_0}")]
    String(String),
    /// 整数值。
    #[from(i64, i32)]
    #[display("{_0}")]
    Int(i64),
    /// 浮点数值。
    #[from(f64)]
    #[display("{_0}")]
    Float(f64),
    /// 布尔值。
    #[from(bool)]
    #[display("{_0}")]
    Bool(bool),
}

impl ParamValue {
    /// 当值是字符串时返回字符串引用。
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    /// 当值是整数时返回整数。
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }

    /// 当值是浮点数时返回浮点数。
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }

    /// 当值是布尔值时返回布尔值。
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }
}

/// 单个位置参数的定义。
#[apply(plain_partial_eq)]
pub struct ParamDef {
    /// 参数名称，用于帮助文本和错误提示。
    pub name: String,
    /// 参数解析类型。
    pub param_type: ParamType,
    /// 帮助文本中的参数说明。
    pub description: String,
    /// 缺省值；存在时参数自动变为非必填。
    pub default_value: Option<ParamValue>,
    /// 是否必须由用户输入。
    pub is_required: bool,
}

impl ParamDef {
    /// 创建必填参数定义。
    pub fn new(
        name: impl Into<String>,
        param_type: ParamType,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            param_type,
            description: description.into(),
            default_value: None,
            is_required: true,
        }
    }

    /// 设置默认值，并把参数标记为非必填。
    pub fn with_default(mut self, default_value: impl Into<ParamValue>) -> Self {
        self.default_value = Some(default_value.into());
        self.is_required = false;
        self
    }

    /// 把参数标记为非必填；缺省时会解析为空字符串值。
    pub fn optional(mut self) -> Self {
        self.is_required = false;
        self
    }
}

/// 按命令参数定义顺序保存的解析结果。
#[apply(plain_partial_eq)]
pub struct ParsedParams(Vec<ParamValue>);

impl ParsedParams {
    /// 按位置读取原始参数值。
    pub fn get(&self, index: usize) -> Option<&ParamValue> {
        self.0.get(index)
    }

    /// 按位置读取字符串参数。
    pub fn get_string(&self, index: usize) -> Option<&str> {
        self.get(index).and_then(ParamValue::as_str)
    }

    /// 按位置读取整数参数。
    pub fn get_i64(&self, index: usize) -> Option<i64> {
        self.get(index).and_then(ParamValue::as_i64)
    }

    /// 按位置读取浮点参数。
    pub fn get_f64(&self, index: usize) -> Option<f64> {
        self.get(index).and_then(ParamValue::as_f64)
    }

    /// 按位置读取布尔参数。
    pub fn get_bool(&self, index: usize) -> Option<bool> {
        self.get(index).and_then(ParamValue::as_bool)
    }

    /// 取出底层参数值列表。
    pub fn into_inner(self) -> Vec<ParamValue> {
        self.0
    }
}

/// 可注册到 [`ReplEngine`] 的命令接口。
pub trait Command {
    /// 命令短名称，用户可直接输入该名称执行命令。
    fn command(&self) -> &str;
    /// 命令帮助文本中的简短说明。
    fn description(&self) -> &str;
    /// 命令的位置参数定义。
    fn param_defs(&self) -> &[ParamDef];
    /// 执行业务逻辑并返回要展示给用户的文本。
    fn eval(&self, params: ParsedParams) -> anyhow::Result<String>;

    /// 将命令执行或参数解析错误转换为展示文本。
    fn handle_error(&self, error: &anyhow::Error) -> String {
        format!("错误: {error}")
    }

    /// 返回命令当前是否参与帮助列表和执行匹配。
    fn support(&self) -> bool {
        true
    }

    /// 渲染当前命令的参数帮助文本。
    fn param_help(&self) -> String {
        self.param_defs()
            .iter()
            .map(|def| {
                let required_mark = if def.is_required { "*" } else { "" };
                let default_hint = def
                    .default_value
                    .as_ref()
                    .map(|value| format!(" (默认: {value})"))
                    .unwrap_or_default();
                format!(
                    "{}{}: {}{} (类型: {})",
                    def.name, required_mark, def.description, default_hint, def.param_type
                )
            })
            .collect::<Vec<_>>()
            .join("\n  ")
    }
}

/// REPL 处理单行输入后的结果。
#[apply(plain_eq)]
pub enum ReplOutcome {
    /// 用户请求退出。
    Exit,
    /// 需要展示给用户的文本。
    Message(String),
    /// 输入为空行。
    Empty,
}

/// REPL 主引擎，负责命令注册、输入分发和帮助文本渲染。
pub struct ReplEngine {
    commands: Vec<Box<dyn Command>>,
    /// 命令行提示符。
    pub prompt: String,
    /// 退出命令文本。
    pub exit_command: String,
    /// 帮助命令文本。
    pub help_command: String,
}

impl ReplEngine {
    /// 使用命令列表创建 REPL 引擎。
    pub fn new(commands: Vec<Box<dyn Command>>) -> Self {
        Self {
            commands,
            prompt: "> ".to_owned(),
            exit_command: EXIT_COMMAND.to_owned(),
            help_command: HELP_COMMAND.to_owned(),
        }
    }

    /// 渲染可执行命令列表，包含数字序号和命令短名称。
    pub fn command_list(&self) -> String {
        let mut lines = vec!["可用命令(键入数字和短名称都可以执行命令):".to_owned()];
        for (index, command) in self.supported_commands().iter().enumerate() {
            lines.push(format!(
                "  {}. {} - {}",
                index + 1,
                command.command(),
                command.description()
            ));
        }
        lines.join("\n")
    }

    /// 渲染所有可用命令的完整帮助文本。
    pub fn help(&self) -> String {
        self.supported_commands()
            .iter()
            .map(|command| {
                format!(
                    "{}: {}\n  {}",
                    command.command(),
                    command.description(),
                    command.param_help()
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// 执行一行用户输入。
    ///
    /// 支持按命令短名称或从 `1` 开始的命令序号定位命令。
    pub fn run_line(&self, input: &str) -> ReplOutcome {
        let input = input.trim();
        if input.is_empty() {
            return ReplOutcome::Empty;
        }

        let (cmd, args) = split_command_and_args(input);

        if cmd.eq_ignore_ascii_case(&self.exit_command) {
            return ReplOutcome::Exit;
        }
        if cmd.eq_ignore_ascii_case(&self.help_command) {
            return ReplOutcome::Message(self.help());
        }

        let command = if let Ok(index) = cmd.parse::<usize>() {
            match self.supported_commands().get(index.saturating_sub(1)) {
                Some(command) => *command,
                None => {
                    return ReplOutcome::Message(format!("错误: invalid command index: {cmd}"));
                }
            }
        } else {
            match self
                .supported_commands()
                .into_iter()
                .find(|candidate| candidate.command().eq_ignore_ascii_case(cmd))
            {
                Some(command) => command,
                None => {
                    return ReplOutcome::Message(format!("错误: unknown command: {cmd}"));
                }
            }
        };

        match parse_params(command.param_defs(), &args) {
            Ok(params) => match command.eval(params) {
                Ok(output) => ReplOutcome::Message(output),
                Err(error) => ReplOutcome::Message(command.handle_error(&error)),
            },
            Err(error) => ReplOutcome::Message(command.handle_error(&error)),
        }
    }

    fn supported_commands(&self) -> Vec<&dyn Command> {
        self.commands
            .iter()
            .filter(|command| command.support())
            .map(|command| command.as_ref())
            .collect()
    }
}

/// 按空白字符把一行输入拆成命令和参数列表。
pub fn split_command_and_args(input: &str) -> (&str, Vec<String>) {
    let mut parts = input.split_whitespace();
    let command = parts.next().unwrap_or_default();
    let args = parts.map(ToOwned::to_owned).collect();
    (command, args)
}

/// 按参数定义解析位置参数。
///
/// 多余输入会被忽略；可选且没有默认值的缺失参数会得到空字符串值。
pub fn parse_params(param_defs: &[ParamDef], input: &[String]) -> anyhow::Result<ParsedParams> {
    let mut values = Vec::with_capacity(param_defs.len());

    for (index, def) in param_defs.iter().enumerate() {
        let input_value = input.get(index).map(String::as_str);
        let value = match input_value {
            Some(value) if !value.is_empty() => parse_value(def, value)?,
            _ => match &def.default_value {
                Some(default_value) => default_value.clone(),
                None if def.is_required => {
                    bail!("missing required parameter: {}", def.name);
                }
                None => ParamValue::String(String::new()),
            },
        };
        values.push(value);
    }

    Ok(ParsedParams(values))
}

fn parse_value(def: &ParamDef, value: &str) -> anyhow::Result<ParamValue> {
    match def.param_type {
        ParamType::String => Ok(ParamValue::String(value.to_owned())),
        ParamType::Int => value
            .parse::<i64>()
            .map(ParamValue::Int)
            .map_err(|_| invalid_value_error(def, value)),
        ParamType::Float => value
            .parse::<f64>()
            .map(ParamValue::Float)
            .map_err(|_| invalid_value_error(def, value)),
        ParamType::Bool => match value.to_ascii_lowercase().as_str() {
            "y" | "yes" | "true" => Ok(ParamValue::Bool(true)),
            "n" | "no" | "false" => Ok(ParamValue::Bool(false)),
            _ => Err(invalid_value_error(def, value)),
        },
    }
}

fn invalid_value_error(def: &ParamDef, value: &str) -> anyhow::Error {
    anyhow!(
        "invalid value `{}` for parameter `{}`, expected {}",
        value,
        def.name,
        def.param_type.code()
    )
}
