use std::{fs, sync::Arc};

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
use web_note_core::Notes;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\nrest";

struct Client {
    _dir: tempfile::TempDir,
    app: Router,
}

impl Client {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let notes = Arc::new(Notes::open(dir.path(), 1024).unwrap());
        Self {
            app: web_note_server::app(notes),
            _dir: dir,
        }
    }

    async fn raw(
        &self,
        method: Method,
        uri: &str,
        body: Body,
        content_type: &str,
    ) -> (StatusCode, Vec<u8>) {
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", content_type)
            .body(body)
            .unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        (
            status,
            res.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
    }

    async fn call(&self, method: Method, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        let body = body.map_or_else(Body::empty, |v| Body::from(v.to_string()));
        let (status, bytes) = self.raw(method, uri, body, "application/json").await;
        let value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        (status, value)
    }

    async fn get(&self, uri: &str) -> (StatusCode, Value) {
        self.call(Method::GET, uri, None).await
    }

    async fn post(&self, uri: &str, body: Value) -> (StatusCode, Value) {
        self.call(Method::POST, uri, Some(body)).await
    }

    async fn create(&self, notebook: &str, body: &str, tags: &[&str]) -> Value {
        let (status, note) = self
            .post(
                "/api/notes",
                json!({"notebook": notebook, "body": body, "tags": tags}),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED);
        note
    }
}

#[tokio::test]
async fn note_lifecycle_with_links_and_conflict() {
    let c = Client::new();
    let target = c.create("Inbox", "# Target\n", &["t"]).await;
    let id = target["meta"]["id"].as_str().unwrap();
    let source = format!("# Source\n\n[[{id}|Target]]\n");
    c.create("Inbox", &source, &[]).await;

    let (_, links) = c.get(&format!("/api/notes/{id}/backlinks")).await;
    assert_eq!(links.as_array().unwrap().len(), 1);

    let version = target["version"].as_str().unwrap();
    let save = |body: &str, base: &str, force: bool| json!({"body": body, "tags": ["t"], "base_version": base, "force": force});
    let (status, saved) = c
        .call(
            Method::PUT,
            &format!("/api/notes/{id}"),
            Some(save("# Renamed\n", version, false)),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["meta"]["title"], "Renamed");

    let (status, err) = c
        .call(
            Method::PUT,
            &format!("/api/notes/{id}"),
            Some(save("# Stale\n", version, false)),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(err["code"], "conflict");

    let (status, _) = c
        .call(
            Method::PUT,
            &format!("/api/notes/{id}"),
            Some(save("# Forced\n", version, true)),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (_, trash) = c.get("/api/trash").await;
    assert_eq!(trash.as_array().unwrap().len(), 1);

    let (status, _) = c
        .call(Method::DELETE, &format!("/api/notes/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, err) = c.get(&format!("/api/notes/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(err["code"], "note_not_found");
}

#[tokio::test]
async fn list_filters_and_search() {
    let c = Client::new();
    c.post("/api/notebooks", json!({"name": "Work"})).await;
    c.create("Inbox", "# A\n\nalpha beta\n", &["x", "y"]).await;
    c.create("Work", "# B\n\nalpha gamma\n", &["x"]).await;

    let count = |v: (StatusCode, Value)| v.1.as_array().unwrap().len();
    assert_eq!(count(c.get("/api/notes").await), 2);
    assert_eq!(count(c.get("/api/notes?tag=x&tag=y").await), 1);
    assert_eq!(count(c.get("/api/notes?notebook=Work&q=alpha").await), 1);
    let (_, hits) = c.get("/api/notes?q=gamma").await;
    assert!(
        hits[0]["snippet"]["text"]
            .as_str()
            .unwrap()
            .contains("gamma")
    );
}

#[tokio::test]
async fn notebook_errors_map_to_codes() {
    let c = Client::new();
    let (status, err) = c.post("/api/notebooks", json!({"name": "../x"})).await;
    assert_eq!(
        (status, err["code"].as_str()),
        (StatusCode::BAD_REQUEST, Some("invalid_name"))
    );

    c.create("Inbox", "# A\n", &[]).await;
    let (status, err) = c.call(Method::DELETE, "/api/notebooks/Inbox", None).await;
    assert_eq!(
        (status, err["code"].as_str()),
        (StatusCode::BAD_REQUEST, Some("protected_notebook"))
    );

    c.post("/api/notebooks", json!({"name": "Work"})).await;
    let (status, err) = c.post("/api/notebooks", json!({"name": "Work"})).await;
    assert_eq!(
        (status, err["code"].as_str()),
        (StatusCode::CONFLICT, Some("notebook_exists"))
    );

    let (status, _) = c
        .call(
            Method::PATCH,
            "/api/notebooks/Work",
            Some(json!({"name": "Done"})),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = c.call(Method::DELETE, "/api/notebooks/Done", None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn tags_rename_merges_and_validates() {
    let c = Client::new();
    c.create("Inbox", "# A\n", &["old"]).await;
    c.create("Inbox", "# B\n", &["new"]).await;

    let (status, result) = c
        .post("/api/tags/rename", json!({"from": "old", "to": "new"}))
        .await;
    assert_eq!((status, &result["affected"]), (StatusCode::OK, &json!(1)));
    let (_, tags) = c.get("/api/tags").await;
    assert_eq!(tags, json!([{"tag": "new", "count": 2}]));

    let (status, err) = c
        .post("/api/tags/rename", json!({"from": "new", "to": " "}))
        .await;
    assert_eq!(
        (status, err["code"].as_str()),
        (StatusCode::BAD_REQUEST, Some("invalid_name"))
    );

    let (status, result) = c
        .post("/api/tags/rename", json!({"from": "ghost", "to": "x"}))
        .await;
    assert_eq!((status, &result["affected"]), (StatusCode::OK, &json!(0)));
}

#[tokio::test]
async fn tags_rename_and_assets() {
    let c = Client::new();
    c.create("Inbox", "# A\n", &["old"]).await;
    let (_, result) = c
        .post("/api/tags/rename", json!({"from": "old", "to": "new"}))
        .await;
    assert_eq!(result["affected"], 1);
    let (_, tags) = c.get("/api/tags").await;
    assert_eq!(tags, json!([{"tag": "new", "count": 1}]));

    let (status, bytes) = c
        .raw(
            Method::POST,
            "/api/assets",
            Body::from(PNG.to_vec()),
            "application/octet-stream",
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let up: Value = serde_json::from_slice(&bytes).unwrap();
    let uri = format!("/api/assets/{}", up["name"].as_str().unwrap());
    let (status, got) = c.raw(Method::GET, &uri, Body::empty(), "").await;
    assert_eq!((status, got.as_slice()), (StatusCode::OK, PNG));

    let (status, _) = c
        .raw(
            Method::POST,
            "/api/assets",
            Body::from("plain text"),
            "text/plain",
        )
        .await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let (status, _) = c
        .raw(
            Method::POST,
            "/api/assets",
            Body::from(vec![0; 4096]),
            "text/plain",
        )
        .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn trash_restore_and_traversal() {
    let c = Client::new();
    let note = c.create("Inbox", "# A\n", &[]).await;
    let id = note["meta"]["id"].as_str().unwrap();
    c.call(Method::DELETE, &format!("/api/notes/{id}"), None)
        .await;

    let (status, _) = c.call(Method::DELETE, "/api/trash/..%2F..%2Fx", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, trash) = c.get("/api/trash").await;
    let key = trash[0]["key"].as_str().unwrap();
    let (status, restored) = c
        .post(&format!("/api/trash/{key}/restore"), json!({}))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(restored["meta"]["id"], id);
}

#[tokio::test]
async fn rescan_reports_corrupt_files() {
    let dir = tempfile::tempdir().unwrap();
    let notes = Arc::new(Notes::open(dir.path(), 1024).unwrap());
    let c = Client {
        app: web_note_server::app(notes),
        _dir: tempfile::tempdir().unwrap(),
    };
    fs::write(dir.path().join("Inbox/bad.md"), "---\nid: x\nno closing\n").unwrap();

    let (status, body) = c.post("/api/system/rescan", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["errors"][0]["path"], "Inbox/bad.md");
}

#[test]
fn committed_openapi_is_up_to_date() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../openapi.json");
    let committed = fs::read_to_string(path).expect("openapi.json is committed");
    assert_eq!(
        committed.trim_end(),
        web_note_server::openapi_json(),
        "regenerate with: cargo run -p web-note-server -- --openapi > openapi.json"
    );
}
