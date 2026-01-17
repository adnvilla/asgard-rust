use asgard_rust::adapters::db;
use asgard_rust::application::services::{OrderService, ProductService, UserService};
use asgard_rust::infrastructure::db as infra_db;
use asgard_rust::{build_app, AppState};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use criterion::{criterion_group, criterion_main, Criterion};
use sqlx::PgPool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_app() -> Option<(AppState, axum::Router)> {
  let database_url = match std::env::var("DATABASE_URL") {
    Ok(v) => v,
    Err(_) => {
      eprintln!("ERROR: DATABASE_URL environment variable is not set");
      eprintln!("Please set DATABASE_URL before running benchmarks");
      return None;
    }
  };

  let pool = match infra_db::create_pool(&database_url).await {
    Ok(p) => p,
    Err(e) => {
      eprintln!("ERROR: Failed to create database pool: {}", e);
      eprintln!("Make sure PostgreSQL is running and DATABASE_URL is correct");
      return None;
    }
  };

  if let Err(e) = infra_db::run_migrations(&pool).await {
    eprintln!("ERROR: Failed to run migrations: {}", e);
    return None;
  }
  
  // Limpiar datos antes de los benchmarks
  if let Err(e) = sqlx::query("TRUNCATE orders, products, users CASCADE")
    .execute(&pool)
    .await
  {
    eprintln!("ERROR: Failed to truncate tables: {}", e);
    return None;
  }

  // Crear datos de prueba
  create_test_data(&pool).await;

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

  let app = build_app(state.clone());
  Some((state, app))
}

async fn create_test_data(pool: &PgPool) {
  // Crear 100 usuarios de prueba
  for i in 0..100 {
    sqlx::query(
      "INSERT INTO users (id, email, name, created_at, updated_at) 
       VALUES ($1, $2, $3, NOW(), NOW()) 
       ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::new_v4())
    .bind(format!("user{}@example.com", i))
    .bind(format!("User {}", i))
    .execute(pool)
    .await
    .ok();
  }

  // Crear 100 productos de prueba
  for i in 0..100 {
    sqlx::query(
      "INSERT INTO products (id, sku, name, price_cents, created_at, updated_at) 
       VALUES ($1, $2, $3, $4, NOW(), NOW()) 
       ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::new_v4())
    .bind(format!("SKU-{:03}", i))
    .bind(format!("Product {}", i))
    .bind((i + 1) as i64 * 1000)
    .execute(pool)
    .await
    .ok();
  }

  // Crear 50 órdenes de prueba
  let user_ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM users LIMIT 50")
    .fetch_all(pool)
    .await
    .unwrap_or_default();

  for (i, user_id) in user_ids.iter().enumerate() {
    sqlx::query(
      "INSERT INTO orders (id, user_id, status, total_cents, created_at, updated_at) 
       VALUES ($1, $2, $3, $4, NOW(), NOW()) 
       ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind("created")
    .bind((i + 1) as i64 * 2000)
    .execute(pool)
    .await
    .ok();
  }
}

// Helper function to setup app or skip benchmark if database is not available
fn setup_app_or_skip(rt: &tokio::runtime::Runtime) -> Option<(AppState, axum::Router)> {
  rt.block_on(setup_app())
}

fn bench_health_check(c: &mut Criterion) {
  let rt = tokio::runtime::Runtime::new().unwrap();
  
  let (_, app) = match setup_app_or_skip(&rt) {
    Some(app) => app,
    None => return,
  };

  c.bench_function("health_check", |b| {
    b.to_async(&rt).iter(|| {
      let app = app.clone();
      async move {
        let req = Request::builder()
          .uri("/health")
          .body(Body::empty())
          .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
      }
    });
  });
}

fn bench_get_users(c: &mut Criterion) {
  let rt = tokio::runtime::Runtime::new().unwrap();
  
  let (_, app) = match setup_app_or_skip(&rt) {
    Some(app) => app,
    None => return,
  };

  c.bench_function("get_users", |b| {
    b.to_async(&rt).iter(|| {
      let app = app.clone();
      async move {
        let req = Request::builder()
          .uri("/users")
          .body(Body::empty())
          .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
      }
    });
  });
}

fn bench_get_products(c: &mut Criterion) {
  let rt = tokio::runtime::Runtime::new().unwrap();
  
  let (_, app) = match setup_app_or_skip(&rt) {
    Some(app) => app,
    None => return,
  };

  c.bench_function("get_products", |b| {
    b.to_async(&rt).iter(|| {
      let app = app.clone();
      async move {
        let req = Request::builder()
          .uri("/products")
          .body(Body::empty())
          .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
      }
    });
  });
}

fn bench_get_orders(c: &mut Criterion) {
  let rt = tokio::runtime::Runtime::new().unwrap();
  
  let (_, app) = match setup_app_or_skip(&rt) {
    Some(app) => app,
    None => return,
  };

  c.bench_function("get_orders", |b| {
    b.to_async(&rt).iter(|| {
      let app = app.clone();
      async move {
        let req = Request::builder()
          .uri("/orders")
          .body(Body::empty())
          .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
      }
    });
  });
}

fn bench_get_user_by_id(c: &mut Criterion) {
  let rt = tokio::runtime::Runtime::new().unwrap();
  
  let (state, app) = match setup_app_or_skip(&rt) {
    Some(app) => app,
    None => return,
  };
  
  // Obtener un ID de usuario existente
  let user_id: Uuid = rt.block_on(async {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM users LIMIT 1")
      .fetch_one(&state.pool)
      .await
      .unwrap()
  });

  c.bench_function("get_user_by_id", |b| {
    let app = app.clone();
    let user_id = user_id;
    b.to_async(&rt).iter(|| {
      let app = app.clone();
      let user_id = user_id;
      async move {
        let req = Request::builder()
          .uri(format!("/users/{}", user_id))
          .body(Body::empty())
          .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
      }
    });
  });
}

fn bench_create_user(c: &mut Criterion) {
  let rt = tokio::runtime::Runtime::new().unwrap();
  
  let (_, app) = match setup_app_or_skip(&rt) {
    Some(app) => app,
    None => return,
  };

  c.bench_function("create_user", |b| {
    let counter = Arc::new(AtomicU64::new(0));
    b.to_async(&rt).iter(|| {
      let app = app.clone();
      let counter = counter.clone();
      async move {
        let val = counter.fetch_add(1, Ordering::Relaxed);
        // Use UUID to ensure unique email for each iteration
        let uuid = Uuid::new_v4();
        let body = format!(
          r#"{{"email":"benchuser-{}-{}@example.com","name":"Bench User {}"}}"#,
          val, uuid, val
        );
        let req = Request::builder()
          .method("POST")
          .uri("/users")
          .header("content-type", "application/json")
          .body(Body::from(body))
          .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
      }
    });
  });
}

fn bench_create_product(c: &mut Criterion) {
  let rt = tokio::runtime::Runtime::new().unwrap();
  
  let (_, app) = match setup_app_or_skip(&rt) {
    Some(app) => app,
    None => return,
  };

  c.bench_function("create_product", |b| {
    let counter = Arc::new(AtomicU64::new(0));
    b.to_async(&rt).iter(|| {
      let app = app.clone();
      let counter = counter.clone();
      async move {
        let val = counter.fetch_add(1, Ordering::Relaxed);
        // Use UUID to ensure unique SKU for each iteration
        let uuid = Uuid::new_v4();
        let body = format!(
          r#"{{"sku":"BENCH-{}-{}","name":"Bench Product {}","price_cents":{}}}"#,
          val, uuid, val, val * 1000
        );
        let req = Request::builder()
          .method("POST")
          .uri("/products")
          .header("content-type", "application/json")
          .body(Body::from(body))
          .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
      }
    });
  });
}

fn bench_create_order(c: &mut Criterion) {
  let rt = tokio::runtime::Runtime::new().unwrap();
  
  let (state, app) = match setup_app_or_skip(&rt) {
    Some(app) => app,
    None => return,
  };
  
  // Obtener un ID de usuario existente
  let user_id: Uuid = rt.block_on(async {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM users LIMIT 1")
      .fetch_one(&state.pool)
      .await
      .unwrap()
  });

  c.bench_function("create_order", |b| {
    let app = app.clone();
    let user_id = user_id;
    let counter = Arc::new(AtomicU64::new(0));
    b.to_async(&rt).iter(|| {
      let app = app.clone();
      let user_id = user_id;
      let counter = counter.clone();
      async move {
        let val = counter.fetch_add(1, Ordering::Relaxed);
        let body = format!(
          r#"{{"user_id":"{}","status":"created","total_cents":{}}}"#,
          user_id, val * 1000
        );
        let req = Request::builder()
          .method("POST")
          .uri("/orders")
          .header("content-type", "application/json")
          .body(Body::from(body))
          .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
      }
    });
  });
}

fn bench_update_user(c: &mut Criterion) {
  let rt = tokio::runtime::Runtime::new().unwrap();
  
  let (state, app) = match setup_app_or_skip(&rt) {
    Some(app) => app,
    None => return,
  };
  
  // Obtener un ID de usuario existente
  let user_id: Uuid = rt.block_on(async {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM users LIMIT 1")
      .fetch_one(&state.pool)
      .await
      .unwrap()
  });

  c.bench_function("update_user", |b| {
    let app = app.clone();
    let user_id = user_id;
    b.to_async(&rt).iter(|| {
      let app = app.clone();
      let user_id = user_id;
      async move {
        let body = r#"{"name":"Updated Name"}"#;
        let req = Request::builder()
          .method("PUT")
          .uri(format!("/users/{}", user_id))
          .header("content-type", "application/json")
          .body(Body::from(body))
          .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
      }
    });
  });
}

fn bench_delete_order(c: &mut Criterion) {
  let rt = tokio::runtime::Runtime::new().unwrap();
  
  let (state, app) = match setup_app_or_skip(&rt) {
    Some(app) => app,
    None => return,
  };

  c.bench_function("delete_order", |b| {
    b.to_async(&rt).iter(|| {
      let app = app.clone();
      let pool = state.pool.clone();
      async move {
        // Crear una orden temporal para cada iteración
        let order_id: Uuid = Uuid::new_v4();
        let user_id: Uuid = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users LIMIT 1")
          .fetch_one(&pool)
          .await
          .unwrap();
        
        sqlx::query(
          "INSERT INTO orders (id, user_id, status, total_cents, created_at, updated_at) 
           VALUES ($1, $2, $3, $4, NOW(), NOW())"
        )
        .bind(order_id)
        .bind(user_id)
        .bind("created")
        .bind(1000)
        .execute(&pool)
        .await
        .unwrap();

        let req = Request::builder()
          .method("DELETE")
          .uri(format!("/orders/{}", order_id))
          .body(Body::empty())
          .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
      }
    });
  });
}

criterion_group!(
  benches,
  bench_health_check,
  bench_get_users,
  bench_get_products,
  bench_get_orders,
  bench_get_user_by_id,
  bench_create_user,
  bench_create_product,
  bench_create_order,
  bench_update_user,
  bench_delete_order
);
criterion_main!(benches);
