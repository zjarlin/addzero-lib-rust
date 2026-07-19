//! Tianyancha company-search response models.

use serde_json::Value;

/// 公司信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Company {
    /// 公司简称
    #[serde(rename = "abbr", default)]
    pub abbr: String,
    /// 基本信息摘要
    #[serde(rename = "abstractsBaseInfo", default)]
    pub abstracts_base_info: String,
    /// 地址
    #[serde(rename = "address", default)]
    pub address: Option<Value>,
    /// 后比率
    #[serde(rename = "afterRatio", default)]
    pub after_ratio: String,
    /// 别名
    #[serde(rename = "alias", default)]
    pub alias: String,
    /// 区域代码列表
    #[serde(
        rename = "areaCodes",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub area_codes: Vec<String>,
    /// 百度URL
    #[serde(rename = "baiduUrl", default)]
    pub baidu_url: Option<Value>,
    /// 基础信息
    #[serde(rename = "base", default)]
    pub base: String,
    /// 业务类型
    #[serde(rename = "bizType", default)]
    pub biz_type: String,
    /// 债券名称
    #[serde(rename = "bondName", default)]
    pub bond_name: Option<Value>,
    /// 债券编号
    #[serde(rename = "bondNum", default)]
    pub bond_num: Option<Value>,
    /// 债券类型
    #[serde(rename = "bondType", default)]
    pub bond_type: Option<Value>,
    /// 奖金分数
    #[serde(rename = "bonusScore", default)]
    pub bonus_score: String,
    /// 业务项目列表
    #[serde(rename = "businessItemList", default)]
    pub business_item_list: Option<Value>,
    /// 经营范围
    #[serde(rename = "businessScope", default)]
    pub business_scope: String,
    /// 分类代码
    #[serde(rename = "categoryCode", default)]
    pub category_code: String,
    /// 2017年分类代码列表
    #[serde(
        rename = "categoryCode2017List",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub category_code2017_list: Vec<String>,
    /// 标准分类代码
    #[serde(rename = "categoryCodeStd", default)]
    pub category_code_std: String,
    /// 分类字符串
    #[serde(rename = "categoryStr", default)]
    pub category_str: String,
    /// 变更金额
    #[serde(rename = "changeAmt", default)]
    pub change_amt: String,
    /// 变更比率
    #[serde(rename = "changeRatio", default)]
    pub change_ratio: String,
    /// 变更时间
    #[serde(rename = "changeTime", default)]
    pub change_time: String,
    /// 城市
    #[serde(rename = "city", default)]
    pub city: String,
    /// 声明信息
    #[serde(rename = "claimInfo", default)]
    pub claim_info: Option<Value>,
    /// 声明包类型
    #[serde(rename = "claimPkgType", default)]
    pub claim_pkg_type: Option<Value>,
    /// 公司品牌信息
    #[serde(rename = "companyBrandInfo", default)]
    pub company_brand_info: Option<Value>,
    /// 公司集团信息
    #[serde(rename = "companyGroupInfo", default)]
    pub company_group_info: Option<Value>,
    /// 公司数量
    #[serde(rename = "companyNum", default)]
    pub company_num: Option<Value>,
    /// 公司组织类型
    #[serde(rename = "companyOrgType", default)]
    pub company_org_type: String,
    /// 公司电话簿
    #[serde(rename = "companyPhoneBook", default)]
    pub company_phone_book: CompanyPhoneBook,
    /// 公司问题
    #[serde(rename = "companyQuestions", default)]
    pub company_questions: CompanyQuestions,
    /// 公司规模
    #[serde(rename = "companyScale", default)]
    pub company_scale: Option<String>,
    /// 公司评分
    #[serde(rename = "companyScore", default)]
    pub company_score: String,
    /// 公司类型
    #[serde(rename = "companyType", default)]
    pub company_type: i64,
    /// 联系人映射
    #[serde(rename = "contantMap", default)]
    pub contant_map: ContantMap,
    /// 信用代码
    #[serde(rename = "creditCode", default)]
    pub credit_code: String,
    /// 部门
    #[serde(rename = "department", default)]
    pub department: String,
    /// 距离
    #[serde(rename = "distance", default)]
    pub distance: Option<Value>,
    /// 区域
    #[serde(rename = "district", default)]
    pub district: String,
    /// 文档特征
    #[serde(rename = "docFeature", default)]
    pub doc_feature: String,
    /// 邮箱列表
    #[serde(
        rename = "emailList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub email_list: Vec<String>,
    /// 邮箱
    #[serde(rename = "emails", default)]
    pub emails: String,
    /// 英文名称
    #[serde(rename = "englishName", default)]
    pub english_name: Option<String>,
    /// 成立时间
    #[serde(rename = "establishmentTime", default)]
    pub establishment_time: String,
    /// 成立时间
    #[serde(rename = "estiblishTime", default)]
    pub estiblish_time: String,
    /// 成立时间显示字符串
    #[serde(rename = "estiblishTimeShowStr", default)]
    pub estiblish_time_show_str: String,
    /// 执行人员
    #[serde(rename = "executive", default)]
    pub executive: Option<Value>,
    /// 融资轮次
    #[serde(rename = "financingRound", default)]
    pub financing_round: String,
    /// 首要职位显示字符串
    #[serde(rename = "firstPositionShowStr", default)]
    pub first_position_show_str: String,
    /// 首要职位值
    #[serde(rename = "firstPositionValue", default)]
    pub first_position_value: String,
    /// 地理位置
    #[serde(rename = "geoLocation", default)]
    pub geo_location: Option<Value>,
    /// B组ID
    #[serde(rename = "gidForB", default)]
    pub gid_for_b: String,
    /// 是否有更多电话
    #[serde(rename = "hasMorePhone", default)]
    pub has_more_phone: Option<Value>,
    /// 是否有视频
    #[serde(rename = "hasVideo", default)]
    pub has_video: Option<Value>,
    /// 隐藏状态
    #[serde(rename = "hidden", default)]
    pub hidden: i64,
    /// 隐藏电话
    #[serde(rename = "hiddenPhones", default)]
    pub hidden_phones: Option<Value>,
    /// 历史名称
    #[serde(rename = "historyNames", default)]
    pub history_names: String,
    /// 人员名称
    #[serde(rename = "humanNames", default)]
    pub human_names: String,
    /// ICP备案
    #[serde(rename = "icp", default)]
    pub icp: String,
    /// ICP备案列表
    #[serde(rename = "icps", default)]
    pub icps: Option<Value>,
    /// ID
    #[serde(rename = "id", default)]
    pub id: i64,
    /// 违法类型
    #[serde(rename = "illegalType", default)]
    pub illegal_type: String,
    /// 行业
    #[serde(rename = "industry", default)]
    pub industry: Option<Value>,
    /// 机构类型列表
    #[serde(
        rename = "institutionTypeList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub institution_type_list: Vec<String>,
    /// 是否为分支机构
    #[serde(rename = "isBranch", default)]
    pub is_branch: i64,
    /// 是否已认领
    #[serde(rename = "isClaimed", default)]
    pub is_claimed: i64,
    /// 是否在内
    #[serde(rename = "isIn", default)]
    pub is_in: String,
    /// 是否推荐
    #[serde(rename = "isRecommend", default)]
    pub is_recommend: Option<Value>,
    /// 标签JSON列表
    #[serde(
        rename = "labelJsonList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub label_json_list: Vec<String>,
    /// 标签列表
    #[serde(rename = "labelList", default)]
    pub label_list: Option<Value>,
    /// 标签列表V2
    #[serde(
        rename = "labelListV2",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub label_list_v2: Vec<String>,
    /// 纬度
    #[serde(rename = "latitude", default)]
    pub latitude: Option<Value>,
    /// 法人
    #[serde(rename = "legalPerson", default)]
    pub legal_person: String,
    /// 法人ID
    #[serde(rename = "legalPersonId", default)]
    pub legal_person_id: String,
    /// 法人姓名
    #[serde(rename = "legalPersonName", default)]
    pub legal_person_name: String,
    /// 法人显示字符串
    #[serde(rename = "legalPersonShowStr", default)]
    pub legal_person_show_str: String,
    /// 法人类型
    #[serde(rename = "legalPersonType", default)]
    pub legal_person_type: String,
    /// logo
    #[serde(rename = "logo", default)]
    pub logo: String,
    /// 经度
    #[serde(rename = "longitude", default)]
    pub longitude: Option<Value>,
    /// 主ID
    #[serde(rename = "mainId", default)]
    pub main_id: String,
    /// 匹配字段
    #[serde(rename = "matchField", default)]
    pub match_field: Option<MatchField>,
    /// 匹配类型
    #[serde(rename = "matchType", default)]
    pub match_type: String,
    /// 多匹配字段
    #[serde(
        rename = "multiMatchField",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub multi_match_field: Vec<MultiMatchField>,
    /// 名称
    #[serde(rename = "name", default)]
    pub name: String,
    /// 新测试名称
    #[serde(rename = "newtestName", default)]
    pub newtest_name: Option<Value>,
    /// 无实际资本
    #[serde(rename = "noActualCapital", default)]
    pub no_actual_capital: Option<Value>,
    /// 组织机构代码
    #[serde(rename = "orgNumber", default)]
    pub org_number: String,
    /// 原始分数
    #[serde(rename = "orginalScore", default)]
    pub orginal_score: String,
    /// 电话
    #[serde(rename = "phone", default)]
    pub phone: String,
    /// 电话信息列表
    #[serde(
        rename = "phoneInfoList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub phone_info_list: Vec<PhoneInfo>,
    /// 电话列表
    #[serde(
        rename = "phoneList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub phone_list: Vec<String>,
    /// 电话号码
    #[serde(rename = "phoneNum", default)]
    pub phone_num: String,
    /// 产品列表
    #[serde(rename = "productList", default)]
    pub product_list: Option<Value>,
    /// 省份
    #[serde(rename = "province", default)]
    pub province: String,
    /// 合格金融产品
    #[serde(rename = "qualifiedFinancialProduct", default)]
    pub qualified_financial_product: Option<Value>,
    /// 实际声明包类型
    #[serde(rename = "realClaimPkgType", default)]
    pub real_claim_pkg_type: Option<Value>,
    /// 注册资本
    #[serde(rename = "regCapital", default)]
    pub reg_capital: String,
    /// 注册资本显示字符串
    #[serde(rename = "regCapitalShowStr", default)]
    pub reg_capital_show_str: String,
    /// 注册地址
    #[serde(rename = "regLocation", default)]
    pub reg_location: String,
    /// 注册号
    #[serde(rename = "regNumber", default)]
    pub reg_number: String,
    /// 注册状态
    #[serde(rename = "regStatus", default)]
    pub reg_status: String,
    /// 注册机构
    #[serde(rename = "registerInstitute", default)]
    pub register_institute: String,
    /// 报告货币
    #[serde(rename = "repCurrency", default)]
    pub rep_currency: Option<Value>,
    /// 住宅楼
    #[serde(rename = "residentialBuilding", default)]
    pub residential_building: Option<Value>,
    /// 分数
    #[serde(rename = "score", default)]
    pub score: String,
    /// 第二职位显示字符串
    #[serde(rename = "secondPositionShowStr", default)]
    pub second_position_show_str: String,
    /// 第二职位值
    #[serde(rename = "secondPositionValue", default)]
    pub second_position_value: String,
    /// 社保员工数量
    #[serde(rename = "socialSecurityStaff_num", default)]
    pub social_security_staff_num: Option<String>,
    /// 员工数量报告年份
    #[serde(rename = "staffNumReportYear", default)]
    pub staff_num_report_year: i64,
    /// 标签列表
    #[serde(rename = "tagList", default)]
    pub tag_list: Option<Value>,
    /// 目标GID
    #[serde(rename = "targetGid", default)]
    pub target_gid: String,
    /// 目标名称
    #[serde(rename = "targetName", default)]
    pub target_name: String,
    /// 目标注册资本金额
    #[serde(rename = "targetRegCapitalAmount", default)]
    pub target_reg_capital_amount: String,
    /// 目标注册资本货币
    #[serde(rename = "targetRegCapitalCurrency", default)]
    pub target_reg_capital_currency: String,
    /// 税务代码
    #[serde(rename = "taxCode", default)]
    pub tax_code: String,
    /// 三个月诉讼
    #[serde(rename = "threeMonthsLawsuit", default)]
    pub three_months_lawsuit: Option<Value>,
    /// 商标列表
    #[serde(rename = "tmList", default)]
    pub tm_list: Option<Value>,
    /// 商标
    #[serde(rename = "trademarks", default)]
    pub trademarks: Option<Value>,
    /// 类型
    #[serde(rename = "type", default)]
    pub r#type: i64,
    /// 曾用债券名称
    #[serde(rename = "usedBondName", default)]
    pub used_bond_name: Option<Value>,
    /// 视频ID
    #[serde(rename = "videoId", default)]
    pub video_id: Option<Value>,
    /// 网站备案数量
    #[serde(rename = "websiteFilingCount", default)]
    pub website_filing_count: i64,
    /// 网站
    #[serde(rename = "websites", default)]
    pub websites: String,
}

/// 搜索结果数据实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompanyData {
    /// 建议查询
    #[serde(rename = "adviceQuery", default)]
    pub advice_query: Option<Value>,
    /// 公司数量
    #[serde(rename = "companyCount", default)]
    pub company_count: i64,
    /// 公司人员数量
    #[serde(rename = "companyHumanCount", default)]
    pub company_human_count: i64,
    /// 公司列表
    #[serde(
        rename = "companyList",
        default,
        deserialize_with = "crate::serde_support::deserialize_null_default"
    )]
    pub company_list: Vec<Company>,
    /// 公司总数
    #[serde(rename = "companyTotal", default)]
    pub company_total: i64,
    /// 公司总页数
    #[serde(rename = "companyTotalPage", default)]
    pub company_total_page: i64,
    /// 公司总数字符串
    #[serde(rename = "companyTotalStr", default)]
    pub company_total_str: String,
    /// 人员数量
    #[serde(rename = "humanCount", default)]
    pub human_count: i64,
    /// 修改后的查询
    #[serde(rename = "modifiedQuery", default)]
    pub modified_query: Option<Value>,
    /// 搜索内容
    #[serde(rename = "searchContent", default)]
    pub search_content: String,
}

/// 公司电话簿信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompanyPhoneBook {
    /// 朋友名称
    #[serde(rename = "friendName", default)]
    pub friend_name: Option<Value>,
    /// 类型
    #[serde(rename = "type", default)]
    pub r#type: Option<Value>,
}

/// 公司问题信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompanyQuestions {
    /// logo列表
    #[serde(rename = "logos", default)]
    pub logos: Option<Value>,
    /// 参与者数量
    #[serde(rename = "participantCount", default)]
    pub participant_count: Option<Value>,
    /// 问题ID
    #[serde(rename = "qid", default)]
    pub qid: Option<Value>,
    /// 文本内容
    #[serde(rename = "textContent", default)]
    pub text_content: Option<Value>,
}

/// 联系人映射信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ContantMap {
    /// 成立时间（长整型）
    #[serde(rename = "establish_time_long", default)]
    pub establish_time_long: String,
    /// 注册资本参数
    #[serde(rename = "param_reg_capital", default)]
    pub param_reg_capital: Option<String>,
    /// 回调来源
    #[serde(rename = "recallSrc", default)]
    pub recall_src: String,
}

/// 匹配字段信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MatchField {
    /// 内容
    #[serde(rename = "content", default)]
    pub content: String,
    /// 字段
    #[serde(rename = "field", default)]
    pub field: String,
}

/// 多匹配字段信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MultiMatchField {
    /// 内容
    #[serde(rename = "content", default)]
    pub content: String,
    /// 字段
    #[serde(rename = "field", default)]
    pub field: String,
}

/// 电话信息实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PhoneInfo {
    /// 评论
    #[serde(rename = "comment", default)]
    pub comment: String,
    /// 标签
    #[serde(rename = "label", default)]
    pub label: String,
    /// 号码
    #[serde(rename = "number", default)]
    pub number: String,
    /// 来源
    #[serde(rename = "source", default)]
    pub source: Option<Value>,
    /// 类型
    #[serde(rename = "type", default)]
    pub r#type: String,
}

/// 搜索结果响应实体类
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SearchRes {
    /// 数据
    #[serde(rename = "data", default)]
    pub data: CompanyData,
    /// 是否已登录
    #[serde(rename = "isLogin", default)]
    pub is_login: i64,
    /// 消息
    #[serde(rename = "message", default)]
    pub message: String,
    /// 特殊信息
    #[serde(rename = "special", default)]
    pub special: String,
    /// 状态
    #[serde(rename = "state", default)]
    pub state: String,
    /// VIP消息
    #[serde(rename = "vipMessage", default)]
    pub vip_message: String,
}
