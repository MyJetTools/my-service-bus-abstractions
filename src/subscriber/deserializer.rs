pub trait MySbMessageDeserializer {
    type Item;
    fn deserialize(src: &[u8]) -> Result<Self::Item, String>;
}
