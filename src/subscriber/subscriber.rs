use super::{deserializer::MySbMessageDeserializer, TopicQueueType};

pub struct MySbSubscriber<TContract: MySbMessageDeserializer<Item = TContract>> {
    pub topic_name: String,
    pub queue_name: String,
    pub queue_type: TopicQueueType,
    pub contract: Option<TContract>,
}

impl<TContract: MySbMessageDeserializer<Item = TContract>> MySbSubscriber<TContract> {
    pub fn new(topic_name: String, queue_name: String, queue_type: TopicQueueType) -> Self {
        Self {
            topic_name,
            queue_name,
            queue_type,
            contract: None,
        }
    }
}
