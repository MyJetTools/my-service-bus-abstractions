use std::collections::HashMap;

use crate::MessageId;

use super::MySbMessageDeserializer;

pub struct MySbDeliveredMessage<TContract: MySbMessageDeserializer<Item = TContract>> {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: Option<HashMap<String, String>>,
    pub content: TContract,
    pub raw: Vec<u8>,
}
