use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::Claims, error::AppError, AppState};
use super::notes::ShareRow;

#[derive(Deserialize)]
pub struct ShareNote {
    pub shared_with_email: String,
    pub permission:        String,  // "view" | "edit"
}

pub async fn share(
    claims: Claims,
    State(state): State<AppState>,
    Path(note_id): Path<Uuid>,
    Json(body): Json<ShareNote>,
) -> Result<(StatusCode, Json<ShareRow>), AppError> {
    // Only the owner can share a note.
    let row = sqlx::query_as::<_, (String,)>("SELECT owner_id FROM notes WHERE id = $1")
        .bind(note_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    if row.0 != claims.sub {
        return Err(AppError::Forbidden);
    }

    if !matches!(body.permission.as_str(), "view" | "edit") {
        return Err(AppError::BadRequest("permission must be 'view' or 'edit'".into()));
    }

    // Upsert: if already shared with this email, update the permission.
    let share = sqlx::query_as::<_, ShareRow>(
        "INSERT INTO note_shares (note_id, shared_with_email, permission)
         VALUES ($1, $2, $3)
         ON CONFLICT (note_id, shared_with_email)
         DO UPDATE SET permission = EXCLUDED.permission
         RETURNING *",
    )
    .bind(note_id)
    .bind(&body.shared_with_email)
    .bind(&body.permission)
    .fetch_one(&state.pool)
    .await?;

    tracing::info!(
        note_id = %note_id,
        email   = %body.shared_with_email,
        perm    = %body.permission,
        "Note shared",
    );

    Ok((StatusCode::CREATED, Json(share)))
}

pub async fn unshare(
    claims: Claims,
    State(state): State<AppState>,
    Path((note_id, share_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let row = sqlx::query_as::<_, (String,)>("SELECT owner_id FROM notes WHERE id = $1")
        .bind(note_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    if row.0 != claims.sub {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM note_shares WHERE id = $1 AND note_id = $2")
        .bind(share_id)
        .bind(note_id)
        .execute(&state.pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
