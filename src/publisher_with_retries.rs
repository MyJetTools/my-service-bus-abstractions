use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{MessageToPublish, MySbMessageSerializer, MyServiceBusPublisherClient, PublishError};

pub struct MyServiceBusPublisherWithRetries<TContract> {
    pub topic_name: String,
    pub client: Arc<dyn MyServiceBusPublisherClient + Send + Sync + 'static>,
    pub serializer: Arc<dyn MySbMessageSerializer<TContract> + Send + Sync + 'static>,
    pub retires_count: usize,
    pub retries_delay: Duration,
}

impl<TContract> MyServiceBusPublisherWithRetries<TContract> {
    pub fn new(
        topic_name: String,
        client: Arc<dyn MyServiceBusPublisherClient + Send + Sync + 'static>,
        serializer: Arc<dyn MySbMessageSerializer<TContract> + Send + Sync + 'static>,
    ) -> Self {
        Self {
            topic_name,
            client,
            serializer,
            retires_count: 5,
            retries_delay: Duration::from_secs(5),
        }
    }

    pub async fn publish(&self, message: &TContract) -> Result<(), PublishError> {
        let content = self.serializer.serialize(message);

        if let Err(err) = content {
            return Err(PublishError::SerializationError(err));
        }

        let content = content.unwrap();

        self.client
            .publish_message_with_retries(
                &self.topic_name,
                MessageToPublish {
                    headers: None,
                    content,
                },
                self.retires_count,
                self.retries_delay,
            )
            .await
    }

    pub async fn publish_with_headers(
        &self,
        message: &TContract,
        headers: HashMap<String, String>,
    ) -> Result<(), PublishError> {
        let content = self.serializer.serialize(message);

        if let Err(err) = content {
            return Err(PublishError::SerializationError(err));
        }

        let content = content.unwrap();

        self.client
            .publish_message_with_retries(
                &self.topic_name,
                MessageToPublish {
                    headers: Some(headers),
                    content,
                },
                self.retires_count,
                self.retries_delay,
            )
            .await
    }

    pub async fn publish_messages(&self, messages: &[TContract]) -> Result<(), PublishError> {
        let mut messages_to_publish = Vec::with_capacity(messages.len());

        for message in messages {
            let content = self.serializer.serialize(message);

            if let Err(err) = content {
                return Err(PublishError::SerializationError(err));
            }

            let content = content.unwrap();

            messages_to_publish.push(MessageToPublish {
                headers: None,
                content,
            });
        }

        self.client
            .publish_messages_with_retries(
                &self.topic_name,
                messages_to_publish,
                self.retires_count,
                self.retries_delay,
            )
            .await
    }

    pub async fn publish_messages_with_header(
        &self,
        messages: Vec<(TContract, Option<HashMap<String, String>>)>,
    ) -> Result<(), PublishError> {
        let mut messages_to_publish = Vec::with_capacity(messages.len());

        for (contract, headers) in messages {
            let content = self.serializer.serialize(&contract);

            if let Err(err) = content {
                return Err(PublishError::SerializationError(err));
            }

            let content = content.unwrap();

            messages_to_publish.push(MessageToPublish { content, headers });
        }

        self.client
            .publish_messages_with_retries(
                &self.topic_name,
                messages_to_publish,
                self.retires_count,
                self.retries_delay,
            )
            .await
    }
}
