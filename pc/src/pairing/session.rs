//! Protocol-level pairing state independent from the platform layer.

use crate::protocol::{Message, PROTOCOL_VERSION};
use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingState {
    AwaitingHello,
    AwaitingConfirmation { device_id: String, fingerprint: String, code: String },
    Trusted,
}

pub struct PairingSession {
    state: PairingState,
}

impl PairingSession {
    pub fn new() -> Self {
        Self { state: PairingState::AwaitingHello }
    }

    pub fn state(&self) -> &PairingState {
        &self.state
    }

    pub fn accept_hello(
        &mut self,
        device_id: String,
        fingerprint: String,
        protocol_version: u32,
        trusted: bool,
        challenge_code: String,
    ) -> Result<Option<Message>> {
        if protocol_version != PROTOCOL_VERSION {
            bail!("protocol version mismatch: peer={protocol_version}, pc={PROTOCOL_VERSION}");
        }

        if trusted {
            self.state = PairingState::Trusted;
            return Ok(None);
        }

        self.state = PairingState::AwaitingConfirmation {
            device_id: device_id.clone(),
            fingerprint: fingerprint.clone(),
            code: challenge_code.clone(),
        };

        Ok(Some(Message::PairChallenge {
            device_id,
            fingerprint,
            short_code: challenge_code,
        }))
    }

    pub fn confirm(&mut self, device_id: &str, code: &str) -> Result<Message> {
        let PairingState::AwaitingConfirmation {
            device_id: expected_id,
            code: expected_code,
            ..
        } = &self.state else {
            bail!("pairing confirmation is not expected in the current state");
        };

        if expected_id != device_id {
            bail!("pairing device id mismatch");
        }
        if expected_code != code {
            bail!("pairing code mismatch");
        }

        self.state = PairingState::Trusted;
        Ok(Message::PairResult {
            device_id: device_id.to_owned(),
            trusted: true,
            message: "device paired successfully".into(),
        })
    }
}

impl Default for PairingSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untrusted_hello_creates_challenge() {
        let mut session = PairingSession::new();
        let result = session
            .accept_hello("phone".into(), "FP".into(), 1, false, "ABCD-EFGH".into())
            .unwrap();
        assert!(matches!(result, Some(Message::PairChallenge { .. })));
    }

    #[test]
    fn valid_confirmation_trusts_device() {
        let mut session = PairingSession::new();
        session
            .accept_hello("phone".into(), "FP".into(), 1, false, "ABCD-EFGH".into())
            .unwrap();
        let result = session.confirm("phone", "ABCD-EFGH").unwrap();
        assert!(matches!(result, Message::PairResult { trusted: true, .. }));
        assert_eq!(session.state(), &PairingState::Trusted);
    }
}
