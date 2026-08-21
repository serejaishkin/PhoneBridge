//! Minimal KDE Connect core packet model used during the migration.
//!
//! KDE Connect's current protocol uses JSON packets with `id`, `type` and
//! `body`. The protocol reference documents identity and pair packets as the
//! core negotiation messages.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Packet {
    pub id: u64,
    #[serde(rename = "type")]
    pub packet_type: String,
    pub body: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdentityPacket {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub incoming_capabilities: Vec<String>,
    pub outgoing_capabilities: Vec<String>,
    pub protocol_version: u8,
    #[serde(flatten)]
    pub transport_fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairPacket {
    pub pair: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

impl Packet {
    pub fn identity(id: u64, identity: &IdentityPacket) -> anyhow::Result<Self> {
        Ok(Self {
            id,
            packet_type: "kdeconnect.identity".into(),
            body: serde_json::to_value(identity)?,
        })
    }

    pub fn pair_request(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            packet_type: "kdeconnect.pair".into(),
            body: serde_json::json!({"pair": true, "timestamp": timestamp}),
        }
    }

    pub fn pair_response(id: u64, accepted: bool) -> Self {
        Self {
            id,
            packet_type: "kdeconnect.pair".into(),
            body: serde_json::json!({"pair": accepted}),
        }
    }

    pub fn as_pair(&self) -> anyhow::Result<PairPacket> {
        if self.packet_type != "kdeconnect.pair" {
            anyhow::bail!("not a KDE Connect pair packet")
        }
        Ok(serde_json::from_value(self.body.clone())?)
    }

    pub fn to_line(&self) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string(self)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_request_round_trips() {
        let packet = Packet::pair_request(7, 1_700_000_000);
        let encoded = packet.to_line().unwrap();
        let decoded: Packet = serde_json::from_str(encoded.trim()).unwrap();
        assert_eq!(decoded.as_pair().unwrap().pair, true);
        assert_eq!(decoded.as_pair().unwrap().timestamp, Some(1_700_000_000));
    }
}
