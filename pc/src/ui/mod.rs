//! Единый интерфейс UI-бэкенда, чтобы main.rs и остальная логика не знали,
//! какой именно тулкит рисует трей/окна на конкретной ОС.
//!
//! Сейчас реализован только `HeadlessUi` (пишет в лог) — он специально
//! существует, чтобы проект собирался и был тестируем без GUI-тулкита на
//! любой машине (в том числе в CI / песочнице без X11).
//!
//! Согласно AI_HANDOFF_GUI.md, следующий шаг — реализовать:
//! - `WindowsUi` (расширение уже существовавшего в PhoneBridge v1 tray.rs на базе `tray-icon`)
//! - `LinuxUi` / `MacUi` (тоже `tray-icon` для трея + `egui`/`eframe` для окна настроек
//!   и pairing wizard — выбор обосновать в PR, не молча тащить третий тулкит)
//!
//! Каждая реализация должна имплементировать этот трейт и подключаться через
//! #[cfg(target_os = "...")] в конце файла — без изменения остального кода.

use crate::protocol::HfpSupport;
use async_trait::async_trait;

#[async_trait]
pub trait UiBackend: Send + Sync {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>);
    async fn notify_call_ended(&self);
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>);
    async fn update_hfp_status(&self, status: HfpSupport);
}

pub struct HeadlessUi;

#[async_trait]
impl UiBackend for HeadlessUi {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>) {
        log::info!(
            "[UI] incoming call: {} ({})",
            caller_name.unwrap_or("unknown"),
            caller_number.unwrap_or("no number")
        );
    }

    async fn notify_call_ended(&self) {
        log::info!("[UI] call ended");
    }

    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>) {
        log::info!(
            "[UI] connection status: {} ({})",
            if connected { "connected" } else { "disconnected" },
            peer_name.unwrap_or("-")
        );
    }

    async fn update_hfp_status(&self, status: HfpSupport) {
        log::info!("[UI] HFP support: {:?}", status);
    }
}
