//! Common control-plane protocol spoken between PC and Android.
//!
//! Transport: TLS stream carrying newline-delimited JSON (one message per line).
//! The control plane covers pairing, calls, media state and SMS. Raw audio is out of scope here.
//!
//! Trust model: both sides own a persistent self-signed certificate. The SHA-256
//! fingerprint of that certificate travels inside Hello/HelloAck so each side can
//! pin and verify the peer (see pc/src/pairing/trust.rs and the Android TrustStore).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Hello {
        device_id: String,
        device_name: String,
        platform: String,
        protocol_version: u32,
        /// SHA-256 hex fingerprint of the sender certificate. Optional on the wire so
        /// older builds still parse, but the PC rejects sessions without it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cert_fingerprint: Option<String>,
    },
    HelloAck {
        device_id: String,
        device_name: String,
        trusted: bool,
        /// SHA-256 hex fingerprint of the PC certificate; lets the phone pin the PC.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        cert_fingerprint: String,
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
    /// PC -> Android: hang up an active call (distinct from CallDecline which
    /// rejects an incoming ringing call).
    #[serde(rename = "call_end")]
    CallEnd,
    /// Android -> PC: call was accepted and is now active.
    #[serde(rename = "call_active")]
    CallActive,
    /// PC -> Android: control command for the active MediaSession.
    MediaCommand { command: MediaCommand },
    /// Android -> PC: state of the active MediaSession.
    MediaState {
        package: Option<String>,
        state: MediaPlaybackState,
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
    },
    /// Android -> PC: incoming SMS.
    #[serde(rename = "sms_received")]
    SmsReceived {
        address: String,
        body: String,
        timestamp: i64,
    },
    /// PC -> Android: send an SMS.
    #[serde(rename = "sms_send")]
    SmsSend {
        address: String,
        body: String,
    },
    /// PC -> Android: request recent incoming SMS history.
    #[serde(rename = "sms_list")]
    SmsList,
    /// Android -> PC: one item of SMS history.
    #[serde(rename = "sms_item")]
    SmsItem {
        id: String,
        address: String,
        body: String,
        timestamp: i64,
    },
    /// Android -> PC: end of sms_list response.
    #[serde(rename = "sms_list_end")]
    SmsListEnd { count: u32 },
    #[serde(rename = "sms_sent")]
    SmsSent { address: String, body: String },
    #[serde(rename = "sms_error")]
    SmsError { error: String },
    /// Android -> PC: start relaying the PC microphone to the phone (:5003).
    #[serde(rename = "mic_start")]
    MicStart,
    /// Android -> PC: stop the microphone relay.
    #[serde(rename = "mic_stop")]
    MicStop,
    PhoneBluetoothStatus { hfp_calls_toggle_enabled: bool },
    PcBluetoothStatus { hfp_supported: HfpSupport },
    Error { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaCommand {
    Play,
    Pause,
    PlayPause,
    Next,
    Previous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaPlaybackState {
    Playing,
    Paused,
    Buffering,
    None,
    Unknown,
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
    fn sms_send_round_trips() {
        let message = Message::SmsSend {
            address: "+79991234567".into(),
            body: "hello".into(),
        };
        let line = message.to_line().unwrap();
        assert_eq!(
            line,
            "{\"type\":\"sms_send\",\"data\":{\"address\":\"+79991234567\",\"body\":\"hello\"}}\n"
        );
        let decoded = Message::from_line(&line).unwrap();
        match decoded {
            Message::SmsSend { address, body } => {
                assert_eq!(address, "+79991234567");
                assert_eq!(body, "hello");
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn hello_round_trips_with_fingerprint() {
        let message = Message::Hello {
            device_id: "pb2-abc".into(),
            device_name: "Pixel".into(),
            platform: "android".into(),
            protocol_version: 1,
            cert_fingerprint: Some("deadbeef".into()),
        };
        let line = message.to_line().unwrap();
        let decoded = Message::from_line(&line).unwrap();
        match decoded {
            Message::Hello { cert_fingerprint, .. } => {
                assert_eq!(cert_fingerprint.as_deref(), Some("deadbeef"));
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn hello_without_fingerprint_still_parses() {
        let line = "{\"type\":\"Hello\",\"data\":{\"device_id\":\"d\",\"device_name\":\"n\",\"platform\":\"android\",\"protocol_version\":1}}\n";
        let decoded = Message::from_line(line).unwrap();
        match decoded {
            Message::Hello { cert_fingerprint, .. } => assert!(cert_fingerprint.is_none()),
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn hello_ack_carries_pc_fingerprint() {
        let message = Message::HelloAck {
            device_id: "pb2-pc".into(),
            device_name: "DESKTOP".into(),
            trusted: true,
            cert_fingerprint: "cafebabe".into(),
        };
        let line = message.to_line().unwrap();
        assert!(line.contains("\"cert_fingerprint\":\"cafebabe\""));
        let decoded = Message::from_line(&line).unwrap();
        match decoded {
            Message::HelloAck { trusted, cert_fingerprint, .. } => {
                assert!(trusted);
                assert_eq!(cert_fingerprint, "cafebabe");
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }
}
