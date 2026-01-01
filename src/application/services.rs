use crate::application::ports::{
  NewOrder, NewProduct, NewUser, OrderRepository, ProductRepository, RepoError, UpdateOrder,
  UpdateProduct, UpdateUser, UserRepository,
};
use crate::domain::models::{Order, Product, User};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService<R: UserRepository> {
  repo: Arc<R>,
}

impl<R: UserRepository> UserService<R> {
  pub fn new(repo: R) -> Self {
    Self { repo: Arc::new(repo) }
  }

  pub async fn create(&self, input: NewUser) -> Result<User, RepoError> {
    self.repo.create(input).await
  }
  pub async fn list(&self) -> Result<Vec<User>, RepoError> {
    self.repo.list().await
  }
  pub async fn get(&self, id: Uuid) -> Result<User, RepoError> {
    self.repo.get(id).await
  }
  pub async fn update(&self, id: Uuid, input: UpdateUser) -> Result<User, RepoError> {
    self.repo.update(id, input).await
  }
  pub async fn delete(&self, id: Uuid) -> Result<(), RepoError> {
    self.repo.delete(id).await
  }
}

#[derive(Clone)]
pub struct ProductService<R: ProductRepository> {
  repo: Arc<R>,
}

impl<R: ProductRepository> ProductService<R> {
  pub fn new(repo: R) -> Self {
    Self { repo: Arc::new(repo) }
  }

  pub async fn create(&self, input: NewProduct) -> Result<Product, RepoError> {
    self.repo.create(input).await
  }
  pub async fn list(&self) -> Result<Vec<Product>, RepoError> {
    self.repo.list().await
  }
  pub async fn get(&self, id: Uuid) -> Result<Product, RepoError> {
    self.repo.get(id).await
  }
  pub async fn update(&self, id: Uuid, input: UpdateProduct) -> Result<Product, RepoError> {
    self.repo.update(id, input).await
  }
  pub async fn delete(&self, id: Uuid) -> Result<(), RepoError> {
    self.repo.delete(id).await
  }
}

#[derive(Clone)]
pub struct OrderService<R: OrderRepository> {
  repo: Arc<R>,
}

impl<R: OrderRepository> OrderService<R> {
  pub fn new(repo: R) -> Self {
    Self { repo: Arc::new(repo) }
  }

  pub async fn create(&self, input: NewOrder) -> Result<Order, RepoError> {
    self.repo.create(input).await
  }
  pub async fn list(&self) -> Result<Vec<Order>, RepoError> {
    self.repo.list().await
  }
  pub async fn get(&self, id: Uuid) -> Result<Order, RepoError> {
    self.repo.get(id).await
  }
  pub async fn update(&self, id: Uuid, input: UpdateOrder) -> Result<Order, RepoError> {
    self.repo.update(id, input).await
  }
  pub async fn delete(&self, id: Uuid) -> Result<(), RepoError> {
    self.repo.delete(id).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::application::ports::{NewUser, UpdateUser, UserRepository};
  use async_trait::async_trait;
  use chrono::Utc;
  use std::collections::HashMap;
  use tokio::sync::Mutex;
  use uuid::Uuid;

  #[derive(Clone, Default)]
  struct FakeUserRepo {
    store: Arc<Mutex<HashMap<Uuid, User>>>,
  }

  #[async_trait]
  impl UserRepository for FakeUserRepo {
    async fn create(&self, input: NewUser) -> Result<User, RepoError> {
      let id = Uuid::new_v4();
      let now = Utc::now();
      let user = User { id, email: input.email, name: input.name, created_at: now, updated_at: now };
      self.store.lock().await.insert(id, user.clone());
      Ok(user)
    }
    async fn list(&self) -> Result<Vec<User>, RepoError> {
      Ok(self.store.lock().await.values().cloned().collect())
    }
    async fn get(&self, id: Uuid) -> Result<User, RepoError> {
      self.store.lock().await.get(&id).cloned().ok_or(RepoError::NotFound)
    }
    async fn update(&self, id: Uuid, input: UpdateUser) -> Result<User, RepoError> {
      let mut guard = self.store.lock().await;
      let u = guard.get_mut(&id).ok_or(RepoError::NotFound)?;
      if let Some(email) = input.email {
        u.email = email;
      }
      if let Some(name) = input.name {
        u.name = name;
      }
      u.updated_at = Utc::now();
      Ok(u.clone())
    }
    async fn delete(&self, id: Uuid) -> Result<(), RepoError> {
      let removed = self.store.lock().await.remove(&id);
      if removed.is_none() {
        return Err(RepoError::NotFound);
      }
      Ok(())
    }
  }

  #[tokio::test]
  async fn user_service_crud_works_with_fake_repo() {
    let svc = UserService::new(FakeUserRepo::default());

    let created = svc
      .create(NewUser { email: "a@b.com".into(), name: "Alice".into() })
      .await
      .unwrap();

    let fetched = svc.get(created.id).await.unwrap();
    assert_eq!(fetched.email, "a@b.com");

    let updated = svc
      .update(created.id, UpdateUser { email: None, name: Some("Alicia".into()) })
      .await
      .unwrap();
    assert_eq!(updated.name, "Alicia");

    svc.delete(created.id).await.unwrap();
    let err = svc.get(created.id).await.unwrap_err();
    assert!(matches!(err, RepoError::NotFound));
  }
}


