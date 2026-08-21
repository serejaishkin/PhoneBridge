//! KDE Connect compatibility layer.
//!
//! This module intentionally starts small: it models the core identity/pairing
//! packets and keeps pairing state independent from the desktop UI. The next
//! migration step can replace the old PhoneBridge pairing protocol without
//! forcing the GUI to know anything about sockets or TLS.

pub mod packet;
pub mod pairing;

pub use packet::{IdentityPacket, PairPacket, Packet};
pub use pairing::{PairingDecision, PairingEvent, PairingSession, PairingState};
