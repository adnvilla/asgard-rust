use asgard_rust::adapters::db;
use asgard_rust::application::services::{OrderService, ProductService, UserService};
use asgard_rust::infrastructure::db as infra_db;
use asgard_rust::{build_app, AppState};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use hyper::body::to_bytes;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::{Arc, Mutex, MutexGuard};
use tower::ServiceExt;
use uuid::Uuid;

static DB_LOCK: Mutex<()> = Mutex::new(());

async fn setup() -> Option<(PgPool, AppState, MutexGuard<'static, ()>)> {
  let guard = DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());

  let database_url = match std::env::var("DATABASE_URL") {
    Ok(v) => v,
    Err(_) => {
      if std::env::var("CI").is_ok() {
        panic!("DATABASE_URL must be set in CI");
      }
      return None;
    }
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

  Some((pool, state, guard))
}

fn json_id(body: &[u8]) -> Uuid {
  let value: Value = serde_json::from_slice(body).unwrap();
  value
    .get("id")
    .unwrap()
    .as_str()
    .unwrap()
    .parse()
    .unwrap()
}

#[tokio::test]
async fn health_is_ok_with_db() {
  let Some((_pool, state, _guard)) = setup().await else {
    return;
  };
  let app = build_app(state);

  let res = app
    .oneshot(
      Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn users_crud_and_conflict() {
  let Some((_pool, state, _guard)) = setup().await else {
    return;
  };
  let app = build_app(state);

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
  let user_id = json_id(&to_bytes(res.into_body()).await.unwrap());

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"email":"u1@example.com","name":"Dup"}"#))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::CONFLICT);

  let res = app
    .clone()
    .oneshot(Request::builder().uri("/users").body(Body::empty()).unwrap())
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);
  let list: Vec<Value> = serde_json::from_slice(&to_bytes(res.into_body()).await.unwrap()).unwrap();
  assert_eq!(list.len(), 1);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .uri(format!("/users/{user_id}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri(format!("/users/{user_id}"))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"name":"User 1 Updated"}"#))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri(format!("/users/{user_id}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NO_CONTENT);

  let missing = Uuid::new_v4();
  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .uri(format!("/users/{missing}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri(format!("/users/{missing}"))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"name":"Nope"}"#))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);

  let res = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri(format!("/users/{missing}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn products_crud_and_conflict() {
  let Some((_pool, state, _guard)) = setup().await else {
    return;
  };
  let app = build_app(state);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/products")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"sku":"sku-1","name":"Prod 1","price_cents":1234}"#,
        ))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::CREATED);
  let product_id = json_id(&to_bytes(res.into_body()).await.unwrap());

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/products")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"sku":"sku-1","name":"Dup","price_cents":1}"#,
        ))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::CONFLICT);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/products")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);
  let list: Vec<Value> = serde_json::from_slice(&to_bytes(res.into_body()).await.unwrap()).unwrap();
  assert_eq!(list.len(), 1);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .uri(format!("/products/{product_id}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri(format!("/products/{product_id}"))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"name":"Prod 1 Updated","price_cents":2000}"#))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri(format!("/products/{product_id}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NO_CONTENT);

  let missing = Uuid::new_v4();
  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .uri(format!("/products/{missing}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri(format!("/products/{missing}"))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"name":"Nope"}"#))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);

  let res = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri(format!("/products/{missing}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn orders_crud_and_not_found() {
  let Some((_pool, state, _guard)) = setup().await else {
    return;
  };
  let app = build_app(state);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"email":"buyer@example.com","name":"Buyer"}"#))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::CREATED);
  let user_id = json_id(&to_bytes(res.into_body()).await.unwrap());

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
  let order_id = json_id(&to_bytes(res.into_body()).await.unwrap());

  let res = app
    .clone()
    .oneshot(Request::builder().uri("/orders").body(Body::empty()).unwrap())
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);
  let list: Vec<Value> = serde_json::from_slice(&to_bytes(res.into_body()).await.unwrap()).unwrap();
  assert_eq!(list.len(), 1);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .uri(format!("/orders/{order_id}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);

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

  let missing = Uuid::new_v4();
  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .uri(format!("/orders/{missing}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);

  let res = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri(format!("/orders/{missing}"))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"status":"cancelled"}"#))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);

  let res = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri(format!("/orders/{missing}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_order_with_unknown_user_returns_conflict() {
  let Some((_pool, state, _guard)) = setup().await else {
    return;
  };
  let app = build_app(state);
  let missing_user = Uuid::new_v4();

  let res = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/orders")
        .header("content-type", "application/json")
        .body(Body::from(format!(
          r#"{{"user_id":"{missing_user}","status":"created","total_cents":100}}"#
        )))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(res.status(), StatusCode::CONFLICT);
}
