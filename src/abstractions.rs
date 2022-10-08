use crate::{publisher::MessageToPublish, subscriber::MessagesReader, PublishError};

pub type MessageId = i64;

#[async_trait::async_trait]
pub trait MyServiceBusPublisherClient {
    async fn publish_message(
        &self,
        topic_id: &str,
        message: MessageToPublish,
        do_retry: bool,
    ) -> Result<(), PublishError>;

    async fn publish_messages(
        &self,
        topic_id: &str,
        message: Vec<MessageToPublish>,
        do_retry: bool,
    ) -> Result<(), PublishError>;
}

pub trait MyServiceBusSubscriberClient {
    fn confirm_delivery(
        &self,
        topic_id: &str,
        queue_id: &str,
        confirmation_id: i64,
        delivered: bool,
    );

    fn confirm_some_messages_ok(
        &self,
        topic_id: &str,
        queue_id: &str,
        confirmation_id: i64,
        ok_messages: crate::queue_with_intervals::QueueWithIntervals,
    );
}

#[async_trait::async_trait]
pub trait MySbSubscriberCallback {
    async fn new_events(&self, messages_reader: MessagesReader);
}
