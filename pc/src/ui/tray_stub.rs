use std::sync::mpsc::{channel, Sender, Receiver};

#[derive(Debug, Clone)]
pub enum TrayCommand {
    AnswerCall,
    EndCall,
    ToggleMute,
    OpenSettings,
    Quit,
}

pub struct AsyncTrayUI;

impl AsyncTrayUI {
    pub fn new() -> Result<(Self, Sender<TrayCommand>), String> {
        let (_tx, _rx) = channel::<TrayCommand>();
        Ok((Self, _tx))
    }

    pub async fn poll(&self) -> Vec<TrayCommand> {
        Vec::new()
    }

    pub async fn notify(&self, _title: &str, _message: &str) {
        // No-op on non-Windows
    }
}
