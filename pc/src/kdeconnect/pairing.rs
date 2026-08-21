//! UI-independent KDE Connect pairing state machine.
//!
//! The state machine mirrors the protocol rule that devices must be paired
//! before normal packets are accepted. Network/TLS code can be attached later.

use super::packet::{IdentityPacket, Packet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingState {
    Idle,
    IncomingRequest,
    AwaitingRemote,
    Paired,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingDecision {
    Accept,
    Reject,
}

#[derive(Debug, Clone)]
pub enum PairingEvent {
    Request { identity: IdentityPacket },
    Accepted { peer_id: String },
    Rejected { peer_id: String },
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PairingSession {
    pub state: PairingState,
    pub peer: Option<IdentityPacket>,
}

impl Default for PairingSession {
    fn default() -> Self {
        Self { state: PairingState::Idle, peer: None }
    }
}

impl PairingSession {
    pub fn receive_identity(&mut self, identity: IdentityPacket) -> Result<PairingEvent, String> {
        if identity.protocol_version < 7 {
            return Err("unsupported KDE Connect protocol version".into());
        }
        self.peer = Some(identity.clone());
        self.state = PairingState::IncomingRequest;
        Ok(PairingEvent::Request { identity })
    }

    pub fn make_pair_request(&mut self) -> Packet {
        self.state = PairingState::AwaitingRemote;
        Packet::pair_request(0, unix_seconds())
    }

    pub fn decide(&mut self, decision: PairingDecision) -> Result<(Packet, PairingEvent), String> {
        let peer_id = self.peer.as_ref().map(|p| p.device_id.clone())
            .ok_or_else(|| "cannot decide pairing without a peer identity".to_string())?;

        match decision {
            PairingDecision::Accept => {
                self.state = PairingState::Paired;
                Ok((Packet::pair_response(0, true), PairingEvent::Accepted { peer_id }))
            }
            PairingDecision::Reject => {
                self.state = PairingState::Rejected;
                Ok((Packet::pair_response(0, false), PairingEvent::Rejected { peer_id }))
            }
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.state == PairingState::Paired
    }
}

fn unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phone() -> IdentityPacket {
        IdentityPacket {
            device_id: "0123456789abcdef0123456789abcdef".into(),
            device_name: "Test Phone".into(),
            device_type: "phone".into(),
            incoming_capabilities: vec![],
            outgoing_capabilities: vec![],
            protocol_version: 8,
            transport_fields: Default::default(),
        }
    }

    #[test]
    fn accept_moves_session_to_paired() {
        let mut session = PairingSession::default();
        session.receive_identity(phone()).unwrap();
        let (_, event) = session.decide(PairingDecision::Accept).unwrap();
        assert!(matches!(event, PairingEvent::Accepted { .. }));
        assert!(session.is_authenticated());
    }
}
