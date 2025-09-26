#![cfg(test)]

use wordmesh_backend::domain::HashedPassword;
use wordmesh_backend::repository::user::{NewUser, PgUserRepository, UserRepository};

#[sqlx::test(migrations = "tests/migrations")]
async fn create_and_fetch_user(pool: sqlx::PgPool) {
    let repo = PgUserRepository::new(pool.clone());
    let new_user = NewUser {
        username: "test_user".into(),
        password_hash: HashedPassword::new("hashed-password".into()).unwrap(),
    };

    let created = repo.create_user(new_user).await.expect("create user");
    assert_eq!(created.username, "test_user");

    let fetched = repo
        .find_by_username("test_user")
        .await
        .expect("fetch user")
        .expect("user present");
    assert_eq!(created.id, fetched.id);
    assert_eq!(fetched.username, "test_user");
}

#[sqlx::test(migrations = "tests/migrations")]
async fn find_by_username_not_found(pool: sqlx::PgPool) {
    let repo = PgUserRepository::new(pool);
    let result = repo.find_by_username("unknown").await.expect("query");
    assert!(result.is_none());
}
