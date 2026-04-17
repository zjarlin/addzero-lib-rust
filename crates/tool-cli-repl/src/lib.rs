use std::fmt;
use thiserror::Error;

pub const EXIT_COMMAND: &str = "q";
pub const HELP_COMMAND: &str = "h";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    String,
    Int,
    Float,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl ParamValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }
}

impl fmt::Display for ParamValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(value) => formatter.write_str(value),
            Self::Int(value) => write!(formatter, "{value}"),
            Self::Float(value) => write!(formatter, "{value}"),
            Self::Bool(value) => write!(formatter, "{value}"),
        }
    }
}

impl From<&str> for ParamValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for ParamValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<i64> for ParamValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<i32> for ParamValue {
    fn from(value: i32) -> Self {
        Self::Int(i64::from(value))
    }
}

impl From<f64> for ParamValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for ParamValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamDef {
    pub name: String,
    pub param_type: ParamType,
    pub description: String,
    pub default_value: Option<ParamValue>,
    pub is_required: bool,
}

impl ParamDef {
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

    pub fn with_default(mut self, default_value: impl Into<ParamValue>) -> Self {
        self.default_value = Some(default_value.into());
        self.is_required = false;
        self
    }

    pub fn optional(mut self) -> Self {
        self.is_required = false;
        self
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ReplError {
    #[error("missing required parameter: {0}")]
    MissingRequiredParameter(String),
    #[error("invalid value `{value}` for parameter `{name}`, expected {expected}")]
    InvalidValue {
        name: String,
        value: String,
        expected: &'static str,
    },
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("invalid command index: {0}")]
    InvalidCommandIndex(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedParams(Vec<ParamValue>);

impl ParsedParams {
    pub fn get(&self, index: usize) -> Option<&ParamValue> {
        self.0.get(index)
    }

    pub fn get_string(&self, index: usize) -> Option<&str> {
        self.get(index).and_then(ParamValue::as_str)
    }

    pub fn get_i64(&self, index: usize) -> Option<i64> {
        self.get(index).and_then(ParamValue::as_i64)
    }

    pub fn get_f64(&self, index: usize) -> Option<f64> {
        self.get(index).and_then(ParamValue::as_f64)
    }

    pub fn get_bool(&self, index: usize) -> Option<bool> {
        self.get(index).and_then(ParamValue::as_bool)
    }

    pub fn into_inner(self) -> Vec<ParamValue> {
        self.0
    }
}

pub trait Command {
    fn command(&self) -> &str;
    fn description(&self) -> &str;
    fn param_defs(&self) -> &[ParamDef];
    fn eval(&self, params: ParsedParams) -> Result<String, ReplError>;

    fn handle_error(&self, error: &ReplError) -> String {
        format!("错误: {error}")
    }

    fn support(&self) -> bool {
        true
    }

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
                    "{}{}: {}{} (类型: {:?})",
                    def.name, required_mark, def.description, default_hint, def.param_type
                )
            })
            .collect::<Vec<_>>()
            .join("\n  ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplOutcome {
    Exit,
    Message(String),
    Empty,
}

pub struct ReplEngine {
    commands: Vec<Box<dyn Command>>,
    pub prompt: String,
    pub exit_command: String,
    pub help_command: String,
}

impl ReplEngine {
    pub fn new(commands: Vec<Box<dyn Command>>) -> Self {
        Self {
            commands,
            prompt: "> ".to_owned(),
            exit_command: EXIT_COMMAND.to_owned(),
            help_command: HELP_COMMAND.to_owned(),
        }
    }

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
                    return ReplOutcome::Message(format!(
                        "错误: {}",
                        ReplError::InvalidCommandIndex(cmd.to_owned())
                    ));
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
                    return ReplOutcome::Message(format!(
                        "错误: {}",
                        ReplError::UnknownCommand(cmd.to_owned())
                    ));
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

pub fn split_command_and_args(input: &str) -> (&str, Vec<String>) {
    let mut parts = input.split_whitespace();
    let command = parts.next().unwrap_or_default();
    let args = parts.map(ToOwned::to_owned).collect();
    (command, args)
}

pub fn parse_params(param_defs: &[ParamDef], input: &[String]) -> Result<ParsedParams, ReplError> {
    let mut values = Vec::with_capacity(param_defs.len());

    for (index, def) in param_defs.iter().enumerate() {
        let input_value = input.get(index).map(String::as_str);
        let value = match input_value {
            Some(value) if !value.is_empty() => parse_value(def, value)?,
            _ => match &def.default_value {
                Some(default_value) => default_value.clone(),
                None if def.is_required => {
                    return Err(ReplError::MissingRequiredParameter(def.name.clone()));
                }
                None => ParamValue::String(String::new()),
            },
        };
        values.push(value);
    }

    Ok(ParsedParams(values))
}

fn parse_value(def: &ParamDef, value: &str) -> Result<ParamValue, ReplError> {
    match def.param_type {
        ParamType::String => Ok(ParamValue::String(value.to_owned())),
        ParamType::Int => {
            value
                .parse::<i64>()
                .map(ParamValue::Int)
                .map_err(|_| ReplError::InvalidValue {
                    name: def.name.clone(),
                    value: value.to_owned(),
                    expected: "Int",
                })
        }
        ParamType::Float => {
            value
                .parse::<f64>()
                .map(ParamValue::Float)
                .map_err(|_| ReplError::InvalidValue {
                    name: def.name.clone(),
                    value: value.to_owned(),
                    expected: "Float",
                })
        }
        ParamType::Bool => match value.to_ascii_lowercase().as_str() {
            "y" | "yes" | "true" => Ok(ParamValue::Bool(true)),
            "n" | "no" | "false" => Ok(ParamValue::Bool(false)),
            _ => Err(ReplError::InvalidValue {
                name: def.name.clone(),
                value: value.to_owned(),
                expected: "Boolean",
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SumCommand {
        params: Vec<ParamDef>,
    }

    impl SumCommand {
        fn new() -> Self {
            Self {
                params: vec![
                    ParamDef::new("left", ParamType::Int, "left operand"),
                    ParamDef::new("right", ParamType::Int, "right operand").with_default(10),
                ],
            }
        }
    }

    impl Command for SumCommand {
        fn command(&self) -> &str {
            "sum"
        }

        fn description(&self) -> &str {
            "sum numbers"
        }

        fn param_defs(&self) -> &[ParamDef] {
            &self.params
        }

        fn eval(&self, params: ParsedParams) -> Result<String, ReplError> {
            Ok(
                (params.get_i64(0).unwrap_or_default() + params.get_i64(1).unwrap_or_default())
                    .to_string(),
            )
        }
    }

    struct EchoCommand {
        params: Vec<ParamDef>,
    }

    impl EchoCommand {
        fn new() -> Self {
            Self {
                params: vec![
                    ParamDef::new("message", ParamType::String, "text"),
                    ParamDef::new("uppercase", ParamType::Bool, "uppercase").with_default(false),
                ],
            }
        }
    }

    impl Command for EchoCommand {
        fn command(&self) -> &str {
            "echo"
        }

        fn description(&self) -> &str {
            "echo text"
        }

        fn param_defs(&self) -> &[ParamDef] {
            &self.params
        }

        fn eval(&self, params: ParsedParams) -> Result<String, ReplError> {
            let message = params.get_string(0).unwrap_or_default();
            let uppercase = params.get_bool(1).unwrap_or(false);
            Ok(if uppercase {
                message.to_uppercase()
            } else {
                message.to_owned()
            })
        }
    }

    #[test]
    fn parse_params_supports_defaults_and_bool_aliases() {
        let params = parse_params(
            &[
                ParamDef::new("message", ParamType::String, "text"),
                ParamDef::new("uppercase", ParamType::Bool, "uppercase").with_default(false),
            ],
            &[String::from("hello"), String::from("y")],
        )
        .expect("params should parse");

        assert_eq!(params.get_string(0), Some("hello"));
        assert_eq!(params.get_bool(1), Some(true));
    }

    #[test]
    fn parse_params_uses_default_value_when_missing() {
        let params = parse_params(
            &[ParamDef::new("right", ParamType::Int, "right operand").with_default(10)],
            &[],
        )
        .expect("params should parse");

        assert_eq!(params.get_i64(0), Some(10));
    }

    #[test]
    fn engine_executes_named_and_indexed_commands() {
        let engine = ReplEngine::new(vec![
            Box::new(SumCommand::new()),
            Box::new(EchoCommand::new()),
        ]);

        assert_eq!(
            engine.run_line("sum 2 3"),
            ReplOutcome::Message("5".to_owned())
        );
        assert_eq!(
            engine.run_line("2 hello true"),
            ReplOutcome::Message("HELLO".to_owned())
        );
    }

    #[test]
    fn engine_help_and_command_list_include_metadata() {
        let engine = ReplEngine::new(vec![Box::new(SumCommand::new())]);
        let help = engine.help();
        let list = engine.command_list();

        assert!(help.contains("sum: sum numbers"));
        assert!(list.contains("1. sum - sum numbers"));
    }

    #[test]
    fn engine_returns_exit_for_exit_command() {
        let engine = ReplEngine::new(vec![Box::new(SumCommand::new())]);

        assert_eq!(engine.run_line("q"), ReplOutcome::Exit);
    }
}
