//! PC-side SMS state and control API.

use crate::protocol::Message;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

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
    pub fn new() -> Self { Self::default() }
    pub fn upsert(&mut self, message: SmsMessage) {
        if let Some(existing) = self.messages.iter_mut().find(|m| m.id == message.id) {
            *existing = message;
            return;
        }
        self.messages.push(message);
        self.messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        self.messages.truncate(500);
    }
    pub fn add_received(&mut self, address: String, body: String, timestamp: i64) {
        let id = format!("rx:{address}:{timestamp}:{body}");
        self.upsert(SmsMessage { id, address, body, timestamp });
    }
    pub fn all(&self) -> Vec<SmsMessage> { self.messages.clone() }
    pub fn len(&self) -> usize { self.messages.len() }
}

#[derive(Clone, Default)]
pub struct SmsController {
    tx: Arc<Mutex<Option<mpsc::Sender<Message>>>>,
}

impl SmsController {
    pub fn new() -> Self { Self::default() }
    pub(crate) async fn attach(&self, tx: mpsc::Sender<Message>) { *self.tx.lock().await = Some(tx); }
    pub(crate) async fn detach(&self) { *self.tx.lock().await = None; }

    pub async fn send_message(&self, message: Message) -> bool {
        match self.tx.lock().await.clone() {
            Some(tx) => tx.send(message).await.is_ok(),
            None => false,
        }
    }

    pub async fn send_sms(&self, address: impl Into<String>, body: impl Into<String>) -> bool {
        self.send_message(Message::SmsSend { address: address.into(), body: body.into() }).await
    }

    pub async fn request_history(&self) -> bool { self.send_message(Message::SmsList).await }
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
