use crate::auth;
use crate::error::{AppError, AppResult};
use crate::guards::AuthUser;
use crate::models::{AuthResponse, LoginReq, User, UserOut, WechatLoginReq};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;

pub async fn wechat_login(
    State(state): State<AppState>,
    Json(req): Json<WechatLoginReq>,
) -> AppResult<Json<AuthResponse>> {
    if req.code.is_empty() {
        return Err(AppError::BadRequest("缺少微信登录 code".into()));
    }
    let openid = auth::wechat_openid(&state.cfg, &state.http, &req.code).await?;
    let user = upsert_student(&state, &openid).await?;
    let token = auth::sign_token(&state.cfg, user.id, &user.role, &user.name)?;
    Ok(Json(AuthResponse {
        token,
        user: UserOut::from(&user),
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> AppResult<Json<AuthResponse>> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest("用户名和密码不能为空".into()));
    }
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&req.username)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".into()))?;
    let hash = user
        .password_hash
        .as_deref()
        .ok_or_else(|| AppError::Unauthorized("该账号不支持密码登录".into()))?;
    if !auth::verify_password(&req.password, hash) {
        return Err(AppError::Unauthorized("用户名或密码错误".into()));
    }
    let token = auth::sign_token(&state.cfg, user.id, &user.role, &user.name)?;
    Ok(Json(AuthResponse {
        token,
        user: UserOut::from(&user),
    }))
}

pub async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<UserOut>> {
    let u = crate::handlers::get_user(&state.db, user.id()).await?;
    Ok(Json(UserOut::from(&u)))
}

async fn upsert_student(state: &AppState, openid: &str) -> AppResult<User> {
    if let Some(u) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE openid = $1")
        .bind(openid)
        .fetch_optional(&state.db)
        .await?
    {
        return Ok(u);
    }
    let suffix: String = openid
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let name = format!("同学{suffix}");
    sqlx::query_as::<_, User>(
        "INSERT INTO users (openid, role, name) VALUES ($1, 'student', $2) RETURNING *",
    )
    .bind(openid)
    .bind(&name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("创建学生账号失败: {e}")))
}
