use inventory_plugin_alipay as _;
use inventory_plugin_other as _;
use inventory_plugin_wechat as _;

#[cfg(test)]
mod tests {
    use pay_api::PayRequest;

    #[test]
    fn app_can_collect_pay_interface_from_multiple_external_crates() {
        let request = PayRequest::new("demo-order", 100);
        let pay_interfaces = pay_api::pay_interfaces();

        assert_eq!(pay_interfaces.len(), 3);

        pay_interfaces.into_iter().for_each(|pay| pay.pay(&request));
    }
}
