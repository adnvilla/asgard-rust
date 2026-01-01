use crate::adapters::web::error::ApiError;
use crate::application::ports::{
  NewOrder, NewProduct, NewUser, UpdateOrder, UpdateProduct, UpdateUser,
};
use crate::{AppState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

pub fn router(state: AppState) -> Router {
  Router::new()
    .route("/health", get(health))
    .route("/users", post(create_user).get(list_users))
    .route(
      "/users/:id",
      get(get_user).put(update_user).delete(delete_user),
    )
    .route("/products", post(create_product).get(list_products))
    .route(
      "/products/:id",
      get(get_product).put(update_product).delete(delete_product),
    )
    .route("/orders", post(create_order).get(list_orders))
    .route(
      "/orders/:id",
      get(get_order).put(update_order).delete(delete_order),
    )
    .with_state(state)
    .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
    .layer(TraceLayer::new_for_http())
}

#[derive(Debug, Serialize)]
struct HealthResponse {
  status: &'static str,
  db: &'static str,
}

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
  let db_ok = db_health(state.pool.clone()).await;
  Ok(Json(HealthResponse { status: "ok", db: if db_ok { "ok" } else { "error" } }))
}

async fn db_health(pool: PgPool) -> bool {
  let fut = sqlx::query_scalar::<_, i64>("SELECT 1").fetch_one(&pool);
  tokio::time::timeout(Duration::from_secs(2), fut)
    .await
    .ok()
    .and_then(|r| r.ok())
    .is_some()
}

// ===== Users =====

#[derive(Debug, Deserialize)]
struct CreateUserBody {
  email: String,
  name: String,
}

async fn create_user(
  State(state): State<AppState>,
  Json(body): Json<CreateUserBody>,
) -> Result<(StatusCode, Json<crate::domain::models::User>), ApiError> {
  let user = state
    .users
    .create(NewUser { email: body.email, name: body.name })
    .await
    .map_err(ApiError::from)?;
  Ok((StatusCode::CREATED, Json(user)))
}

async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<crate::domain::models::User>>, ApiError> {
  let users = state.users.list().await.map_err(ApiError::from)?;
  Ok(Json(users))
}

async fn get_user(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
) -> Result<Json<crate::domain::models::User>, ApiError> {
  let user = state.users.get(id).await.map_err(ApiError::from)?;
  Ok(Json(user))
}

#[derive(Debug, Deserialize)]
struct UpdateUserBody {
  email: Option<String>,
  name: Option<String>,
}

async fn update_user(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
  Json(body): Json<UpdateUserBody>,
) -> Result<Json<crate::domain::models::User>, ApiError> {
  let user = state
    .users
    .update(
      id,
      UpdateUser {
        email: body.email,
        name: body.name,
      },
    )
    .await
    .map_err(ApiError::from)?;
  Ok(Json(user))
}

async fn delete_user(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
  state.users.delete(id).await.map_err(ApiError::from)?;
  Ok(StatusCode::NO_CONTENT)
}

// ===== Products =====

#[derive(Debug, Deserialize)]
struct CreateProductBody {
  sku: String,
  name: String,
  price_cents: i64,
}

async fn create_product(
  State(state): State<AppState>,
  Json(body): Json<CreateProductBody>,
) -> Result<(StatusCode, Json<crate::domain::models::Product>), ApiError> {
  let product = state
    .products
    .create(NewProduct { sku: body.sku, name: body.name, price_cents: body.price_cents })
    .await
    .map_err(ApiError::from)?;
  Ok((StatusCode::CREATED, Json(product)))
}

async fn list_products(
  State(state): State<AppState>,
) -> Result<Json<Vec<crate::domain::models::Product>>, ApiError> {
  let products = state.products.list().await.map_err(ApiError::from)?;
  Ok(Json(products))
}

async fn get_product(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
) -> Result<Json<crate::domain::models::Product>, ApiError> {
  let product = state.products.get(id).await.map_err(ApiError::from)?;
  Ok(Json(product))
}

#[derive(Debug, Deserialize)]
struct UpdateProductBody {
  sku: Option<String>,
  name: Option<String>,
  price_cents: Option<i64>,
}

async fn update_product(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
  Json(body): Json<UpdateProductBody>,
) -> Result<Json<crate::domain::models::Product>, ApiError> {
  let product = state
    .products
    .update(
      id,
      UpdateProduct { sku: body.sku, name: body.name, price_cents: body.price_cents },
    )
    .await
    .map_err(ApiError::from)?;
  Ok(Json(product))
}

async fn delete_product(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
  state.products.delete(id).await.map_err(ApiError::from)?;
  Ok(StatusCode::NO_CONTENT)
}

// ===== Orders =====

#[derive(Debug, Deserialize)]
struct CreateOrderBody {
  user_id: Uuid,
  status: String,
  total_cents: i64,
}

async fn create_order(
  State(state): State<AppState>,
  Json(body): Json<CreateOrderBody>,
) -> Result<(StatusCode, Json<crate::domain::models::Order>), ApiError> {
  let order = state
    .orders
    .create(NewOrder { user_id: body.user_id, status: body.status, total_cents: body.total_cents })
    .await
    .map_err(ApiError::from)?;
  Ok((StatusCode::CREATED, Json(order)))
}

async fn list_orders(
  State(state): State<AppState>,
) -> Result<Json<Vec<crate::domain::models::Order>>, ApiError> {
  let orders = state.orders.list().await.map_err(ApiError::from)?;
  Ok(Json(orders))
}

async fn get_order(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
) -> Result<Json<crate::domain::models::Order>, ApiError> {
  let order = state.orders.get(id).await.map_err(ApiError::from)?;
  Ok(Json(order))
}

#[derive(Debug, Deserialize)]
struct UpdateOrderBody {
  status: Option<String>,
  total_cents: Option<i64>,
}

async fn update_order(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
  Json(body): Json<UpdateOrderBody>,
) -> Result<Json<crate::domain::models::Order>, ApiError> {
  let order = state
    .orders
    .update(id, UpdateOrder { status: body.status, total_cents: body.total_cents })
    .await
    .map_err(ApiError::from)?;
  Ok(Json(order))
}

async fn delete_order(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
  state.orders.delete(id).await.map_err(ApiError::from)?;
  Ok(StatusCode::NO_CONTENT)
}


