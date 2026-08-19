use crate::connection::state::ConnectionState;
use crate::pairing::session::{PairingSession, PairingState};
use crate::protocol::Message;
use anyhow::{bail, Result};

/// Shared control-plane session used after TLS has been established.
/// Transport code only feeds messages into this state machine.
pub struct ControlSession {
    state: ConnectionState,
    pairing: PairingSession,
}

impl ControlSession {
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Handshaking,
            pairing: PairingSession::new(),
        }
    }

    pub fn state(&self) -> &ConnectionState { &self.state }
    pub fn pairing(&self) -> &PairingSession { &self.pairing }

    pub fn handle(&mut self, message: Message, trusted: bool) -> Result<Vec<Message>> {
        let mut outgoing = Vec::new();
        match message {
            Message::Hello { device_id, device_name: _, platform: _, protocol_version, fingerprint } => {
                if !matches!(self.state, ConnectionState::Handshaking) {
                    bail!("Hello received after handshake");
                }
                if let Some(challenge) = self.pairing.accept_hello(
                    device_id,
                    fingerprint,
                    protocol_version,
                    trusted,
                    crate::pairing::trust::short_code(&fingerprint),
                )? {
                    outgoing.push(challenge);
                }
                self.state = if trusted || matches!(self.pairing.state(), PairingState::Trusted) {
                    ConnectionState::Connected
                } else {
                    ConnectionState::Handshaking
                };
            }
            Message::PairConfirm { device_id, short_code } => {
                outgoing.push(self.pairing.confirm(&device_id, &short_code)?);
                self.state = ConnectionState::Connected;
            }
            Message::Ping => outgoing.push(Message::Pong),
            Message::Pong => {}
            _ => {
                if !matches!(self.state, ConnectionState::Connected) {
                    bail!("message is not allowed before pairing completes");
                }
            }
        }
        Ok(outgoing)
    }
}

impl Default for ControlSession {
    fn default() -> Self { Self::new() }
}
