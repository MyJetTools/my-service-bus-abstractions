use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use rust_extensions::Logger;

use crate::{
    queue_with_intervals::QueueWithIntervals, MyServiceBusSubscriberClient,
    MyServiceBusSubscriberClientCallback,
};

use super::{
    MessagesReader, MySbDeliveredMessage, MySbMessageDeserializer, MySbMessageToDeliver,
    TopicQueueType,
};

#[async_trait::async_trait]
pub trait MySbCallback<TContract: MySbMessageDeserializer<Item = TContract> + Send + Sync + 'static>
{
    async fn handle_messages(&self, messages_reader: MessagesReader<TContract>);
}

pub struct SubscriberData {
    pub topic_id: String,
    pub queue_id: String,
    pub queue_type: TopicQueueType,
    pub logger: Arc<dyn Logger + Sync + Send + 'static>,
    pub client: Arc<dyn MyServiceBusSubscriberClient + Sync + Send + 'static>,
}

pub struct Subscriber<TContract: MySbMessageDeserializer<Item = TContract>> {
    data: Arc<SubscriberData>,
    pub callback: Arc<dyn MySbCallback<TContract> + Sync + Send + 'static>,
}

impl<TContract: MySbMessageDeserializer<Item = TContract> + Send + Sync + 'static>
    Subscriber<TContract>
{
    pub fn new(
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
        callback: Arc<dyn MySbCallback<TContract> + Sync + Send + 'static>,
        logger: Arc<dyn Logger + Sync + Send + 'static>,
        client: Arc<dyn MyServiceBusSubscriberClient + Sync + Send + 'static>,
    ) -> Self {
        let data = SubscriberData {
            topic_id,
            queue_id,
            queue_type,
            client,
            logger,
        };
        Self {
            callback,
            data: Arc::new(data),
        }
    }
}

#[async_trait::async_trait]
impl<TContract: MySbMessageDeserializer<Item = TContract> + Send + Sync + 'static>
    MyServiceBusSubscriberClientCallback for Subscriber<TContract>
{
    fn get_topic_id(&self) -> &str {
        self.data.topic_id.as_str()
    }

    fn get_queue_id(&self) -> &str {
        self.data.queue_id.as_str()
    }
    fn get_queue_type(&self) -> TopicQueueType {
        self.data.queue_type
    }

    async fn new_events(
        &self,
        messages_to_deliver: Vec<MySbMessageToDeliver>,
        confirmation_id: i64,
    ) {
        let mut messages = VecDeque::with_capacity(messages_to_deliver.len());

        let mut can_not_serialize_messages = QueueWithIntervals::new();

        let mut deserialize_error = None;

        for msg in messages_to_deliver {
            let content_result = TContract::deserialize(&msg.content, &msg.headers);

            match content_result {
                Ok(contract) => {
                    let msg = MySbDeliveredMessage {
                        id: msg.id,
                        attempt_no: msg.attempt_no,
                        headers: msg.headers,
                        content: contract,
                        raw: msg.content,
                    };

                    messages.push_back(msg);
                }
                Err(err) => {
                    if deserialize_error.is_none() {
                        deserialize_error = Some(format!(
                            "Can not deserialize one of the messages. Err:{:?}",
                            err
                        ));
                    }
                    can_not_serialize_messages.enqueue(msg.id);
                }
            }
        }

        if messages.len() == 0 {
            self.data.client.confirm_delivery(
                &self.data.topic_id,
                &self.data.queue_id,
                confirmation_id,
                true,
            );

            let mut ctx = HashMap::new();

            ctx.insert("topicId".to_string(), self.data.topic_id.to_string());
            ctx.insert("queueId".to_string(), self.data.queue_id.to_string());
            ctx.insert(
                "messages".to_string(),
                format!("{:?}", can_not_serialize_messages),
            );
            ctx.insert("confirmationId".to_string(), confirmation_id.to_string());

            self.data.logger.write_fatal_error(
                "new_events".to_string(),
                format!("Can not serialize messages"),
                Some(ctx),
            );
            return;
        }

        let reader = MessagesReader::new(self.data.clone(), messages, confirmation_id);

        let callback = self.callback.clone();
        tokio::spawn(async move {
            callback.handle_messages(reader).await;
        });
    }
}
