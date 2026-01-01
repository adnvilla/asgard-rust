use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
  pub id: Uuid,
  pub email: String,
  pub name: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Product {
  pub id: Uuid,
  pub sku: String,
  pub name: String,
  pub price_cents: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Order {
  pub id: Uuid,
  pub user_id: Uuid,
  pub status: String,
  pub total_cents: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}


