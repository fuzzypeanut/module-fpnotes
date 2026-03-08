use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{auth::Claims, error::AppError, AppState};

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow, Clone)]
pub struct NoteRow {
    pub id:         Uuid,
    pub owner_id:   String,
    pub title:      String,
    pub content:    String,
    pub color:      String,
    pub pinned:     bool,
    pub archived:   bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow, Clone)]
pub struct TodoRow {
    pub id:       Uuid,
    pub note_id:  Uuid,
    pub text:     String,
    pub checked:  bool,
    pub position: i32,
}

#[derive(Serialize, sqlx::FromRow, Clone)]
pub struct ShareRow {
    pub id:                Uuid,
    pub note_id:           Uuid,
    pub shared_with_email: String,
    pub shared_with_id:    Option<String>,
    pub permission:        String,
}

#[derive(Serialize)]
pub struct NoteResponse {
    #[serde(flatten)]
    pub note:       NoteRow,
    pub todos:      Vec<TodoRow>,
    pub shares:     Vec<ShareRow>,
    pub permission: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Fetch a note with its todos and shares, resolving the calling user's permission.
/// Returns None if the note doesn't exist or the user has no access.
pub async fn fetch_note_full(
    pool:    &PgPool,
    note_id: Uuid,
    user_id: &str,
    email:   &str,
) -> Result<Option<NoteResponse>, AppError> {
    let Some(note) = sqlx::query_as::<_, NoteRow>("SELECT * FROM notes WHERE id = $1")
        .bind(note_id)
        .fetch_optional(pool)
        .await?
    else {
        return Ok(None);
    };

    // Determine permission — update shared_with_id lazily while we're here.
    let permission = if note.owner_id == user_id {
        "owner".to_string()
    } else {
        let row = sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT permission, shared_with_id FROM note_shares
             WHERE note_id = $1 AND shared_with_email = $2",
        )
        .bind(note_id)
        .bind(email)
        .fetch_optional(pool)
        .await?;

        let Some((perm, sid)) = row else {
            return Ok(None); // No access — return 404, not 403 (don't leak existence)
        };

        // If we haven't recorded the uid for this share yet, fill it in now.
        if sid.as_deref() != Some(user_id) {
            let _ = sqlx::query(
                "UPDATE note_shares SET shared_with_id = $1
                 WHERE note_id = $2 AND shared_with_email = $3",
            )
            .bind(user_id)
            .bind(note_id)
            .bind(email)
            .execute(pool)
            .await;
        }

        perm
    };

    let todos = sqlx::query_as::<_, TodoRow>(
        "SELECT * FROM todos WHERE note_id = $1 ORDER BY position, id",
    )
    .bind(note_id)
    .fetch_all(pool)
    .await?;

    // Only the owner sees the full share list.
    let shares = if permission == "owner" {
        sqlx::query_as::<_, ShareRow>("SELECT * FROM note_shares WHERE note_id = $1")
            .bind(note_id)
            .fetch_all(pool)
            .await?
    } else {
        vec![]
    };

    Ok(Some(NoteResponse { note, todos, shares, permission }))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<NoteResponse>>, AppError> {
    let email = claims.email.as_deref().unwrap_or_default();

    let ids = sqlx::query_as::<_, (Uuid,)>(
        "SELECT DISTINCT n.id FROM notes n
         LEFT JOIN note_shares s ON s.note_id = n.id
         WHERE n.archived = false
           AND (n.owner_id = $1 OR s.shared_with_email = $2)
         ORDER BY n.id",
    )
    .bind(&claims.sub)
    .bind(email)
    .fetch_all(&state.pool)
    .await?;

    let mut notes = Vec::with_capacity(ids.len());
    for (id,) in ids {
        if let Some(n) = fetch_note_full(&state.pool, id, &claims.sub, email).await? {
            notes.push(n);
        }
    }

    Ok(Json(notes))
}

#[derive(Deserialize)]
pub struct CreateNote {
    pub title:   String,
    pub content: String,
    pub color:   String,
    pub pinned:  bool,
}

pub async fn create(
    claims: Claims,
    State(state): State<AppState>,
    Json(body): Json<CreateNote>,
) -> Result<(StatusCode, Json<NoteResponse>), AppError> {
    let row = sqlx::query_as::<_, NoteRow>(
        "INSERT INTO notes (owner_id, title, content, color, pinned)
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(&claims.sub)
    .bind(&body.title)
    .bind(&body.content)
    .bind(&body.color)
    .bind(body.pinned)
    .fetch_one(&state.pool)
    .await?;

    let resp = NoteResponse {
        note: row,
        todos: vec![],
        shares: vec![],
        permission: "owner".to_string(),
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn get_one(
    claims: Claims,
    State(state): State<AppState>,
    Path(note_id): Path<Uuid>,
) -> Result<Json<NoteResponse>, AppError> {
    let email = claims.email.as_deref().unwrap_or_default();
    fetch_note_full(&state.pool, note_id, &claims.sub, email)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

#[derive(Deserialize)]
pub struct UpdateNote {
    pub title:    Option<String>,
    pub content:  Option<String>,
    pub color:    Option<String>,
    pub pinned:   Option<bool>,
    pub archived: Option<bool>,
}

pub async fn update(
    claims: Claims,
    State(state): State<AppState>,
    Path(note_id): Path<Uuid>,
    Json(body): Json<UpdateNote>,
) -> Result<Json<NoteResponse>, AppError> {
    let email = claims.email.as_deref().unwrap_or_default();

    let note = fetch_note_full(&state.pool, note_id, &claims.sub, email)
        .await?
        .ok_or(AppError::NotFound)?;

    if note.permission == "view" {
        return Err(AppError::Forbidden);
    }

    // Build a partial SET clause — only update provided fields.
    let mut sets: Vec<String> = vec![];
    let mut i: i32 = 2;

    macro_rules! maybe {
        ($field:expr) => {
            if $field.is_some() {
                sets.push(format!("{} = ${i}", stringify!($field)));
                i += 1;
            }
        };
    }
    maybe!(body.title);
    maybe!(body.content);
    maybe!(body.color);
    maybe!(body.pinned);
    maybe!(body.archived);

    if !sets.is_empty() {
        let sql = format!(
            "UPDATE notes SET {}, updated_at = now() WHERE id = $1",
            sets.join(", ")
        );
        let mut q = sqlx::query(&sql).bind(note_id);
        if let Some(v) = &body.title    { q = q.bind(v); }
        if let Some(v) = &body.content  { q = q.bind(v); }
        if let Some(v) = &body.color    { q = q.bind(v); }
        if let Some(v) = body.pinned    { q = q.bind(v); }
        if let Some(v) = body.archived  { q = q.bind(v); }
        q.execute(&state.pool).await?;
    }

    fetch_note_full(&state.pool, note_id, &claims.sub, email)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn delete(
    claims: Claims,
    State(state): State<AppState>,
    Path(note_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let row = sqlx::query_as::<_, (String,)>("SELECT owner_id FROM notes WHERE id = $1")
        .bind(note_id)
        .fetch_optional(&state.pool)
        .await?;

    match row {
        Some((owner_id,)) if owner_id == claims.sub => {
            sqlx::query("DELETE FROM notes WHERE id = $1")
                .bind(note_id)
                .execute(&state.pool)
                .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        Some(_) => Err(AppError::Forbidden),
        None    => Err(AppError::NotFound),
    }
}
