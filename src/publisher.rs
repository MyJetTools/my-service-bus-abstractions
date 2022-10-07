use std::{collections::HashMap, sync::Arc};

use crate::{MessageToPublish, MySbMessageSerializer, MyServiceBusPublisherClient, PublishError};

pub struct MyServiceBusPublisher<TContract> {
    pub topic_name: String,
    pub client: Arc<dyn MyServiceBusPublisherClient + Send + Sync + 'static>,
    pub serializer: Arc<dyn MySbMessageSerializer<TContract> + Send + Sync + 'static>,
    pub do_retries: bool,
}

impl<TContract> MyServiceBusPublisher<TContract> {
    pub fn new(
        topic_name: String,
        client: Arc<dyn MyServiceBusPublisherClient + Send + Sync + 'static>,
        serializer: Arc<dyn MySbMessageSerializer<TContract> + Send + Sync + 'static>,
        do_retries: bool,
    ) -> Self {
        Self {
            topic_name,
            client,
            serializer,
            do_retries,
        }
    }

    pub async fn publish(&self, message: &TContract) -> Result<(), PublishError> {
        let content = self.serializer.serialize(message);

        if let Err(err) = content {
            return Err(PublishError::SerializationError(err));
        }

        let content = content.unwrap();

        self.client
            .publish_message(
                &self.topic_name,
                MessageToPublish {
                    headers: None,
                    content,
                },
                self.do_retries,
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
            .publish_message(
                &self.topic_name,
                MessageToPublish {
                    headers: Some(headers),
                    content,
                },
                self.do_retries,
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
            .publish_messages(&self.topic_name, messages_to_publish, self.do_retries)
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
            .publish_messages(&self.topic_name, messages_to_publish, self.do_retries)
            .await
    }
}
