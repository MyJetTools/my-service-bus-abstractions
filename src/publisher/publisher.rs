use std::{collections::HashMap, sync::Arc};

use rust_extensions::Logger;

use crate::{MyServiceBusPublisherClient, PublishError};

use super::{MessageToPublish, MySbMessageSerializer};

pub struct MyServiceBusPublisher<TContract: MySbMessageSerializer> {
    pub topic_id: String,
    pub client: Arc<dyn MyServiceBusPublisherClient + Send + Sync + 'static>,
    pub do_retries: bool,
    pub itm: Option<TContract>,
    pub logger: Arc<dyn Logger + Send + Sync + 'static>,
}

impl<TContract: MySbMessageSerializer> MyServiceBusPublisher<TContract> {
    pub fn new(
        topic_id: String,
        client: Arc<dyn MyServiceBusPublisherClient + Send + Sync + 'static>,
        do_retries: bool,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        Self {
            topic_id,
            client,
            do_retries,
            logger,
            itm: None,
        }
    }

    pub async fn publish(&self, message: &TContract) -> Result<(), PublishError> {
        let content = message.serialize(None);

        if let Err(err) = content {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger
                .write_fatal_error("publish".to_string(), err.clone(), Some(ctx));
            return Err(PublishError::SerializationError(err));
        }

        let (content, headers) = content.unwrap();

        let result = self
            .client
            .publish_message(
                &self.topic_id,
                MessageToPublish { headers, content },
                self.do_retries,
            )
            .await;

        if let Err(err) = &result {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger.write_error(
                "publish".to_string(),
                format!("Can not publish message. Error: {:?}", err),
                Some(ctx),
            );
        }

        result
    }

    pub async fn publish_with_headers(
        &self,
        message: &TContract,
        headers: HashMap<String, String>,
    ) -> Result<(), PublishError> {
        let content = message.serialize(Some(headers));

        if let Err(err) = content {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger.write_fatal_error(
                "publish_with_headers".to_string(),
                err.clone(),
                Some(ctx),
            );

            return Err(PublishError::SerializationError(err));
        }

        let (content, headers) = content.unwrap();

        let result = self
            .client
            .publish_message(
                &self.topic_id,
                MessageToPublish { headers, content },
                self.do_retries,
            )
            .await;

        if let Err(err) = &result {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger.write_error(
                "publish_messages".to_string(),
                format!("Can not publish message. Error: {:?}", err),
                Some(ctx),
            );
        }

        result
    }

    pub async fn publish_messages(&self, messages: &[TContract]) -> Result<(), PublishError> {
        let mut messages_to_publish = Vec::with_capacity(messages.len());

        for message in messages {
            let content = message.serialize(None);

            if let Err(err) = content {
                let mut ctx = HashMap::new();
                ctx.insert("topicId".to_string(), self.topic_id.to_string());
                self.logger.write_fatal_error(
                    "publish_messages".to_string(),
                    err.clone(),
                    Some(ctx),
                );

                return Err(PublishError::SerializationError(err));
            }

            let (content, headers) = content.unwrap();

            messages_to_publish.push(MessageToPublish { headers, content });
        }

        let result = self
            .client
            .publish_messages(&self.topic_id, messages_to_publish, self.do_retries)
            .await;

        if let Err(err) = &result {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger.write_error(
                "publish_messages".to_string(),
                format!("Can not publish message. Error: {:?}", err),
                Some(ctx),
            );
        }

        result
    }

    pub async fn publish_messages_with_header(
        &self,
        messages: Vec<(TContract, Option<HashMap<String, String>>)>,
    ) -> Result<(), PublishError> {
        let mut messages_to_publish = Vec::with_capacity(messages.len());

        for (contract, headers) in messages {
            let content = contract.serialize(headers);

            if let Err(err) = content {
                let mut ctx = HashMap::new();
                ctx.insert("topicId".to_string(), self.topic_id.to_string());
                self.logger.write_fatal_error(
                    "publish_messages_with_header".to_string(),
                    err.clone(),
                    Some(ctx),
                );

                return Err(PublishError::SerializationError(err));
            }

            let (content, headers) = content.unwrap();

            messages_to_publish.push(MessageToPublish { content, headers });
        }

        let result = self
            .client
            .publish_messages(&self.topic_id, messages_to_publish, self.do_retries)
            .await;

        if let Err(err) = &result {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger.write_error(
                "publish_messages_with_header".to_string(),
                format!("Can not publish message. Error: {:?}", err),
                Some(ctx),
            );
        }

        result
    }
}
