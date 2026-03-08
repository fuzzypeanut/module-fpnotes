use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, Json};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

use crate::{error::AppError, AppState};

fn check_secret(headers: &HeaderMap, expected: &str) -> Result<(), AppError> {
    let provided = headers
        .get("X-Provisioning-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // Constant-time comparison to prevent timing attacks.
    if provided.as_bytes().ct_eq(expected.as_bytes()).into() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

#[derive(Deserialize)]
pub struct ProvisionUser {
    pub user_id: String,
    pub email:   String,
}

#[derive(Serialize)]
pub struct ProvisionResponse {
    pub status:  &'static str,
    pub user_id: String,
}

/// Called by the provisioning service when a new user is created in Authentik.
/// Notes are created on demand, so there's nothing to set up per-user.
/// Returns 201 to satisfy the provisioning contract.
pub async fn provision(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ProvisionUser>,
) -> Result<(StatusCode, Json<ProvisionResponse>), AppError> {
    check_secret(&headers, &state.prov_secret)?;
    tracing::info!(user_id = %body.user_id, email = %body.email, "Provisioned user");
    Ok((
        StatusCode::CREATED,
        Json(ProvisionResponse { status: "ok", user_id: body.user_id }),
    ))
}

/// Called by the provisioning service when a user is deleted in Authentik.
/// Deletes all owned notes (cascades to todos/shares) and removes recipient shares.
pub async fn deprovision(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
) -> Result<StatusCode, AppError> {
    check_secret(&headers, &state.prov_secret)?;

    // ON DELETE CASCADE handles todos and shares owned by deleted notes.
    let r = sqlx::query("DELETE FROM notes WHERE owner_id = $1")
        .bind(&user_id)
        .execute(&state.pool)
        .await?;

    // Also remove shares where this user was a recipient.
    sqlx::query("DELETE FROM note_shares WHERE shared_with_id = $1 OR shared_with_email = (SELECT email FROM note_shares WHERE shared_with_id = $1 LIMIT 1)")
        .bind(&user_id)
        .execute(&state.pool)
        .await
        .ok(); // best-effort

    tracing::info!(user_id = %user_id, deleted = r.rows_affected(), "Deprovisioned user");
    Ok(StatusCode::NO_CONTENT)
}
