use crate::{
    SmsActivationRequest, SmsError, SmsHostingRequest, SmsInbox, SmsOrder, SmsResult,
    WaitForSmsOptions,
};
use std::time::Instant;

/// Common async interface implemented by SMS providers.
#[async_trait::async_trait]
pub trait SmsProvider: Send + Sync {
    /// Buy a one-time activation number.
    async fn buy_activation_number(&self, request: SmsActivationRequest) -> SmsResult<SmsOrder>;

    /// Buy a hosted/rented number when the provider supports it.
    async fn buy_hosting_number(&self, request: SmsHostingRequest) -> SmsResult<SmsOrder>;

    /// Fetch the current order state and attached SMS messages.
    async fn check_order(&self, order_id: u64) -> SmsResult<SmsOrder>;

    /// Mark an order as successfully finished.
    async fn finish_order(&self, order_id: u64) -> SmsResult<SmsOrder>;

    /// Cancel an order that should no longer be used.
    async fn cancel_order(&self, order_id: u64) -> SmsResult<SmsOrder>;

    /// Reject an order because the number or received SMS is unusable.
    async fn ban_order(&self, order_id: u64) -> SmsResult<SmsOrder>;

    /// Fetch SMS inbox messages for a hosted/rented order.
    async fn inbox(&self, order_id: u64) -> SmsResult<SmsInbox>;

    /// Poll `check_order` until an SMS arrives, the order closes, or the timeout expires.
    async fn wait_for_sms(&self, order_id: u64, options: WaitForSmsOptions) -> SmsResult<SmsOrder> {
        options.validate()?;
        let deadline = Instant::now() + options.timeout;

        loop {
            let order = self.check_order(order_id).await?;
            if !order.sms.is_empty() {
                return Ok(order);
            }
            if order.status.is_terminal() {
                return Err(SmsError::OrderClosed {
                    order_id,
                    status: order.status,
                });
            }
            if Instant::now() >= deadline {
                return Err(SmsError::Timeout {
                    order_id,
                    timeout_secs: options.timeout.as_secs(),
                });
            }
            tokio::time::sleep(options.interval).await;
        }
    }
}
