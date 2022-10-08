use std::collections::HashMap;

pub trait MySbMessageDeserializer {
    type Item;
    fn deserialize(
        src: &[u8],
        headers: &Option<HashMap<String, String>>,
    ) -> Result<Self::Item, String>;
}
