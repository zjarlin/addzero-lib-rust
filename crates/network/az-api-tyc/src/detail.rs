//! Tianyancha company-detail response models.

use serde_json::Value;

/// AB测试信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AbInfo {
    /// AB测试键
    #[serde(rename = "abKey", default)]
    pub ab_key: String,
    /// AB测试值
    #[serde(rename = "abValue", default)]
    pub ab_value: String,
}

/// 地址信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Address {
    /// 地址
    #[serde(rename = "address", default)]
    pub address: String,
    /// 纬度
    #[serde(rename = "latitude", default)]
    pub latitude: String,
    /// 经度
    #[serde(rename = "longitude", default)]
    pub longitude: String,
    /// 报告年份
    #[serde(rename = "reportYear", default)]
    pub report_year: Option<String>,
    /// 来源显示
    #[serde(rename = "showSource", default)]
    pub show_source: String,
    /// 来源显示权重
    #[serde(rename = "sourceDisplayWeight", default)]
    pub source_display_weight: i64,
}

/// 标签信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Boxinfo {
    /// 事件类型
    #[serde(rename = "eventType", default)]
    pub event_type: String,
    /// 图片标题
    #[serde(rename = "imgTitle", default)]
    pub img_title: String,
    /// 视频图片
    #[serde(rename = "videoImg", default)]
    pub video_img: String,
}

/// 天眼查企业详细信息数据实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompanyDetailData {
    /// 企业相关信息列表
    #[serde(
        rename = "abInfo",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub ab_info: Vec<AbInfo>,
    /// 实收资本
    #[serde(rename = "actualCapital", default)]
    pub actual_capital: String,
    /// 地址列表
    #[serde(
        rename = "addressList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub address_list: Vec<Address>,
    /// 企业简称
    #[serde(rename = "alias", default)]
    pub alias: String,
    /// 核准时间
    #[serde(rename = "approvedTime", default)]
    pub approved_time: i64,
    /// 企业所属省份简称
    #[serde(rename = "base", default)]
    pub base: String,
    /// 企业基本信息
    #[serde(rename = "baseInfo", default)]
    pub base_info: String,
    /// 分支机构参保人数
    #[serde(rename = "branchSocialStaffNum", default)]
    pub branch_social_staff_num: String,
    /// 经营范围
    #[serde(rename = "businessScope", default)]
    pub business_scope: String,
    /// 企业状态信息
    #[serde(rename = "businessStatusInfo", default)]
    pub business_status_info: Option<Value>,
    /// 企业资产信息
    #[serde(rename = "companyAssets", default)]
    pub company_assets: Option<Value>,
    /// 企业业务组织类型
    #[serde(rename = "companyBizOrgType", default)]
    pub company_biz_org_type: String,
    /// 企业业务类型
    #[serde(rename = "companyBizType", default)]
    pub company_biz_type: i64,
    /// 统一社会信用代码
    #[serde(rename = "companyCreditCode", default)]
    pub company_credit_code: String,
    /// 企业类型
    #[serde(rename = "companyOrgType", default)]
    pub company_org_type: String,
    /// 企业简介纯文本
    #[serde(rename = "companyProfilePlainText", default)]
    pub company_profile_plain_text: String,
    /// 企业简介纯文本（用于复制）
    #[serde(rename = "companyProfilePlainText4Copy", default)]
    pub company_profile_plain_text4_copy: String,
    /// 企业简介富文本
    #[serde(
        rename = "companyProfileRichText",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub company_profile_rich_text: Vec<CompanyProfileRichText>,
    /// 企业业务类型名称
    #[serde(rename = "companyShowBizTypeName", default)]
    pub company_show_biz_type_name: String,
    /// 企业时间说明
    #[serde(rename = "companyTimeExplain", default)]
    pub company_time_explain: String,
    /// 企业时间标题
    #[serde(rename = "companyTimeTitle", default)]
    pub company_time_title: String,
    /// 企业类型
    #[serde(rename = "companyType", default)]
    pub company_type: i64,
    /// 企业复杂名称
    #[serde(rename = "complexName", default)]
    pub complex_name: String,
    /// 信用代码
    #[serde(rename = "creditCode", default)]
    pub credit_code: String,
    /// 邮箱
    #[serde(rename = "email", default)]
    pub email: String,
    /// 邮箱详情列表
    #[serde(
        rename = "emailDetailList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub email_detail_list: Vec<EmailDetail>,
    /// 邮箱列表
    #[serde(
        rename = "emailList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub email_list: Vec<String>,
    /// 英文名称来源
    #[serde(rename = "enNameSource", default)]
    pub en_name_source: String,
    /// 企业规模信息
    #[serde(rename = "enterpriseScaleInfo", default)]
    pub enterprise_scale_info: EnterpriseScaleInfo,
    /// 实体类型
    #[serde(rename = "entityType", default)]
    pub entity_type: i64,
    /// 股权结构图URL
    #[serde(rename = "equityUrl", default)]
    pub equity_url: String,
    /// 成立时间
    #[serde(rename = "estiblishTime", default)]
    pub estiblish_time: i64,
    /// 成立时间标题名称
    #[serde(rename = "estiblishTimeTitleName", default)]
    pub estiblish_time_title_name: String,
    /// 额外信息
    #[serde(rename = "extraInfo", default)]
    pub extra_info: Option<Value>,
    /// 经营开始时间
    #[serde(rename = "fromTime", default)]
    pub from_time: i64,
    /// 企业ID
    #[serde(rename = "id", default)]
    pub id: i64,
    /// 行业信息
    #[serde(rename = "industry", default)]
    pub industry: Option<Value>,
    /// 2017年行业分类
    #[serde(rename = "industry2017", default)]
    pub industry2017: String,
    /// 产业链列表
    #[serde(
        rename = "industryChainList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub industry_chain_list: Vec<IndustryChain>,
    /// 行业信息详情
    #[serde(rename = "industryInfo", default)]
    pub industry_info: IndustryInfo,
    /// 国家标准行业三级代码
    #[serde(rename = "industryNationalStdLv3Code", default)]
    pub industry_national_std_lv3_code: String,
    /// 是否为分支机构
    #[serde(rename = "isBranch", default)]
    pub is_branch: bool,
    /// 是否已认领
    #[serde(rename = "isClaimed", default)]
    pub is_claimed: i64,
    /// 法人信息
    #[serde(rename = "legalInfo", default)]
    pub legal_info: LegalInfo,
    /// 法人ID
    #[serde(rename = "legalPersonId", default)]
    pub legal_person_id: i64,
    /// 法人姓名
    #[serde(rename = "legalPersonName", default)]
    pub legal_person_name: String,
    /// 法人PID
    #[serde(rename = "legalPersonPid", default)]
    pub legal_person_pid: String,
    /// 法人类型
    #[serde(rename = "legalPersonType", default)]
    pub legal_person_type: i64,
    /// 法人标题名称
    #[serde(rename = "legalTitleName", default)]
    pub legal_title_name: String,
    /// 链接
    #[serde(rename = "link", default)]
    pub link: i64,
    /// 上市板块列表
    #[serde(
        rename = "listedPlateList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub listed_plate_list: Vec<Value>,
    /// 上市状态类型
    #[serde(rename = "listedStatusType", default)]
    pub listed_status_type: i64,
    /// 高管上市状态类型
    #[serde(rename = "listedStatusTypeForSenior", default)]
    pub listed_status_type_for_senior: i64,
    /// 企业logo
    #[serde(rename = "logo", default)]
    pub logo: String,
    /// 企业名称
    #[serde(rename = "name", default)]
    pub name: String,
    /// 新公司收入
    #[serde(rename = "newCompanyIncome", default)]
    pub new_company_income: Option<Value>,
    /// 新公司利润
    #[serde(rename = "newCompanyProfit", default)]
    pub new_company_profit: Option<Value>,
    /// 营业收入信息
    #[serde(rename = "operatingIncomeInfo", default)]
    pub operating_income_info: Option<Value>,
    /// 营业利润信息
    #[serde(rename = "operatingProfitInfo", default)]
    pub operating_profit_info: Option<Value>,
    /// 组织机构代码
    #[serde(rename = "orgNumber", default)]
    pub org_number: String,
    /// 组织类型名称
    #[serde(rename = "orgTypeName", default)]
    pub org_type_name: String,
    /// 原始百分制评分
    #[serde(rename = "originalPercentileScore", default)]
    pub original_percentile_score: i64,
    /// 父级ID
    #[serde(rename = "parentId", default)]
    pub parent_id: Option<Value>,
    /// 百分制评分
    #[serde(rename = "percentileScore", default)]
    pub percentile_score: i64,
    /// 电话列表
    #[serde(
        rename = "phoneList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub phone_list: Vec<String>,
    /// 电话号码
    #[serde(rename = "phoneNumber", default)]
    pub phone_number: String,
    /// 电话来源列表
    #[serde(
        rename = "phoneSourceList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub phone_source_list: Vec<PhoneSource>,
    /// 英文名称
    #[serde(rename = "property3", default)]
    pub property3: String,
    /// 注册资本
    #[serde(rename = "regCapital", default)]
    pub reg_capital: String,
    /// 注册资本金额
    #[serde(rename = "regCapitalAmount", default)]
    pub reg_capital_amount: String,
    /// 注册资本金额单位
    #[serde(rename = "regCapitalAmountUnit", default)]
    pub reg_capital_amount_unit: String,
    /// 注册资本金额（整数）
    #[serde(rename = "regCapitalAmt", default)]
    pub reg_capital_amt: i64,
    /// 注册资本币种
    #[serde(rename = "regCapitalCurrency", default)]
    pub reg_capital_currency: String,
    /// 注册资本标签
    #[serde(rename = "regCapitalLabel", default)]
    pub reg_capital_label: String,
    /// 登记机关
    #[serde(rename = "regInstitute", default)]
    pub reg_institute: String,
    /// 注册地址
    #[serde(rename = "regLocation", default)]
    pub reg_location: String,
    /// 注册地址标题
    #[serde(rename = "regLocationTitle", default)]
    pub reg_location_title: String,
    /// 注册号
    #[serde(rename = "regNumber", default)]
    pub reg_number: String,
    /// 企业经营状态
    #[serde(rename = "regStatus", default)]
    pub reg_status: String,
    /// 注册标题名称
    #[serde(rename = "regTitleName", default)]
    pub reg_title_name: String,
    /// 相关产业链数量
    #[serde(rename = "relatedIndustryChainCount", default)]
    pub related_industry_chain_count: i64,
    /// 安全类型
    #[serde(rename = "safetype", default)]
    pub safetype: String,
    /// 科技信息
    #[serde(rename = "scienceTechnologyInfo", default)]
    pub science_technology_info: ScienceTechnologyInfo,
    /// 敏感实体类型
    #[serde(rename = "sensitiveEntityType", default)]
    pub sensitive_entity_type: i64,
    /// 企业简称
    #[serde(rename = "shortname", default)]
    pub shortname: String,
    /// 是否显示分支机构趋势图
    #[serde(rename = "showBranchTrendChart", default)]
    pub show_branch_trend_chart: bool,
    /// 是否显示减资公告数量
    #[serde(rename = "showCapitalReductionNoticeCount", default)]
    pub show_capital_reduction_notice_count: bool,
    /// 是否显示趋势图
    #[serde(rename = "showTrendChart", default)]
    pub show_trend_chart: bool,
    /// 社保参保人数
    #[serde(rename = "socialStaffNum", default)]
    pub social_staff_num: String,
    /// 员工数量信息
    #[serde(rename = "staffNumInfo", default)]
    pub staff_num_info: StaffNumInfo,
    /// 员工数量信息列表
    #[serde(rename = "staffNumInfoList", default)]
    pub staff_num_info_list: StaffNumInfoList,
    /// 近三年员工数量信息列表
    #[serde(
        rename = "staffNumInfoListThreeYear",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub staff_num_info_list_three_year: Vec<StaffNumInfoThreeYear>,
    /// 员工规模范围
    #[serde(rename = "staffNumRange", default)]
    pub staff_num_range: String,
    /// 标签列表
    #[serde(
        rename = "tagList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub tag_list: Vec<Tag>,
    /// 标签列表V2
    #[serde(
        rename = "tagListV2",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub tag_list_v2: Vec<TagV2>,
    /// 标签列表V3
    #[serde(
        rename = "tagListV3",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub tag_list_v3: Vec<TagV3>,
    /// 标签
    #[serde(rename = "tags", default)]
    pub tags: String,
    /// 税务地址
    #[serde(rename = "taxAddress", default)]
    pub tax_address: String,
    /// 纳税人识别号
    #[serde(rename = "taxNumber", default)]
    pub tax_number: String,
    /// 税务电话
    #[serde(rename = "taxPhone", default)]
    pub tax_phone: String,
    /// 税务资格
    #[serde(rename = "taxQualification", default)]
    pub tax_qualification: String,
    /// 企业类型
    #[serde(rename = "type", default)]
    pub r#type: i64,
    /// 索引更新时间
    #[serde(rename = "updateTime4Index", default)]
    pub update_time4_index: i64,
    /// 更新时间
    #[serde(rename = "updateTimes", default)]
    pub update_times: i64,
    /// 更新时间
    #[serde(rename = "updatetime", default)]
    pub updatetime: i64,
    /// 网站列表
    #[serde(rename = "websiteList", default)]
    pub website_list: Option<Value>,
    /// 微信公众号数量
    #[serde(rename = "wechatCount", default)]
    pub wechat_count: i64,
    /// 分支机构参保年份
    #[serde(rename = "year4BranchSocialStaffNum", default)]
    pub year4_branch_social_staff_num: i64,
    /// 社保参保年份
    #[serde(rename = "year4SocialStaffNum", default)]
    pub year4_social_staff_num: i64,
}

impl CompanyDetailData {
    /// Converts the upstream micro-enterprise numeric flag to a Chinese label.
    pub const fn parse_is_micro_ent(value: i64) -> Option<&'static str> {
        match value {
            0 => Some("不是"),
            1 => Some("是"),
            _ => None,
        }
    }

    /// Converts the upstream legal-person type code to a Chinese label.
    pub const fn parse_type(value: i64) -> Option<&'static str> {
        match value {
            1 => Some("人"),
            2 => Some("公司"),
            _ => None,
        }
    }

    /// Returns the label for this record's `type` code.
    pub const fn type_name(&self) -> Option<&'static str> {
        Self::parse_type(self.r#type)
    }

    /// Returns the label for this record's company-type code.
    pub const fn company_type_name(&self) -> Option<&'static str> {
        Self::parse_company_type(self.company_type)
    }

    /// Converts the upstream company-type code to a Chinese label.
    pub const fn parse_company_type(value: i64) -> Option<&'static str> {
        match value {
            1 => Some("公司"),
            2 => Some("香港公司"),
            3 => Some("社会组织"),
            4 => Some("律所"),
            5 => Some("事业单位"),
            6 => Some("基金会"),
            _ => None,
        }
    }
}

/// CompanyInfoRes response model.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompanyInfoRes {
    /// Original `data` field returned by Tianyancha.
    #[serde(rename = "data", default)]
    pub data: CompanyDetailData,
    /// Original `errorCode` field returned by Tianyancha.
    #[serde(rename = "errorCode", default)]
    pub error_code: Option<Value>,
    /// Original `errorMessage` field returned by Tianyancha.
    #[serde(rename = "errorMessage", default)]
    pub error_message: Option<Value>,
    /// Original `isLogin` field returned by Tianyancha.
    #[serde(rename = "isLogin", default)]
    pub is_login: Option<i64>,
    /// Original `message` field returned by Tianyancha.
    #[serde(rename = "message", default)]
    pub message: Option<String>,
    /// Original `special` field returned by Tianyancha.
    #[serde(rename = "special", default)]
    pub special: Option<String>,
    /// Original `state` field returned by Tianyancha.
    #[serde(rename = "state", default)]
    pub state: Option<String>,
    /// Original `vipMessage` field returned by Tianyancha.
    #[serde(rename = "vipMessage", default)]
    pub vip_message: Option<String>,
}

/// 公司简介富文本信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompanyProfileRichText {
    /// 内容
    #[serde(rename = "content", default)]
    pub content: String,
    /// 标题
    #[serde(rename = "title", default)]
    pub title: String,
}

/// 公司员工信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompanyStaff {
    /// 分支机构数量
    #[serde(rename = "branchNum", default)]
    pub branch_num: i64,
    /// 是否存在分支机构
    #[serde(rename = "existBranch", default)]
    pub exist_branch: bool,
    /// 说明文本
    #[serde(rename = "explainText", default)]
    pub explain_text: String,
    /// 员工数量
    #[serde(rename = "num", default)]
    pub num: i64,
    /// 路由
    #[serde(rename = "route", default)]
    pub route: String,
    /// 是否显示分支机构趋势图
    #[serde(rename = "showBranchTrendChart", default)]
    pub show_branch_trend_chart: bool,
    /// 是否显示趋势图
    #[serde(rename = "showTrendChart", default)]
    pub show_trend_chart: bool,
    /// 来源
    #[serde(rename = "source", default)]
    pub source: String,
    /// APP端来源显示
    #[serde(rename = "sourceForApp", default)]
    pub source_for_app: String,
    /// 年份
    #[serde(rename = "year", default)]
    pub year: i64,
}

/// 邮箱详情信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmailDetail {
    /// 邮箱
    #[serde(rename = "email", default)]
    pub email: String,
    /// 报告年份
    #[serde(rename = "reportYear", default)]
    pub report_year: String,
    /// 相同邮箱数量
    #[serde(rename = "sameEmailCount", default)]
    pub same_email_count: Option<String>,
    /// 来源显示
    #[serde(rename = "showSource", default)]
    pub show_source: String,
    /// 来源显示权重
    #[serde(rename = "sourceDisplayWeight", default)]
    pub source_display_weight: String,
}

/// 企业规模信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnterpriseScaleInfo {
    /// 说明文本
    #[serde(rename = "explainText", default)]
    pub explain_text: String,
    /// HTML格式说明文本
    #[serde(rename = "explainTextHtml", default)]
    pub explain_text_html: String,
    /// 图标类型
    #[serde(rename = "iconType", default)]
    pub icon_type: String,
    /// 企业规模
    #[serde(rename = "scale", default)]
    pub scale: String,
}

/// 产业链信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IndustryChain {
    /// 代码
    #[serde(rename = "code", default)]
    pub code: String,
    /// 名称
    #[serde(rename = "name", default)]
    pub name: String,
}

/// 行业信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IndustryInfo {
    /// 行业代码
    #[serde(rename = "code", default)]
    pub code: String,
    /// 说明文本
    #[serde(rename = "explainText", default)]
    pub explain_text: String,
    /// 一级行业名称
    #[serde(rename = "nameLevel1", default)]
    pub name_level1: String,
    /// 二级行业名称
    #[serde(rename = "nameLevel2", default)]
    pub name_level2: String,
    /// 三级行业名称
    #[serde(rename = "nameLevel3", default)]
    pub name_level3: String,
    /// 四级行业名称
    #[serde(rename = "nameLevel4", default)]
    pub name_level4: Option<Value>,
}

/// 法人信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LegalInfo {
    /// 别名
    #[serde(rename = "alias", default)]
    pub alias: Option<Value>,
    /// 老板证书
    #[serde(rename = "bossCertificate", default)]
    pub boss_certificate: i64,
    /// 公司ID
    #[serde(rename = "cid", default)]
    pub cid: i64,
    /// 公司数量
    #[serde(rename = "companyNum", default)]
    pub company_num: i64,
    /// 公司列表
    #[serde(rename = "companys", default)]
    pub companys: Option<Value>,
    /// 合作次数
    #[serde(rename = "coopCount", default)]
    pub coop_count: i64,
    /// 事件信息
    #[serde(rename = "event", default)]
    pub event: Option<Value>,
    /// 头像URL
    #[serde(rename = "headUrl", default)]
    pub head_url: Option<Value>,
    /// 法人_hid
    #[serde(rename = "hid", default)]
    pub hid: i64,
    /// 介绍信息
    #[serde(rename = "introduction", default)]
    pub introduction: Option<Value>,
    /// 法人姓名
    #[serde(rename = "name", default)]
    pub name: String,
    /// 办公室列表
    #[serde(
        rename = "office",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub office: Vec<Office>,
    /// 办公室列表V1
    #[serde(
        rename = "officeV1",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub office_v1: Vec<OfficeV1>,
    /// 合伙人数量
    #[serde(rename = "partnerNum", default)]
    pub partner_num: i64,
    /// 合伙人信息
    #[serde(rename = "partners", default)]
    pub partners: Option<Value>,
    /// PID
    #[serde(rename = "pid", default)]
    pub pid: Option<Value>,
    /// 角色
    #[serde(rename = "role", default)]
    pub role: Option<Value>,
    /// 服务次数
    #[serde(rename = "serviceCount", default)]
    pub service_count: i64,
    /// 服务类型
    #[serde(rename = "serviceType", default)]
    pub service_type: i64,
    /// 类型连接
    #[serde(rename = "typeJoin", default)]
    pub type_join: Option<Value>,
}

/// 办公室信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Office {
    /// 地区
    #[serde(rename = "area", default)]
    pub area: String,
    /// 公司ID
    #[serde(rename = "cid", default)]
    pub cid: i64,
    /// 公司名称
    #[serde(rename = "companyName", default)]
    pub company_name: String,
    /// 分数
    #[serde(rename = "score", default)]
    pub score: i64,
    /// 状态
    #[serde(rename = "state", default)]
    pub state: Option<Value>,
    /// 总数
    #[serde(rename = "total", default)]
    pub total: i64,
}

/// 办公室信息V1实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfficeV1 {
    /// 地区
    #[serde(rename = "area", default)]
    pub area: String,
    /// 公司ID
    #[serde(rename = "cid", default)]
    pub cid: i64,
    /// 公司名称
    #[serde(rename = "companyName", default)]
    pub company_name: String,
    /// 分数
    #[serde(rename = "score", default)]
    pub score: i64,
    /// 状态
    #[serde(rename = "state", default)]
    pub state: Option<Value>,
    /// 总数
    #[serde(rename = "total", default)]
    pub total: i64,
}

/// 电话来源信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PhoneSource {
    /// 公司ID
    #[serde(rename = "cid", default)]
    pub cid: i64,
    /// 城市
    #[serde(rename = "city", default)]
    pub city: String,
    /// 公司数量
    #[serde(rename = "companyCount", default)]
    pub company_count: i64,
    /// 公司数量字符串
    #[serde(rename = "companyCountStr", default)]
    pub company_count_str: String,
    /// 公司名称
    #[serde(rename = "companyName", default)]
    pub company_name: String,
    /// 公司总数字符串
    #[serde(rename = "companyTotalStr", default)]
    pub company_total_str: String,
    /// 公司类型
    #[serde(rename = "companyType", default)]
    pub company_type: i64,
    /// GID
    #[serde(rename = "gid", default)]
    pub gid: i64,
    /// 是否有更多公司
    #[serde(rename = "hasMoreCompany", default)]
    pub has_more_company: i64,
    /// 原始电话号码
    #[serde(rename = "oriPhoneNumber", default)]
    pub ori_phone_number: Option<Value>,
    /// 电话号码
    #[serde(rename = "phoneNumber", default)]
    pub phone_number: String,
    /// 电话标签
    #[serde(rename = "phoneTag", default)]
    pub phone_tag: Option<Value>,
    /// 电话标签列表
    #[serde(rename = "phoneTagList", default)]
    pub phone_tag_list: Option<Value>,
    /// 电话标签类型
    #[serde(rename = "phoneTagType", default)]
    pub phone_tag_type: i64,
    /// 电话提示
    #[serde(rename = "phoneTips", default)]
    pub phone_tips: String,
    /// 电话类型
    #[serde(rename = "phoneType", default)]
    pub phone_type: i64,
    /// 省份
    #[serde(rename = "province", default)]
    pub province: String,
    /// 报告年份
    #[serde(rename = "reportYear", default)]
    pub report_year: Option<String>,
    /// 来源显示
    #[serde(rename = "showSource", default)]
    pub show_source: String,
    /// 疑似账户标签
    #[serde(rename = "suspectedAccountTag", default)]
    pub suspected_account_tag: Option<Value>,
    /// 疑似账户标签URL
    #[serde(rename = "suspectedAccountTagUrl", default)]
    pub suspected_account_tag_url: Option<Value>,
}

/// 科技信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScienceTechnologyInfo {
    /// 超过百分比
    #[serde(rename = "exceedPercent", default)]
    pub exceed_percent: String,
    /// 等级
    #[serde(rename = "grade", default)]
    pub grade: String,
    /// 等级颜色
    #[serde(rename = "gradeColor", default)]
    pub grade_color: String,
    /// 分数
    #[serde(rename = "score", default)]
    pub score: i64,
}

/// 员工数量信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StaffNumInfo {
    /// 分支机构数量
    #[serde(rename = "branchNum", default)]
    pub branch_num: i64,
    /// 是否存在分支机构
    #[serde(rename = "existBranch", default)]
    pub exist_branch: bool,
    /// 说明文本
    #[serde(rename = "explainText", default)]
    pub explain_text: String,
    /// 员工数量
    #[serde(rename = "num", default)]
    pub num: i64,
    /// 路由
    #[serde(rename = "route", default)]
    pub route: String,
    /// 是否显示分支机构趋势图
    #[serde(rename = "showBranchTrendChart", default)]
    pub show_branch_trend_chart: bool,
    /// 是否显示趋势图
    #[serde(rename = "showTrendChart", default)]
    pub show_trend_chart: bool,
    /// 来源
    #[serde(rename = "source", default)]
    pub source: String,
    /// APP端来源显示
    #[serde(rename = "sourceForApp", default)]
    pub source_for_app: String,
    /// 年份
    #[serde(rename = "year", default)]
    pub year: i64,
}

/// 员工数量信息列表实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StaffNumInfoList {
    /// 公司员工列表
    #[serde(
        rename = "companyStaffs",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub company_staffs: Vec<CompanyStaff>,
    /// 说明文本
    #[serde(rename = "explainText", default)]
    pub explain_text: String,
    /// 是否弹出
    #[serde(rename = "isPop", default)]
    pub is_pop: bool,
}

/// 近三年员工数量信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StaffNumInfoThreeYear {
    /// 分支机构数量
    #[serde(rename = "branchNum", default)]
    pub branch_num: i64,
    /// 是否存在分支机构
    #[serde(rename = "existBranch", default)]
    pub exist_branch: bool,
    /// 说明文本
    #[serde(rename = "explainText", default)]
    pub explain_text: String,
    /// 员工数量
    #[serde(rename = "num", default)]
    pub num: i64,
    /// 路由
    #[serde(rename = "route", default)]
    pub route: String,
    /// 是否显示分支机构趋势图
    #[serde(rename = "showBranchTrendChart", default)]
    pub show_branch_trend_chart: bool,
    /// 是否显示趋势图
    #[serde(rename = "showTrendChart", default)]
    pub show_trend_chart: bool,
    /// 来源
    #[serde(rename = "source", default)]
    pub source: String,
    /// APP端来源显示
    #[serde(rename = "sourceForApp", default)]
    pub source_for_app: String,
    /// 年份
    #[serde(rename = "year", default)]
    pub year: i64,
}

/// 标签信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    /// 背景颜色
    #[serde(rename = "background", default)]
    pub background: String,
    /// 标签信息
    #[serde(rename = "boxinfo", default)]
    pub boxinfo: Option<Boxinfo>,
    /// 字体颜色
    #[serde(rename = "color", default)]
    pub color: String,
    /// 层级信息
    #[serde(rename = "layer", default)]
    pub layer: String,
    /// 层级信息数组
    #[serde(rename = "layerArray", default)]
    pub layer_array: Option<Vec<String>>,
    /// 排序
    #[serde(rename = "sort", default)]
    pub sort: i64,
    /// 标题
    #[serde(rename = "title", default)]
    pub title: String,
    /// 类型
    #[serde(rename = "type", default)]
    pub r#type: i64,
    /// 值
    #[serde(rename = "value", default)]
    pub value: String,
}

/// 标签信息V2实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TagV2 {
    /// 背景颜色
    #[serde(rename = "background", default)]
    pub background: String,
    /// 点击超链接类型
    #[serde(rename = "clickHyperLinkType", default)]
    pub click_hyper_link_type: i64,
    /// 点击URL
    #[serde(rename = "clickUrl", default)]
    pub click_url: String,
    /// 字体颜色
    #[serde(rename = "color", default)]
    pub color: String,
    /// 悬停信息
    #[serde(rename = "hover", default)]
    pub hover: String,
    /// logo
    #[serde(rename = "logo", default)]
    pub logo: String,
    /// 名称
    #[serde(rename = "name", default)]
    pub name: String,
    /// 标签ID
    #[serde(rename = "tagId", default)]
    pub tag_id: i64,
    /// 标题
    #[serde(rename = "title", default)]
    pub title: String,
}

/// 标签信息V3实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TagV3 {
    /// 动作类型
    #[serde(rename = "actionType", default)]
    pub action_type: String,
    /// Android版本
    #[serde(rename = "androidVersion", default)]
    pub android_version: Option<Value>,
    /// 背景颜色
    #[serde(rename = "background", default)]
    pub background: String,
    /// 边框颜色
    #[serde(rename = "borderColor", default)]
    pub border_color: String,
    /// 边框透明度
    #[serde(rename = "borderTransparency", default)]
    pub border_transparency: f64,
    /// 边框宽度
    #[serde(rename = "borderWidth", default)]
    pub border_width: f64,
    /// 字体颜色
    #[serde(rename = "color", default)]
    pub color: String,
    /// 字体族
    #[serde(rename = "fontFamily", default)]
    pub font_family: String,
    /// 字体大小
    #[serde(rename = "fontSize", default)]
    pub font_size: i64,
    /// 引导颜色
    #[serde(rename = "guideColor", default)]
    pub guide_color: String,
    /// 引导透明度
    #[serde(rename = "guideTransparency", default)]
    pub guide_transparency: f64,
    /// 悬停通知内容
    #[serde(rename = "hoverNoticeContent", default)]
    pub hover_notice_content: String,
    /// 悬停通知类型
    #[serde(rename = "hoverNoticeType", default)]
    pub hover_notice_type: i64,
    /// iOS版本
    #[serde(rename = "iOSVersion", default)]
    pub i_os_version: Option<Value>,
    /// ID
    #[serde(rename = "id", default)]
    pub id: i64,
    /// logo
    #[serde(rename = "logo", default)]
    pub logo: String,
    /// 名称
    #[serde(rename = "name", default)]
    pub name: String,
    /// 排序
    #[serde(rename = "order", default)]
    pub order: i64,
    /// 弹出名称
    #[serde(rename = "popName", default)]
    pub pop_name: String,
    /// 标签点击超链接详情
    #[serde(rename = "profileTagClickHyperlinkDetails", default)]
    pub profile_tag_click_hyperlink_details: String,
    /// 标签点击超链接类型
    #[serde(rename = "profileTagClickHyperlinkType", default)]
    pub profile_tag_click_hyperlink_type: i64,
    /// 标签类型ID
    #[serde(rename = "profileTagTypeId", default)]
    pub profile_tag_type_id: i64,
    /// 标签类型排名
    #[serde(rename = "profileTagTypeRanking", default)]
    pub profile_tag_type_ranking: i64,
    /// 路由动作
    #[serde(rename = "routingAction", default)]
    pub routing_action: bool,
    /// 路由地址
    #[serde(rename = "routingAddr", default)]
    pub routing_addr: Option<String>,
    /// 路由名称
    #[serde(rename = "routingName", default)]
    pub routing_name: Option<String>,
    /// 显示条件
    #[serde(rename = "showCondition", default)]
    pub show_condition: Option<Value>,
    /// 标题
    #[serde(rename = "title", default)]
    pub title: String,
    /// 类型
    #[serde(rename = "type", default)]
    pub r#type: i64,
}
