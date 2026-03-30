use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn maty(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("MatyMemory").unwrap();
    cmd.arg("--db").arg(dir.path().join("test.db"));
    cmd
}

/// Run `remember` and return the created memory ID by parsing JSON output.
fn remember(dir: &TempDir, content: &str) -> String {
    let output = maty(dir)
        .args(["--json", "remember", content, "--type", "semantic", "--tags", "test"])
        .output()
        .unwrap();
    assert!(output.status.success(), "remember failed: {}", String::from_utf8_lossy(&output.stderr));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    v["data"]["memory"]["id"].as_str().unwrap().to_string()
}

// ---------------------------------------------------------------------------
// Remember + Get roundtrip
// ---------------------------------------------------------------------------

#[test]
fn remember_and_get_roundtrip() {
    let dir = TempDir::new().unwrap();

    // Create
    let output = maty(&dir)
        .args([
            "--json", "remember", "Auth uses JWT",
            "--type", "semantic",
            "--tags", "auth,jwt",
            "--importance", "0.9",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let created: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let id = created["data"]["memory"]["id"].as_str().unwrap();
    assert!(!id.is_empty());
    assert_eq!(created["data"]["memory"]["content"].as_str().unwrap(), "Auth uses JWT");

    // Get
    let output = maty(&dir)
        .args(["--json", "get", id])
        .output()
        .unwrap();
    assert!(output.status.success());
    let fetched: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(fetched["data"]["memory"]["content"].as_str().unwrap(), "Auth uses JWT");
    assert_eq!(fetched["data"]["memory"]["memory_type"].as_str().unwrap(), "semantic");
}

// ---------------------------------------------------------------------------
// Recall
// ---------------------------------------------------------------------------

#[test]
fn recall_text_search() {
    let dir = TempDir::new().unwrap();
    remember(&dir, "Auth uses JWT RS256");
    remember(&dir, "Prefers Rust over Go");
    remember(&dir, "Database uses PostgreSQL");

    let output = maty(&dir)
        .args(["--json", "recall", "JWT"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let results: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let list = results["data"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0]["content"].as_str().unwrap().contains("JWT"));
}

#[test]
fn recall_type_filter() {
    let dir = TempDir::new().unwrap();

    // Create semantic (default from helper)
    remember(&dir, "semantic memory");

    // Create episodic
    maty(&dir)
        .args(["remember", "episodic event", "--type", "episodic", "--tags", "test"])
        .assert()
        .success();

    let output = maty(&dir)
        .args(["--json", "recall", "--type", "semantic"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let results: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let list = results["data"].as_array().unwrap();
    assert!(list.iter().all(|m| m["memory_type"].as_str().unwrap() == "semantic"));
}

// ---------------------------------------------------------------------------
// Inspect
// ---------------------------------------------------------------------------

#[test]
fn inspect_shows_provenance_and_tags() {
    let dir = TempDir::new().unwrap();
    let id = remember(&dir, "Inspect me");

    let output = maty(&dir)
        .args(["--json", "inspect", &id])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let data = &v["data"];

    // Has memory
    assert_eq!(data["memory"]["memory"]["content"].as_str().unwrap(), "Inspect me");
    // Has provenance with actor
    assert!(data["provenance"]["actor"].as_str().is_some());
    // Has tags
    assert!(data["memory"]["tags"].as_array().is_some());
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

#[test]
fn update_memory_content() {
    let dir = TempDir::new().unwrap();
    let id = remember(&dir, "old content");

    maty(&dir)
        .args(["--json", "update", &id, "--content", "new content"])
        .assert()
        .success();

    let output = maty(&dir)
        .args(["--json", "get", &id])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["data"]["memory"]["content"].as_str().unwrap(), "new content");
}

// ---------------------------------------------------------------------------
// Pin
// ---------------------------------------------------------------------------

#[test]
fn pin_changes_type() {
    let dir = TempDir::new().unwrap();
    let id = remember(&dir, "important fact");

    let output = maty(&dir)
        .args(["--json", "pin", &id])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["data"]["new_type"].as_str().unwrap(), "pinned");
}

// ---------------------------------------------------------------------------
// Archive + repeat archive fails
// ---------------------------------------------------------------------------

#[test]
fn archive_and_double_archive_fails() {
    let dir = TempDir::new().unwrap();
    let id = remember(&dir, "to archive");

    // First archive succeeds
    let output = maty(&dir)
        .args(["--json", "archive", &id])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["data"]["new_status"].as_str().unwrap(), "archived");

    // Second archive fails
    maty(&dir)
        .args(["archive", &id])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// Invalidate
// ---------------------------------------------------------------------------

#[test]
fn invalidate_memory() {
    let dir = TempDir::new().unwrap();
    let id = remember(&dir, "wrong info");

    let output = maty(&dir)
        .args(["--json", "invalidate", &id])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["data"]["new_status"].as_str().unwrap(), "invalidated");
}

// ---------------------------------------------------------------------------
// Supersede
// ---------------------------------------------------------------------------

#[test]
fn supersede_memory() {
    let dir = TempDir::new().unwrap();
    let old_id = remember(&dir, "old fact");
    let new_id = remember(&dir, "new fact");

    let output = maty(&dir)
        .args(["--json", "supersede", &old_id, &new_id])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["data"]["new_status"].as_str().unwrap(), "superseded");
    assert_eq!(v["data"]["id"].as_str().unwrap(), old_id);
}

// ---------------------------------------------------------------------------
// Forget
// ---------------------------------------------------------------------------

#[test]
fn forget_deletes_memory() {
    let dir = TempDir::new().unwrap();
    let id = remember(&dir, "forget me");

    // Forget succeeds
    maty(&dir)
        .args(["forget", &id])
        .assert()
        .success();

    // Get after forget returns error
    maty(&dir)
        .args(["get", &id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Not found")));
}

// ---------------------------------------------------------------------------
// Tag operations (tag + untag)
// ---------------------------------------------------------------------------

#[test]
fn tag_add_and_remove() {
    let dir = TempDir::new().unwrap();
    let id = remember(&dir, "taggable");

    // Add tags
    let output = maty(&dir)
        .args(["--json", "tag", &id, "rust,memory"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let tags: Vec<&str> = v["data"]["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect();
    assert!(tags.contains(&"rust"));
    assert!(tags.contains(&"memory"));

    // Remove tag
    let output = maty(&dir)
        .args(["--json", "untag", &id, "rust"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let tags: Vec<&str> = v["data"]["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect();
    assert!(!tags.contains(&"rust"));
    assert!(tags.contains(&"memory"));
}

// ---------------------------------------------------------------------------
// Relate
// ---------------------------------------------------------------------------

#[test]
fn relate_creates_relation() {
    let dir = TempDir::new().unwrap();
    let id1 = remember(&dir, "concept A");
    let id2 = remember(&dir, "concept B");

    let output = maty(&dir)
        .args(["--json", "relate", &id1, &id2, "--relation-type", "related_to"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["data"]["from_id"].as_str().unwrap(), id1);
    assert_eq!(v["data"]["to_id"].as_str().unwrap(), id2);

    // Verify via inspect
    let output = maty(&dir)
        .args(["--json", "inspect", &id1])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let rels = v["data"]["relations"].as_array().unwrap();
    assert!(!rels.is_empty());
    assert_eq!(rels[0]["relation_type"].as_str().unwrap(), "related_to");
}

// ---------------------------------------------------------------------------
// List + Pagination
// ---------------------------------------------------------------------------

#[test]
fn list_with_pagination() {
    let dir = TempDir::new().unwrap();
    for i in 0..5 {
        remember(&dir, &format!("memory {i}"));
    }

    // Page 1
    let output = maty(&dir)
        .args(["--json", "list", "--limit", "2"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let page1 = v["data"].as_array().unwrap();
    assert_eq!(page1.len(), 2);

    // Page 2
    let output = maty(&dir)
        .args(["--json", "list", "--limit", "2", "--offset", "2"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let page2 = v["data"].as_array().unwrap();
    assert_eq!(page2.len(), 2);

    // Pages have different entries
    assert_ne!(page1[0]["id"], page2[0]["id"]);
}

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

#[test]
fn stats_reports_counts() {
    let dir = TempDir::new().unwrap();
    remember(&dir, "one");
    remember(&dir, "two");

    let output = maty(&dir)
        .args(["--json", "stats"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let stats = &v["data"];
    assert_eq!(stats["total"].as_u64().unwrap(), 2);
    assert!(stats["by_type"]["semantic"].as_u64().unwrap() >= 2);
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn get_nonexistent_fails() {
    let dir = TempDir::new().unwrap();
    maty(&dir)
        .args(["get", "nonexistent-id"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Not found")));
}

#[test]
fn forget_nonexistent_fails() {
    let dir = TempDir::new().unwrap();
    maty(&dir)
        .args(["forget", "nonexistent-id"])
        .assert()
        .failure();
}

#[test]
fn get_nonexistent_json_error() {
    let dir = TempDir::new().unwrap();
    let output = maty(&dir)
        .args(["--json", "get", "nonexistent-id"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert!(v["error"].as_str().is_some());
}

// ---------------------------------------------------------------------------
// DB path via --db flag
// ---------------------------------------------------------------------------

#[test]
fn db_flag_creates_database() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("custom.db");

    let mut cmd = Command::cargo_bin("MatyMemory").unwrap();
    cmd.arg("--db")
        .arg(&db_path)
        .args(["--json", "remember", "db flag test", "--type", "semantic", "--tags", "test"])
        .assert()
        .success();

    assert!(db_path.exists());
}

// ---------------------------------------------------------------------------
// DB path via MATY_DB_PATH env var
// ---------------------------------------------------------------------------

#[test]
fn env_var_db_path() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("env.db");

    let mut cmd = Command::cargo_bin("MatyMemory").unwrap();
    cmd.env("MATY_DB_PATH", &db_path)
        .args(["--json", "remember", "env test", "--type", "semantic", "--tags", "test"])
        .assert()
        .success();

    assert!(db_path.exists());
}

// ---------------------------------------------------------------------------
// Quiet mode
// ---------------------------------------------------------------------------

#[test]
fn quiet_mode_returns_id_only() {
    let dir = TempDir::new().unwrap();

    let output = maty(&dir)
        .args(["--quiet", "remember", "quiet test", "--type", "semantic", "--tags", "test"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let id = stdout.trim();
    // Should be a UUID-like string with no extra formatting
    assert!(id.len() >= 32, "Expected UUID, got: {id}");
    assert!(!id.contains('\n'));
}
