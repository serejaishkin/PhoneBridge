use crate::call::{CallState, SharedState};
use crate::connection::state::ConnectionState;
use crate::pairing::session::{PairingSession, PairingState};
use crate::protocol::Message;
use anyhow::{bail, Result};
use std::sync::Arc;

pub struct ControlSession {
    state: ConnectionState,
    pairing: PairingSession,
    calls: Option<Arc<SharedState>>,
}

impl ControlSession {
    pub fn new() -> Self {
        Self { state: ConnectionState::Handshaking, pairing: PairingSession::new(), calls: None }
    }

    pub fn with_calls(mut self, calls: Arc<SharedState>) -> Self {
        self.calls = Some(calls);
        self
    }

    pub fn state(&self) -> &ConnectionState { &self.state }
    pub fn pairing(&self) -> &PairingSession { &self.pairing }

    pub async fn handle(&mut self, message: Message, trusted: bool) -> Result<Vec<Message>> {
        let mut outgoing = Vec::new();
        match message {
            Message::Hello { device_id, fingerprint, protocol_version, .. } => {
                if !matches!(self.state, ConnectionState::Handshaking) { bail!("Hello received after handshake"); }
                if let Some(challenge) = self.pairing.accept_hello(
                    device_id, fingerprint.clone(), protocol_version, trusted,
                    crate::pairing::trust::short_code(&fingerprint),
                )? { outgoing.push(challenge); }
                self.state = if trusted || matches!(self.pairing.state(), PairingState::Trusted) {
                    ConnectionState::Connected
                } else { ConnectionState::Handshaking };
            }
            Message::PairConfirm { device_id, short_code } => {
                outgoing.push(self.pairing.confirm(&device_id, &short_code)?);
                self.state = ConnectionState::Connected;
            }
            Message::Ping => outgoing.push(Message::Pong),
            Message::Pong => {}
            ref message @ (Message::IncomingCall { .. } | Message::CallAnswer | Message::CallDecline | Message::CallEnded | Message::PhoneBluetoothStatus { .. }) => {
                if !matches!(self.state, ConnectionState::Connected) { bail!("call message before pairing completes"); }
                if let Some(calls) = &self.calls {
                    let controller = crate::call::CallController::new(calls.clone(), crate::call::hfp::create_backend());
                    if let Some(response) = controller.handle(message).await.map_err(anyhow::Error::msg)? { outgoing.push(response); }
                }
            }
            _ => {
                if !matches!(self.state, ConnectionState::Connected) { bail!("message is not allowed before pairing completes"); }
            }
        }
        Ok(outgoing)
    }
}

impl Default for ControlSession { fn default() -> Self { Self::new() } }
