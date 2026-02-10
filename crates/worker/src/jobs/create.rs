use plast_mem_core::{EpisodicMemory, Message};
use plast_mem_db_schema::episodic_memory;
use plast_mem_shared::AppError;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::jobs::WorkerError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateEpisodicMemoryJob {
  pub conversation_id: Uuid,
  pub segment_messages: Vec<Message>,
}

pub async fn handle_create_job(
  job: CreateEpisodicMemoryJob,
  db: DatabaseConnection,
) -> Result<(), WorkerError> {
  let episodic = EpisodicMemory::new(job.conversation_id, job.segment_messages).await?;
  let model = episodic.to_model()?;
  let active_model: episodic_memory::ActiveModel = model.into();

  episodic_memory::Entity::insert(active_model)
    .exec(&db)
    .await
    .map_err(AppError::from)?;

  Ok(())
}
