## asgard-rust (REST API)

REST API en **Rust** con **Axum + SQLx + PostgreSQL**, siguiendo **Clean Architecture**.

### Requisitos

- **Rust** (edition 2021)
- **Docker** (para PostgreSQL)

### Estructura (Clean Architecture)

- `src/domain/`: modelos del dominio (`User`, `Product`, `Order`)
- `src/application/`: puertos (traits) y casos de uso (services)
- `src/adapters/`:
  - `web/`: HTTP (Axum) router/handlers + mapeo de errores
  - `db/`: repositorios PostgreSQL (SQLx)
- `src/infrastructure/`: config, pool, migraciones, logging
- `migrations/`: migraciones SQL (SQLx)
- `tests/`: tests de integración HTTP + BD

### Levantar PostgreSQL (Docker)

```bash
docker compose up -d db
```

### Configuración

Copia `env.example` a `.env` y ajusta valores:

- `DATABASE_URL` (requerido)
- `APP_HOST` (default `127.0.0.1`)
- `APP_PORT` (default `8080`)
- `RUST_LOG` (default `info`)

### Ejecutar la API

```bash
cargo run
```

### Endpoints

- `GET /health`
- `GET /users` / `POST /users`
- `GET /users/:id` / `PUT /users/:id` / `DELETE /users/:id`
- `GET /products` / `POST /products`
- `GET /products/:id` / `PUT /products/:id` / `DELETE /products/:id`
- `GET /orders` / `POST /orders`
- `GET /orders/:id` / `PUT /orders/:id` / `DELETE /orders/:id`

### Tests

Con PostgreSQL levantado y `DATABASE_URL` configurada:

```bash
cargo test
```

### Benchmarks

Para medir el rendimiento de la API, ejecuta los benchmarks con:

```bash
cargo bench
```

Los benchmarks evalúan la capacidad y rendimiento de los siguientes endpoints:

- `GET /health` - Health check (sin base de datos)
- `GET /users`, `/products`, `/orders` - Listado de recursos
- `GET /users/:id` - Obtener recurso por ID
- `POST /users`, `/products`, `/orders` - Crear recursos
- `PUT /users/:id` - Actualizar recurso
- `DELETE /orders/:id` - Eliminar recurso

**Requisitos:**
- PostgreSQL debe estar levantado
- `DATABASE_URL` debe estar configurada
- Los benchmarks crearán datos de prueba temporalmente

Los resultados se guardan en `target/criterion/` con reportes HTML detallados.


