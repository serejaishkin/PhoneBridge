//! Общий протокол сообщений PC <-> Android поверх TLS-соединения.
//!
//! Формат: одна JSON-строка на сообщение, разделитель — '\n' (newline-delimited JSON).
//! Control-plane содержит только звонки и управление медиа. Аудиопоток сюда не входит.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Hello {
        device_id: String,
        device_name: String,
        platform: String,
        protocol_version: u32,
    },
    HelloAck {
        device_id: String,
        device_name: String,
        trusted: bool,
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
    /// PC -> Android: управление активной MediaSession.
    MediaCommand { command: MediaCommand },
    /// Android -> PC: состояние активной MediaSession.
    MediaState {
        package: Option<String>,
        state: MediaPlaybackState,
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
    },
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
