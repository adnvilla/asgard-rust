use asgard_rust::adapters::db;
use asgard_rust::application::services::{OrderService, ProductService, UserService};
use asgard_rust::infrastructure::db as infra_db;
use asgard_rust::{build_app, AppState};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use hyper::body::to_bytes;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup() -> Option<(PgPool, AppState)> {
  let database_url = match std::env::var("DATABASE_URL") {
    Ok(v) => v,
    Err(_) => return None,
  };

  let pool = infra_db::create_pool(&database_url).await.ok()?;
  infra_db::run_migrations(&pool).await.ok()?;
  sqlx::query("TRUNCATE orders, products, users CASCADE")
    .execute(&pool)
    .await
    .ok()?;

  let users_repo = db::users_repo::PgUserRepository::new(pool.clone());
  let products_repo = db::products_repo::PgProductRepository::new(pool.clone());
  let orders_repo = db::orders_repo::PgOrderRepository::new(pool.clone());

  let state = AppState {
    pool: pool.clone(),
    users: Arc::new(UserService::new(users_repo)),
    products: Arc::new(ProductService::new(products_repo)),
    orders: Arc::new(OrderService::new(orders_repo)),
    config: asgard_rust::infrastructure::config::AppConfig {
      host: "127.0.0.1".into(),
      port: 0,
      database_url,
    },
  };

  Some((pool, state))
}

#[tokio::test]
async fn health_is_ok_with_db() {
  let Some((_pool, state)) = setup().await else { return };
  let app = build_app(state);

  let res = app
    .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
    .await
    .unwrap();

  assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn users_products_orders_crud_smoke() {
  let Some((_pool, state)) = setup().await else { return };
  let app = build_app(state);

  // Create user
  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"email":"u1@example.com","name":"User 1"}"#))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::CREATED);
  let body = to_bytes(res.into_body()).await.unwrap();
  let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
  let user_id: Uuid = user.get("id").unwrap().as_str().unwrap().parse().unwrap();

  // Create product
  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/products")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"sku":"sku-1","name":"Prod 1","price_cents":1234}"#))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::CREATED);

  // Create order (depends on user)
  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/orders")
        .header("content-type", "application/json")
        .body(Body::from(format!(
          r#"{{"user_id":"{user_id}","status":"created","total_cents":1234}}"#
        )))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::CREATED);
  let body = to_bytes(res.into_body()).await.unwrap();
  let order: serde_json::Value = serde_json::from_slice(&body).unwrap();
  let order_id: Uuid = order.get("id").unwrap().as_str().unwrap().parse().unwrap();

  // Get order
  let res = app
    .clone()
    .oneshot(Request::builder().uri(format!("/orders/{order_id}")).body(Body::empty()).unwrap())
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);

  // Update order
  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri(format!("/orders/{order_id}"))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"status":"paid"}"#))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);

  // Delete order
  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri(format!("/orders/{order_id}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NO_CONTENT);
}


