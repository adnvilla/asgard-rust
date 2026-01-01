use crate::application::ports::{NewProduct, ProductRepository, RepoError, UpdateProduct};
use crate::domain::models::Product;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct PgProductRepository {
  pool: PgPool,
}

impl PgProductRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

fn map_sqlx_err(err: sqlx::Error) -> RepoError {
  match &err {
    sqlx::Error::RowNotFound => RepoError::NotFound,
    sqlx::Error::Database(db_err) => {
      if db_err.code().as_deref() == Some("23505") {
        RepoError::Conflict
      } else {
        RepoError::Unexpected(err.to_string())
      }
    }
    _ => RepoError::Unexpected(err.to_string()),
  }
}

#[async_trait]
impl ProductRepository for PgProductRepository {
  async fn create(&self, input: NewProduct) -> Result<Product, RepoError> {
    let row = sqlx::query(
      r#"
      INSERT INTO products (sku, name, price_cents)
      VALUES ($1, $2, $3)
      RETURNING id, sku, name, price_cents, created_at, updated_at
      "#,
    )
    .bind(input.sku)
    .bind(input.name)
    .bind(input.price_cents)
    .fetch_one(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(Product {
      id: row.get::<Uuid, _>("id"),
      sku: row.get::<String, _>("sku"),
      name: row.get::<String, _>("name"),
      price_cents: row.get::<i64, _>("price_cents"),
      created_at: row.get::<DateTime<Utc>, _>("created_at"),
      updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
  }

  async fn list(&self) -> Result<Vec<Product>, RepoError> {
    let rows = sqlx::query(
      r#"
      SELECT id, sku, name, price_cents, created_at, updated_at
      FROM products
      ORDER BY created_at DESC
      "#,
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(rows
      .into_iter()
      .map(|row| Product {
        id: row.get::<Uuid, _>("id"),
        sku: row.get::<String, _>("sku"),
        name: row.get::<String, _>("name"),
        price_cents: row.get::<i64, _>("price_cents"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
      })
      .collect())
  }

  async fn get(&self, id: Uuid) -> Result<Product, RepoError> {
    let row = sqlx::query(
      r#"
      SELECT id, sku, name, price_cents, created_at, updated_at
      FROM products
      WHERE id = $1
      "#,
    )
    .bind(id)
    .fetch_one(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(Product {
      id: row.get::<Uuid, _>("id"),
      sku: row.get::<String, _>("sku"),
      name: row.get::<String, _>("name"),
      price_cents: row.get::<i64, _>("price_cents"),
      created_at: row.get::<DateTime<Utc>, _>("created_at"),
      updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
  }

  async fn update(&self, id: Uuid, input: UpdateProduct) -> Result<Product, RepoError> {
    let row = sqlx::query(
      r#"
      UPDATE products
      SET
        sku = COALESCE($2, sku),
        name = COALESCE($3, name),
        price_cents = COALESCE($4, price_cents),
        updated_at = now()
      WHERE id = $1
      RETURNING id, sku, name, price_cents, created_at, updated_at
      "#,
    )
    .bind(id)
    .bind(input.sku)
    .bind(input.name)
    .bind(input.price_cents)
    .fetch_one(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(Product {
      id: row.get::<Uuid, _>("id"),
      sku: row.get::<String, _>("sku"),
      name: row.get::<String, _>("name"),
      price_cents: row.get::<i64, _>("price_cents"),
      created_at: row.get::<DateTime<Utc>, _>("created_at"),
      updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
  }

  async fn delete(&self, id: Uuid) -> Result<(), RepoError> {
    let res = sqlx::query("DELETE FROM products WHERE id = $1")
      .bind(id)
      .execute(&self.pool)
      .await
      .map_err(map_sqlx_err)?;
    if res.rows_affected() == 0 {
      return Err(RepoError::NotFound);
    }
    Ok(())
  }
}


