use lib::utils::{parse_multipart, AnalogRead, Celsius, DeviceTimestamp, MultipartDeserialize,
                 Percentage, UID};

#[derive(Debug, Deserialize)]
pub struct EventForm {
    pub pid: UID,
    pub at: Celsius,
    pub ah: Percentage,
    pub st: Celsius,
    pub sr: AnalogRead,
    pub l: AnalogRead,
    pub t: DeviceTimestamp,
}

impl MultipartDeserialize for EventForm {
    fn from_multipart(content: &[u8], boundary: &[u8]) -> Option<Self> {
        let values = parse_multipart(content, boundary);
        Some(EventForm {
            pid: try_get_num!(values, pid, UID),
            at: try_get_num!(values, at, Celsius),
            ah: try_get_num!(values, ah, Percentage),
            st: try_get_num!(values, st, Celsius),
            sr: try_get_num!(values, sr, AnalogRead),
            l: try_get_num!(values, l, AnalogRead),
            t: try_get_num!(values, t, DeviceTimestamp),
        })
    }
}
