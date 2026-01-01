pub mod adapters;
pub mod application;
pub mod domain;
pub mod infrastructure;

use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

use crate::adapters::{db, web};
use crate::application::services::{OrderService, ProductService, UserService};
use crate::infrastructure::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
  pub pool: PgPool,
  pub users: Arc<UserService<db::users_repo::PgUserRepository>>,
  pub products: Arc<ProductService<db::products_repo::PgProductRepository>>,
  pub orders: Arc<OrderService<db::orders_repo::PgOrderRepository>>,
  pub config: AppConfig,
}

pub fn build_app(state: AppState) -> Router {
  web::router::router(state)
}


