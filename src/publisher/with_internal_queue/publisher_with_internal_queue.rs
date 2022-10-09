use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

use crate::{MyServiceBusPublisherClient, PublishError};

use super::{
    super::{MessageToPublish, MySbMessageSerializer},
    PublisherWithInternalQueueData, QueueToPublish,
};

pub struct PublisherWithInternalQueue<TMessageModel: MySbMessageSerializer> {
    data: Arc<PublisherWithInternalQueueData>,
    event_sender: UnboundedSender<()>,
    pub item: Option<TMessageModel>,
}

impl<TMessageModel: MySbMessageSerializer> PublisherWithInternalQueue<TMessageModel> {
    pub async fn new(
        topic_id: String,
        client: Arc<dyn MyServiceBusPublisherClient + Send + Sync + 'static>,
        logger: Arc<dyn rust_extensions::Logger + Send + Sync + 'static>,
    ) -> Self {
        let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();

        let data = PublisherWithInternalQueueData {
            client,
            topic_id,
            queue_to_publish: Mutex::new(QueueToPublish::new()),
            logger,
        };

        let result = Self {
            event_sender,
            data: Arc::new(data),
            item: None,
        };

        let data = result.data.clone();
        tokio::spawn(events_publisher(data, event_receiver));

        result
    }

    pub async fn publish_and_forget(&self, message: TMessageModel) -> Result<(), PublishError> {
        let result = message.serialize(None);

        if let Err(err) = result {
            return Err(PublishError::SerializationError(err));
        }

        let (content, headers) = result.unwrap();

        let mut write_access = self.data.queue_to_publish.lock().await;
        write_access
            .queue
            .push_back(MessageToPublish { headers, content });

        if let Err(err) = self.event_sender.send(()) {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.data.topic_id.to_string());
            self.data.logger.write_error(
                "publish_and_forget".to_string(),
                format!("Can not publish message. Err: {}", err),
                Some(ctx),
            )
        }

        Ok(())
    }

    pub async fn get_queue_size(&self) -> usize {
        let read_access = self.data.queue_to_publish.lock().await;
        read_access.queue.len() + read_access.being_published
    }
}

async fn events_publisher(
    data: Arc<PublisherWithInternalQueueData>,
    mut event_receiver: UnboundedReceiver<()>,
) {
    let mut to_publish = None;
    loop {
        tokio::sync::mpsc::UnboundedReceiver::recv(&mut event_receiver).await;

        if to_publish.is_none() {
            to_publish = data.get_messages_to_publish().await;
        }

        if to_publish.is_none() {
            return;
        }

        if data.publish(to_publish.as_ref().unwrap()).await {
            data.messages_are_published().await;
            to_publish = None;
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }
}
