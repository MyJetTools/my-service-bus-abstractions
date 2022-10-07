use std::sync::Arc;

use crate::{MySbMessageSerializer, MyServiceBusClient};

pub struct MyServiceBusPublisher<TContract> {
    pub topic_name: String,
    client: Arc<dyn MyServiceBusClient + Send + Sync + 'static>,
    serializer: Arc<dyn MySbMessageSerializer<TContract> + Send + Sync + 'static>,
}

impl<TContract> MyServiceBusPublisher<TContract> {
    pub fn new(
        topic_name: String,
        client: Arc<dyn MyServiceBusClient + Send + Sync + 'static>,
        serializer: Arc<dyn MySbMessageSerializer<TContract> + Send + Sync + 'static>,
    ) -> Self {
        Self {
            topic_name,
            client,
            serializer,
        }
    }

    pub async fn publish(&self, message: &TContract) {
        let bytes = self.serializer.serialize(message).unwrap();
        self.client.publish_message(&self.topic_name, bytes).await;
    }

    pub async fn publish_messages(&self, messages: &[TContract]) {
        let mut payloads = Vec::with_capacity(messages.len());

        for message in messages {
            let bytes = self.serializer.serialize(message).unwrap();
            payloads.push(bytes);
        }

        self.client
            .publish_messages(&self.topic_name, payloads)
            .await;
    }
}
