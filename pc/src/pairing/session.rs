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
    pub fn new() -> Self { Self { state: PairingState::AwaitingHello } }
    pub fn state(&self) -> &PairingState { &self.state }

    pub fn accept_hello(
        &mut self,
        device_id: String,
        fingerprint: String,
        protocol_version: u32,
        trusted: bool,
        challenge_code: String,
    ) -> Result<Option<Message>> {
        if protocol_version != PROTOCOL_VERSION { bail!("protocol version mismatch: peer={protocol_version}, pc={PROTOCOL_VERSION}"); }

        match &self.state {
            PairingState::Trusted if trusted => return Ok(None),
            PairingState::AwaitingConfirmation { device_id: old_id, fingerprint: old_fp, .. }
                if old_id == &device_id && old_fp == &fingerprint => {
                    bail!("duplicate Hello while pairing confirmation is pending");
                }
            PairingState::AwaitingConfirmation { .. } => bail!("pairing session already belongs to another device"),
            _ => {}
        }

        if trusted {
            self.state = PairingState::Trusted;
            return Ok(None);
        }

        self.state = PairingState::AwaitingConfirmation {
            device_id: device_id.clone(), fingerprint: fingerprint.clone(), code: challenge_code.clone(),
        };
        Ok(Some(Message::PairChallenge { device_id, fingerprint, short_code: challenge_code }))
    }

    pub fn confirm(&mut self, device_id: &str, fingerprint: &str, code: &str) -> Result<Message> {
        let PairingState::AwaitingConfirmation { device_id: expected_id, fingerprint: expected_fp, code: expected_code } = &self.state else {
            bail!("pairing confirmation is not expected in the current state");
        };
        if expected_id != device_id { bail!("pairing device id mismatch"); }
        if expected_fp != fingerprint { bail!("pairing fingerprint mismatch"); }
        if expected_code != code { bail!("pairing code mismatch"); }
        self.state = PairingState::Trusted;
        Ok(Message::PairResult { device_id: device_id.to_owned(), trusted: true, message: "device paired successfully".into() })
    }
}

impl Default for PairingSession { fn default() -> Self { Self::new() } }
