#[cfg(feature = "with-telemetry")]
use my_telemetry::{EventDurationTracker, MyTelemetryContext};
use std::collections::HashMap;

use crate::MessageId;

use super::MySbMessageDeserializer;

pub struct MySbDeliveredMessage<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: Option<HashMap<String, String>>,
    pub content: TMessageModel,
    pub raw: Vec<u8>,
    #[cfg(feature = "with-telemetry")]
    pub my_telemetry_ctx: Option<MyTelemetryContext>,
    #[cfg(feature = "with-telemetry")]
    pub event_tracker: Option<EventDurationTracker>,
}

#[cfg(feature = "with-telemetry")]
impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>>
    MySbDeliveredMessage<TMessageModel>
{
    pub fn init_telemetry_context(&mut self, topic_id: &str, queue_id: &str) {
        use crate::MY_TELEMETRY_HEADER;

        if let Some(headers) = self.headers.as_ref() {
            if let Some(telemetry_value) = headers.get(MY_TELEMETRY_HEADER) {
                if let Ok(result) = telemetry_value.parse() {
                    let my_telemery = MyTelemetryContext::restore(result);
                    let event_duration_tracker = my_telemery.start_event_tracking(format!(
                        "Handling event {}/{}. MsgId: {}",
                        topic_id, queue_id, self.id
                    ));
                    self.my_telemetry_ctx = Some(my_telemery);
                    self.event_tracker = Some(event_duration_tracker)
                }
            }
        }
    }
}
