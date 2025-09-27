// Word module Neo4j schema initialization
// Constraints ensure unique identifiers for Word and UserSense nodes
CREATE CONSTRAINT word_id_unique IF NOT EXISTS FOR (w:Word) REQUIRE w.word_id IS UNIQUE;
CREATE CONSTRAINT sense_id_unique IF NOT EXISTS FOR (s:UserSense) REQUIRE s.sense_id IS UNIQUE;

// Optional index on UserSense.user_id to accelerate queries scoped by user
CREATE INDEX user_sense_user_id_idx IF NOT EXISTS FOR (s:UserSense) ON (s.user_id);

// Relationship indexes to support filtering by user_id + kind
CREATE INDEX word_to_word_rel_idx IF NOT EXISTS FOR ()-[r:WORD_TO_WORD]-() ON (r.user_id, r.kind);
CREATE INDEX sense_to_word_rel_idx IF NOT EXISTS FOR ()-[r:SENSE_TO_WORD]-() ON (r.user_id, r.kind);

// The application should execute MERGE statements for nodes/relationships as part of normal operations.
RETURN "Neo4j schema initialization complete";