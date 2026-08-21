//! Desktop pairing playground for the KDE Connect migration.
//!
//! The GUI owns the decision for a real TLS pairing session. Network
//! callbacks are marshalled into the UI thread through a standard channel.

#[path = "../kdeconnect/mod.rs"]
mod kdeconnect;

use eframe::egui;
use kdeconnect::{default_security_dir, IncomingPairing, LocalTlsIdentity, PairingResponder, TlsPairingServer, TLS_PAIRING_PORT};
use kdeconnect::tls_server::PairingDecision as TlsPairingDecision;
use std::sync::mpsc::{self, Receiver, Sender};

struct PendingPairing {
    request: IncomingPairing,
    responder: PairingResponder,
}

struct PairingDemoApp {
    pending: Option<PendingPairing>,
    event_rx: Receiver<PendingPairing>,
    status: String,
    server_error: Option<String>,
}

impl PairingDemoApp {
    fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        let mut app = Self {
            pending: None,
            event_rx,
            status: format!("Starting TLS pairing server on port {TLS_PAIRING_PORT}…"),
            server_error: None,
        };

        let security_dir = default_security_dir();
        let identity = match LocalTlsIdentity::load_or_create(&security_dir) {
            Ok(identity) => identity,
            Err(error) => {
                app.server_error = Some(format!("TLS identity error: {error:#}"));
                return app;
            }
        };

        let trust_path = security_dir.join("trusted_peers.json");
        let server = TlsPairingServer::new(identity, trust_path);
        match server.spawn(&format!("0.0.0.0:{TLS_PAIRING_PORT}"), move |request, responder| {
            let _ = event_tx.send(PendingPairing { request, responder });
        }) {
            Ok(_) => {
                app.status = format!("TLS pairing server listening on 0.0.0.0:{TLS_PAIRING_PORT}");
            }
            Err(error) => {
                app.server_error = Some(format!("TLS server error: {error:#}"));
            }
        }
        app
    }

    fn poll_events(&mut self) {
        while let Ok(pending) = self.event_rx.try_recv() {
            self.status = format!("Pairing request from {}", pending.request.identity.device_name);
            self.pending = Some(pending);
        }
    }

    fn decide(&mut self, decision: TlsPairingDecision) {
        let Some(pending) = self.pending.take() else { return };
        let name = pending.request.identity.device_name.clone();
        match pending.responder.decide(decision.clone()) {
            Ok(()) => {
                self.status = match decision {
                    TlsPairingDecision::Accept => format!("Paired with {name}"),
                    TlsPairingDecision::Reject => format!("Rejected {name}"),
                };
            }
            Err(error) => self.status = format!("Pairing response error: {error:#}"),
        }
    }
}

impl eframe::App for PairingDemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_events();
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        let pending_view = self.pending.as_ref().map(|pending| {
            (
                pending.request.identity.device_name.clone(),
                pending.request.identity.device_id.clone(),
                pending.request.identity.device_type.clone(),
                pending.request.identity.protocol_version,
                pending.request.fingerprint.clone(),
            )
        });

        let mut decision = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PhoneBridge — KDE Connect pairing");
            ui.label("Live TLS pairing endpoint");
            ui.separator();
            ui.label(&self.status);

            if let Some(error) = &self.server_error {
                ui.colored_label(egui::Color32::RED, error);
            }

            if let Some((name, id, device_type, protocol, fingerprint)) = &pending_view {
                ui.add_space(10.0);
                ui.group(|ui| {
                    ui.heading("Incoming pairing request");
                    ui.label(format!("Name: {name}"));
                    ui.label(format!("ID: {id}"));
                    ui.label(format!("Type: {device_type}"));
                    ui.label(format!("Protocol: {protocol}"));
                    ui.label(format!("Certificate fingerprint: {fingerprint}"));
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Allow").clicked() {
                            // Apply the decision after the immutable UI snapshot is released.
                            decision = Some(TlsPairingDecision::Accept);
                        }
                        if ui.button("Reject").clicked() {
                            decision = Some(TlsPairingDecision::Reject);
                        }
                    });
                });
            } else {
                ui.add_space(20.0);
                ui.label("Waiting for Android pairing request…");
                ui.monospace(format!("TLS endpoint: 0.0.0.0:{TLS_PAIRING_PORT}"));
            }
        });

        if let Some(decision) = decision {
            self.decide(decision);
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "PhoneBridge Pairing",
        options,
        Box::new(|_cc| Ok(Box::new(PairingDemoApp::new()))),
    )
}
