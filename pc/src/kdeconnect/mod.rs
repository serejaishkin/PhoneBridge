//! KDE Connect compatibility layer.
//!
//! This module models protocol primitives independently from the desktop UI.
//! TLS identity/trust is kept here so transports can share the same security
//! boundary without recreating pairing state.

pub mod packet;
pub mod pairing;
pub mod tls;
pub mod tls_server;

pub use packet::{IdentityPacket, PairPacket, Packet};
pub use pairing::{PairingDecision, PairingEvent, PairingSession, PairingState};
pub use tls::{certificate_fingerprint, default_security_dir, LocalTlsIdentity, TrustStore, TrustedPeer};
pub use tls_server::{IncomingPairing, PairingResponder, TlsPairingServer, TLS_PAIRING_PORT};
