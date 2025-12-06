use sqlx::{PgPool, Row};
use Wordmesh_backend::config::settings::Settings;
use Wordmesh_backend::domain::word::CanonicalKey;
use Wordmesh_backend::repository::word::{
    PgWordRepository, SearchParams, SearchScope, UpsertUserWord, WordRepository,
};

async fn setup_test_pool() -> PgPool {
    // Load testing configuration
    let settings = Settings::load_for_environment("testing")
        .expect("Failed to load testing configuration");
    
    let connection_string = settings.database.connection_string();
    PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to test database")
}

async fn create_test_user(pool: &PgPool, user_id: i64) -> i64 {
    // Create test user if not exists
    // First try to get existing user
    let existing = sqlx::query("SELECT id FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    
    if existing.is_some() {
        return user_id;
    }
    
    // Create new user
    let result = sqlx::query(
        "INSERT INTO users (id, username, password, created_at) 
         VALUES ($1, $2, $3, NOW()) 
         RETURNING id"
    )
    .bind(user_id)
    .bind(format!("test_user_{}", user_id))
    .bind("test_password")
    .fetch_optional(pool)
    .await
    .ok();
    
    if let Some(Some(row)) = result {
        row.try_get("id").unwrap_or(user_id)
    } else {
        user_id
    }
}

async fn cleanup_test_data(pool: &PgPool, user_id: i64) {
    // Clean up test data
    sqlx::query("DELETE FROM user_senses WHERE user_word_id IN (SELECT id FROM user_words WHERE user_id = $1)")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM user_words WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM words WHERE id NOT IN (SELECT DISTINCT word_id FROM user_words)")
        .execute(pool)
        .await
        .ok();
    // Optionally delete test user (commented out to avoid affecting other tests)
    // sqlx::query("DELETE FROM users WHERE id = $1")
    //     .bind(user_id)
    //     .execute(pool)
    //     .await
    //     .ok();
}

#[tokio::test]
async fn test_upsert_word() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    
    let canonical = CanonicalKey::new("test-word").unwrap();
    let result = repo.upsert_word(&canonical, "test-word").await;
    
    assert!(result.is_ok());
    let word = result.unwrap();
    assert_eq!(word.text, "test-word");
    assert_eq!(word.canonical_key.as_str(), "test-word");
    
    // Cleanup
    sqlx::query("DELETE FROM words WHERE canonical_key = $1")
        .bind(canonical.as_str())
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_upsert_word_idempotent() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    
    let canonical = CanonicalKey::new("idempotent-word").unwrap();
    
    // Cleanup first
    sqlx::query("DELETE FROM words WHERE canonical_key = $1")
        .bind(canonical.as_str())
        .execute(&pool)
        .await
        .ok();
    
    // First insert
    let result1 = repo.upsert_word(&canonical, "idempotent-word").await;
    assert!(result1.is_ok());
    let word1 = result1.unwrap();
    let word_id = word1.id;
    
    // Second insert (should update, not create new)
    let result2 = repo.upsert_word(&canonical, "idempotent-word-updated").await;
    assert!(result2.is_ok());
    let word2 = result2.unwrap();
    
    // Should be same ID
    assert_eq!(word1.id, word2.id);
    assert_eq!(word2.text, "idempotent-word-updated");
    
    // Cleanup
    sqlx::query("DELETE FROM words WHERE id = $1")
        .bind(word_id)
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_upsert_user_word() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    let test_user_id = 99999;
    
    create_test_user(&pool, test_user_id).await;
    cleanup_test_data(&pool, test_user_id).await;
    
    let canonical = CanonicalKey::new("user-test-word").unwrap();
    let payload = UpsertUserWord {
        user_id: test_user_id,
        word_text: "user-test-word".to_string(),
        canonical_key: canonical,
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        note: Some("test note".to_string()),
    };
    
    let result = repo.upsert_user_word(payload).await;
    assert!(result.is_ok());
    
    let aggregate = result.unwrap();
    assert_eq!(aggregate.word.text, "user-test-word");
    assert_eq!(aggregate.user_word.tags().len(), 2);
    assert_eq!(aggregate.user_word.note(), Some("test note"));
    
    cleanup_test_data(&pool, test_user_id).await;
}

#[tokio::test]
async fn test_find_user_word() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    let test_user_id = 99998;
    
    create_test_user(&pool, test_user_id).await;
    cleanup_test_data(&pool, test_user_id).await;
    
    // Create a user word first
    let canonical = CanonicalKey::new("find-test-word").unwrap();
    let payload = UpsertUserWord {
        user_id: test_user_id,
        word_text: "find-test-word".to_string(),
        canonical_key: canonical,
        tags: vec![],
        note: None,
    };
    
    let aggregate = repo.upsert_user_word(payload).await.unwrap();
    let user_word_id = aggregate.user_word.id.unwrap();
    
    // Find it
    let result = repo.find_user_word(test_user_id, user_word_id).await;
    assert!(result.is_ok());
    let found = result.unwrap();
    assert!(found.is_some());
    let found_aggregate = found.unwrap();
    assert_eq!(found_aggregate.word.text, "find-test-word");
    
    // Find non-existent
    let result = repo.find_user_word(test_user_id, 999999).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    cleanup_test_data(&pool, test_user_id).await;
}

#[tokio::test]
async fn test_add_user_sense() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    let test_user_id = 99997;
    
    create_test_user(&pool, test_user_id).await;
    cleanup_test_data(&pool, test_user_id).await;
    
    // Create a user word first
    let canonical = CanonicalKey::new("sense-test-word").unwrap();
    let payload = UpsertUserWord {
        user_id: test_user_id,
        word_text: "sense-test-word".to_string(),
        canonical_key: canonical,
        tags: vec![],
        note: None,
    };
    
    let aggregate = repo.upsert_user_word(payload).await.unwrap();
    let user_word_id = aggregate.user_word.id.unwrap();
    
    // Add a sense
    use Wordmesh_backend::repository::word::NewUserSense;
    let sense = NewUserSense {
        user_word_id,
        text: "test meaning".to_string(),
        is_primary: true,
        sort_order: 0,
        note: None,
    };
    
    let result = repo.add_user_sense(sense).await;
    assert!(result.is_ok());
    let created_sense = result.unwrap();
    assert_eq!(created_sense.text(), "test meaning");
    assert!(created_sense.is_primary);
    
    cleanup_test_data(&pool, test_user_id).await;
}

#[tokio::test]
async fn test_update_user_sense() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    let test_user_id = 99996;
    
    create_test_user(&pool, test_user_id).await;
    cleanup_test_data(&pool, test_user_id).await;
    
    // Create a user word with a sense
    let canonical = CanonicalKey::new("update-sense-word").unwrap();
    let payload = UpsertUserWord {
        user_id: test_user_id,
        word_text: "update-sense-word".to_string(),
        canonical_key: canonical,
        tags: vec![],
        note: None,
    };
    
    let aggregate = repo.upsert_user_word(payload).await.unwrap();
    let user_word_id = aggregate.user_word.id.unwrap();
    
    use Wordmesh_backend::repository::word::NewUserSense;
    let sense = NewUserSense {
        user_word_id,
        text: "original meaning".to_string(),
        is_primary: true,
        sort_order: 0,
        note: None,
    };
    
    let created_sense = repo.add_user_sense(sense).await.unwrap();
    let sense_id = created_sense.id().unwrap();
    
    // Update the sense
    use Wordmesh_backend::repository::word::SenseUpdate;
    let update = SenseUpdate {
        text: Some("updated meaning".to_string()),
        is_primary: Some(false),
        sort_order: Some(1),
        note: Some(Some("updated note".to_string())),
    };
    
    let result = repo.update_user_sense(test_user_id, sense_id, update).await;
    assert!(result.is_ok());
    let updated_sense = result.unwrap();
    assert_eq!(updated_sense.text(), "updated meaning");
    assert!(!updated_sense.is_primary);
    assert_eq!(updated_sense.sort_order, 1);
    
    cleanup_test_data(&pool, test_user_id).await;
}

#[tokio::test]
async fn test_remove_user_sense() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    let test_user_id = 99995;
    
    create_test_user(&pool, test_user_id).await;
    cleanup_test_data(&pool, test_user_id).await;
    
    // Create a user word with a sense
    let canonical = CanonicalKey::new("remove-sense-word").unwrap();
    let payload = UpsertUserWord {
        user_id: test_user_id,
        word_text: "remove-sense-word".to_string(),
        canonical_key: canonical,
        tags: vec![],
        note: None,
    };
    
    let aggregate = repo.upsert_user_word(payload).await.unwrap();
    let user_word_id = aggregate.user_word.id.unwrap();
    
    use Wordmesh_backend::repository::word::NewUserSense;
    let sense = NewUserSense {
        user_word_id,
        text: "to be removed".to_string(),
        is_primary: true,
        sort_order: 0,
        note: None,
    };
    
    let created_sense = repo.add_user_sense(sense).await.unwrap();
    let sense_id = created_sense.id().unwrap();
    
    // Remove the sense
    let result = repo.remove_user_sense(test_user_id, sense_id).await;
    assert!(result.is_ok());
    let removed_sense = result.unwrap();
    assert_eq!(removed_sense.text(), "to be removed");
    
    cleanup_test_data(&pool, test_user_id).await;
}

#[tokio::test]
async fn test_remove_user_word() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    let test_user_id = 99994;
    
    create_test_user(&pool, test_user_id).await;
    cleanup_test_data(&pool, test_user_id).await;
    
    // Create a user word
    let canonical = CanonicalKey::new("remove-word").unwrap();
    let payload = UpsertUserWord {
        user_id: test_user_id,
        word_text: "remove-word".to_string(),
        canonical_key: canonical,
        tags: vec![],
        note: None,
    };
    
    let aggregate = repo.upsert_user_word(payload).await.unwrap();
    let user_word_id = aggregate.user_word.id.unwrap();
    
    // Remove it
    let result = repo.remove_user_word(test_user_id, user_word_id).await;
    assert!(result.is_ok());
    
    // Verify it's gone
    let found = repo.find_user_word(test_user_id, user_word_id).await.unwrap();
    assert!(found.is_none());
    
    cleanup_test_data(&pool, test_user_id).await;
}

#[tokio::test]
async fn test_find_sense_by_id() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    let test_user_id = 99993;
    
    create_test_user(&pool, test_user_id).await;
    cleanup_test_data(&pool, test_user_id).await;
    
    // Create a user word with a sense
    let canonical = CanonicalKey::new("find-sense-word").unwrap();
    let payload = UpsertUserWord {
        user_id: test_user_id,
        word_text: "find-sense-word".to_string(),
        canonical_key: canonical,
        tags: vec![],
        note: None,
    };
    
    let aggregate = repo.upsert_user_word(payload).await.unwrap();
    let user_word_id = aggregate.user_word.id.unwrap();
    let word_id = aggregate.word.id;
    
    use Wordmesh_backend::repository::word::NewUserSense;
    let sense = NewUserSense {
        user_word_id,
        text: "findable sense".to_string(),
        is_primary: true,
        sort_order: 0,
        note: None,
    };
    
    let created_sense = repo.add_user_sense(sense).await.unwrap();
    let sense_id = created_sense.id().unwrap();
    
    // Find it
    let result = repo.find_sense_by_id(test_user_id, sense_id).await;
    assert!(result.is_ok());
    let found = result.unwrap();
    assert!(found.is_some());
    let (found_sense, found_word_id) = found.unwrap();
    assert_eq!(found_sense.text(), "findable sense");
    assert_eq!(found_word_id, word_id);
    
    cleanup_test_data(&pool, test_user_id).await;
}

#[tokio::test]
async fn test_find_word_by_id() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    
    // Cleanup first
    let canonical = CanonicalKey::new("find-word-by-id").unwrap();
    sqlx::query("DELETE FROM words WHERE canonical_key = $1")
        .bind(canonical.as_str())
        .execute(&pool)
        .await
        .ok();
    
    // Create a word
    let word = repo.upsert_word(&canonical, "find-word-by-id").await.unwrap();
    let word_id = word.id;
    
    // Find it
    let result = repo.find_word_by_id(word_id).await;
    assert!(result.is_ok());
    let found = result.unwrap();
    assert!(found.is_some(), "Word with id {} should exist", word_id);
    let found_word = found.unwrap();
    assert_eq!(found_word.text, "find-word-by-id");
    
    // Cleanup
    sqlx::query("DELETE FROM words WHERE id = $1")
        .bind(word_id)
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_search_word_scope() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    let test_user_id = 99992;
    
    create_test_user(&pool, test_user_id).await;
    cleanup_test_data(&pool, test_user_id).await;
    
    // Create test words
    let canonical1 = CanonicalKey::new("searchable-word").unwrap();
    let payload1 = UpsertUserWord {
        user_id: test_user_id,
        word_text: "searchable-word".to_string(),
        canonical_key: canonical1,
        tags: vec![],
        note: None,
    };
    repo.upsert_user_word(payload1).await.unwrap();
    
    let canonical2 = CanonicalKey::new("another-word").unwrap();
    let payload2 = UpsertUserWord {
        user_id: test_user_id,
        word_text: "another-word".to_string(),
        canonical_key: canonical2,
        tags: vec![],
        note: None,
    };
    repo.upsert_user_word(payload2).await.unwrap();
    
    // Search by word scope
    let params = SearchParams {
        user_id: test_user_id,
        query: "searchable".to_string(),
        scope: SearchScope::Word,
        limit: 10,
        offset: 0,
    };
    
    let result = repo.search(params).await;
    assert!(result.is_ok());
    let results = result.unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|r| r.word.text == "searchable-word"));
    
    cleanup_test_data(&pool, test_user_id).await;
}

#[tokio::test]
async fn test_search_sense_scope() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    let test_user_id = 99991;
    
    create_test_user(&pool, test_user_id).await;
    cleanup_test_data(&pool, test_user_id).await;
    
    // Create a user word with a sense
    let canonical = CanonicalKey::new("sense-search-word").unwrap();
    let payload = UpsertUserWord {
        user_id: test_user_id,
        word_text: "sense-search-word".to_string(),
        canonical_key: canonical,
        tags: vec![],
        note: None,
    };
    
    let aggregate = repo.upsert_user_word(payload).await.unwrap();
    let user_word_id = aggregate.user_word.id.unwrap();
    
    use Wordmesh_backend::repository::word::NewUserSense;
    let sense = NewUserSense {
        user_word_id,
        text: "searchable meaning".to_string(),
        is_primary: true,
        sort_order: 0,
        note: None,
    };
    repo.add_user_sense(sense).await.unwrap();
    
    // Search by sense scope
    let params = SearchParams {
        user_id: test_user_id,
        query: "searchable".to_string(),
        scope: SearchScope::Sense,
        limit: 10,
        offset: 0,
    };
    
    let result = repo.search(params).await;
    assert!(result.is_ok());
    let results = result.unwrap();
    assert!(!results.is_empty());
    
    cleanup_test_data(&pool, test_user_id).await;
}

#[tokio::test]
async fn test_search_both_scope() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    let test_user_id = 99990;
    
    create_test_user(&pool, test_user_id).await;
    cleanup_test_data(&pool, test_user_id).await;
    
    // Create test data
    let canonical = CanonicalKey::new("both-search-word").unwrap();
    let payload = UpsertUserWord {
        user_id: test_user_id,
        word_text: "both-search-word".to_string(),
        canonical_key: canonical,
        tags: vec![],
        note: None,
    };
    
    let aggregate = repo.upsert_user_word(payload).await.unwrap();
    let user_word_id = aggregate.user_word.id.unwrap();
    
    use Wordmesh_backend::repository::word::NewUserSense;
    let sense = NewUserSense {
        user_word_id,
        text: "both searchable meaning".to_string(),
        is_primary: true,
        sort_order: 0,
        note: None,
    };
    repo.add_user_sense(sense).await.unwrap();
    
    // Search by both scope
    let params = SearchParams {
        user_id: test_user_id,
        query: "both".to_string(),
        scope: SearchScope::Both,
        limit: 10,
        offset: 0,
    };
    
    let result = repo.search(params).await;
    assert!(result.is_ok());
    let results = result.unwrap();
    assert!(!results.is_empty());
    
    cleanup_test_data(&pool, test_user_id).await;
}

#[tokio::test]
async fn test_search_pagination() {
    let pool = setup_test_pool().await;
    let repo = PgWordRepository::new(pool.clone());
    let test_user_id = 99989;
    
    create_test_user(&pool, test_user_id).await;
    cleanup_test_data(&pool, test_user_id).await;
    
    // Create multiple words
    for i in 0..5 {
        let canonical = CanonicalKey::new(&format!("paginated-word-{}", i)).unwrap();
        let payload = UpsertUserWord {
            user_id: test_user_id,
            word_text: format!("paginated-word-{}", i),
            canonical_key: canonical,
            tags: vec![],
            note: None,
        };
        repo.upsert_user_word(payload).await.unwrap();
    }
    
    // First page
    let params = SearchParams {
        user_id: test_user_id,
        query: String::new(),
        scope: SearchScope::Both,
        limit: 2,
        offset: 0,
    };
    
    let result = repo.search(params).await;
    assert!(result.is_ok());
    let page1 = result.unwrap();
    assert_eq!(page1.len(), 2);
    
    // Second page
    let params = SearchParams {
        user_id: test_user_id,
        query: String::new(),
        scope: SearchScope::Both,
        limit: 2,
        offset: 2,
    };
    
    let result = repo.search(params).await;
    assert!(result.is_ok());
    let page2 = result.unwrap();
    assert_eq!(page2.len(), 2);
    
    cleanup_test_data(&pool, test_user_id).await;
}

