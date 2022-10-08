use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use crate::{
    queue_with_intervals::QueueWithIntervals,
    subscriber::{MySbDeliveredMessage, MySbMessageDeserializer},
};

use super::SubscriberData;

pub struct MessagesReader<TContract: MySbMessageDeserializer<Item = TContract>> {
    data: Arc<SubscriberData>,
    total_messages_amount: i64,
    messages: VecDeque<MySbDeliveredMessage<TContract>>,
    pub confirmation_id: i64,
    delivered: QueueWithIntervals,
    connection_id: i32,
}

impl<TContract: MySbMessageDeserializer<Item = TContract>> MessagesReader<TContract> {
    pub fn new(
        data: Arc<SubscriberData>,
        messages: VecDeque<MySbDeliveredMessage<TContract>>,
        confirmation_id: i64,
        connection_id: i32,
    ) -> Self {
        let total_messages_amount = messages.len() as i64;
        Self {
            data,
            messages,
            confirmation_id,
            delivered: QueueWithIntervals::new(),
            total_messages_amount,
            connection_id,
        }
    }

    pub fn handled_ok(&mut self, msg: &MySbDeliveredMessage<TContract>) {
        self.delivered.enqueue(msg.id);
    }

    pub fn get_next_message(&mut self) -> Option<MySbDeliveredMessage<TContract>> {
        self.messages.pop_front()
    }
}

impl<TContract: MySbMessageDeserializer<Item = TContract>> Drop for MessagesReader<TContract> {
    fn drop(&mut self) {
        if self.delivered.len() == self.total_messages_amount {
            self.data.client.confirm_delivery(
                self.data.topic_id.as_str(),
                self.data.queue_id.as_str(),
                self.confirmation_id,
                self.connection_id,
                true,
            );
        } else if self.delivered.len() == 0 {
            let mut log_context = HashMap::new();
            log_context.insert(
                "ConfirmationId".to_string(),
                self.confirmation_id.to_string(),
            );

            log_context.insert("TopicId".to_string(), self.data.topic_id.to_string());
            log_context.insert("QueueId".to_string(), self.data.queue_id.to_string());

            self.data.logger.write_error(
                "Sending delivery confirmation".to_string(),
                "All messages confirmed as fail".to_string(),
                Some(log_context),
            );

            self.data.client.confirm_delivery(
                self.data.topic_id.as_str(),
                self.data.queue_id.as_str(),
                self.confirmation_id,
                self.connection_id,
                false,
            );
        } else {
            let mut log_context = HashMap::new();
            log_context.insert(
                "ConfirmationId".to_string(),
                self.confirmation_id.to_string(),
            );

            log_context.insert("TopicId".to_string(), self.data.topic_id.to_string());
            log_context.insert("QueueId".to_string(), self.data.queue_id.to_string());

            self.data.logger.write_error(
                "Sending delivery confirmation".to_string(),
                format!(
                    "{} messages out of {} confirmed as Delivered",
                    self.delivered.len(),
                    self.total_messages_amount
                ),
                Some(log_context),
            );
            self.data.client.confirm_some_messages_ok(
                self.data.topic_id.as_str(),
                self.data.queue_id.as_str(),
                self.confirmation_id,
                self.connection_id,
                self.delivered.get_snapshot(),
            );
        };
    }
}
