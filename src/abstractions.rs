#[async_trait::async_trait]
pub trait MyServiceBusClient {
    async fn publish_message(&self, topic_name: &str, payload: Vec<u8>);
    async fn publish_messages(&self, topic_name: &str, payloads: Vec<Vec<u8>>);
}

pub trait MySbMessageSerializer<TContract> {
    fn serialize(&self, contract: &TContract) -> Result<Vec<u8>, String>;
    fn deserialize(&self, contracts: &[u8]) -> Result<TContract, String>;
}
