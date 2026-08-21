//! Desktop pairing playground for the KDE Connect migration.
//!
//! The GUI now controls a real framed TCP pairing listener. TLS certificate
//! authentication is intentionally kept as the next transport milestone.

#[path = "../kdeconnect/mod.rs"]
mod kdeconnect;

use eframe::egui;
use kdeconnect::{
    IdentityPacket, PairingDecision, PairingEvent, PairingServer, PairingServerEvent,
    PairingSession, PairingState, DEFAULT_PAIRING_PORT,
};
use std::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;

struct PairingDemoApp {
    session: PairingSession,
    last_packet: String,
    event_text: String,
    server_text: String,
    events: Receiver<PairingServerEvent>,
    decision: Option<oneshot::Sender<bool>>,
}

impl PairingDemoApp {
    fn new() -> Self {
        let (events_tx, events_rx) = mpsc::channel();
        let mut app = Self {
            session: PairingSession::default(),
            last_packet: String::new(),
            event_text: "Starting pairing listener...".into(),
            server_text: format!("TCP 0.0.0.0:{DEFAULT_PAIRING_PORT}"),
            events: events_rx,
            decision: None,
        };

        if let Err(error) = PairingServer::spawn(DEFAULT_PAIRING_PORT, events_tx) {
            app.event_text = format!("Failed to start listener: {error:#}");
        }
        app
    }

    fn process_server_events(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                PairingServerEvent::Listening { address } => {
                    self.server_text = format!("Listening: {address}");
                    self.event_text =
                        "Pairing listener is ready. Waiting for a KDE Connect device.".into();
                }
                PairingServerEvent::Connected { address } => {
                    self.event_text = format!("Incoming TCP connection from {address}");
                }
                PairingServerEvent::Request { identity, decision } => {
                    self.handle_identity(identity);
                    self.decision = Some(decision);
                }
                PairingServerEvent::Error(error) => {
                    self.event_text = format!("Server error: {error}");
                }
            }
        }
    }

    fn handle_identity(&mut self, identity: IdentityPacket) {
        match self.session.receive_identity(identity) {
            Ok(PairingEvent::Request { identity }) => {
                self.event_text = format!("Pairing request from \"{}\"", identity.device_name);
                self.last_packet = "kdeconnect.identity received".into();
            }
            Ok(_) => {}
            Err(error) => self.event_text = format!("Identity error: {error}"),
        }
    }

    fn decide(&mut self, decision: PairingDecision) {
        let accepted = decision == PairingDecision::Accept;
        if let Some(sender) = self.decision.take() {
            let _ = sender.send(accepted);
        }

        match self.session.decide(decision) {
            Ok((packet, event)) => {
                self.last_packet = packet
                    .to_line()
                    .unwrap_or_else(|error| format!("packet error: {error}"));
                self.event_text = match event {
                    PairingEvent::Accepted { peer_id } => format!("Paired with {peer_id}"),
                    PairingEvent::Rejected { peer_id } => format!("Rejected {peer_id}"),
                    PairingEvent::Request { .. } => "Pairing request".into(),
                    PairingEvent::Error(error) => format!("Error: {error}"),
                };
            }
            Err(error) => self.event_text = format!("Pairing error: {error}"),
        }
    }

    fn simulate_phone_request(&mut self) {
        let identity = IdentityPacket {
            device_id: "0123456789abcdef0123456789abcdef".into(),
            device_name: "Demo Android Phone".into(),
            device_type: "phone".into(),
            incoming_capabilities: vec!["kdeconnect.ping".into(), "kdeconnect.share".into()],
            outgoing_capabilities: vec!["kdeconnect.ping".into(), "kdeconnect.share".into()],
            protocol_version: 8,
            transport_fields: Default::default(),
        };
        self.handle_identity(identity);
    }
}

impl Default for PairingDemoApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for PairingDemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_server_events();
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PhoneBridge — KDE Connect pairing");
            ui.label("Live TCP pairing endpoint. TLS authentication is the next transport step.");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Server:");
                ui.monospace(&self.server_text);
            });
            ui.horizontal(|ui| {
                ui.label("State:");
                ui.monospace(format!("{:?}", self.session.state));
            });
            ui.label(&self.event_text);

            if let Some(peer) = &self.session.peer {
                ui.group(|ui| {
                    ui.heading("Remote device");
                    ui.label(format!("Name: {}", peer.device_name));
                    ui.label(format!("ID: {}", peer.device_id));
                    ui.label(format!("Type: {}", peer.device_type));
                    ui.label(format!("Protocol: {}", peer.protocol_version));
                    ui.label(format!("Incoming capabilities: {}", peer.incoming_capabilities.len()));
                    ui.label(format!("Outgoing capabilities: {}", peer.outgoing_capabilities.len()));
                });
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Simulate Android request").clicked() {
                    self.simulate_phone_request();
                }

                let can_decide = self.session.state == PairingState::IncomingRequest;
                ui.add_enabled_ui(can_decide, |ui| {
                    if ui.button("Allow").clicked() {
                        self.decide(PairingDecision::Accept);
                    }
                    if ui.button("Reject").clicked() {
                        self.decide(PairingDecision::Reject);
                    }
                });
            });

            if !self.last_packet.is_empty() {
                ui.separator();
                ui.label("Last protocol packet:");
                ui.code(&self.last_packet);
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "PhoneBridge Pairing",
        options,
        Box::new(|_cc| Ok(Box::new(PairingDemoApp::default()))),
    )
}
