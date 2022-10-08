pub trait MySbMessageSerializer {
    fn serialize(&self) -> Result<Vec<u8>, String>;
}
