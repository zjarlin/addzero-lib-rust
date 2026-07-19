//! engine PostgreSQL 集成测试。

use az_engine::{
    ComputedDependency, EngineStore, FieldInput, ModelInput, PageParams, verify_database_url,
};
use serde_json::json;

#[tokio::test]
async fn postgres_record_pipeline_and_computed_query() -> anyhow::Result<()> {
    let Some(database_url) = test_database_url()? else {
        return Ok(());
    };
    let store = EngineStore::connect(&database_url).await?;
    let suffix = unique_suffix();
    let user_model = format!("it_user_{suffix}");
    let order_model = format!("it_order_{suffix}");

    store
        .create_model(ModelInput {
            name: user_model.clone(),
            display_name: "集成用户".to_string(),
        })
        .await?;
    store
        .create_model(ModelInput {
            name: order_model.clone(),
            display_name: "集成订单".to_string(),
        })
        .await?;

    store
        .create_field(
            &user_model,
            FieldInput {
                name: "vip_level".to_string(),
                display_name: "VIP".to_string(),
                field_type: "int".to_string(),
                is_required: true,
                expression: None,
                dependency_json: None,
                order_index: 1,
            },
        )
        .await?;
    store
        .create_field(
            &order_model,
            FieldInput {
                name: "user_id".to_string(),
                display_name: "用户".to_string(),
                field_type: "string".to_string(),
                is_required: true,
                expression: None,
                dependency_json: None,
                order_index: 1,
            },
        )
        .await?;
    store
        .create_field(
            &order_model,
            FieldInput {
                name: "amount".to_string(),
                display_name: "金额".to_string(),
                field_type: "int".to_string(),
                is_required: true,
                expression: None,
                dependency_json: None,
                order_index: 2,
            },
        )
        .await?;
    store
        .create_field(
            &order_model,
            FieldInput {
                name: "vip_amount".to_string(),
                display_name: "VIP 金额".to_string(),
                field_type: "computed".to_string(),
                is_required: false,
                expression: Some("amount + user_vip_level".to_string()),
                dependency_json: Some(serde_json::to_string(&[ComputedDependency {
                    alias: "user_vip_level".to_string(),
                    source_model_name: user_model.clone(),
                    local_field: "user_id".to_string(),
                    source_payload_field: "vip_level".to_string(),
                }])?),
                order_index: 3,
            },
        )
        .await?;

    let user = store
        .executor()
        .insert_record(&user_model, json!({ "vip_level": 7 }))
        .await?;
    store
        .executor()
        .insert_record(&order_model, json!({ "user_id": user.id, "amount": 30 }))
        .await?;

    let page = store
        .executor()
        .list_records(&order_model, PageParams { o: 0, s: 10 })
        .await?;

    // 查询路径应批量加载外部模型依赖并只在返回 payload 中注入计算结果。
    assert_eq!(page.t, 1);
    assert_eq!(page.p, PageParams { o: 0, s: 10 });
    assert_eq!(page.d[0].payload["vip_amount"], json!(37));

    store.delete_model(&order_model).await?;
    store.delete_model(&user_model).await?;
    Ok(())
}

fn test_database_url() -> anyhow::Result<Option<String>> {
    match std::env::var("DATABASE_URL") {
        Ok(value) => Ok(Some(verify_database_url(&value)?.to_string())),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn unique_suffix() -> String {
    let millis = az_engine::timestamp_ms();
    let pid = std::process::id();
    format!("{pid}_{millis}")
}
