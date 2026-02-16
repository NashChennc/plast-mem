# plastmem_migration

Database schema migrations using Sea-ORM Migration.

## Overview

Manages database schema evolution. Migrations are applied automatically
at application startup by the main `plastmem` binary.

## Migrations

| File | Description |
|------|-------------|
| `m20260216_01_create_message_queue_table.rs` | Creates `message_queue` table |
| `m20260216_02_create_episodic_memory_table.rs` | Creates `episodic_memory` table with pgvector support |

## Running Migrations

### Programmatically (recommended)

```rust
use plastmem_migration::Migrator;
use sea_orm_migration::MigratorTrait;

Migrator::up(db, None).await?;
```

### CLI

```bash
# Install sea-orm-cli
cargo install sea-orm-cli

# Run pending migrations
sea-orm-cli migrate up

# Create new migration
sea-orm-cli migrate generate create_new_table
```

## Requirements

- PostgreSQL 14+
- `pgvector` extension (automatically enabled by migrations)

## Creating New Migrations

1. Generate migration file:
   ```bash
   sea-orm-cli migrate generate your_migration_name
   ```

2. Implement `up` and `down` in the generated file

3. Test migration:
   ```bash
   sea-orm-cli migrate up
   sea-orm-cli migrate down
   ```

4. Regenerate entities in `crates/entities`
