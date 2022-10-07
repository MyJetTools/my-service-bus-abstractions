use crate::MessageToPublish;

#[async_trait::async_trait]
pub trait MyServiceBusPublisherClient {
    async fn publish_message(&self, topic_name: &str, message: MessageToPublish);
    async fn publish_messages(&self, topic_name: &str, message: Vec<MessageToPublish>);
}

pub trait MySbMessageSerializer<TContract> {
    fn serialize(&self, contract: &TContract) -> Result<Vec<u8>, String>;
    fn deserialize(&self, contracts: &[u8]) -> Result<TContract, String>;
}
