#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageId(i64);

impl MessageId {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn from(value: Option<i64>) -> Option<Self> {
        let value = value?;
        Some(Self(value))
    }

    pub fn get_value(&self) -> i64 {
        self.0
    }
}

impl AsRef<i64> for MessageId {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl Into<MessageId> for i64 {
    fn into(self) -> MessageId {
        MessageId::new(self)
    }
}

impl Into<i64> for MessageId {
    fn into(self) -> i64 {
        self.0
    }
}
