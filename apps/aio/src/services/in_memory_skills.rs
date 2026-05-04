use std::cell::RefCell;

use chrono::Utc;

use super::skills::{
    LocalBoxFuture, SkillDto, SkillServiceError, SkillServiceResult, SkillSourceDto,
    SkillUpsertDto, SkillsApi, SyncReportDto,
};

#[derive(Default)]
pub struct InMemorySkillsApi;

thread_local! {
    static SKILLS: RefCell<Vec<SkillDto>> = RefCell::new(seed_skills());
    static LAST_REPORT: RefCell<SyncReportDto> = RefCell::new(SyncReportDto {
        finished_at: Some(Utc::now()),
        fs_root: "~/.agents/skills".to_string(),
        ..SyncReportDto::default()
    });
}

impl SkillsApi for InMemorySkillsApi {
    fn list_skills(&self) -> LocalBoxFuture<'_, SkillServiceResult<Vec<SkillDto>>> {
        Box::pin(async { Ok(SKILLS.with(|skills| skills.borrow().clone())) })
    }

    fn get_skill(&self, name: String) -> LocalBoxFuture<'_, SkillServiceResult<Option<SkillDto>>> {
        Box::pin(async move {
            Ok(SKILLS.with(|skills| {
                skills
                    .borrow()
                    .iter()
                    .find(|skill| skill.name == name)
                    .cloned()
            }))
        })
    }

    fn upsert_skill(
        &self,
        input: SkillUpsertDto,
    ) -> LocalBoxFuture<'_, SkillServiceResult<SkillDto>> {
        Box::pin(async move {
            let name = input.name.trim().to_string();
            if name.is_empty() {
                return Err(SkillServiceError::new("名称不能为空"));
            }

            let normalized = SkillUpsertDto {
                name: name.clone(),
                keywords: input.keywords,
                description: input.description,
                body: input.body,
            };
            let skill = SkillDto {
                name: name.clone(),
                keywords: normalized.keywords.clone(),
                description: normalized.description.clone(),
                body: normalized.body.clone(),
                content_hash: make_hash(&normalized),
                updated_at: Utc::now(),
                source: SkillSourceDto::FileSystem,
            };

            SKILLS.with(|skills| {
                let mut skills = skills.borrow_mut();
                if let Some(existing) = skills.iter_mut().find(|current| current.name == name) {
                    *existing = skill.clone();
                } else {
                    skills.push(skill.clone());
                    skills.sort_by(|left, right| left.name.cmp(&right.name));
                }
            });

            Ok(skill)
        })
    }

    fn delete_skill(&self, name: String) -> LocalBoxFuture<'_, SkillServiceResult<()>> {
        Box::pin(async move {
            let deleted = SKILLS.with(|skills| {
                let mut skills = skills.borrow_mut();
                let before = skills.len();
                skills.retain(|skill| skill.name != name);
                before != skills.len()
            });

            if deleted {
                Ok(())
            } else {
                Err(SkillServiceError::new("未找到该技能"))
            }
        })
    }

    fn sync_skills(&self) -> LocalBoxFuture<'_, SkillServiceResult<SyncReportDto>> {
        Box::pin(async {
            let names = SKILLS.with(|skills| {
                skills
                    .borrow()
                    .iter()
                    .take(2)
                    .map(|skill| skill.name.clone())
                    .collect::<Vec<_>>()
            });
            let report = SyncReportDto {
                updated_in_fs: names,
                finished_at: Some(Utc::now()),
                pg_online: false,
                fs_root: "~/.agents/skills".to_string(),
                ..SyncReportDto::default()
            };
            LAST_REPORT.with(|last| last.replace(report.clone()));
            Ok(report)
        })
    }

    fn server_status(&self) -> LocalBoxFuture<'_, SkillServiceResult<SyncReportDto>> {
        Box::pin(async { Ok(LAST_REPORT.with(|last| last.borrow().clone())) })
    }
}

fn seed_skills() -> Vec<SkillDto> {
    vec![
        SkillDto {
            name: "frontend-design".to_string(),
            keywords: vec!["frontend".into(), "ui".into(), "design".into()],
            description: "构建有设计感的前端页面与组件。".to_string(),
            body: "# frontend-design\n\n聚焦布局、视觉层次、响应式和交互节奏。".to_string(),
            content_hash: "demo_frontend_design".to_string(),
            updated_at: Utc::now(),
            source: SkillSourceDto::FileSystem,
        },
        SkillDto {
            name: "debug-pro".to_string(),
            keywords: vec!["debug".into(), "排查".into(), "错误".into()],
            description: "系统化定位编译、运行时和环境问题。".to_string(),
            body: "# debug-pro\n\n先缩小范围，再验证假设，最后固化修复。".to_string(),
            content_hash: "demo_debug_pro".to_string(),
            updated_at: Utc::now(),
            source: SkillSourceDto::Both,
        },
        SkillDto {
            name: "rust-testing".to_string(),
            keywords: vec!["rust".into(), "test".into(), "测试".into()],
            description: "为 Rust crate 设计并执行测试策略。".to_string(),
            body: "# rust-testing\n\n覆盖单元测试、集成测试和边界条件。".to_string(),
            content_hash: "demo_rust_testing".to_string(),
            updated_at: Utc::now(),
            source: SkillSourceDto::Postgres,
        },
    ]
}

fn make_hash(input: &SkillUpsertDto) -> String {
    let basis = format!(
        "{}|{}|{}|{}",
        input.name,
        input.keywords.join(","),
        input.description,
        input.body
    );
    let mut acc: u64 = 1469598103934665603;
    for byte in basis.bytes() {
        acc ^= u64::from(byte);
        acc = acc.wrapping_mul(1099511628211);
    }
    format!("{acc:016x}")
}
