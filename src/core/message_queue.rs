use super::Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageQueue {
  pub conversation_id: Uuid,
  pub messages: Vec<Message>,
}

impl MessageQueue {
  pub fn new(conversation_id: Uuid) -> Self {
    Self {
      conversation_id,
      messages: vec![],
    }
  }
}
