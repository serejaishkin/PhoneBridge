//! Состояние звонка на стороне PC.
//!
//! Важно ещё раз: сам звук звонка сюда не попадает и не должен — он идёт
//! нативным Bluetooth Classic HFP (телефон = Audio Gateway, PC = Hands-Free
//! unit), см. корень README. Этот модуль только про control-plane: кто
//! звонит, кнопки Answer/Decline, и диагностику "почему у меня нет звука".

use crate::protocol::HfpSupport;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallState {
    Idle,
    Ringing {
        caller_number: Option<String>,
        caller_name: Option<String>,
    },
    Active,
}

pub struct SharedState {
    pub call: Mutex<CallState>,
    pub hfp_support: Mutex<HfpSupport>,
}

impl SharedState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            call: Mutex::new(CallState::Idle),
            hfp_support: Mutex::new(HfpSupport::Unknown),
        })
    }
}

/// Проверка, поддерживает ли Bluetooth-адаптер этого PC роль Hands-Free Unit
/// (клиент HFP) — от этого зависит, будет ли вообще слышен звук звонка.
///
/// Это ЗАГЛУШКА. Реальная проверка платформо-зависима:
/// - Windows: нужно смотреть через Bluetooth/WinRT API, какие профили
///   поддерживает адаптер (нет простого публичного API "поддерживает ли HFP
///   client role" — вероятно, придётся проверять по факту наличия
///   "Handsfree Telephony" в списке служб сопряжённого устройства).
/// - Linux: через BlueZ D-Bus — есть ли профиль `hfp_hf` в адаптере/оrg.bluez.
/// - macOS: через IOBluetooth — аналогично, публичного простого API нет.
///
/// TODO(Kimi): реализовать per-platform проверку в отдельных файлах
/// call/hfp_check_windows.rs / hfp_check_linux.rs / hfp_check_macos.rs
/// и подключить сюда через #[cfg(target_os = ...)].
pub async fn check_hfp_support() -> HfpSupport {
    log::warn!(
        "check_hfp_support() is a stub — real platform detection not implemented yet, \
         returning Unknown. UI must show this as 'нужно проверить вручную', не как ошибку."
    );
    HfpSupport::Unknown
}
