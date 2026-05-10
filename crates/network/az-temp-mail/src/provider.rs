use crate::TempMailResult;
use crate::model::{
    CreateMailboxRequest, ListResponse, PageRequest, TempMailMailbox, TempMailMessageDetail,
    TempMailMessageSummary, TempMailProviderKind,
};

/// Common receive-mail contract for temporary email providers.
///
/// The trait intentionally models only the stable cross-provider surface:
/// create an address, list messages, and fetch a message. Provider-specific
/// capabilities such as Cloudflare send-mail access stay on concrete clients.
pub trait TempMailProvider: Send + Sync {
    /// Identifies the provider implementation.
    fn provider_kind(&self) -> TempMailProviderKind;

    /// Creates a mailbox and returns the provider credential needed later.
    fn create_mailbox(&self, request: &CreateMailboxRequest) -> TempMailResult<TempMailMailbox>;

    /// Lists messages for a previously created mailbox.
    fn list_messages(
        &self,
        mailbox: &TempMailMailbox,
        page: PageRequest,
    ) -> TempMailResult<ListResponse<TempMailMessageSummary>>;

    /// Fetches a single message by provider message id.
    fn get_message(
        &self,
        mailbox: &TempMailMailbox,
        message_id: &str,
    ) -> TempMailResult<Option<TempMailMessageDetail>>;
}
