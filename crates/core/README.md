# plastmem_core

Core domain logic for Plast Mem.

## Overview

This crate implements the central business logic:

- **Episodic Memory** - Storage and retrieval of conversation segments
- **Message Queue** - Buffering and event segmentation
- **Boundary Detection** - Dual-channel (surprise + topic) event boundary detection
- **Episode Creation** - LLM-based title/summary generation with FSRS initialization

## Key Types

### EpisodicMemory

The main memory type representing a conversation segment:

```rust
pub struct EpisodicMemory {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub messages: Vec<Message>,     // Original conversation
    pub title: String,              // LLM-generated title
    pub summary: String,            // LLM-generated summary
    pub stability: f32,             // FSRS stability
    pub difficulty: f32,            // FSRS difficulty
    pub surprise: f32,              // 0.0-1.0 significance score
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    // ... timestamps
}
```

### MessageQueue

Per-conversation message buffer:

```rust
// Load or create queue for a conversation
let queue = MessageQueue::get(conversation_id, db).await?;

// Push a message
queue.push(message, db).await?;

// Check segmentation rules
let check = queue.check_segmentation(db).await?;
```

### PendingReview

Tracks memories retrieved for later review:

```rust
pub struct PendingReview {
    pub query: String,              // The search query
    pub memory_ids: Vec<Uuid>,      // Retrieved memory IDs
}
```

## Key Functions

### Retrieval

```rust
use plastmem_core::{EpisodicMemory, DetailLevel};

// Hybrid search with FSRS re-ranking
let results = EpisodicMemory::retrieve(
    "user query",
    5,                              // limit
    Some(conversation_id),         // scope (None for global)
    db,
).await?;
```

### Episode Creation

```rust
use plastmem_core::create_episode;

// Create episode from buffered messages
let created = create_episode(
    conversation_id,
    &messages,
    drain_count,                   // How many messages to drain
    next_event_embedding,          // Embedding for next event context
    surprise_signal,               // 0.0-1.0 surprise score
    db,
).await?;
```

### Boundary Detection

```rust
use plastmem_core::detect_boundary;

let result = detect_boundary(
    &messages,
    event_model,
    last_embedding,
    event_model_embedding,
).await?;

// result.is_boundary indicates if a boundary was detected
```

## Modules

- `memory/episodic.rs` - `EpisodicMemory` struct and retrieval
- `memory/creation.rs` - Episode generation and FSRS initialization
- `memory/mod.rs` - Tool result formatting, `DetailLevel` enum
- `message_queue/mod.rs` - `MessageQueue` and `PendingReview`
- `message_queue/boundary.rs` - Dual-channel boundary detection
- `message_queue/segmentation.rs` - Rule-based segmentation triggers
- `message_queue/state.rs` - Queue state management
- `message_queue/pending_reviews.rs` - Review tracking helpers

## Architecture Notes

- Core logic is pure domain code - no HTTP or job queue specifics
- Database operations use Sea-ORM
- LLM calls go through `plastmem_ai`
