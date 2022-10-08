use std::collections::HashMap;

use crate::MessageId;

#[derive(Debug, Clone)]
pub struct MySbMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: Option<HashMap<String, String>>,
    pub content: Vec<u8>,
}
