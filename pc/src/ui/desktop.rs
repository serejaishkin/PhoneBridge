//! Cross-platform desktop dashboard built with iced.
//! Pairing actions are emitted into the live core command channel.

use super::{PairingUiCommand, PairingUiCommandSender};
use iced::widget::{button, column, container, row, scrollable, text, toggler};
use iced::{Element, Length, Task};

#[derive(Debug, Clone)]
pub enum Message {
    ShowPairing,
    ShowDashboard,
    ShowDiagnostics,
    ToggleMedia(bool),
    ToggleMicrophone(bool),
    ApprovePairing,
    RejectPairing,
    ForgetPhone,
    CommandSent(Result<(), String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen { Dashboard, Pairing, Diagnostics }

#[derive(Debug, Clone)]
pub struct DesktopApp {
    screen: Screen,
    connected: bool,
    peer_name: String,
    peer_address: String,
    peer_device_id: String,
    hfp: String,
    pairing_code: String,
    pairing_pending: bool,
    pairing_status: String,
    media_enabled: bool,
    microphone_enabled: bool,
    diagnostics: Vec<String>,
    command_sender: Option<PairingUiCommandSender>,
}

impl DesktopApp {
    pub fn new(pairing_code: impl Into<String>) -> Self {
        Self {
            screen: Screen::Dashboard,
            connected: false,
            peer_name: "No phone paired".into(),
            peer_address: "—".into(),
            peer_device_id: String::new(),
            hfp: "Checking…".into(),
            pairing_code: pairing_code.into(),
            pairing_pending: false,
            pairing_status: "No pending request".into(),
            media_enabled: true,
            microphone_enabled: true,
            diagnostics: Vec::new(),
            command_sender: None,
        }
    }

    /// Attach the live command channel owned by PairingServer.
    pub fn with_command_sender(mut self, sender: PairingUiCommandSender) -> Self {
        self.command_sender = Some(sender);
        self
    }

    pub fn set_connection(&mut self, connected: bool, peer_name: Option<&str>, address: Option<&str>) {
        self.connected = connected;
        if let Some(name) = peer_name { self.peer_name = name.to_owned(); }
        if let Some(address) = address { self.peer_address = address.to_owned(); }
    }

    /// Populate the live request received from ControlSession.
    pub fn set_pairing_request(&mut self, device_id: &str, code: &str) {
        self.peer_device_id = device_id.to_owned();
        self.pairing_code = code.to_owned();
        self.pairing_pending = true;
        self.pairing_status = "Verify the code on the phone, then choose Allow or Reject.".into();
        self.screen = Screen::Pairing;
    }

    pub fn set_pairing_result(&mut self, trusted: bool, message: &str) {
        self.pairing_pending = false;
        self.pairing_status = message.to_owned();
        if trusted { self.connected = true; }
    }

    pub fn set_hfp(&mut self, status: impl Into<String>) { self.hfp = status.into(); }
    pub fn push_diagnostic(&mut self, line: impl Into<String>) { self.diagnostics.push(line.into()); }

    pub fn run(self) -> iced::Result {
        iced::application("PhoneBridge", update, view)
            .run_with(|| (self, Task::none()))
    }
}

fn send_command(sender: Option<PairingUiCommandSender>, command: PairingUiCommand) -> Task<Message> {
    let Some(sender) = sender else { return Task::done(Message::CommandSent(Err("pairing command channel is not connected".into()))); };
    Task::perform(async move { sender.send(command).await.map_err(|_| "live pairing session is no longer available".to_owned()) }, Message::CommandSent)
}

fn update(app: &mut DesktopApp, message: Message) -> Task<Message> {
    match message {
        Message::ShowPairing => app.screen = Screen::Pairing,
        Message::ShowDashboard => app.screen = Screen::Dashboard,
        Message::ShowDiagnostics => app.screen = Screen::Diagnostics,
        Message::ToggleMedia(value) => app.media_enabled = value,
        Message::ToggleMicrophone(value) => app.microphone_enabled = value,
        Message::ApprovePairing => {
            if app.pairing_pending {
                app.pairing_status = "Allow requested".into();
                return send_command(app.command_sender.clone(), PairingUiCommand::Approve { device_id: app.peer_device_id.clone(), short_code: app.pairing_code.clone() });
            }
        }
        Message::RejectPairing => {
            if app.pairing_pending {
                app.pairing_status = "Reject requested".into();
                return send_command(app.command_sender.clone(), PairingUiCommand::Reject { device_id: app.peer_device_id.clone(), reason: "rejected by user".into() });
            }
        }
        Message::ForgetPhone => {
            app.connected = false;
            app.pairing_pending = false;
            app.pairing_status = "Trust revoke requested".into();
            if !app.peer_device_id.is_empty() {
                return send_command(app.command_sender.clone(), PairingUiCommand::Forget { device_id: app.peer_device_id.clone() });
            }
        }
        Message::CommandSent(result) => {
            if let Err(error) = result { app.pairing_status = error; }
        }
    }
    Task::none()
}

fn view(app: &DesktopApp) -> Element<'_, Message> {
    let navigation = row![
        button("Dashboard").on_press(Message::ShowDashboard),
        button("Pairing").on_press(Message::ShowPairing),
        button("Diagnostics").on_press(Message::ShowDiagnostics),
    ].spacing(8);
    let body = match app.screen { Screen::Dashboard => dashboard(app), Screen::Pairing => pairing(app), Screen::Diagnostics => diagnostics(app) };
    container(column![navigation, body].spacing(18).padding(24)).width(Length::Fill).height(Length::Fill).into()
}

fn dashboard(app: &DesktopApp) -> Element<'_, Message> {
    let connection = if app.connected { "Connected" } else { "Offline" };
    column![
        text("PhoneBridge").size(32),
        text(format!("Status: {}", connection)),
        text(format!("Phone: {}", app.peer_name)),
        text(format!("Address: {}", app.peer_address)),
        text(format!("Bluetooth HFP: {}", app.hfp)),
        button("Forget phone").on_press(Message::ForgetPhone),
        row![toggler(app.media_enabled).label("Media stream").on_toggle(Message::ToggleMedia), toggler(app.microphone_enabled).label("PC microphone").on_toggle(Message::ToggleMicrophone)].spacing(24),
    ].spacing(12).into()
}

fn pairing(app: &DesktopApp) -> Element<'_, Message> {
    let actions = if app.pairing_pending {
        row![button("Allow").on_press(Message::ApprovePairing), button("Reject").on_press(Message::RejectPairing)].spacing(12)
    } else { row![].spacing(12) };
    column![
        text("Pair Android phone").size(28),
        text(format!("Device: {}", if app.peer_device_id.is_empty() { "waiting" } else { &app.peer_device_id })),
        text("Verify that both devices show the same code."),
        container(text(&app.pairing_code).size(42)).padding(20),
        text(&app.pairing_status),
        actions,
    ].spacing(14).into()
}

fn diagnostics(app: &DesktopApp) -> Element<'_, Message> {
    let lines = app.diagnostics.iter().fold(column![], |column, line| column.push(text(line)));
    column![text("Diagnostics").size(28), scrollable(lines).height(Length::Fill)].spacing(12).into()
}
