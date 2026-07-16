#![forbid(unsafe_code)]

automod::dir!(pub "src");

pub use plugin::AlgorithmCenterPlugin;

rudi::enable! {
    az_algorithm::enable();
}

#[cfg(test)]
mod tests {
    use az_algorithm::di::resolve_algorithm_catalog;
    use rudi::Context;

    #[test]
    fn enable_registers_algorithm_catalog_provider() -> anyhow::Result<()> {
        super::enable();
        let mut context = Context::auto_register();
        let catalog = resolve_algorithm_catalog(&mut context)?;

        // algorithm-center 启用时必须同时暴露 az-algorithm 的 Rudi 服务。
        assert_eq!(catalog.components().len(), 9);
        Ok(())
    }
}
