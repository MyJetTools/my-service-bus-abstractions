use std::collections::HashMap;

pub struct MessageToPublishContent {
    pub headers: Option<HashMap<String, String>>,
    pub content: Vec<u8>,
}

pub trait MessageToPublish {
    fn get_content(self) -> MessageToPublishContent;
}
