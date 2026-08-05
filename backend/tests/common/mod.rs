use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use axum::Router;
use campus_voice_backend::auth;
use campus_voice_backend::models::User;
use campus_voice_backend::state::AppState;
use campus_voice_backend::{connect_pool, migrate, AppConfig};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct TestApp {
    pub router: Router,
    pub state: AppState,
}

pub async fn setup() -> anyhow::Result<TestApp> {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://campus_voice:campus_voice@localhost:5432/campus_voice_test".to_string()
    });
    let pool = connect_pool(&database_url).await?;
    migrate(&pool).await?;
    sqlx::query(
        "TRUNCATE users, issues, issue_progress, replies, attachments, notifications, audit_logs, public_cases
         RESTART IDENTITY CASCADE",
    )
    .execute(&pool)
    .await?;
    sqlx::query("SELECT setval('issue_no_seq', 1, false)")
        .execute(&pool)
        .await?;

    let cfg = AppConfig {
        database_url,
        server_addr: "0.0.0.0:0".to_string(),
        jwt_secret: "test-secret-test-secret-test-secret-test".to_string(),
        jwt_expires_hours: 1,
        wechat_appid: None,
        wechat_secret: None,
        admin_username: "admin".to_string(),
        admin_password: "admin-pw".to_string(),
        upload_dir: std::env::temp_dir().join("campus_voice_test_uploads"),
        max_upload_bytes: 5 * 1024 * 1024,
        max_attachments: 3,
    };

    let state = AppState {
        db: pool,
        cfg: Arc::new(cfg),
        http: reqwest::Client::new(),
    };
    let router = campus_voice_backend::routes::build_router(state.clone());
    Ok(TestApp { router, state })
}

pub async fn register_admin(app: &TestApp) -> anyhow::Result<()> {
    let hash = auth::hash_password("admin-pw")?;
    sqlx::query(
        "INSERT INTO users (username, password_hash, role, name) VALUES ($1, $2, 'admin', '管理员')",
    )
    .bind("admin")
    .bind(hash)
    .execute(&app.state.db)
    .await?;
    Ok(())
}

pub async fn create_department_staff(
    app: &TestApp,
    username: &str,
    password: &str,
) -> anyhow::Result<User> {
    let dept =
        sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM departments WHERE code = 'logistics'")
            .fetch_one(&app.state.db)
            .await?;
    let hash = auth::hash_password(password)?;
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password_hash, role, name, department_id)
         VALUES ($1, $2, 'department_staff', $3, $4) RETURNING *",
    )
    .bind(username)
    .bind(hash)
    .bind(username)
    .bind(dept)
    .fetch_one(&app.state.db)
    .await?;
    Ok(user)
}

pub async fn request(
    app: &TestApp,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> anyhow::Result<(StatusCode, Value)> {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let req = match body {
        Some(v) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))?,
        None => builder.body(Body::empty())?,
    };
    let resp = tower::ServiceExt::oneshot(app.router.clone(), req).await?;
    let status = resp.status();
    let bytes = resp.into_body().collect().await?.to_bytes();
    let body: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)?
    };
    Ok((status, body))
}

pub async fn login(app: &TestApp, username: &str, password: &str) -> anyhow::Result<String> {
    let (status, body) = request(
        app,
        Method::POST,
        "/api/v1/auth/login",
        None,
        Some(json!({ "username": username, "password": password })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "登录失败: {body}");
    Ok(body["token"].as_str().unwrap().to_string())
}

pub async fn student_token(app: &TestApp) -> anyhow::Result<String> {
    let (status, body) = request(
        app,
        Method::POST,
        "/api/v1/auth/wechat",
        None,
        Some(json!({ "code": "devstudent01" })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "学生登录失败: {body}");
    Ok(body["token"].as_str().unwrap().to_string())
}
