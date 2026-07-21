rudi::enable! {
    di_plugin_alipay::enable();
    di_plugin_other::enable();
    di_plugin_wechat::enable();
}

#[cfg(test)]
mod tests {
    use pay_api::{PayRequest, pay_interfaces};
    use rudi::Context;

    #[test]
    fn app_can_resolve_pay_interfaces_from_multiple_external_crates() {
        let request = PayRequest::new("demo-order", 100);
        crate::enable();
        let mut context = Context::auto_register();
        let pay_interfaces = pay_interfaces(&mut context);

        // 三个支付实现都必须由聚合后的 Rudi 上下文提供。
        assert_eq!(pay_interfaces.len(), 3);

        pay_interfaces.into_iter().for_each(|pay| pay.pay(&request));
    }
}
