use super::hfp::HfpBackend;
use super::{CallState, SharedState};
use crate::protocol::Message;
use std::sync::Arc;

/// Translates authenticated PhoneBridge call commands into platform HFP actions.
pub struct CallController {
    state: Arc<SharedState>,
    hfp: Box<dyn HfpBackend>,
}

impl CallController {
    pub fn new(state: Arc<SharedState>, hfp: Box<dyn HfpBackend>) -> Self {
        Self { state, hfp }
    }

    pub async fn capability_message(&self) -> Message {
        Message::PcBluetoothStatus {
            hfp_supported: self.hfp.support(),
        }
    }

    pub async fn handle(&self, message: &Message) -> Result<Option<Message>, String> {
        match message {
            Message::IncomingCall { caller_number, caller_name } => {
                *self.state.call.lock().await = CallState::Ringing {
                    caller_number: caller_number.clone(),
                    caller_name: caller_name.clone(),
                };
                Ok(None)
            }
            Message::CallAnswer => {
                self.hfp.answer_call()?;
                *self.state.call.lock().await = CallState::Active;
                Ok(None)
            }
            Message::CallDecline => {
                self.hfp.decline_call()?;
                *self.state.call.lock().await = CallState::Idle;
                Ok(None)
            }
            Message::CallEnded => {
                self.hfp.end_call()?;
                *self.state.call.lock().await = CallState::Idle;
                Ok(None)
            }
            Message::PhoneBluetoothStatus { hfp_calls_toggle_enabled } => {
                log::debug!("Android HFP call control enabled={hfp_calls_toggle_enabled}");
                Ok(Some(self.capability_message().await))
            }
            _ => Ok(None),
        }
    }
}
