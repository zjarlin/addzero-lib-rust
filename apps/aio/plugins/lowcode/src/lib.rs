#![forbid(unsafe_code)]

pub mod plugin;
pub mod routes;
pub mod state;
pub mod ui;

use az_aio_platform::core::db::ToastyModelContribution;

pub use plugin::LowcodePlugin;

#[rudi::Singleton(name = "lowcode-toasty-models")]
pub fn lowcode_model_contribution() -> ToastyModelContribution {
    ToastyModelContribution::new(az_engine::engine_models())
}

rudi::enable! {}
