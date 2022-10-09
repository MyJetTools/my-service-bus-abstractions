use std::collections::HashMap;

use crate::MessageId;

use super::MySbMessageDeserializer;

pub struct MySbDeliveredMessage<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: Option<HashMap<String, String>>,
    pub content: TMessageModel,
    pub raw: Vec<u8>,
}
