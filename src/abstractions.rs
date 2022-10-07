use crate::{MessageToPublish, PublishError};

#[async_trait::async_trait]
pub trait MyServiceBusPublisherClient {
    async fn publish_message(
        &self,
        topic_name: &str,
        message: MessageToPublish,
        do_retry: bool,
    ) -> Result<(), PublishError>;

    async fn publish_messages(
        &self,
        topic_name: &str,
        message: Vec<MessageToPublish>,
        do_retry: bool,
    ) -> Result<(), PublishError>;
}

pub trait MySbMessageSerializer {
    fn serialize(&self) -> Result<Vec<u8>, String>;
}

pub trait MySbMessageDeserializer<T> {
    fn deserialize(src: &[u8]) -> Result<T, String>;
}
