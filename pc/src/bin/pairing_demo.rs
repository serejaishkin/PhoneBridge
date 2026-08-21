//! Small desktop pairing playground for the KDE Connect migration.
//!
//! This is intentionally a standalone binary so the new pairing UX can be
//! exercised before it is wired into the real network daemon.

#[path = "../kdeconnect/mod.rs"]
mod kdeconnect;

use eframe::egui;
use kdeconnect::{IdentityPacket, PairingDecision, PairingEvent, PairingSession};

struct PairingDemoApp {
    session: PairingSession,
    last_packet: String,
    event_text: String,
}

impl Default for PairingDemoApp {
    fn default() -> Self {
        Self {
            session: PairingSession::default(),
            last_packet: String::new(),
            event_text: "Waiting for a pairing request".into(),
        }
    }
}

impl PairingDemoApp {
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

        match self.session.receive_identity(identity) {
            Ok(PairingEvent::Request { identity }) => {
                self.event_text = format!("Pairing request from {}", identity.device_name);
                self.last_packet = "kdeconnect.identity received".into();
            }
            Ok(_) => {}
            Err(error) => self.event_text = format!("Error: {error}"),
        }
    }

    fn decide(&mut self, decision: PairingDecision) {
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
            Err(error) => self.event_text = format!("Error: {error}"),
        }
    }
}

impl eframe::App for PairingDemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PhoneBridge — KDE Connect pairing");
            ui.label("Pairing playground. Network/TLS wiring is intentionally not connected yet.");
            ui.separator();

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
                });
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Simulate Android pairing request").clicked() {
                    self.simulate_phone_request();
                }

                let can_decide = self.session.state == kdeconnect::PairingState::IncomingRequest;
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
