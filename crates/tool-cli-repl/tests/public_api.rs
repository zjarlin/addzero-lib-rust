use addzero_cli_repl::*;

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
