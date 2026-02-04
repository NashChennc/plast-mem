use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
  User,
  Assistant,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
  pub role: MessageRole,
  pub content: String,
  #[serde(
    with = "chrono::serde::ts_milliseconds_option",
    skip_serializing_if = "Option::is_none"
  )]
  pub date: Option<DateTime<Utc>>,
}
