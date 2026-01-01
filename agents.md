## Agent guide (AI / quick contributions)

### Goal

This repo implements a REST API using **Clean Architecture**:

- **Domain**: entities and pure logic (no frameworks)
- **Application**: use cases and outbound ports (traits)
- **Adapters**: concrete implementations (HTTP/DB)
- **Infrastructure**: configuration, wiring, migrations, logging

### Rules

- **Do not mix layers**:
  - `domain/` must not import anything from Axum/SQLx.
  - `application/` depends on `domain/` and defines **traits** (ports).
  - `adapters/` implements ports using Axum/SQLx.
- **Errors**:
  - In repositories, map DB errors to `RepoError::{NotFound,Conflict,Unexpected}`.
  - In the web layer, map `RepoError` to HTTP (`404`, `409`, `500`).
- **SQLx**:
  - Use runtime queries (`sqlx::query`) to avoid requiring a DB at compile time.
  - Keep migrations in `migrations/` and run them at boot.
- **Tests**:
  - Unit: application layer with fake/in-memory repos.
  - Integration: HTTP + PostgreSQL using `DATABASE_URL`.

### Add a new resource

- `domain/models.rs`: domain model (if applicable)
- `application/ports.rs`: inputs + repository trait
- `application/services.rs`: service/use case
- `adapters/db/*_repo.rs`: SQLx implementation
- `adapters/web/router.rs`: routes + handlers
- `migrations/`: schema changes if needed
- `tests/`: endpoint smoke test


