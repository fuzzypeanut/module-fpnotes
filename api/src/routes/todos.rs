use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::Claims, error::AppError, AppState};
use super::notes::{fetch_note_full, TodoRow};

#[derive(Deserialize)]
pub struct CreateTodo {
    pub text:     String,
    pub position: Option<i32>,
}

pub async fn create(
    claims: Claims,
    State(state): State<AppState>,
    Path(note_id): Path<Uuid>,
    Json(body): Json<CreateTodo>,
) -> Result<(StatusCode, Json<TodoRow>), AppError> {
    let email = claims.email.as_deref().unwrap_or_default();
    let note = fetch_note_full(&state.pool, note_id, &claims.sub, email)
        .await?
        .ok_or(AppError::NotFound)?;

    if note.permission == "view" {
        return Err(AppError::Forbidden);
    }

    let row = sqlx::query_as::<_, TodoRow>(
        "INSERT INTO todos (note_id, text, position) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(note_id)
    .bind(&body.text)
    .bind(body.position.unwrap_or(0))
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Deserialize)]
pub struct UpdateTodo {
    pub text:     Option<String>,
    pub checked:  Option<bool>,
    pub position: Option<i32>,
}

pub async fn update(
    claims: Claims,
    State(state): State<AppState>,
    Path((note_id, todo_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateTodo>,
) -> Result<Json<TodoRow>, AppError> {
    let email = claims.email.as_deref().unwrap_or_default();
    let note = fetch_note_full(&state.pool, note_id, &claims.sub, email)
        .await?
        .ok_or(AppError::NotFound)?;

    if note.permission == "view" {
        return Err(AppError::Forbidden);
    }

    let mut sets: Vec<String> = vec![];
    let mut i: i32 = 3;

    if body.text.is_some()     { sets.push(format!("text = ${i}"));     i += 1; }
    if body.checked.is_some()  { sets.push(format!("checked = ${i}"));  i += 1; }
    if body.position.is_some() { sets.push(format!("position = ${i}")); }

    if !sets.is_empty() {
        let sql = format!("UPDATE todos SET {} WHERE id = $1 AND note_id = $2", sets.join(", "));
        let mut q = sqlx::query(&sql).bind(todo_id).bind(note_id);
        if let Some(v) = &body.text     { q = q.bind(v); }
        if let Some(v) = body.checked   { q = q.bind(v); }
        if let Some(v) = body.position  { q = q.bind(v); }
        q.execute(&state.pool).await?;
    }

    sqlx::query_as::<_, TodoRow>("SELECT * FROM todos WHERE id = $1 AND note_id = $2")
        .bind(todo_id)
        .bind(note_id)
        .fetch_optional(&state.pool)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn delete(
    claims: Claims,
    State(state): State<AppState>,
    Path((note_id, todo_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let email = claims.email.as_deref().unwrap_or_default();
    let note = fetch_note_full(&state.pool, note_id, &claims.sub, email)
        .await?
        .ok_or(AppError::NotFound)?;

    if note.permission == "view" {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM todos WHERE id = $1 AND note_id = $2")
        .bind(todo_id)
        .bind(note_id)
        .execute(&state.pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
