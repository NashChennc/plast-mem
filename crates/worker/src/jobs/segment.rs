use apalis::prelude::TaskSink;
use apalis_postgres::PostgresStorage;
use plast_mem_core::{Message, MessageQueue, MessageRole, SegmentDecision, rule_segmenter};
use plast_mem_llm::{InputMessage, Role, decide_split};
use plast_mem_shared::AppError;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::jobs::{CreateEpisodicMemoryJob, WorkerError, WorkerJob};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageQueueSegmentJob {
  pub conversation_id: Uuid,
}

fn to_input_messages(messages: &[Message]) -> Vec<InputMessage> {
  messages
    .iter()
    .map(|m| InputMessage {
      role: match m.role {
        MessageRole::User => Role::User,
        MessageRole::Assistant => Role::Assistant,
      },
      content: m.content.clone(),
    })
    .collect()
}

pub async fn handle_segment_job(
  job: MessageQueueSegmentJob,
  db: DatabaseConnection,
  mut backend: PostgresStorage<WorkerJob>,
) -> Result<(), WorkerError> {
  let queue = MessageQueue::get(job.conversation_id, &db).await?;
  let messages = queue.messages;
  let Some(incoming) = messages.last().cloned() else {
    return Ok(());
  };

  let recent = &messages[..messages.len().saturating_sub(1)];
  let decision = rule_segmenter(recent, &incoming);

  let should_split = match decision {
    SegmentDecision::Split => true,
    SegmentDecision::NoSplit => false,
    SegmentDecision::CallLlm => {
      let recent_input = to_input_messages(recent);
      let incoming_input = to_input_messages(std::slice::from_ref(&incoming))
        .pop()
        .expect("incoming message exists");
      decide_split(&recent_input, &incoming_input).await?
    }
  };

  if should_split {
    let segment_messages = recent.to_vec();

    // Atomically drain segment from queue front, preserving any newly pushed messages
    MessageQueue::drain(job.conversation_id, segment_messages.len(), &db).await?;

    backend
      .push(WorkerJob::Create(CreateEpisodicMemoryJob {
        conversation_id: job.conversation_id,
        segment_messages,
      }))
      .await
      .map_err(AppError::from)?;
  }

  Ok(())
}
