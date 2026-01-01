use crate::domain::models::{Order, Product, User};
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepoError {
  #[error("not found")]
  NotFound,
  #[error("conflict")]
  Conflict,
  #[error("unexpected repository error: {0}")]
  Unexpected(String),
}

#[derive(Debug, Clone)]
pub struct NewUser {
  pub email: String,
  pub name: String,
}

#[derive(Debug, Clone)]
pub struct UpdateUser {
  pub email: Option<String>,
  pub name: Option<String>,
}

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
  async fn create(&self, input: NewUser) -> Result<User, RepoError>;
  async fn list(&self) -> Result<Vec<User>, RepoError>;
  async fn get(&self, id: Uuid) -> Result<User, RepoError>;
  async fn update(&self, id: Uuid, input: UpdateUser) -> Result<User, RepoError>;
  async fn delete(&self, id: Uuid) -> Result<(), RepoError>;
}

#[derive(Debug, Clone)]
pub struct NewProduct {
  pub sku: String,
  pub name: String,
  pub price_cents: i64,
}

#[derive(Debug, Clone)]
pub struct UpdateProduct {
  pub sku: Option<String>,
  pub name: Option<String>,
  pub price_cents: Option<i64>,
}

#[async_trait]
pub trait ProductRepository: Send + Sync + 'static {
  async fn create(&self, input: NewProduct) -> Result<Product, RepoError>;
  async fn list(&self) -> Result<Vec<Product>, RepoError>;
  async fn get(&self, id: Uuid) -> Result<Product, RepoError>;
  async fn update(&self, id: Uuid, input: UpdateProduct) -> Result<Product, RepoError>;
  async fn delete(&self, id: Uuid) -> Result<(), RepoError>;
}

#[derive(Debug, Clone)]
pub struct NewOrder {
  pub user_id: Uuid,
  pub status: String,
  pub total_cents: i64,
}

#[derive(Debug, Clone)]
pub struct UpdateOrder {
  pub status: Option<String>,
  pub total_cents: Option<i64>,
}

#[async_trait]
pub trait OrderRepository: Send + Sync + 'static {
  async fn create(&self, input: NewOrder) -> Result<Order, RepoError>;
  async fn list(&self) -> Result<Vec<Order>, RepoError>;
  async fn get(&self, id: Uuid) -> Result<Order, RepoError>;
  async fn update(&self, id: Uuid, input: UpdateOrder) -> Result<Order, RepoError>;
  async fn delete(&self, id: Uuid) -> Result<(), RepoError>;
}


