use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{Action, ProtocolError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFrame {
    action: Action,
    payload: String,
}

impl TextFrame {
    pub fn new(action: Action, payload: impl Into<String>) -> Self {
        Self {
            action,
            payload: payload.into(),
        }
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }

    pub fn decode_payload<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(serde_json::from_str(&self.payload)?)
    }

    pub fn into_parts(self) -> (Action, String) {
        (self.action, self.payload)
    }
}

pub fn encode_text_frame<T>(action: Action, payload: &T) -> Result<String>
where
    T: Serialize,
{
    let action = action.as_str();
    let payload = serde_json::to_string(payload)?;

    Ok(format!("{action}|{payload}"))
}

pub fn decode_text_frame(frame: &str) -> Result<TextFrame> {
    let (action, payload) = frame
        .split_once('|')
        .ok_or(ProtocolError::MissingSeparator)?;

    if action.is_empty() {
        return Err(ProtocolError::EmptyAction);
    }

    Ok(TextFrame::new(Action::try_from(action)?, payload))
}
