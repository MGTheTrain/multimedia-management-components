# Models

## Migrations

### Option A — Manual (CLI)

**Prerequisite:** `export DATABASE_URL=postgres://user:password@localhost:5432/dbname`
```bash
diesel setup                             # creates migrations/ and diesel.toml
diesel migration generate create_tables  # creates up.sql and down.sql
diesel migration run                     # applies up.sql, regenerates src/schema.rs
diesel migration redo                    # runs down.sql then up.sql (verify rollback)
# Ref: https://diesel.rs/guides/getting-started/#setup-diesel-for-your-project
```

After `run`, check that `src/schema.rs` aligns with the domain structs in `container_meta.rs` and `track.rs`.

### Option B — Automatic (at runtime)

`psql_repository.rs` calls `run_pending_migrations(MIGRATIONS)` on startup via `embed_migrations!`, so no CLI steps are needed.