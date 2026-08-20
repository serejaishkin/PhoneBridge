use crate::call::SharedState;
use crate::connection::state::ConnectionState;
use crate::pairing::session::{PairingSession, PairingState};
use crate::protocol::Message;
use anyhow::{bail, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(15);
const IDLE_TIMEOUT: Duration = Duration::from_secs(45);

pub struct ControlSession {
    state: ConnectionState,
    pairing: PairingSession,
    calls: Option<Arc<SharedState>>,
    last_activity: Instant,
}

impl ControlSession {
    pub fn new() -> Self { Self { state: ConnectionState::Handshaking, pairing: PairingSession::new(), calls: None, last_activity: Instant::now() } }
    pub fn with_calls(mut self, calls: Arc<SharedState>) -> Self { self.calls = Some(calls); self }
    pub fn state(&self) -> &ConnectionState { &self.state }
    pub fn pairing(&self) -> &PairingSession { &self.pairing }
    pub fn timeout(&self) -> Duration { let limit = if matches!(self.state, ConnectionState::Connected) { IDLE_TIMEOUT } else { HANDSHAKE_TIMEOUT }; limit.saturating_sub(self.last_activity.elapsed()) }
    pub fn is_expired(&self) -> bool { self.timeout().is_zero() }
    pub fn is_authenticated(&self) -> bool { matches!(self.state, ConnectionState::Connected) }

    pub async fn handle(&mut self, message: Message, trusted: bool) -> Result<Vec<Message>> { self.handle_with_peer(message, trusted, None).await }

    pub async fn handle_with_peer(&mut self, message: Message, trusted: bool, peer_fingerprint: Option<&str>) -> Result<Vec<Message>> {
        if self.is_expired() { bail!("control session timed out"); }
        self.last_activity = Instant::now();
        let mut outgoing = Vec::new();
        match message {
            Message::Hello { device_id, fingerprint, protocol_version, .. } => {
                if !matches!(self.state, ConnectionState::Handshaking) { bail!("Hello received after handshake"); }
                if let Some(challenge) = self.pairing.accept_hello(device_id, fingerprint.clone(), protocol_version, trusted, crate::pairing::trust::short_code(&fingerprint))? { outgoing.push(challenge); }
                self.state = if trusted || matches!(self.pairing.state(), PairingState::Trusted) { ConnectionState::Connected } else { ConnectionState::Handshaking };
            }
            Message::PairConfirm { device_id, short_code } => {
                let fingerprint = peer_fingerprint.ok_or_else(|| anyhow::anyhow!("peer fingerprint is required for pairing confirmation"))?;
                let result = self.pairing.confirm(&device_id, fingerprint, &short_code)?;
                if matches!(&result, Message::PairResult { trusted: true, .. }) { self.state = ConnectionState::Connected; }
                outgoing.push(result);
            }
            Message::PairApprove { device_id, short_code } => {
                let result = self.pairing.approve(&device_id, &short_code)?;
                if matches!(&result, Message::PairResult { trusted: true, .. }) { self.state = ConnectionState::Connected; }
                outgoing.push(result);
            }
            Message::PairReject { device_id, reason } => {
                outgoing.push(self.pairing.reject(&device_id, &reason)?);
            }
            Message::Ping => { if !self.is_authenticated() { bail!("Ping before pairing completes"); } outgoing.push(Message::Pong); }
            Message::Pong => {}
            Message::Disconnect { .. } => {}
            ref message @ (Message::IncomingCall { .. } | Message::CallAnswer | Message::CallDecline | Message::CallEnded | Message::PhoneBluetoothStatus { .. }) => {
                if !self.is_authenticated() { bail!("call message before pairing completes"); }
                if let Some(calls) = &self.calls {
                    let controller = crate::call::CallController::new(calls.clone(), crate::call::hfp::create_backend());
                    if let Some(response) = controller.handle(message).await.map_err(anyhow::Error::msg)? { outgoing.push(response); }
                }
            }
            _ => { if !self.is_authenticated() { bail!("message is not allowed before pairing completes"); } }
        }
        Ok(outgoing)
    }
}

impl Default for ControlSession { fn default() -> Self { Self::new() } }
