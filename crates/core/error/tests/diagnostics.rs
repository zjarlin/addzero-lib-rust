use anyhow::{Context, Result, anyhow};
use az_error::diagnostics::Diagnostics;

#[test]
fn capture_returns_success_without_diagnostic() {
    let mut diagnostics = Diagnostics::default();
    let value = diagnostics.capture(Ok::<_, anyhow::Error>(7));

    // 成功路径必须原样保留值，且不能产生伪诊断。
    assert_eq!(value, Some(7));
    assert!(diagnostics.is_empty());
}

#[test]
fn capture_records_contextual_error() {
    let mut diagnostics = Diagnostics::default();
    let result = Err::<u32, _>(anyhow!("原始错误").context("解析端口"));
    let value = diagnostics.capture(result);
    let messages = diagnostics
        .iter()
        .map(|error| format!("{error:#}"))
        .collect::<Vec<_>>();

    // 降级前必须保留完整错误上下文，不能只留下底层消息。
    assert_eq!(value, None);
    assert_eq!(messages, vec!["解析端口: 原始错误"]);
}

#[test]
fn recover_uses_explicit_fallback_only_on_failure() {
    let mut diagnostics = Diagnostics::default();
    let success = diagnostics.recover(Ok::<_, anyhow::Error>(3), || {
        panic!("成功路径不应执行降级闭包")
    });
    let recovered = diagnostics.recover(
        Err::<u32, _>(anyhow!("缺少可选值").context("读取可选配置")),
        || 5,
    );

    // 降级值由调用方显式决定，并且只记录真实失败。
    assert_eq!(success, 3);
    assert_eq!(recovered, 5);
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn into_result_preserves_all_diagnostics_in_order() {
    let mut diagnostics = Diagnostics::default();
    diagnostics.record(anyhow!("根因一").context("读取配置"));
    diagnostics.record(anyhow!("根因二").context("加载模型"));

    let Err(error) = diagnostics.into_result() else {
        panic!("存在诊断时必须返回聚合错误");
    };
    let Some(collected) = error.downcast_ref::<Diagnostics>() else {
        panic!("聚合错误必须允许向下转型回 Diagnostics");
    };

    // 阶段边界必须保留全部诊断及其稳定发生顺序。
    assert_eq!(collected.len(), 2);
    assert_eq!(
        format!("{error:#}"),
        "收集到 2 个可恢复诊断\n[1] 读取配置: 根因一\n[2] 加载模型: 根因二"
    );
}

#[test]
fn finish_returns_value_when_stage_has_no_diagnostics() {
    let diagnostics = Diagnostics::default();
    let result = diagnostics.finish("完整结果");

    // 无诊断阶段应直接交付原始结果。
    assert_eq!(result.ok(), Some("完整结果"));
}

#[test]
fn fatal_error_still_returns_before_recoverable_work() {
    fn run_stage(required: &str, diagnostics: &mut Diagnostics) -> Result<u32> {
        let required = required.parse::<u32>().context("解析必填值")?;
        let optional = diagnostics.recover(
            Err::<u32, _>(anyhow!("可选值损坏").context("解析可选值")),
            || 0,
        );

        Ok(required + optional)
    }

    let mut diagnostics = Diagnostics::default();
    let result = run_stage("不是数字", &mut diagnostics);

    // 致命错误必须立即返回，后续可恢复步骤不应被执行或记录。
    assert!(result.is_err());
    assert!(diagnostics.is_empty());
}
