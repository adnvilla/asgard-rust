use asgard_rust::adapters::db;
use asgard_rust::application::services::{OrderService, ProductService, UserService};
use asgard_rust::infrastructure::{config::AppConfig, db as infra_db, logging};
use asgard_rust::{build_app, AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Load .env if present (optional).
  let _ = dotenvy::dotenv();

  logging::init_tracing();
  let config = AppConfig::from_env()?;

  let pool = infra_db::create_pool(&config.database_url).await?;
  infra_db::run_migrations(&pool).await?;

  let users_repo = db::users_repo::PgUserRepository::new(pool.clone());
  let products_repo = db::products_repo::PgProductRepository::new(pool.clone());
  let orders_repo = db::orders_repo::PgOrderRepository::new(pool.clone());

  let state = AppState {
    pool: pool.clone(),
    users: Arc::new(UserService::new(users_repo)),
    products: Arc::new(ProductService::new(products_repo)),
    orders: Arc::new(OrderService::new(orders_repo)),
    config: config.clone(),
  };

  let app = build_app(state);
  let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;

  info!("listening on http://{addr}");
  axum::Server::bind(&addr).serve(app.into_make_service()).await?;
  Ok(())
}


