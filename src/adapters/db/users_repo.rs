use crate::application::ports::{NewUser, RepoError, UpdateUser, UserRepository};
use crate::domain::models::User;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct PgUserRepository {
  pool: PgPool,
}

impl PgUserRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

fn map_sqlx_err(err: sqlx::Error) -> RepoError {
  match &err {
    sqlx::Error::RowNotFound => RepoError::NotFound,
    sqlx::Error::Database(db_err) => {
      // Postgres: unique_violation = 23505
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
impl UserRepository for PgUserRepository {
  async fn create(&self, input: NewUser) -> Result<User, RepoError> {
    let row = sqlx::query(
      r#"
      INSERT INTO users (email, name)
      VALUES ($1, $2)
      RETURNING id, email, name, created_at, updated_at
      "#,
    )
    .bind(input.email)
    .bind(input.name)
    .fetch_one(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(User {
      id: row.get::<Uuid, _>("id"),
      email: row.get::<String, _>("email"),
      name: row.get::<String, _>("name"),
      created_at: row.get::<DateTime<Utc>, _>("created_at"),
      updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
  }

  async fn list(&self) -> Result<Vec<User>, RepoError> {
    let rows = sqlx::query(
      r#"
      SELECT id, email, name, created_at, updated_at
      FROM users
      ORDER BY created_at DESC
      "#,
    )
    .fetch_all(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(rows
      .into_iter()
      .map(|row| User {
        id: row.get::<Uuid, _>("id"),
        email: row.get::<String, _>("email"),
        name: row.get::<String, _>("name"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
      })
      .collect())
  }

  async fn get(&self, id: Uuid) -> Result<User, RepoError> {
    let row = sqlx::query(
      r#"
      SELECT id, email, name, created_at, updated_at
      FROM users
      WHERE id = $1
      "#,
    )
    .bind(id)
    .fetch_one(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(User {
      id: row.get::<Uuid, _>("id"),
      email: row.get::<String, _>("email"),
      name: row.get::<String, _>("name"),
      created_at: row.get::<DateTime<Utc>, _>("created_at"),
      updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
  }

  async fn update(&self, id: Uuid, input: UpdateUser) -> Result<User, RepoError> {
    let row = sqlx::query(
      r#"
      UPDATE users
      SET
        email = COALESCE($2, email),
        name = COALESCE($3, name),
        updated_at = now()
      WHERE id = $1
      RETURNING id, email, name, created_at, updated_at
      "#,
    )
    .bind(id)
    .bind(input.email)
    .bind(input.name)
    .fetch_one(&self.pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(User {
      id: row.get::<Uuid, _>("id"),
      email: row.get::<String, _>("email"),
      name: row.get::<String, _>("name"),
      created_at: row.get::<DateTime<Utc>, _>("created_at"),
      updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
  }

  async fn delete(&self, id: Uuid) -> Result<(), RepoError> {
    let res = sqlx::query("DELETE FROM users WHERE id = $1")
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


