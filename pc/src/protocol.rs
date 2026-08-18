//! Общий протокол сообщений PC <-> Android поверх TLS-соединения.
//!
//! Формат: одна JSON-строка на сообщение, разделитель — '\n' (newline-delimited JSON).
//! Это сознательно упрощённый вариант по сравнению с бинарным протоколом v1 —
//! на этапе pairing/control-plane важнее читаемость и лёгкость отладки, чем
//! производительность (аудио по-прежнему идёт отдельным UDP-потоком, сюда не входит).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    /// Первое сообщение после установления TLS-соединения: кто я такой.
    Hello {
        device_id: String,
        device_name: String,
        /// "android" | "windows" | "macos" | "linux"
        platform: String,
        /// Версия протокола, чтобы будущие несовместимые изменения не роняли соединение молча.
        protocol_version: u32,
    },

    /// Ответ на Hello: подтверждение + статус доверия (см. pairing::TrustStore).
    HelloAck {
        device_id: String,
        device_name: String,
        trusted: bool,
    },

    /// Keepalive в обе стороны.
    Ping,
    Pong,

    /// Android -> PC: входящий звонок. Аудио сюда НЕ входит — оно пойдёт через
    /// нативный Bluetooth HFP (см. AI_HANDOFF_GUI.md, раздел 1). Это только
    /// control-plane для отображения на ПК и кнопок Answer/Decline.
    IncomingCall {
        caller_number: Option<String>,
        caller_name: Option<String>,
    },

    /// Звонок завершился (принят где-то ещё / сброшен / положили трубку).
    CallEnded,

    /// PC -> Android: пользователь нажал "Answer" в трее/окне.
    CallAnswer,

    /// PC -> Android: пользователь нажал "Decline".
    CallDecline,

    /// Android -> PC: статус поддержки Bluetooth Classic HFP самим телефоном
    /// (обычно true) — полезно для диагностики на экране статуса.
    PhoneBluetoothStatus { hfp_calls_toggle_enabled: bool },

    /// PC -> Android: статус проверки HFP-клиента на стороне PC.
    /// См. pc/src/call/hfp_check.rs — реальная детекция pending, это заготовка API.
    PcBluetoothStatus { hfp_supported: HfpSupport },

    /// Общая ошибка протокола / отказ.
    Error { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HfpSupport {
    Supported,
    Unsupported,
    /// Проверка ещё не реализована для этой платформы / не удалось определить.
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
