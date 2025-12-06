use std::time::Duration;
use neo4rs::{Graph, query};
use Wordmesh_backend::config::settings::Settings;
use Wordmesh_backend::repository::graph::{
    Neo4jGraphRepository, SenseLinkFilter, SenseWordLinkKind, WordLinkFilter, WordLinkKind,
    GraphRepository,
};

async fn setup_test_graph() -> Neo4jGraphRepository {
    // Load testing configuration
    let settings = Settings::load_for_environment("testing")
        .expect("Failed to load testing configuration");
    
    let graph = Graph::new(
        &settings.neo4j.uri,
        settings.neo4j.username.clone(),
        settings.neo4j.password.clone(),
    )
    .expect("Failed to connect to test Neo4j");
    
    Neo4jGraphRepository::new(
        graph,
        Duration::from_secs(settings.neo4j.query_timeout_seconds),
    )
}

async fn get_cleanup_graph() -> Graph {
    let settings = Settings::load_for_environment("testing")
        .expect("Failed to load testing configuration");
    
    Graph::new(
        &settings.neo4j.uri,
        settings.neo4j.username.clone(),
        settings.neo4j.password.clone(),
    )
    .expect("Failed to connect for cleanup")
}

async fn cleanup_test_data(user_id: i64) {
    // Clean up test relationships by deleting all links for test user
    let cleanup_graph = get_cleanup_graph().await;
    
    // Delete word links
    let mut q = query("MATCH ()-[r:WORD_TO_WORD { user_id: $user_id }]->() DELETE r");
    q = q.param("user_id", user_id);
    if let Ok(mut result) = cleanup_graph.execute(q).await {
        while let Ok(Some(_)) = result.next().await {}
    }
    
    // Delete sense-word links
    let mut q = query("MATCH ()-[r:SENSE_TO_WORD { user_id: $user_id }]->() DELETE r");
    q = q.param("user_id", user_id);
    if let Ok(mut result) = cleanup_graph.execute(q).await {
        while let Ok(Some(_)) = result.next().await {}
    }
    
    // Clean up orphaned nodes
    let q = query("MATCH (n:Word) WHERE NOT (n)-[]-() DELETE n");
    if let Ok(mut result) = cleanup_graph.execute(q).await {
        while let Ok(Some(_)) = result.next().await {}
    }
    
    let q = query("MATCH (n:UserSense) WHERE NOT (n)-[]-() DELETE n");
    if let Ok(mut result) = cleanup_graph.execute(q).await {
        while let Ok(Some(_)) = result.next().await {}
    }
}

async fn cleanup_node(graph: &Graph, label: &str, prop: &str, value: i64) {
    let mut q = query(&format!("MATCH (n:{} {{ {}: ${} }}) DELETE n", label, prop, prop));
    q = q.param(prop, value);
    if let Ok(mut result) = graph.execute(q).await {
        while let Ok(Some(_)) = result.next().await {}
    }
}

#[tokio::test]
async fn test_upsert_node_word() {
    let repo = setup_test_graph().await;
    let test_word_id = 99999;
    
    // Cleanup first
    let cleanup_graph = get_cleanup_graph().await;
    cleanup_node(&cleanup_graph, "Word", "word_id", test_word_id).await;
    
    let result = repo.upsert_node_word(test_word_id).await;
    if let Err(e) = &result {
        eprintln!("upsert_node_word error: {:?}", e);
    }
    assert!(result.is_ok());
    
    // Verify node exists
    let mut q = query("MATCH (n:Word { word_id: $word_id }) RETURN n");
    q = q.param("word_id", test_word_id);
    let mut result = cleanup_graph.execute(q).await.unwrap();
    assert!(result.next().await.unwrap().is_some());
    
    // Cleanup
    cleanup_node(&cleanup_graph, "Word", "word_id", test_word_id).await;
}

#[tokio::test]
async fn test_upsert_node_sense() {
    let repo = setup_test_graph().await;
    let test_sense_id = 99999;
    let test_user_id = 99999;
    
    // Cleanup first
    let cleanup_graph = get_cleanup_graph().await;
    let mut q = query("MATCH (n:UserSense { sense_id: $sense_id }) DELETE n");
    q = q.param("sense_id", test_sense_id);
    if let Ok(mut result) = cleanup_graph.execute(q).await {
        while let Ok(Some(_)) = result.next().await {}
    }
    
    let result = repo.upsert_node_sense(test_sense_id, test_user_id).await;
    assert!(result.is_ok());
    
    // Verify node exists
    let mut q = query("MATCH (n:UserSense { sense_id: $sense_id, user_id: $user_id }) RETURN n");
    q = q.param("sense_id", test_sense_id).param("user_id", test_user_id);
    let mut result = cleanup_graph.execute(q).await.unwrap();
    assert!(result.next().await.unwrap().is_some());
    
    // Cleanup
    let mut q = query("MATCH (n:UserSense { sense_id: $sense_id }) DELETE n");
    q = q.param("sense_id", test_sense_id);
    if let Ok(mut result) = cleanup_graph.execute(q).await {
        while let Ok(Some(_)) = result.next().await {}
    }
}

#[tokio::test]
async fn test_create_word_link() {
    let repo = setup_test_graph().await;
    let test_user_id = 99998;
    let word_a_id = 100;
    let word_b_id = 200;
    
    cleanup_test_data(test_user_id).await;
    
    // Create nodes first
    repo.upsert_node_word(word_a_id).await.unwrap();
    repo.upsert_node_word(word_b_id).await.unwrap();
    
    let result = repo
        .create_word_link(test_user_id, word_a_id, word_b_id, WordLinkKind::SimilarForm, None)
        .await;
    
    assert!(result.is_ok());
    let link = result.unwrap();
    assert_eq!(link.user_id, test_user_id);
    assert_eq!(link.kind, WordLinkKind::SimilarForm);
    
    cleanup_test_data(test_user_id).await;
}

#[tokio::test]
async fn test_create_word_link_self_forbidden() {
    let repo = setup_test_graph().await;
    let test_user_id = 99997;
    let word_id = 100;
    
    let cleanup_graph = get_cleanup_graph().await;
    cleanup_node(&cleanup_graph, "Word", "word_id", word_id).await;
    
    repo.upsert_node_word(word_id).await.unwrap();
    
    let result = repo
        .create_word_link(test_user_id, word_id, word_id, WordLinkKind::SimilarForm, None)
        .await;
    
    assert!(result.is_err());
    
    // Cleanup
    cleanup_node(&cleanup_graph, "Word", "word_id", word_id).await;
}

#[tokio::test]
async fn test_delete_word_link() {
    let repo = setup_test_graph().await;
    let test_user_id = 99996;
    let word_a_id = 101;
    let word_b_id = 201;
    
    cleanup_test_data(test_user_id).await;
    
    // Create nodes and link
    repo.upsert_node_word(word_a_id).await.unwrap();
    repo.upsert_node_word(word_b_id).await.unwrap();
    repo.create_word_link(test_user_id, word_a_id, word_b_id, WordLinkKind::SimilarForm, None)
        .await
        .unwrap();
    
    // Delete the link
    let result = repo
        .delete_word_link(test_user_id, word_a_id, word_b_id, WordLinkKind::SimilarForm)
        .await;
    
    assert!(result.is_ok());
    
    cleanup_test_data(test_user_id).await;
}

#[tokio::test]
async fn test_list_word_links() {
    let repo = setup_test_graph().await;
    let test_user_id = 99995;
    let word_a_id = 102;
    let word_b_id = 202;
    let word_c_id = 203;
    
    cleanup_test_data(test_user_id).await;
    
    // Create nodes and links
    repo.upsert_node_word(word_a_id).await.unwrap();
    repo.upsert_node_word(word_b_id).await.unwrap();
    repo.upsert_node_word(word_c_id).await.unwrap();
    
    repo.create_word_link(test_user_id, word_a_id, word_b_id, WordLinkKind::SimilarForm, None)
        .await
        .unwrap();
    repo.create_word_link(test_user_id, word_a_id, word_c_id, WordLinkKind::RootAffix, None)
        .await
        .unwrap();
    
    // List links
    let filter = WordLinkFilter {
        user_id: test_user_id,
        word_id: word_a_id,
        kind: None,
        limit: 10,
        offset: 0,
    };
    
    let result = repo.list_word_links(filter).await;
    assert!(result.is_ok());
    let links = result.unwrap();
    assert!(links.len() >= 2);
    
    cleanup_test_data(test_user_id).await;
}

#[tokio::test]
async fn test_create_sense_word_link() {
    let repo = setup_test_graph().await;
    let test_user_id = 99994;
    let sense_id = 1000;
    let source_word_id = 300;
    let target_word_id = 400;
    
    cleanup_test_data(test_user_id).await;
    
    // Create nodes
    repo.upsert_node_sense(sense_id, test_user_id).await.unwrap();
    repo.upsert_node_word(target_word_id).await.unwrap();
    
    let result = repo
        .create_sense_word_link(
            test_user_id,
            sense_id,
            source_word_id,
            target_word_id,
            SenseWordLinkKind::Synonym,
            Some("test note".to_string()),
        )
        .await;
    
    assert!(result.is_ok());
    let link = result.unwrap();
    assert_eq!(link.sense_id, sense_id);
    assert_eq!(link.target_word_id, target_word_id);
    assert_eq!(link.kind, SenseWordLinkKind::Synonym);
    assert_eq!(link.note, Some("test note".to_string()));
    
    cleanup_test_data(test_user_id).await;
}

#[tokio::test]
async fn test_create_sense_word_link_self_forbidden() {
    let repo = setup_test_graph().await;
    let test_user_id = 99993;
    let sense_id = 1001;
    let word_id = 301;
    
    let cleanup_graph = get_cleanup_graph().await;
    let mut q = query("MATCH (n:UserSense { sense_id: $sense_id }) DELETE n");
    q = q.param("sense_id", sense_id);
    if let Ok(mut result) = cleanup_graph.execute(q).await {
        while let Ok(Some(_)) = result.next().await {}
    }
    cleanup_node(&cleanup_graph, "Word", "word_id", word_id).await;
    
    repo.upsert_node_sense(sense_id, test_user_id).await.unwrap();
    repo.upsert_node_word(word_id).await.unwrap();
    
    // Try to create self-link (source_word_id == target_word_id)
    let result = repo
        .create_sense_word_link(
            test_user_id,
            sense_id,
            word_id,
            word_id,
            SenseWordLinkKind::Synonym,
            None,
        )
        .await;
    
    assert!(result.is_err());
    
    // Cleanup
    let mut q = query("MATCH (n:UserSense { sense_id: $sense_id }) DELETE n");
    q = q.param("sense_id", sense_id);
    if let Ok(mut result) = cleanup_graph.execute(q).await {
        while let Ok(Some(_)) = result.next().await {}
    }
    cleanup_node(&cleanup_graph, "Word", "word_id", word_id).await;
}

#[tokio::test]
async fn test_delete_sense_word_link() {
    let repo = setup_test_graph().await;
    let test_user_id = 99992;
    let sense_id = 1002;
    let source_word_id = 302;
    let target_word_id = 402;
    
    cleanup_test_data(test_user_id).await;
    
    // Create nodes and link
    repo.upsert_node_sense(sense_id, test_user_id).await.unwrap();
    repo.upsert_node_word(target_word_id).await.unwrap();
    repo.create_sense_word_link(
        test_user_id,
        sense_id,
        source_word_id,
        target_word_id,
        SenseWordLinkKind::Synonym,
        None,
    )
    .await
    .unwrap();
    
    // Delete the link
    let result = repo
        .delete_sense_word_link(test_user_id, sense_id, target_word_id, SenseWordLinkKind::Synonym)
        .await;
    
    assert!(result.is_ok());
    
    cleanup_test_data(test_user_id).await;
}

#[tokio::test]
async fn test_list_sense_word_links() {
    let repo = setup_test_graph().await;
    let test_user_id = 99991;
    let sense_id = 1003;
    let source_word_id = 303;
    let target_word_id_1 = 403;
    let target_word_id_2 = 404;
    
    cleanup_test_data(test_user_id).await;
    
    // Create nodes and links
    repo.upsert_node_sense(sense_id, test_user_id).await.unwrap();
    repo.upsert_node_word(target_word_id_1).await.unwrap();
    repo.upsert_node_word(target_word_id_2).await.unwrap();
    
    repo.create_sense_word_link(
        test_user_id,
        sense_id,
        source_word_id,
        target_word_id_1,
        SenseWordLinkKind::Synonym,
        None,
    )
    .await
    .unwrap();
    repo.create_sense_word_link(
        test_user_id,
        sense_id,
        source_word_id,
        target_word_id_2,
        SenseWordLinkKind::Antonym,
        None,
    )
    .await
    .unwrap();
    
    // List links
    let filter = SenseLinkFilter {
        user_id: test_user_id,
        sense_id,
        kind: None,
        limit: 10,
        offset: 0,
    };
    
    let result = repo.list_sense_word_links(filter).await;
    assert!(result.is_ok());
    let links = result.unwrap();
    assert!(links.len() >= 2);
    
    cleanup_test_data(test_user_id).await;
}

#[tokio::test]
async fn test_remove_links_for_sense() {
    let repo = setup_test_graph().await;
    let test_user_id = 99990;
    let sense_id = 1004;
    let source_word_id = 304;
    let target_word_id_1 = 405;
    let target_word_id_2 = 406;
    
    cleanup_test_data(test_user_id).await;
    
    // Create nodes and links
    repo.upsert_node_sense(sense_id, test_user_id).await.unwrap();
    repo.upsert_node_word(target_word_id_1).await.unwrap();
    repo.upsert_node_word(target_word_id_2).await.unwrap();
    
    repo.create_sense_word_link(
        test_user_id,
        sense_id,
        source_word_id,
        target_word_id_1,
        SenseWordLinkKind::Synonym,
        None,
    )
    .await
    .unwrap();
    repo.create_sense_word_link(
        test_user_id,
        sense_id,
        source_word_id,
        target_word_id_2,
        SenseWordLinkKind::Antonym,
        None,
    )
    .await
    .unwrap();
    
    // Remove all links for sense
    let result = repo.remove_links_for_sense(sense_id).await;
    assert!(result.is_ok());
    
    // Verify links are gone
    let filter = SenseLinkFilter {
        user_id: test_user_id,
        sense_id,
        kind: None,
        limit: 10,
        offset: 0,
    };
    let links = repo.list_sense_word_links(filter).await.unwrap();
    assert_eq!(links.len(), 0);
    
    cleanup_test_data(test_user_id).await;
}
