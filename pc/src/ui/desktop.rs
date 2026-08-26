use crate::protocol::{HfpSupport, MediaCommand, MediaPlaybackState, Message};
use crate::sms::{SmsController, SmsStore};
use crate::ui::{PairingRequest, UiBackend};
#[cfg(any(windows, target_os = "macos"))]
use crate::ui::tray::{TrayCommand, TrayUI};
use async_trait::async_trait;
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;
use tokio::sync::oneshot;

pub struct DesktopState {
    pub connected: bool,
    pub peer_name: String,
    pub hfp: HfpSupport,
    pub caller_name: String,
    pub caller_number: String,
    pub ringing: bool,
    pub media_package: String,
    pub media_state: MediaPlaybackState,
    pub media_title: String,
    pub media_artist: String,
    pub media_album: String,
    pub sms_notice: String,
    pub sms_error: String,
    /// Short code of this PC certificate, shown so the user can compare it
    /// against the phone screen during pairing.
    pub local_code: String,
    /// Pairing attempts waiting for an explicit Allow/Reject decision.
    pub pending_pairing: Vec<(PairingRequest, oneshot::Sender<bool>)>,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            connected: false,
            peer_name: String::new(),
            hfp: HfpSupport::Unknown,
            caller_name: String::new(),
            caller_number: String::new(),
            ringing: false,
            media_package: String::new(),
            media_state: MediaPlaybackState::None,
            media_title: String::new(),
            media_artist: String::new(),
            media_album: String::new(),
            sms_notice: String::new(),
            sms_error: String::new(),
            local_code: String::new(),
            pending_pairing: Vec::new(),
        }
    }
}

pub struct DesktopUi { state: Arc<Mutex<DesktopState>> }
impl DesktopUi { pub fn new(state: Arc<Mutex<DesktopState>>) -> Self { Self { state } } }

#[async_trait]
impl UiBackend for DesktopUi {
    async fn notify_incoming_call(&self, name: Option<&str>, number: Option<&str>) {
        let mut s = self.state.lock().unwrap();
        s.ringing = true;
        s.caller_name = name.unwrap_or("").to_string();
        s.caller_number = number.unwrap_or("").to_string();
    }

    async fn notify_call_ended(&self) { self.state.lock().unwrap().ringing = false; }

    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>) {
        let mut s = self.state.lock().unwrap();
        s.connected = connected;
        s.peer_name = peer_name.unwrap_or("").to_string();
        if !connected {
            s.ringing = false;
            s.media_state = MediaPlaybackState::None;
            s.media_title.clear();
            s.media_artist.clear();
            s.media_album.clear();
        }
    }

    async fn update_hfp_status(&self, status: HfpSupport) { self.state.lock().unwrap().hfp = status; }

    async fn update_media_state(&self, package: Option<&str>, state: MediaPlaybackState, title: Option<&str>, artist: Option<&str>, album: Option<&str>) {
        let mut s = self.state.lock().unwrap();
        s.media_package = package.unwrap_or("").to_string();
        s.media_state = state;
        s.media_title = title.unwrap_or("").to_string();
        s.media_artist = artist.unwrap_or("").to_string();
        s.media_album = album.unwrap_or("").to_string();
    }

    async fn notify_sms_received(&self, address: &str, body: &str, _timestamp: i64) {
        let mut s = self.state.lock().unwrap();
        s.sms_error.clear();
        s.sms_notice = format!("Новое SMS от {address}: {body}");
    }

    async fn notify_sms_sent(&self, address: &str, _body: &str) {
        let mut s = self.state.lock().unwrap();
        s.sms_error.clear();
        s.sms_notice = format!("SMS отправлено на {address}");
    }

    async fn notify_sms_error(&self, error: &str) {
        self.state.lock().unwrap().sms_error = error.to_string();
    }

    async fn request_pairing_decision(&self, request: PairingRequest) -> bool {
        let (tx, rx) = oneshot::channel();
        self.state.lock().unwrap().pending_pairing.push((request, tx));
        // Park the connection task until the user answers in the GUI thread.
        // A dropped responder (state reset) counts as rejection.
        matches!(rx.await, Ok(true))
    }
}

pub struct PhoneBridgeApp {
    state: Arc<Mutex<DesktopState>>,
    sms_store: Arc<tokio::sync::Mutex<SmsStore>>,
    controller: SmsController,
    runtime: Handle,
    selected_address: String,
    sms_body: String,
    selected_sms: Option<usize>,
    /// Tray lives as long as the app; created on the main thread by eframe.
    #[cfg(any(windows, target_os = "macos"))]
    _tray: Option<TrayUI>,
}

impl PhoneBridgeApp {
    pub fn new(state: Arc<Mutex<DesktopState>>, sms_store: Arc<tokio::sync::Mutex<SmsStore>>, controller: SmsController, runtime: Handle) -> Self {
        #[cfg(any(windows, target_os = "macos"))]
        let _tray = match TrayUI::new() {
            Ok(tray) => Some(tray),
            Err(e) => {
                log::warn!("system tray unavailable: {e}");
                None
            }
        };
        Self {
            state,
            sms_store,
            controller,
            runtime,
            selected_address: String::new(),
            sms_body: String::new(),
            selected_sms: None,
            #[cfg(any(windows, target_os = "macos"))]
            _tray,
        }
    }

    fn send(&self, message: Message) {
        let controller = self.controller.clone();
        self.runtime.spawn(async move { let _ = controller.send_message(message).await; });
    }

    fn request_sms_history(&self) {
        let controller = self.controller.clone();
        self.runtime.spawn(async move { let _ = controller.request_history().await; });
    }

    fn resolve_pairing(&self, index: usize, accepted: bool) {
        // The index comes from the last rendered frame; re-validate under the
        // lock because the pending list may have changed since.
        let entry = {
            let mut s = self.state.lock().unwrap();
            if index >= s.pending_pairing.len() { return; }
            s.pending_pairing.remove(index)
        };
        let (_request, tx) = entry;
        let _ = tx.send(accepted);
    }
}

impl eframe::App for PhoneBridgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(250));

        // Drain tray menu commands first so they act within this frame.
        #[cfg(any(windows, target_os = "macos"))]
        if let Some(tray) = &self._tray {
            for command in tray.poll_commands() {
                match command {
                    TrayCommand::AnswerCall => self.send(Message::CallAnswer),
                    TrayCommand::EndCall => self.send(Message::CallDecline),
                    // No audio pipeline is wired yet; keep the item responsive.
                    TrayCommand::ToggleMute => log::info!("tray: mute toggle requested (no audio path yet)"),
                    TrayCommand::OpenSettings => log::info!("tray: settings requested (not implemented yet)"),
                    TrayCommand::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                }
            }
        }

        let (connected, peer_name, hfp, caller_name, caller_number, ringing, media_state, media_title, media_artist, media_album, sms_notice, sms_error, local_code) = {
            let state = self.state.lock().unwrap();
            (state.connected, state.peer_name.clone(), state.hfp, state.caller_name.clone(), state.caller_number.clone(), state.ringing, state.media_state, state.media_title.clone(), state.media_artist.clone(), state.media_album.clone(), state.sms_notice.clone(), state.sms_error.clone(), state.local_code.clone())
        };

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("PhoneBridge");
                ui.separator();
                ui.label(if connected { format!("● {}", peer_name) } else { "○ Телефон не подключён".into() });
                ui.separator();
                ui.label(format!("HFP: {:?}", hfp));
                ui.separator();
                if !local_code.is_empty() { ui.label(format!("Код ПК: {local_code}")); }
            });
        });

        egui::SidePanel::left("calls").resizable(false).default_width(260.0).show(ctx, |ui| {
            ui.heading("Звонки");
            ui.separator();
            if ringing {
                ui.strong(if caller_name.is_empty() { "Входящий звонок" } else { &caller_name });
                if !caller_number.is_empty() { ui.label(&caller_number); }
                ui.horizontal(|ui| {
                    if ui.button("Ответить").clicked() { self.send(Message::CallAnswer); }
                    if ui.button("Отклонить").clicked() { self.send(Message::CallDecline); }
                });
            } else {
                ui.label("Нет активного звонка");
            }

            ui.add_space(12.0);
            ui.heading("Медиа");
            ui.separator();
            if !media_title.is_empty() { ui.strong(&media_title); } else { ui.label("Нет активного трека"); }
            if !media_artist.is_empty() { ui.label(&media_artist); }
            if !media_album.is_empty() { ui.label(&media_album); }
            ui.label(format!("Состояние: {:?}", media_state));
            ui.horizontal(|ui| {
                if ui.button("⏮").clicked() { self.send(Message::MediaCommand { command: MediaCommand::Previous }); }
                let toggle = match media_state { MediaPlaybackState::Playing => "Ⅱ", _ => "▶" };
                if ui.button(toggle).clicked() { self.send(Message::MediaCommand { command: MediaCommand::PlayPause }); }
                if ui.button("⏭").clicked() { self.send(Message::MediaCommand { command: MediaCommand::Next }); }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SMS");
            ui.horizontal(|ui| {
                if ui.button("Обновить историю").clicked() { self.request_sms_history(); }
                ui.label(if connected { "Телефон подключён" } else { "Нет подключения" });
            });
            if !sms_notice.is_empty() { ui.label(&sms_notice); }
            if !sms_error.is_empty() { ui.colored_label(egui::Color32::RED, format!("Ошибка: {sms_error}")); }

            let messages = self.runtime.block_on(async { self.sms_store.lock().await.all() });
            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                for (i, sms) in messages.iter().enumerate() {
                    let label = format!("{}  —  {}", sms.address, sms.body.replace('\n', " "));
                    if ui.selectable_label(self.selected_sms == Some(i), label).clicked() {
                        self.selected_sms = Some(i);
                        self.selected_address = sms.address.clone();
                    }
                }
            });

            ui.separator();
            ui.label("Получатель");
            ui.text_edit_singleline(&mut self.selected_address);
            ui.label("Сообщение");
            ui.add(egui::TextEdit::multiline(&mut self.sms_body).desired_rows(4));
            let can_send = connected && !self.selected_address.trim().is_empty() && !self.sms_body.trim().is_empty();
            if ui.add_enabled(can_send, egui::Button::new("Отправить SMS")).clicked() {
                let address = self.selected_address.trim().to_string();
                let body = std::mem::take(&mut self.sms_body);
                self.send(Message::SmsSend { address, body });
            }
        });

        // Pairing dialog: shown above everything while a decision is pending.
        let pending: Vec<(usize, PairingRequest)> = {
            let state = self.state.lock().unwrap();
            state.pending_pairing.iter().enumerate().map(|(i, (r, _))| (i, r.clone())).collect()
        };
        if !pending.is_empty() {
            egui::Window::new("Сопряжение устройства")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Новое устройство запрашивает доступ к этому ПК.");
                    ui.add_space(4.0);
                    for (_idx, request) in &pending {
                        ui.strong(format!("Устройство: {}", request.device_name));
                        ui.label(format!("ID: {}", request.device_id));
                        ui.add_space(4.0);
                        ui.label("Сверьте коды безопасности на обоих устройствах:");
                        ui.monospace(format!("Телефон: {}", request.peer_code));
                        ui.monospace(format!("Этот ПК: {}", request.local_code));
                        ui.add_space(8.0);
                    }
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("Разрешить")).clicked() {
                            self.resolve_pairing(pending[0].0, true);
                        }
                        if ui.add(egui::Button::new("Отклонить")).clicked() {
                            self.resolve_pairing(pending[0].0, false);
                        }
                    });
                });
        }
    }
}
