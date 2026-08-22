use crate::protocol::{HfpSupport, MediaCommand, Message};
use crate::sms::{SmsController, SmsStore};
use crate::ui::UiBackend;
use async_trait::async_trait;
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;

#[derive(Default)]
pub struct DesktopState {
    pub connected: bool,
    pub peer_name: String,
    pub hfp: HfpSupport,
    pub caller_name: String,
    pub caller_number: String,
    pub ringing: bool,
    pub sms_notice: String,
}

pub struct DesktopUi { state: Arc<Mutex<DesktopState>> }
impl DesktopUi {
    pub fn new(state: Arc<Mutex<DesktopState>>) -> Self { Self { state } }
}

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
        if !connected { s.ringing = false; }
    }
    async fn update_hfp_status(&self, status: HfpSupport) { self.state.lock().unwrap().hfp = status; }
    async fn notify_sms_received(&self, address: &str, body: &str, _timestamp: i64) {
        self.state.lock().unwrap().sms_notice = format!("Новое SMS от {address}: {body}");
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
}

impl PhoneBridgeApp {
    pub fn new(state: Arc<Mutex<DesktopState>>, sms_store: Arc<tokio::sync::Mutex<SmsStore>>, controller: SmsController, runtime: Handle) -> Self {
        Self { state, sms_store, controller, runtime, selected_address: String::new(), sms_body: String::new(), selected_sms: None }
    }
    fn send(&self, message: Message) {
        let controller = self.controller.clone();
        self.runtime.spawn(async move { let _ = controller.send_message(message).await; });
    }
}

impl eframe::App for PhoneBridgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
        let state = self.state.lock().unwrap();
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("PhoneBridge");
                ui.separator();
                ui.label(if state.connected { format!("● {}", state.peer_name) } else { "○ Телефон не подключён".into() });
                ui.separator();
                ui.label(format!("HFP: {:?}", state.hfp));
            });
        });

        egui::SidePanel::left("calls").resizable(false).show(ctx, |ui| {
            ui.heading("Звонки");
            ui.separator();
            if state.ringing {
                ui.label(if state.caller_name.is_empty() { "Входящий звонок" } else { &state.caller_name });
                ui.label(&state.caller_number);
                ui.horizontal(|ui| {
                    if ui.button("Ответить").clicked() { self.send(Message::CallAnswer); }
                    if ui.button("Отклонить").clicked() { self.send(Message::CallDecline); }
                });
            } else { ui.label("Нет активного звонка"); }
            ui.separator();
            ui.heading("Медиа");
            for (title, command) in [("▶/Ⅱ", MediaCommand::PlayPause), ("⏮", MediaCommand::Previous), ("⏭", MediaCommand::Next)] {
                if ui.button(title).clicked() { self.send(Message::MediaCommand { command }); }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SMS");
            if !state.sms_notice.is_empty() { ui.label(&state.sms_notice); }
            drop(state);
            let messages = self.runtime.block_on(async { self.sms_store.lock().await.all() });
            egui::ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
                for (i, sms) in messages.iter().enumerate() {
                    if ui.selectable_label(self.selected_sms == Some(i), format!("{} — {}", sms.address, sms.body)).clicked() {
                        self.selected_sms = Some(i);
                        self.selected_address = sms.address.clone();
                    }
                }
            });
            ui.separator();
            ui.horizontal(|ui| { ui.label("Номер:"); ui.text_edit_singleline(&mut self.selected_address); });
            ui.text_edit_multiline(&mut self.sms_body);
            if ui.button("Отправить SMS").clicked() && !self.selected_address.trim().is_empty() && !self.sms_body.trim().is_empty() {
                let address = self.selected_address.clone();
                let body = std::mem::take(&mut self.sms_body);
                self.send(Message::SmsSend { address, body });
            }
        });
    }
}
