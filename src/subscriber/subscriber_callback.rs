use super::{MessagesReader, MySbMessageDeserializer};

#[async_trait::async_trait]
pub trait SubscriberCallback<
    TMessageModel: MySbMessageDeserializer<Item = TMessageModel> + Send + Sync + 'static,
>
{
    async fn handle_messages(&self, messages_reader: MessagesReader<TMessageModel>);
}
