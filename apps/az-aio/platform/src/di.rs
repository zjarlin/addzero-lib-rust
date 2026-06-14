//! 依赖注入模块注册。
//!
//! 所有 shaku bean 在此统一注册。

use shaku::module;

use crate::config::ConfigCenterConfig;

module! {
    pub AppModule {
        components = [ConfigCenterConfig],
        providers = [],
    }
}
