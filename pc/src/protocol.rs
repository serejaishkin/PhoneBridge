//! Общий control-plane протокол PC <-> Android поверх TLS.
//!
//! Формат: newline-delimited JSON. Все варианты с payload используют
//! {"type":"Variant","data":{...}}, а варианты без payload — {"type":"Ping"}.

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Hello {
        device_id: String,
        device_name: String,
        /// "android" | "windows" | "macos" | "linux"
        platform: String,
        protocol_version: u32,
        /// SHA-256 fingerprint of the peer's persistent identity certificate.
        fingerprint: String,
    },

    HelloAck {
        device_id: String,
        device_name: String,
        protocol_version: u32,
        trusted: bool,
        fingerprint: String,
    },

    /// Sent by an untrusted peer after Hello. The fingerprint is the identity
    /// fingerprint of the device, not the TLS transport certificate of the PC.
    PairRequest {
        device_id: String,
        device_name: String,
        fingerprint: String,
    },

    /// PC tells Android which human-readable code must be confirmed by the user.
    PairChallenge {
        device_id: String,
        fingerprint: String,
        short_code: String,
    },

    /// Explicit user confirmation. The code must match the challenge currently
    /// associated with this connection.
    PairConfirm {
        device_id: String,
        short_code: String,
    },

    PairResult {
        device_id: String,
        trusted: bool,
        message: String,
    },

    Ping,
    Pong,

    IncomingCall {
        caller_number: Option<String>,
        caller_name: Option<String>,
    },
    CallEnded,
    CallAnswer,
    CallDecline,

    PhoneBluetoothStatus { hfp_calls_toggle_enabled: bool },
    PcBluetoothStatus { hfp_supported: HfpSupport },

    Error { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HfpSupport {
    Supported,
    Unsupported,
    Unknown,
}

impl Message {
    pub fn to_line(&self) -> anyhow::Result<String> {
        let mut s = serde_json::to_string(self)?;
        s.push('\n');
        Ok(s)
    }

    pub fn from_line(line: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(line.trim_end())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_messages_use_expected_wire_format() {
        let msg = Message::PairChallenge {
            device_id: "pb2-phone".into(),
            fingerprint: "AABBCCDD".into(),
            short_code: "AABB-CCDD".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"PairChallenge","data":{"device_id":"pb2-phone","fingerprint":"AABBCCDD","short_code":"AABB-CCDD"}}"#);
        assert!(matches!(Message::from_line(&json).unwrap(), Message::PairChallenge { .. }));
    }

    #[test]
    fn empty_messages_have_no_data_field() {
        assert_eq!(serde_json::to_string(&Message::Ping).unwrap(), r#"{"type":"Ping"}"#);
    }
}
