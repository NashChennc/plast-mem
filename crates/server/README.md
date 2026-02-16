# plastmem_server

HTTP API server for Plast Mem.

## Overview

Axum-based HTTP server providing REST API endpoints for:
- Adding messages to conversations
- Retrieving episodic memories

Includes OpenAPI documentation served via Scalar UI.

## API Endpoints

### POST /api/v0/add_message

Add a message to a conversation queue (triggers background segmentation):

```json
{
  "conversation_id": "550e8400-e29b-41d4-a716-446655440001",
  "message": {
    "role": "user",
    "content": "Hello, how are you?"
  }
}
```

### POST /api/v0/retrieve_memory

Search memories with hybrid retrieval (BM25 + vector + FSRS reranking).
Returns Markdown-formatted results optimized for LLM consumption:

```json
{
  "query": "what did we discuss about Rust",
  "conversation_id": "550e8400-e29b-41d4-a716-446655440001",
  "limit": 5,
  "detail": "auto",
  "scope": "550e8400-e29b-41d4-a716-446655440001"
}
```

Response format (Markdown):
```markdown
## Memory 1 [rank: 1, score: 0.92, key moment]
**When:** 2 days ago
**Summary:** User switching careers from Python to Rust...

**Details:**
- user: "I've been doing Python for 5 years..."
```

### POST /api/v0/retrieve_memory/raw

Same search, returns JSON:

```json
[
  {
    "id": "...",
    "title": "...",
    "summary": "...",
    "stability": 3.5,
    "difficulty": 5.0,
    "surprise": 0.85,
    "score": 0.92
  }
]
```

## Running the Server

```rust
use plastmem_server::server;

server(listener, app_state).await?;
```

## OpenAPI Documentation

Available at `/openapi/` when server is running:
- Interactive docs (Scalar UI)
- Raw spec at `/openapi.json`

## Request Types

### AddMessage

```rust
pub struct AddMessage {
    pub conversation_id: Uuid,
    pub message: AddMessageMessage,
}

pub struct AddMessageMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: Option<DateTime<Utc>>,
}
```

### RetrieveMemory

```rust
pub struct RetrieveMemory {
    pub conversation_id: Uuid,
    pub query: String,
    pub limit: u64,                  // default: 5
    pub detail: DetailLevel,         // default: Auto
    pub scope: Option<Uuid>,         // None = global search, Some(id) = conversation scope
}
```

The `scope` field is optional:
- `null` or omitted - search all conversations globally
- `uuid` - search only within the specified conversation

`DetailLevel` controls whether full message details are included:
- `None` - Never include details
- `Low` - Only rank 1 with high surprise
- `Auto` - Ranks 1-2 with surprise >= 0.7 (default)
- `High` - Always include details

## Architecture

- `api/` - HTTP handlers and request/response types
- `utils/` - Server utilities (AppState, shutdown handling)
- `server.rs` - Axum server setup

All endpoints return `Result<Json<T>, AppError>` with proper status codes.
