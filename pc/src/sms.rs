//! PC-side SMS state and control API.
//!
//! The transport itself is handled by pairing::server. This module keeps the
//! latest SMS history in memory and provides a small, platform-neutral API for
//! the future desktop UI to send SMS through the active Android connection.

use crate::protocol::Message;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmsMessage {
    pub id: String,
    pub address: String,
    pub body: String,
    pub timestamp: i64,
}

#[derive(Debug, Default)]
pub struct SmsStore {
    messages: Vec<SmsMessage>,
}

impl SmsStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&mut self, message: SmsMessage) {
        if let Some(existing) = self.messages.iter_mut().find(|m| m.id == message.id) {
            *existing = message;
            return;
        }
        self.messages.push(message);
        self.messages
            .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        self.messages.truncate(500);
    }

    pub fn add_received(&mut self, address: String, body: String, timestamp: i64) {
        let id = format!("rx:{address}:{timestamp}:{body}");
        self.upsert(SmsMessage {
            id,
            address,
            body,
            timestamp,
        });
    }

    pub fn all(&self) -> Vec<SmsMessage> {
        self.messages.clone()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

/// Handle shared by the daemon/UI to talk to the currently connected Android
/// device. The connection owns the actual WebSocket/TLS writer; this handle
/// only queues protocol messages to it.
#[derive(Clone, Default)]
pub struct SmsController {
    tx: Arc<Mutex<Option<mpsc::Sender<Message>>>>,
}

impl SmsController {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) async fn attach(&self, tx: mpsc::Sender<Message>) {
        *self.tx.lock().await = Some(tx);
    }

    pub(crate) async fn detach(&self) {
        *self.tx.lock().await = None;
    }

    pub async fn send_sms(&self, address: impl Into<String>, body: impl Into<String>) -> bool {
        let tx = self.tx.lock().await.clone();
        match tx {
            Some(tx) => tx
                .send(Message::SmsSend {
                    address: address.into(),
                    body: body.into(),
                })
                .await
                .is_ok(),
            None => false,
        }
    }

    pub async fn request_history(&self) -> bool {
        let tx = self.tx.lock().await.clone();
        match tx {
            Some(tx) => tx.send(Message::SmsList).await.is_ok(),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_deduplicates_and_sorts() {
        let mut store = SmsStore::new();
        store.add_received("+7999".into(), "old".into(), 10);
        store.add_received("+7999".into(), "new".into(), 20);
        assert_eq!(store.len(), 2);
        assert_eq!(store.all()[0].body, "new");

        store.add_received("+7999".into(), "new".into(), 20);
        assert_eq!(store.len(), 2);
    }
}
