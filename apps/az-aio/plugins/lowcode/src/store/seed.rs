use crate::model::{AppScreen, MetaField, MetaModel};

use super::LowcodeStore;

impl LowcodeStore {
    pub fn seed_demo(&self) {
        let now = chrono::Utc::now().to_rfc3339();
        let mut models = self.mem.models.lock();
        if !models.is_empty() {
            return;
        }
        let mut fields = self.mem.fields.lock();

        let proj_id = "demo-proj-001".to_string();
        models.push(MetaModel {
            id: proj_id.clone(),
            name: "Project".into(),
            label: "项目".into(),
            description: "项目管理模型".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-001".into(),
            model_id: proj_id.clone(),
            name: "name".into(),
            label: "名称".into(),
            field_type: "String".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: true,
            is_unique: false,
            order: 1,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-002".into(),
            model_id: proj_id.clone(),
            name: "status".into(),
            label: "状态".into(),
            field_type: "String".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: false,
            is_unique: false,
            order: 2,
            default_value: Some("draft".into()),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-003".into(),
            model_id: proj_id.clone(),
            name: "start_date".into(),
            label: "开始日期".into(),
            field_type: "DateTime".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: false,
            is_unique: false,
            order: 3,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-004".into(),
            model_id: proj_id.clone(),
            name: "budget".into(),
            label: "预算".into(),
            field_type: "Float".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: false,
            is_unique: false,
            order: 4,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });

        let emp_id = "demo-emp-001".to_string();
        models.push(MetaModel {
            id: emp_id.clone(),
            name: "Employee".into(),
            label: "员工".into(),
            description: "员工信息模型".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-010".into(),
            model_id: emp_id.clone(),
            name: "name".into(),
            label: "姓名".into(),
            field_type: "String".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: true,
            is_unique: false,
            order: 1,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-011".into(),
            model_id: emp_id.clone(),
            name: "email".into(),
            label: "邮箱".into(),
            field_type: "String".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: false,
            is_unique: true,
            order: 2,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-012".into(),
            model_id: emp_id.clone(),
            name: "project_id".into(),
            label: "所属项目".into(),
            field_type: "Relation".into(),
            relation_type: Some("OneToMany".into()),
            relation_model_id: Some(proj_id.clone()),
            is_required: false,
            is_unique: false,
            order: 3,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-013".into(),
            model_id: emp_id.clone(),
            name: "manager_id".into(),
            label: "上级主管".into(),
            field_type: "Relation".into(),
            relation_type: Some("SelfRecursive".into()),
            relation_model_id: Some(emp_id.clone()),
            is_required: false,
            is_unique: false,
            order: 4,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });

        let org_id = "demo-org-001".to_string();
        models.push(MetaModel {
            id: org_id.clone(),
            name: "Organization".into(),
            label: "组织架构".into(),
            description: "树形组织架构".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-020".into(),
            model_id: org_id.clone(),
            name: "name".into(),
            label: "名称".into(),
            field_type: "String".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: true,
            is_unique: false,
            order: 1,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-021".into(),
            model_id: org_id.clone(),
            name: "parent_id".into(),
            label: "上级部门".into(),
            field_type: "Relation".into(),
            relation_type: Some("SelfRecursive".into()),
            relation_model_id: Some(org_id.clone()),
            is_required: false,
            is_unique: false,
            order: 2,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-022".into(),
            model_id: org_id.clone(),
            name: "level".into(),
            label: "层级".into(),
            field_type: "Integer".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: false,
            is_unique: false,
            order: 3,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });

        let mut screens = self.mem.screens.lock();
        screens.push(AppScreen {
            id: "demo-screen-table".into(),
            name: "project-table".into(),
            label: "项目列表".into(),
            layout: "Table".into(),
            model_id: proj_id.clone(),
            config_json: r#"{"columns":[{"field_name":"name","label":"名称","sortable":true},{"field_name":"status","label":"状态"},{"field_name":"start_date","label":"开始日期","sortable":true},{"field_name":"budget","label":"预算"}],"searchable_fields":["name"],"page_size":20}"#.into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        screens.push(AppScreen {
            id: "demo-screen-md".into(),
            name: "project-org".into(),
            label: "项目-组织左树右表".into(),
            layout: "MasterDetail".into(),
            model_id: org_id.clone(),
            config_json: r#"{"tree_field_id":"name","detail_columns":[{"field_name":"name","label":"名称","sortable":true},{"field_name":"level","label":"层级"},{"field_name":"parent_id","label":"上级部门"}],"detail_searchable":["name"]}"#.into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        screens.push(AppScreen {
            id: "demo-screen-acc".into(),
            name: "employee-detail".into(),
            label: "员工详情手风琴".into(),
            layout: "Accordion".into(),
            model_id: emp_id.clone(),
            config_json: r#"{"groups":[{"label":"基本信息","fields":["name","email"]},{"label":"组织关系","fields":["project_id","manager_id"]}]}"#.into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        screens.push(AppScreen {
            id: "demo-screen-form".into(),
            name: "employee-form".into(),
            label: "员工录入表单".into(),
            layout: "Form".into(),
            model_id: emp_id.clone(),
            config_json: r#"{"fields":[{"field_name":"name","label":"姓名","field_type":"string","required":true,"placeholder":"输入员工姓名"},{"field_name":"email","label":"邮箱","field_type":"string","required":false,"placeholder":"email@example.com"},{"field_name":"project_id","label":"所属项目","field_type":"string","required":false,"placeholder":"选择项目"}],"submit_label":"保存"}"#.into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        screens.push(AppScreen {
            id: "demo-screen-tree".into(),
            name: "org-tree".into(),
            label: "组织架构树".into(),
            layout: "TreeTable".into(),
            model_id: org_id.clone(),
            config_json: r#"{"tree_field":"parent_id","label_field":"name","columns":[{"field_name":"name","label":"名称"},{"field_name":"level","label":"层级"}]}"#.into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
    }
}
