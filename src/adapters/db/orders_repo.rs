use crate::application::ports::{NewOrder, OrderRepository, RepoError, UpdateOrder};
use crate::domain::models::Order;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct PgOrderRepository {
  pool: PgPool,
}

impl PgOrderRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

fn map_sqlx_err(err: sqlx::Error) -> RepoError {
  match &err {
    sqlx::Error::RowNotFound => RepoError::NotFound,
    sqlx::Error::Database(db_err) => {
      // foreign_key_violation = 23503, unique_violation = 23505
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
impl OrderRepository for PgOrderRepository {
  async fn create(&self, input: NewOrder) -> Result<Order, RepoError> {
    let row = sqlx::query(
      r#"
      INSERT INTO orders (user_id, status, total_cents)
      VALUES ($1, $2, $3)
      RETURNING id, user_id, status, total_cents, created_at, updated_at
      "#,
    )
    .bind(input.user_id)
    .bind(input.status)
    .bind(input.total_cents)
    .fetch_one(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(Order {
      id: row.get::<Uuid, _>("id"),
      user_id: row.get::<Uuid, _>("user_id"),
      status: row.get::<String, _>("status"),
      total_cents: row.get::<i64, _>("total_cents"),
      created_at: row.get::<DateTime<Utc>, _>("created_at"),
      updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
  }

  async fn list(&self) -> Result<Vec<Order>, RepoError> {
    let rows = sqlx::query(
      r#"
      SELECT id, user_id, status, total_cents, created_at, updated_at
      FROM orders
      ORDER BY created_at DESC
      "#,
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(rows
      .into_iter()
      .map(|row| Order {
        id: row.get::<Uuid, _>("id"),
        user_id: row.get::<Uuid, _>("user_id"),
        status: row.get::<String, _>("status"),
        total_cents: row.get::<i64, _>("total_cents"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
      })
      .collect())
  }

  async fn get(&self, id: Uuid) -> Result<Order, RepoError> {
    let row = sqlx::query(
      r#"
      SELECT id, user_id, status, total_cents, created_at, updated_at
      FROM orders
      WHERE id = $1
      "#,
    )
    .bind(id)
    .fetch_one(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(Order {
      id: row.get::<Uuid, _>("id"),
      user_id: row.get::<Uuid, _>("user_id"),
      status: row.get::<String, _>("status"),
      total_cents: row.get::<i64, _>("total_cents"),
      created_at: row.get::<DateTime<Utc>, _>("created_at"),
      updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
  }

  async fn update(&self, id: Uuid, input: UpdateOrder) -> Result<Order, RepoError> {
    let row = sqlx::query(
      r#"
      UPDATE orders
      SET
        status = COALESCE($2, status),
        total_cents = COALESCE($3, total_cents),
        updated_at = now()
      WHERE id = $1
      RETURNING id, user_id, status, total_cents, created_at, updated_at
      "#,
    )
    .bind(id)
    .bind(input.status)
    .bind(input.total_cents)
    .fetch_one(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(Order {
      id: row.get::<Uuid, _>("id"),
      user_id: row.get::<Uuid, _>("user_id"),
      status: row.get::<String, _>("status"),
      total_cents: row.get::<i64, _>("total_cents"),
      created_at: row.get::<DateTime<Utc>, _>("created_at"),
      updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
  }

  async fn delete(&self, id: Uuid) -> Result<(), RepoError> {
    let res = sqlx::query("DELETE FROM orders WHERE id = $1")
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


