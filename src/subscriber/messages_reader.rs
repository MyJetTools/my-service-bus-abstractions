use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use rust_extensions::Logger;

use crate::{
    queue_with_intervals::QueueWithIntervals,
    subscriber::{MySbDeliveredMessage, MySbMessageDeserializer},
    MyServiceBusSubscriberClient,
};

use super::MySbTypedDeliveredMessage;

pub struct MessagesReader {
    total_messages_amount: i64,

    pub topic_id: String,
    pub queue_id: String,
    messages: VecDeque<MySbDeliveredMessage>,
    pub confirmation_id: i64,
    delivered: QueueWithIntervals,
    subscriber_client: Arc<dyn MyServiceBusSubscriberClient + Send + Sync + 'static>,

    logger: Arc<dyn Logger + Send + Sync + 'static>,
}

impl MessagesReader {
    pub fn new(
        topic_id: String,
        queue_id: String,
        messages: VecDeque<MySbDeliveredMessage>,
        confirmation_id: i64,
        subscriber_client: Arc<dyn MyServiceBusSubscriberClient + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        let total_messages_amount = messages.len() as i64;
        Self {
            topic_id,
            queue_id,
            messages,
            confirmation_id,
            subscriber_client,
            delivered: QueueWithIntervals::new(),
            total_messages_amount,
            logger,
        }
    }

    pub fn handled_ok(&mut self, msg: &MySbDeliveredMessage) {
        self.delivered.enqueue(msg.id);
    }

    pub fn get_next_message<TContract: MySbMessageDeserializer<Item = TContract>>(
        &mut self,
    ) -> Option<MySbTypedDeliveredMessage<TContract>> {
        loop {
            let next_message = self.messages.pop_front()?;

            match TContract::deserialize(&next_message.content, &next_message.headers) {
                Ok(content) => {
                    let result = MySbTypedDeliveredMessage {
                        id: next_message.id,
                        attempt_no: next_message.attempt_no,
                        headers: next_message.headers,
                        content,
                        raw: next_message.content,
                    };

                    return Some(result);
                }
                Err(err) => {
                    let mut ctx = HashMap::new();

                    ctx.insert("topicId".to_string(), self.topic_id.clone());
                    ctx.insert("queueId".to_string(), self.queue_id.clone());
                    ctx.insert("messageId".to_string(), next_message.id.to_string());
                    ctx.insert("attemptNo".to_string(), next_message.attempt_no.to_string());

                    self.logger.write_fatal_error(
                        "get_next_message".to_string(),
                        format!("Can not deserialize message. Err: {:?}", err),
                        Some(ctx),
                    )
                }
            }
        }
    }
}

impl Drop for MessagesReader {
    fn drop(&mut self) {
        if self.delivered.len() == self.total_messages_amount {
            self.subscriber_client.confirm_delivery(
                self.topic_id.as_str(),
                self.queue_id.as_str(),
                self.confirmation_id,
                true,
            );
        } else if self.delivered.len() == 0 {
            let mut log_context = HashMap::new();
            log_context.insert(
                "ConfirmationId".to_string(),
                self.confirmation_id.to_string(),
            );

            log_context.insert("TopicId".to_string(), self.topic_id.to_string());
            log_context.insert("QueueId".to_string(), self.queue_id.to_string());

            self.logger.write_error(
                "Sending delivery confirmation".to_string(),
                "All messages confirmed as fail".to_string(),
                Some(log_context),
            );

            self.subscriber_client.confirm_delivery(
                self.topic_id.as_str(),
                self.queue_id.as_str(),
                self.confirmation_id,
                false,
            );
        } else {
            let mut log_context = HashMap::new();
            log_context.insert(
                "ConfirmationId".to_string(),
                self.confirmation_id.to_string(),
            );

            log_context.insert("TopicId".to_string(), self.topic_id.to_string());
            log_context.insert("QueueId".to_string(), self.queue_id.to_string());

            self.logger.write_error(
                "Sending delivery confirmation".to_string(),
                format!(
                    "{} messages out of {} confirmed as Delivered",
                    self.delivered.len(),
                    self.total_messages_amount
                ),
                Some(log_context),
            );
            self.subscriber_client.confirm_some_messages_ok(
                self.topic_id.as_str(),
                self.queue_id.as_str(),
                self.confirmation_id,
                self.delivered.clone(),
            );
        };
    }
}
