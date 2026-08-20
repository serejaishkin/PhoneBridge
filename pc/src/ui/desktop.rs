//! Cross-platform desktop dashboard built with iced.
//! The window is intentionally transport-agnostic: pairing and connection state remain in the core.

use iced::widget::{button, column, container, row, scrollable, text, toggler};
use iced::{Element, Length, Task};

#[derive(Debug, Clone)]
pub enum Message {
    ShowPairing,
    ShowDashboard,
    ShowDiagnostics,
    ToggleMedia(bool),
    ToggleMicrophone(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen { Dashboard, Pairing, Diagnostics }

#[derive(Debug, Clone)]
pub struct DesktopApp {
    screen: Screen,
    connected: bool,
    peer_name: String,
    peer_address: String,
    hfp: String,
    pairing_code: String,
    media_enabled: bool,
    microphone_enabled: bool,
    diagnostics: Vec<String>,
}

impl DesktopApp {
    pub fn new(pairing_code: impl Into<String>) -> Self {
        Self {
            screen: Screen::Dashboard,
            connected: false,
            peer_name: "No phone paired".into(),
            peer_address: "—".into(),
            hfp: "Checking…".into(),
            pairing_code: pairing_code.into(),
            media_enabled: true,
            microphone_enabled: true,
            diagnostics: Vec::new(),
        }
    }

    pub fn set_connection(&mut self, connected: bool, peer_name: Option<&str>, address: Option<&str>) {
        self.connected = connected;
        if let Some(name) = peer_name { self.peer_name = name.to_owned(); }
        if let Some(address) = address { self.peer_address = address.to_owned(); }
    }

    pub fn set_hfp(&mut self, status: impl Into<String>) { self.hfp = status.into(); }
    pub fn push_diagnostic(&mut self, line: impl Into<String>) { self.diagnostics.push(line.into()); }

    pub fn run(self) -> iced::Result {
        iced::application("PhoneBridge", update, view)
            .run_with(|| (self, Task::none()))
    }
}

fn update(app: &mut DesktopApp, message: Message) -> Task<Message> {
    match message {
        Message::ShowPairing => app.screen = Screen::Pairing,
        Message::ShowDashboard => app.screen = Screen::Dashboard,
        Message::ShowDiagnostics => app.screen = Screen::Diagnostics,
        Message::ToggleMedia(value) => app.media_enabled = value,
        Message::ToggleMicrophone(value) => app.microphone_enabled = value,
    }
    Task::none()
}

fn view(app: &DesktopApp) -> Element<'_, Message> {
    let navigation = row![
        button("Dashboard").on_press(Message::ShowDashboard),
        button("Pairing").on_press(Message::ShowPairing),
        button("Diagnostics").on_press(Message::ShowDiagnostics),
    ].spacing(8);

    let body = match app.screen {
        Screen::Dashboard => dashboard(app),
        Screen::Pairing => pairing(app),
        Screen::Diagnostics => diagnostics(app),
    };

    container(column![navigation, body].spacing(18).padding(24))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn dashboard(app: &DesktopApp) -> Element<'_, Message> {
    let connection = if app.connected { "Connected" } else { "Offline" };
    column![
        text("PhoneBridge").size(32),
        text(format!("Status: {}", connection)),
        text(format!("Phone: {}", app.peer_name)),
        text(format!("Address: {}", app.peer_address)),
        text(format!("Bluetooth HFP: {}", app.hfp)),
        row![
            toggler(app.media_enabled).label("Media stream").on_toggle(Message::ToggleMedia),
            toggler(app.microphone_enabled).label("PC microphone").on_toggle(Message::ToggleMicrophone),
        ].spacing(24),
    ].spacing(12).into()
}

fn pairing(app: &DesktopApp) -> Element<'_, Message> {
    column![
        text("Pair Android phone").size(28),
        text("Connect the phone to the same Wi-Fi network, then select this PC."),
        text("Verify that both devices show the same code before confirming."),
        container(text(&app.pairing_code).size(42)).padding(20),
        text("After confirmation the device identity is trusted and future reconnects can be automatic."),
    ].spacing(14).into()
}

fn diagnostics(app: &DesktopApp) -> Element<'_, Message> {
    let lines = app.diagnostics.iter().fold(column![], |column, line| column.push(text(line)));
    column![
        text("Diagnostics").size(28),
        scrollable(lines).height(Length::Fill),
    ].spacing(12).into()
}
