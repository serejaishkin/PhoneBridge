//! Routing for pairing decisions from the desktop UI to the live TLS session.

use crate::ui::PairingUiCommand;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[derive(Clone, Default)]
pub struct PairingCommandHub {
    peers: Arc<Mutex<HashMap<String, mpsc::Sender<PairingUiCommand>>>>,
}

impl PairingCommandHub {
    pub fn new() -> Self { Self::default() }

    pub async fn register(&self, device_id: String) -> mpsc::Receiver<PairingUiCommand> {
        let (tx, rx) = mpsc::channel(8);
        self.peers.lock().await.insert(device_id, tx);
        rx
    }

    pub async fn unregister(&self, device_id: &str) {
        self.peers.lock().await.remove(device_id);
    }

    /// Route a GUI command only to the live session owning this device identity.
    pub async fn send(&self, command: PairingUiCommand) -> bool {
        let device_id = match &command {
            PairingUiCommand::Approve { device_id, .. } => device_id,
            PairingUiCommand::Reject { device_id, .. } => device_id,
            PairingUiCommand::Forget { device_id } => device_id,
        };
        let tx = self.peers.lock().await.get(device_id).cloned();
        match tx { Some(tx) => tx.send(command).await.is_ok(), None => false }
    }
}
