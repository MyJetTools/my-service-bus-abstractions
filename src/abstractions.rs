use crate::{MessageToPublish, PublishError};

#[async_trait::async_trait]
pub trait MyServiceBusPublisherClient {
    async fn publish_message(
        &self,
        topic_name: &str,
        message: &MessageToPublish,
    ) -> Result<(), PublishError>;
    async fn publish_messages(
        &self,
        topic_name: &str,
        message: &[MessageToPublish],
    ) -> Result<(), PublishError>;
}

pub trait MySbMessageSerializer<TContract> {
    fn serialize(&self, contract: &TContract) -> Result<Vec<u8>, String>;
    fn deserialize(&self, contracts: &[u8]) -> Result<TContract, String>;
}
