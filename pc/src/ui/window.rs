//! First cross-platform PhoneBridge desktop window.

use iced::{Element, Task, Theme};
use iced::widget::{button, column, container, row, text, Space};
use super::{BasicUi, UiState};

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    Pair,
    Reconnect,
    Disconnect,
}

pub struct PhoneBridgeWindow {
    ui: BasicUi,
    state: UiState,
}

impl PhoneBridgeWindow {
    pub fn new(ui: BasicUi) -> (Self, Task<Message>) {
        (Self { ui, state: UiState::default() }, Task::perform(ui.state().read().map(|s| s.clone()), |_| Message::Refresh))
    }

    pub fn title(&self) -> String { "PhoneBridge".into() }
    pub fn theme(&self) -> Theme { Theme::Dark }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => {
                let state = self.ui.state();
                return Task::perform(async move { state.read().await.clone() }, |state| {
                    // The current UI is state-only; transport actions are wired by the app shell next.
                    Message::Refresh
                });
            }
            Message::Pair | Message::Reconnect | Message::Disconnect => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let connection = if self.state.connected { "● Connected" } else { "○ Disconnected" };
        let hfp = format!("HFP: {:?}", self.state.hfp_support);
        let peer = self.state.peer_name.as_deref().unwrap_or("No phone");
        let code = self.state.pairing_code.as_deref().unwrap_or("—");

        let actions = row![
            button(text("Pair")).on_press(Message::Pair),
            button(text("Reconnect")).on_press(Message::Reconnect),
            button(text("Disconnect")).on_press(Message::Disconnect),
        ].spacing(8);

        container(column![
            text("PhoneBridge").size(28),
            Space::with_height(10),
            text(connection).size(18),
            text(format!("Phone: {peer}")),
            text(hfp),
            text(format!("Pairing code: {code}")),
            text(format!("Status: {}", self.state.status)),
            Space::with_height(12),
            actions,
        ].spacing(8).padding(20)).into()
    }
}
