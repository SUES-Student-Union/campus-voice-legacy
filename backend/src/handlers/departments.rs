use crate::error::{AppError, AppResult};
use crate::guards::AuthUser;
use crate::models::{Category, CreateDepartmentReq, Department};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;

pub async fn list_departments(
    State(state): State<AppState>,
    _user: AuthUser,
) -> AppResult<Json<Vec<Department>>> {
    let rows = sqlx::query_as::<_, Department>("SELECT * FROM departments ORDER BY name")
        .fetch_all(&state.db)
        .await?;
    Ok(Json(rows))
}

pub async fn list_categories(
    State(state): State<AppState>,
    _user: AuthUser,
) -> AppResult<Json<Vec<Category>>> {
    let rows =
        sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE enabled ORDER BY sort, name")
            .fetch_all(&state.db)
            .await?;
    Ok(Json(rows))
}

pub async fn create_department(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateDepartmentReq>,
) -> AppResult<Json<Department>> {
    user.check(
        crate::permissions::DEPARTMENT_MANAGE,
        &crate::permissions::Ctx::none(),
    )?;
    let name = req.name.trim();
    let code = req.code.trim();
    if name.is_empty() || code.is_empty() {
        return Err(AppError::BadRequest("部门名称与编码不能为空".into()));
    }
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM departments WHERE name = $1 OR code = $2)",
    )
    .bind(name)
    .bind(code)
    .fetch_one(&state.db)
    .await?;
    if exists {
        return Err(AppError::Conflict("部门名称或编码已存在".into()));
    }
    let row = sqlx::query_as::<_, Department>(
        "INSERT INTO departments (name, code, description) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(name)
    .bind(code)
    .bind(req.description.as_deref())
    .fetch_one(&state.db)
    .await
    .map_err(AppError::internal)?;
    Ok(Json(row))
}
