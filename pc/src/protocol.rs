//! Общий control-plane протокол PC <-> Android поверх TLS.
//! Формат: newline-delimited JSON.

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Hello { device_id: String, device_name: String, platform: String, protocol_version: u32, fingerprint: String },
    HelloAck { device_id: String, device_name: String, protocol_version: u32, trusted: bool, fingerprint: String },
    PairRequest { device_id: String, device_name: String, fingerprint: String },
    PairChallenge { device_id: String, fingerprint: String, short_code: String },
    PairConfirm { device_id: String, short_code: String },
    PairResult { device_id: String, trusted: bool, message: String },
    Ping,
    Pong,
    Disconnect { reason: String },
    IncomingCall { caller_number: Option<String>, caller_name: Option<String> },
    CallEnded,
    CallAnswer,
    CallDecline,
    PhoneBluetoothStatus { hfp_calls_toggle_enabled: bool },
    PcBluetoothStatus { hfp_supported: HfpSupport },
    Error { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HfpSupport { Supported, Unsupported, Unknown }

impl Message {
    pub fn to_line(&self) -> anyhow::Result<String> { let mut s = serde_json::to_string(self)?; s.push('\n'); Ok(s) }
    pub fn from_line(line: &str) -> anyhow::Result<Self> { Ok(serde_json::from_str(line.trim_end())?) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disconnect_is_wire_compatible() {
        let msg = Message::Disconnect { reason: "peer shutdown".into() };
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"Disconnect","data":{"reason":"peer shutdown"}}"#);
        assert!(matches!(Message::from_line(&json).unwrap(), Message::Disconnect { .. }));
    }
    #[test]
    fn empty_messages_have_no_data_field() { assert_eq!(serde_json::to_string(&Message::Ping).unwrap(), r#"{"type":"Ping"}"#); }
}
