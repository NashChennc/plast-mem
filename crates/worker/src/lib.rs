use std::time::Duration;

use apalis::prelude::{Monitor, WorkerBuilder};
use apalis_postgres::PostgresStorage;
use plast_mem_shared::AppError;
use sea_orm::DatabaseConnection;

mod jobs;
pub use jobs::{
  CreateEpisodicMemoryJob, MessageQueueSegmentJob, WorkerJob, handle_create_job, handle_segment_job,
};

pub async fn worker(
  db: &DatabaseConnection,
  backend: PostgresStorage<WorkerJob>,
) -> Result<(), AppError> {
  let db = db.clone();

  Monitor::new()
    .register(move |_run_id| {
      let db = db.clone();
      let backend = backend.clone();

      WorkerBuilder::new("plast-mem-worker")
        .backend(backend.clone())
        .build(move |job: WorkerJob| {
          let db = db.clone();
          let backend = backend.clone();
          async move {
            match job {
              WorkerJob::Segment(job) => handle_segment_job(job, db, backend).await,
              WorkerJob::Create(job) => handle_create_job(job, db).await,
            }
          }
        })
    })
    .shutdown_timeout(Duration::from_secs(5))
    .run_with_signal(tokio::signal::ctrl_c())
    .await
    .map_err(|err| AppError::from(anyhow::Error::new(err)))?;

  Ok(())
}
