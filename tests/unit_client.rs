use serde_json::{json, Value};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// We test the client directly via reqwest against a wiremock server,
// replicating what UmamiClient does internally.

mod helpers;
use helpers::test_client;

#[tokio::test]
async fn login_success() {
    let server = MockServer::start().await;
    let response_body = json!({
        "token": "test-jwt-token",
        "user": {
            "id": "user-1",
            "username": "admin",
            "role": "admin",
            "createdAt": "2025-01-01T00:00:00.000Z",
            "isAdmin": true
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/auth/login"))
        .and(body_json(json!({ "username": "admin", "password": "umami" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&server)
        .await;

    let mut client = test_client(&server, None);
    let result = client.login("admin", "umami").await;
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data["token"], "test-jwt-token");
    assert_eq!(data["user"]["username"], "admin");
}

#[tokio::test]
async fn login_failure_invalid_credentials() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/auth/login"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let mut client = test_client(&server, None);
    let result = client.login("admin", "wrong").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("401"));
}

#[tokio::test]
async fn get_requires_auth() {
    let server = MockServer::start().await;
    let client = test_client(&server, None);
    let result: Result<Value, _> = client.get("/api/websites").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Not authenticated"));
}

#[tokio::test]
async fn get_websites_success() {
    let server = MockServer::start().await;
    let websites = json!({
        "data": [
            { "id": "web-1", "name": "My Site", "domain": "example.com", "createdAt": "2025-01-01" },
            { "id": "web-2", "name": "Blog", "domain": "blog.example.com", "createdAt": "2025-02-01" }
        ],
        "count": 2,
        "page": 1,
        "pageSize": 20
    });

    Mock::given(method("GET"))
        .and(path("/api/websites"))
        .and(header("Authorization", "Bearer test-token"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&websites))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let query = vec![("page".to_string(), "1".to_string())];
    let result: Value = client.get_with_query("/api/websites", &query).await.unwrap();
    let data = result["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["name"], "My Site");
}

#[tokio::test]
async fn create_website_success() {
    let server = MockServer::start().await;
    let created = json!({
        "id": "new-web-id",
        "name": "New Site",
        "domain": "new.example.com",
        "createdAt": "2025-03-01"
    });

    Mock::given(method("POST"))
        .and(path("/api/websites"))
        .and(header("Authorization", "Bearer test-token"))
        .and(body_json(json!({ "name": "New Site", "domain": "new.example.com" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(&created))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let body = json!({ "name": "New Site", "domain": "new.example.com" });
    let result: Value = client.post("/api/websites", &body).await.unwrap();
    assert_eq!(result["id"], "new-web-id");
}

#[tokio::test]
async fn delete_website_success() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/websites/web-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true })))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let result: Value = client.delete("/api/websites/web-1").await.unwrap();
    assert_eq!(result["ok"], true);
}

#[tokio::test]
async fn api_error_propagated() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/websites/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let result: Result<Value, _> = client.get("/api/websites/nonexistent").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("404"));
}

#[tokio::test]
async fn get_website_stats() {
    let server = MockServer::start().await;
    let stats = json!({
        "pageviews": { "value": 1234, "prev": 1100 },
        "visitors": { "value": 456, "prev": 400 },
        "visits": { "value": 789, "prev": 700 },
        "bounces": { "value": 123, "prev": 100 },
        "totaltime": { "value": 56789, "prev": 50000 }
    });

    Mock::given(method("GET"))
        .and(path("/api/websites/web-1/stats"))
        .and(query_param("startAt", "1000"))
        .and(query_param("endAt", "2000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&stats))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let query = vec![
        ("startAt".to_string(), "1000".to_string()),
        ("endAt".to_string(), "2000".to_string()),
    ];
    let result: Value = client
        .get_with_query("/api/websites/web-1/stats", &query)
        .await
        .unwrap();
    assert!(result["pageviews"]["value"].as_u64().unwrap() == 1234);
}

#[tokio::test]
async fn get_active_visitors() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/websites/web-1/active"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "visitors": 42 })))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let result: Value = client.get("/api/websites/web-1/active").await.unwrap();
    assert_eq!(result["visitors"], 42);
}

#[tokio::test]
async fn get_realtime_data() {
    let server = MockServer::start().await;
    let realtime = json!({
        "countries": { "US": 9, "DE": 3 },
        "urls": { "/": 43, "/about": 12 },
        "referrers": {},
        "events": [],
        "series": { "views": [], "visitors": [] },
        "totals": { "views": 55, "visitors": 12, "events": 0, "countries": 2 },
        "timestamp": 1704067200000_i64
    });

    Mock::given(method("GET"))
        .and(path("/api/realtime/web-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&realtime))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let result: Value = client.get("/api/realtime/web-1").await.unwrap();
    assert_eq!(result["totals"]["views"], 55);
    assert_eq!(result["countries"]["US"], 9);
}

#[tokio::test]
async fn list_sessions() {
    let server = MockServer::start().await;
    let sessions = json!({
        "data": [{
            "id": "session-1",
            "websiteId": "web-1",
            "browser": "chrome",
            "os": "Mac OS",
            "device": "desktop",
            "country": "US",
            "visits": 2,
            "views": 18
        }],
        "count": 1,
        "page": 1,
        "pageSize": 20
    });

    Mock::given(method("GET"))
        .and(path("/api/websites/web-1/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sessions))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let query = vec![
        ("startAt".to_string(), "1000".to_string()),
        ("endAt".to_string(), "2000".to_string()),
        ("page".to_string(), "1".to_string()),
    ];
    let result: Value = client
        .get_with_query("/api/websites/web-1/sessions", &query)
        .await
        .unwrap();
    let data = result["data"].as_array().unwrap();
    assert_eq!(data[0]["browser"], "chrome");
}

#[tokio::test]
async fn create_team() {
    let server = MockServer::start().await;
    let team = json!({
        "id": "team-1",
        "name": "Dev Team",
        "accessCode": "abc123",
        "createdAt": "2025-01-01"
    });

    Mock::given(method("POST"))
        .and(path("/api/teams"))
        .and(body_json(json!({ "name": "Dev Team" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(&team))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let body = json!({ "name": "Dev Team" });
    let result: Value = client.post("/api/teams", &body).await.unwrap();
    assert_eq!(result["name"], "Dev Team");
    assert_eq!(result["accessCode"], "abc123");
}

#[tokio::test]
async fn create_report_funnel() {
    let server = MockServer::start().await;
    let report_result = json!({
        "data": [
            { "url": "/signup", "visitors": 100, "dropoff": 0 },
            { "url": "/onboard", "visitors": 75, "dropoff": 25 },
            { "url": "/activate", "visitors": 50, "dropoff": 25 }
        ]
    });

    Mock::given(method("POST"))
        .and(path("/api/reports/funnel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&report_result))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let body = json!({
        "websiteId": "web-1",
        "type": "funnel",
        "parameters": {
            "startDate": "2025-01-01",
            "endDate": "2025-01-31",
            "timezone": "UTC",
            "urls": ["/signup", "/onboard", "/activate"],
            "window": 7
        }
    });
    let result: Value = client.post("/api/reports/funnel", &body).await.unwrap();
    let data = result["data"].as_array().unwrap();
    assert_eq!(data.len(), 3);
    assert_eq!(data[0]["visitors"], 100);
}

#[tokio::test]
async fn create_link() {
    let server = MockServer::start().await;
    let link = json!({
        "id": "link-1",
        "name": "My Link",
        "url": "https://example.com",
        "slug": "mylink12"
    });

    Mock::given(method("POST"))
        .and(path("/api/links"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&link))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let body = json!({ "name": "My Link", "url": "https://example.com", "slug": "mylink12" });
    let result: Value = client.post("/api/links", &body).await.unwrap();
    assert_eq!(result["slug"], "mylink12");
}

#[tokio::test]
async fn create_pixel() {
    let server = MockServer::start().await;
    let pixel = json!({
        "id": "pixel-1",
        "name": "My Pixel",
        "slug": "mypixel1"
    });

    Mock::given(method("POST"))
        .and(path("/api/pixels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&pixel))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let body = json!({ "name": "My Pixel", "slug": "mypixel1" });
    let result: Value = client.post("/api/pixels", &body).await.unwrap();
    assert_eq!(result["name"], "My Pixel");
}

#[tokio::test]
async fn admin_list_users() {
    let server = MockServer::start().await;
    let users = json!({
        "data": [
            { "id": "user-1", "username": "admin", "role": "admin", "createdAt": "2025-01-01" },
            { "id": "user-2", "username": "viewer", "role": "view-only", "createdAt": "2025-02-01" }
        ],
        "count": 2,
        "page": 1,
        "pageSize": 20
    });

    Mock::given(method("GET"))
        .and(path("/api/admin/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&users))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let query = vec![("page".to_string(), "1".to_string())];
    let result: Value = client
        .get_with_query("/api/admin/users", &query)
        .await
        .unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn share_create() {
    let server = MockServer::start().await;
    let share = json!({
        "id": "share-1",
        "entityId": "web-1",
        "shareType": 1,
        "name": "Public Dashboard",
        "slug": "my-dashboard"
    });

    Mock::given(method("POST"))
        .and(path("/api/share"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&share))
        .mount(&server)
        .await;

    let client = test_client(&server, Some("test-token"));
    let body = json!({
        "entityId": "web-1",
        "shareType": 1,
        "name": "Public Dashboard",
        "slug": "my-dashboard"
    });
    let result: Value = client.post("/api/share", &body).await.unwrap();
    assert_eq!(result["slug"], "my-dashboard");
}
