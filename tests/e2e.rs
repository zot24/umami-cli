//! End-to-end tests that run against a real Umami instance via Docker.
//!
//! Prerequisites:
//!   docker compose -f docker-compose.test.yml up -d
//!   Wait for umami to be healthy (port 3099)
//!
//! Run with:
//!   cargo test --test e2e -- --test-threads=1
//!
//! These tests are ignored by default. Run explicitly:
//!   cargo test --test e2e -- --ignored --test-threads=1

use serde_json::Value;
use serial_test::serial;

const BASE_URL: &str = "http://localhost:3099";
const USERNAME: &str = "admin";
const PASSWORD: &str = "umami";

fn client_no_auth() -> umami_cli::api::UmamiClient {
    umami_cli::api::UmamiClient::new(BASE_URL, None)
}

fn client_with_token(token: &str) -> umami_cli::api::UmamiClient {
    umami_cli::api::UmamiClient::new(BASE_URL, Some(token.to_string()))
}

async fn login() -> String {
    let mut client = client_no_auth();
    let result = client.login(USERNAME, PASSWORD).await.unwrap();
    result["token"].as_str().unwrap().to_string()
}

// ========== Auth Tests ==========

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_auth_login_success() {
    let mut client = client_no_auth();
    let result = client.login(USERNAME, PASSWORD).await;
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data["token"].as_str().is_some());
    assert_eq!(data["user"]["username"], USERNAME);
}

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_auth_login_bad_password() {
    let mut client = client_no_auth();
    let result = client.login(USERNAME, "wrong-password").await;
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_auth_verify() {
    let token = login().await;
    let client = client_with_token(&token);
    let result = client.verify().await;
    assert!(result.is_ok());
}

// ========== User/Me Tests ==========

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_me() {
    let token = login().await;
    let client = client_with_token(&token);
    let me: Value = client.get("/api/me").await.unwrap();
    assert_eq!(me["username"], USERNAME);
    assert!(me["isAdmin"].as_bool().unwrap_or(false));
}

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_me_websites() {
    let token = login().await;
    let client = client_with_token(&token);
    let query = vec![];
    let result: Value = client.get_with_query("/api/me/websites", &query).await.unwrap();
    // Should return an object with data array or an array
    assert!(result.is_object() || result.is_array());
}

// ========== Website CRUD Tests ==========

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_website_full_lifecycle() {
    let token = login().await;
    let client = client_with_token(&token);

    // Create
    let body = serde_json::json!({
        "name": "E2E Test Site",
        "domain": "e2e-test.example.com"
    });
    let created: Value = client.post("/api/websites", &body).await.unwrap();
    let website_id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["name"], "E2E Test Site");
    assert_eq!(created["domain"], "e2e-test.example.com");

    // Read
    let fetched: Value = client
        .get(&format!("/api/websites/{website_id}"))
        .await
        .unwrap();
    assert_eq!(fetched["id"], website_id);
    assert_eq!(fetched["name"], "E2E Test Site");

    // Update
    let update_body = serde_json::json!({ "name": "E2E Updated Site" });
    let updated: Value = client
        .post(&format!("/api/websites/{website_id}"), &update_body)
        .await
        .unwrap();
    assert_eq!(updated["name"], "E2E Updated Site");

    // List
    let query = vec![("page".to_string(), "1".to_string())];
    let list: Value = client
        .get_with_query("/api/websites", &query)
        .await
        .unwrap();
    let sites = list["data"].as_array().unwrap();
    assert!(sites.iter().any(|s| s["id"] == website_id));

    // Active visitors
    let active: Value = client
        .get(&format!("/api/websites/{website_id}/active"))
        .await
        .unwrap();
    assert!(active.get("visitors").is_some());

    // Stats (empty but should not error)
    let now = chrono::Utc::now().timestamp_millis();
    let week_ago = now - 7 * 24 * 60 * 60 * 1000;
    let stats_query = vec![
        ("startAt".to_string(), week_ago.to_string()),
        ("endAt".to_string(), now.to_string()),
    ];
    let stats: Value = client
        .get_with_query(&format!("/api/websites/{website_id}/stats"), &stats_query)
        .await
        .unwrap();
    assert!(stats.is_object());

    // Pageviews
    let pv_query = vec![
        ("startAt".to_string(), week_ago.to_string()),
        ("endAt".to_string(), now.to_string()),
        ("unit".to_string(), "day".to_string()),
        ("timezone".to_string(), "UTC".to_string()),
    ];
    let pageviews: Value = client
        .get_with_query(
            &format!("/api/websites/{website_id}/pageviews"),
            &pv_query,
        )
        .await
        .unwrap();
    assert!(pageviews.is_object());

    // Metrics
    let metrics_query = vec![
        ("startAt".to_string(), week_ago.to_string()),
        ("endAt".to_string(), now.to_string()),
        ("type".to_string(), "url".to_string()),
    ];
    let metrics: Value = client
        .get_with_query(
            &format!("/api/websites/{website_id}/metrics"),
            &metrics_query,
        )
        .await
        .unwrap();
    assert!(metrics.is_array() || metrics.is_object());

    // Sessions list
    let sessions_query = vec![
        ("startAt".to_string(), week_ago.to_string()),
        ("endAt".to_string(), now.to_string()),
        ("page".to_string(), "1".to_string()),
    ];
    let sessions: Value = client
        .get_with_query(
            &format!("/api/websites/{website_id}/sessions"),
            &sessions_query,
        )
        .await
        .unwrap();
    assert!(sessions.is_object());

    // Events list
    let events: Value = client
        .get_with_query(
            &format!("/api/websites/{website_id}/events"),
            &sessions_query,
        )
        .await
        .unwrap();
    assert!(events.is_object() || events.is_array());

    // Reset
    let reset: Value = client
        .post(
            &format!("/api/websites/{website_id}/reset"),
            &serde_json::json!({}),
        )
        .await
        .unwrap();
    assert!(reset.is_object());

    // Delete
    let deleted: Value = client
        .delete(&format!("/api/websites/{website_id}"))
        .await
        .unwrap();
    assert!(deleted.is_object() || deleted.is_null());
}

// ========== Team CRUD Tests ==========

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_team_full_lifecycle() {
    let token = login().await;
    let client = client_with_token(&token);

    // Create team
    let body = serde_json::json!({ "name": "E2E Test Team" });
    let created: Value = client.post("/api/teams", &body).await.unwrap();
    let team_id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["name"], "E2E Test Team");

    // Get team
    let fetched: Value = client
        .get(&format!("/api/teams/{team_id}"))
        .await
        .unwrap();
    assert_eq!(fetched["id"], team_id);

    // Update team
    let update_body = serde_json::json!({ "name": "E2E Updated Team" });
    let updated: Value = client
        .post(&format!("/api/teams/{team_id}"), &update_body)
        .await
        .unwrap();
    assert_eq!(updated["name"], "E2E Updated Team");

    // List teams
    let teams: Value = client.get("/api/teams").await.unwrap();
    assert!(teams.is_array() || teams.is_object());

    // List team members
    let members: Value = client
        .get(&format!("/api/teams/{team_id}/users"))
        .await
        .unwrap();
    assert!(members.is_array() || members.is_object());

    // List team websites
    let websites: Value = client
        .get(&format!("/api/teams/{team_id}/websites"))
        .await
        .unwrap();
    assert!(websites.is_array() || websites.is_object());

    // Delete team
    let deleted: Value = client
        .delete(&format!("/api/teams/{team_id}"))
        .await
        .unwrap();
    assert!(deleted.is_object() || deleted.is_null());
}

// ========== User Management Tests (Admin) ==========

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_user_crud() {
    let token = login().await;
    let client = client_with_token(&token);

    // Create user
    let body = serde_json::json!({
        "username": "e2e-test-user",
        "password": "test-password-123",
        "role": "user"
    });
    let created: Value = client.post("/api/users", &body).await.unwrap();
    let user_id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["username"], "e2e-test-user");

    // Get user
    let fetched: Value = client
        .get(&format!("/api/users/{user_id}"))
        .await
        .unwrap();
    assert_eq!(fetched["username"], "e2e-test-user");

    // Update user
    let update_body = serde_json::json!({ "role": "view-only" });
    let _updated: Value = client
        .post(&format!("/api/users/{user_id}"), &update_body)
        .await
        .unwrap();

    // Delete user
    let deleted: Value = client
        .delete(&format!("/api/users/{user_id}"))
        .await
        .unwrap();
    assert!(deleted.is_object() || deleted.is_null());
}

// ========== Admin Endpoints Tests ==========

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_admin_list_users() {
    let token = login().await;
    let client = client_with_token(&token);
    let query = vec![("page".to_string(), "1".to_string())];
    let result: Value = client
        .get_with_query("/api/admin/users", &query)
        .await
        .unwrap();
    assert!(result.is_object());
    let data = result.get("data").and_then(|d| d.as_array());
    assert!(data.is_some());
    assert!(!data.unwrap().is_empty()); // At least the admin user
}

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_admin_list_websites() {
    let token = login().await;
    let client = client_with_token(&token);
    let query = vec![("page".to_string(), "1".to_string())];
    let result: Value = client
        .get_with_query("/api/admin/websites", &query)
        .await
        .unwrap();
    assert!(result.is_object());
}

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_admin_list_teams() {
    let token = login().await;
    let client = client_with_token(&token);
    let query = vec![("page".to_string(), "1".to_string())];
    let result: Value = client
        .get_with_query("/api/admin/teams", &query)
        .await
        .unwrap();
    assert!(result.is_object());
}

// ========== Event Tracking Tests ==========

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_send_event_no_auth() {
    // First create a website to get an ID
    let token = login().await;
    let client = client_with_token(&token);
    let body = serde_json::json!({
        "name": "Event Test Site",
        "domain": "event-test.example.com"
    });
    let site: Value = client.post("/api/websites", &body).await.unwrap();
    let website_id = site["id"].as_str().unwrap();

    // Send event via /api/send (no auth required)
    let http = reqwest::Client::new();
    let payload = serde_json::json!({
        "payload": {
            "website": website_id,
            "url": "/test-page",
            "hostname": "event-test.example.com"
        }
    });
    let resp = http
        .post(format!("{BASE_URL}/api/send"))
        .json(&payload)
        .send()
        .await
        .unwrap();
    // Umami may return 200 or may reject if hostname doesn't match — both are valid behaviors
    assert!(resp.status().as_u16() < 500);

    // Cleanup
    let _: Value = client
        .delete(&format!("/api/websites/{website_id}"))
        .await
        .unwrap();
}

// ========== Realtime Tests ==========

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_realtime() {
    let token = login().await;
    let client = client_with_token(&token);

    // Create a website first
    let body = serde_json::json!({
        "name": "Realtime Test Site",
        "domain": "realtime.example.com"
    });
    let site: Value = client.post("/api/websites", &body).await.unwrap();
    let website_id = site["id"].as_str().unwrap().to_string();

    // Get realtime data
    let result: Value = client
        .get(&format!("/api/realtime/{website_id}"))
        .await
        .unwrap();
    assert!(result.is_object());
    // Should have totals
    assert!(result.get("totals").is_some() || result.get("series").is_some());

    // Cleanup
    let _: Value = client
        .delete(&format!("/api/websites/{website_id}"))
        .await
        .unwrap();
}

// ========== Report Tests ==========

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_reports_list() {
    let token = login().await;
    let client = client_with_token(&token);
    let query = vec![("page".to_string(), "1".to_string())];
    let result: Value = client
        .get_with_query("/api/reports", &query)
        .await
        .unwrap();
    assert!(result.is_object() || result.is_array());
}

#[tokio::test]
#[serial]
#[ignore]
async fn e2e_report_funnel() {
    let token = login().await;
    let client = client_with_token(&token);

    // Create website for the report
    let site_body = serde_json::json!({
        "name": "Funnel Test Site",
        "domain": "funnel.example.com"
    });
    let site: Value = client.post("/api/websites", &site_body).await.unwrap();
    let website_id = site["id"].as_str().unwrap().to_string();

    // Run funnel report
    let report_body = serde_json::json!({
        "websiteId": website_id,
        "type": "funnel",
        "parameters": {
            "startDate": "2025-01-01T00:00:00.000Z",
            "endDate": "2025-12-31T23:59:59.999Z",
            "timezone": "UTC",
            "urls": ["/step1", "/step2", "/step3"],
            "window": 7
        }
    });
    let result: Value = client
        .post("/api/reports/funnel", &report_body)
        .await
        .unwrap();
    assert!(result.is_object() || result.is_array());

    // Cleanup
    let _: Value = client
        .delete(&format!("/api/websites/{website_id}"))
        .await
        .unwrap();
}
