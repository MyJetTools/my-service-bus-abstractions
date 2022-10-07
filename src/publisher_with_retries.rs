use std::{collections::HashMap, time::Duration};

use crate::{MessageToPublish, MyServiceBusPublisher, PublishError};

pub struct MyServiceBusPublisherWithRetries<TContract> {
    publisher: MyServiceBusPublisher<TContract>,
    pub retires_count: usize,
    pub retries_delay: Duration,
}

impl<TContract> MyServiceBusPublisherWithRetries<TContract> {
    pub fn new(publisher: MyServiceBusPublisher<TContract>) -> Self {
        Self {
            publisher,
            retires_count: 5,
            retries_delay: Duration::from_secs(5),
        }
    }

    pub async fn publish(&self, message: &TContract) -> Result<(), PublishError> {
        let mut attempt_no = 0;

        loop {
            let result = self.publisher.publish(message).await;

            if result.is_ok() {
                return result;
            }

            attempt_no += 1;

            if attempt_no >= self.retires_count {
                return result;
            }

            self.handle_error(result.unwrap_err()).await?;
        }
    }

    pub async fn publish_with_headers(
        &self,
        message: &TContract,
        headers: HashMap<String, String>,
    ) -> Result<(), PublishError> {
        let mut attempt_no = 0;

        loop {
            let result = self
                .publisher
                .publish_with_headers(message, headers.clone())
                .await;

            if result.is_ok() {
                return result;
            }

            attempt_no += 1;

            if attempt_no >= self.retires_count {
                return result;
            }

            self.handle_error(result.unwrap_err()).await?;
        }
    }

    pub async fn publish_messages(&self, messages: &[TContract]) -> Result<(), PublishError> {
        let mut attempt_no = 0;

        loop {
            let result = self.publisher.publish_messages(messages).await;

            if result.is_ok() {
                return result;
            }

            attempt_no += 1;

            if attempt_no >= self.retires_count {
                return result;
            }

            self.handle_error(result.unwrap_err()).await?;
        }
    }

    pub async fn publish_messages_with_header(
        &self,
        messages: &[(TContract, Option<HashMap<String, String>>)],
    ) -> Result<(), PublishError> {
        let mut attempt_no = 0;

        let mut messages_to_publish = Vec::with_capacity(messages.len());

        for (contract, headers) in messages {
            let content = self.publisher.serializer.serialize(contract);

            if let Err(err) = content {
                return Err(PublishError::SerializationError(err));
            }

            let content = content.unwrap();

            messages_to_publish.push(MessageToPublish {
                headers: headers.clone(),
                content,
            });
        }

        loop {
            let result = self
                .publisher
                .client
                .publish_messages(self.publisher.topic_name.as_str(), &messages_to_publish)
                .await;

            if result.is_ok() {
                return result;
            }

            attempt_no += 1;

            if attempt_no >= self.retires_count {
                return result;
            }

            self.handle_error(result.unwrap_err()).await?;
        }
    }

    async fn handle_error(&self, err: PublishError) -> Result<(), PublishError> {
        match err {
            PublishError::NoConnectionToPublish => {}
            PublishError::SerializationError(err) => {
                return Err(PublishError::SerializationError(err));
            }
            PublishError::Disconnected => {}
            PublishError::Other(_) => {}
        }

        tokio::time::sleep(self.retries_delay).await;

        Ok(())
    }
}
