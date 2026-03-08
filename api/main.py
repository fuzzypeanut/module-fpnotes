"""
FuzzyPeanut Notes Module — API backend
Python 3.12+ / FastAPI / asyncpg / PyJWT

Environment variables (see .env.example):
  DATABASE_URL        — asyncpg connection string
  JWKS_URL            — Authentik JWKS endpoint
  PROVISIONING_SECRET — shared secret for /admin/* endpoints
  CORS_ORIGINS        — comma-separated allowed origins (default: *)
"""

import hmac
import logging
import os
from contextlib import asynccontextmanager
from pathlib import Path
from typing import Any, Optional

import asyncpg
import jwt
from fastapi import Depends, FastAPI, Header, HTTPException, status
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, EmailStr

logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")
log = logging.getLogger(__name__)

# ── Config ────────────────────────────────────────────────────────────────────

DATABASE_URL        = os.environ["DATABASE_URL"]
JWKS_URL            = os.environ["JWKS_URL"]
PROVISIONING_SECRET = os.environ.get("PROVISIONING_SECRET", "")
CORS_ORIGINS        = os.environ.get("CORS_ORIGINS", "*").split(",")

# ── JWKS token validation ─────────────────────────────────────────────────────
# PyJWKClient fetches the JWKS endpoint once and caches the keys internally.
# It re-fetches automatically when it encounters an unknown key ID (kid).

_jwks_client: Optional[jwt.PyJWKClient] = None


def jwks_client() -> jwt.PyJWKClient:
    global _jwks_client
    if _jwks_client is None:
        _jwks_client = jwt.PyJWKClient(JWKS_URL, cache_keys=True)
    return _jwks_client


async def validate_token(authorization: str = Header()) -> dict[str, Any]:
    """
    FastAPI dependency — validates the Bearer token and returns its claims.

    Raises HTTP 401 if the header is missing, malformed, or the token is invalid.
    """
    if not authorization.startswith("Bearer "):
        raise HTTPException(status.HTTP_401_UNAUTHORIZED, "Missing Bearer token")

    token = authorization.removeprefix("Bearer ")

    try:
        signing_key = jwks_client().get_signing_key_from_jwt(token)
        claims = jwt.decode(
            token,
            signing_key.key,
            algorithms=["RS256"],
            options={"verify_aud": False},  # Authentik doesn't set aud by default
        )
        return claims
    except jwt.ExpiredSignatureError:
        raise HTTPException(status.HTTP_401_UNAUTHORIZED, "Token expired")
    except Exception as exc:
        raise HTTPException(status.HTTP_401_UNAUTHORIZED, f"Invalid token: {exc}")


def require_provisioning(x_provisioning_secret: str = Header(alias="X-Provisioning-Secret")) -> None:
    """FastAPI dependency — gates admin endpoints behind the provisioning secret."""
    if not hmac.compare_digest(x_provisioning_secret, PROVISIONING_SECRET):
        raise HTTPException(status.HTTP_403_FORBIDDEN, "Invalid provisioning secret")


# ── Database ──────────────────────────────────────────────────────────────────

_pool: Optional[asyncpg.Pool] = None


async def get_db() -> asyncpg.Pool:  # type: ignore[return]
    if _pool is None:
        raise RuntimeError("Database pool not initialized")
    return _pool


@asynccontextmanager
async def lifespan(app: FastAPI):
    global _pool
    log.info("Connecting to database…")
    _pool = await asyncpg.create_pool(DATABASE_URL, min_size=2, max_size=10)

    # Run migrations — simple approach: read the SQL file and execute it.
    migrations_dir = Path(__file__).parent / "migrations"
    for sql_file in sorted(migrations_dir.glob("*.sql")):
        log.info("Applying migration: %s", sql_file.name)
        async with _pool.acquire() as conn:
            await conn.execute(sql_file.read_text())

    log.info("Database ready.")
    yield

    await _pool.close()
    log.info("Database pool closed.")


# ── App ───────────────────────────────────────────────────────────────────────

app = FastAPI(title="FuzzyPeanut Notes API", lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# ── Pydantic models ───────────────────────────────────────────────────────────

class NoteCreate(BaseModel):
    title: str = ""
    content: str = ""
    color: str = "#ffffff"
    pinned: bool = False


class NoteUpdate(BaseModel):
    title: Optional[str] = None
    content: Optional[str] = None
    color: Optional[str] = None
    pinned: Optional[bool] = None
    archived: Optional[bool] = None


class TodoCreate(BaseModel):
    text: str
    position: int = 0


class TodoUpdate(BaseModel):
    text: Optional[str] = None
    checked: Optional[bool] = None
    position: Optional[int] = None


class ShareCreate(BaseModel):
    shared_with_email: str
    permission: str = "view"  # 'view' | 'edit'


class ProvisionUser(BaseModel):
    user_id: str
    email: str


# ── Helpers ───────────────────────────────────────────────────────────────────

async def fetch_note_full(conn: asyncpg.Connection, note_id: str, user_id: str) -> dict:  # type: ignore[type-arg]
    """
    Fetches a note, its todos, and its shares.
    Also determines the calling user's permission level.
    Returns None if the note doesn't exist or the user has no access.
    """
    note = await conn.fetchrow("SELECT * FROM notes WHERE id = $1", note_id)
    if note is None:
        return None  # type: ignore[return-value]

    # Determine permission.
    if note["owner_id"] == user_id:
        permission = "owner"
    else:
        share = await conn.fetchrow(
            "SELECT permission FROM note_shares WHERE note_id = $1 AND shared_with_id = $2",
            note_id, user_id,
        )
        if share is None:
            return None  # No access.
        permission = share["permission"]

    todos = await conn.fetch(
        "SELECT * FROM todos WHERE note_id = $1 ORDER BY position, id", note_id
    )
    shares = await conn.fetch("SELECT * FROM note_shares WHERE note_id = $1", note_id)

    return {
        **dict(note),
        "id": str(note["id"]),
        "todos": [dict(t) | {"id": str(t["id"]), "note_id": str(t["note_id"])} for t in todos],
        "shares": [
            dict(s) | {"id": str(s["id"]), "note_id": str(s["note_id"])}
            for s in shares
        ],
        "permission": permission,
    }


# ── Health ────────────────────────────────────────────────────────────────────

@app.get("/health")
async def health():
    return {"status": "ok"}


# ── Notes ─────────────────────────────────────────────────────────────────────

@app.get("/notes")
async def list_notes(
    claims: dict = Depends(validate_token),
    pool: asyncpg.Pool = Depends(get_db),
):
    user_id = claims["sub"]
    async with pool.acquire() as conn:
        # Fetch notes the user owns plus notes shared with them.
        rows = await conn.fetch(
            """
            SELECT DISTINCT n.id FROM notes n
            LEFT JOIN note_shares s ON s.note_id = n.id
            WHERE (n.owner_id = $1 OR s.shared_with_id = $1)
              AND n.archived = false
            ORDER BY n.id
            """,
            user_id,
        )
        result = []
        for row in rows:
            note = await fetch_note_full(conn, str(row["id"]), user_id)
            if note:
                result.append(note)
    return result


@app.post("/notes", status_code=201)
async def create_note(
    body: NoteCreate,
    claims: dict = Depends(validate_token),
    pool: asyncpg.Pool = Depends(get_db),
):
    user_id = claims["sub"]
    async with pool.acquire() as conn:
        row = await conn.fetchrow(
            """
            INSERT INTO notes (owner_id, title, content, color, pinned)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            """,
            user_id, body.title, body.content, body.color, body.pinned,
        )
        return {**dict(row), "id": str(row["id"]), "todos": [], "shares": [], "permission": "owner"}


@app.get("/notes/{note_id}")
async def get_note(
    note_id: str,
    claims: dict = Depends(validate_token),
    pool: asyncpg.Pool = Depends(get_db),
):
    user_id = claims["sub"]
    async with pool.acquire() as conn:
        note = await fetch_note_full(conn, note_id, user_id)
    if note is None:
        raise HTTPException(404, "Note not found")
    return note


@app.put("/notes/{note_id}")
async def update_note(
    note_id: str,
    body: NoteUpdate,
    claims: dict = Depends(validate_token),
    pool: asyncpg.Pool = Depends(get_db),
):
    user_id = claims["sub"]
    async with pool.acquire() as conn:
        note = await fetch_note_full(conn, note_id, user_id)
        if note is None:
            raise HTTPException(404, "Note not found")
        if note["permission"] == "view":
            raise HTTPException(403, "Read-only access")

        # Build a partial update — only update fields that were provided.
        updates: dict[str, Any] = {}
        if body.title    is not None: updates["title"]    = body.title
        if body.content  is not None: updates["content"]  = body.content
        if body.color    is not None: updates["color"]    = body.color
        if body.pinned   is not None: updates["pinned"]   = body.pinned
        if body.archived is not None: updates["archived"] = body.archived

        if updates:
            set_clause = ", ".join(f"{k} = ${i+2}" for i, k in enumerate(updates))
            await conn.execute(
                f"UPDATE notes SET {set_clause}, updated_at = now() WHERE id = $1",
                note_id, *updates.values(),
            )

        return await fetch_note_full(conn, note_id, user_id)


@app.delete("/notes/{note_id}", status_code=204)
async def delete_note(
    note_id: str,
    claims: dict = Depends(validate_token),
    pool: asyncpg.Pool = Depends(get_db),
):
    user_id = claims["sub"]
    async with pool.acquire() as conn:
        note = await conn.fetchrow("SELECT owner_id FROM notes WHERE id = $1", note_id)
        if note is None or note["owner_id"] != user_id:
            raise HTTPException(403, "Only the owner can delete a note")
        await conn.execute("DELETE FROM notes WHERE id = $1", note_id)


# ── Todos ─────────────────────────────────────────────────────────────────────

@app.post("/notes/{note_id}/todos", status_code=201)
async def create_todo(
    note_id: str,
    body: TodoCreate,
    claims: dict = Depends(validate_token),
    pool: asyncpg.Pool = Depends(get_db),
):
    user_id = claims["sub"]
    async with pool.acquire() as conn:
        note = await fetch_note_full(conn, note_id, user_id)
        if note is None:
            raise HTTPException(404, "Note not found")
        if note["permission"] == "view":
            raise HTTPException(403, "Read-only access")

        row = await conn.fetchrow(
            "INSERT INTO todos (note_id, text, position) VALUES ($1, $2, $3) RETURNING *",
            note_id, body.text, body.position,
        )
        return {**dict(row), "id": str(row["id"]), "note_id": str(row["note_id"])}


@app.put("/notes/{note_id}/todos/{todo_id}")
async def update_todo(
    note_id: str,
    todo_id: str,
    body: TodoUpdate,
    claims: dict = Depends(validate_token),
    pool: asyncpg.Pool = Depends(get_db),
):
    user_id = claims["sub"]
    async with pool.acquire() as conn:
        note = await fetch_note_full(conn, note_id, user_id)
        if note is None:
            raise HTTPException(404, "Note not found")
        if note["permission"] == "view":
            raise HTTPException(403, "Read-only access")

        updates: dict[str, Any] = {}
        if body.text     is not None: updates["text"]     = body.text
        if body.checked  is not None: updates["checked"]  = body.checked
        if body.position is not None: updates["position"] = body.position

        if updates:
            set_clause = ", ".join(f"{k} = ${i+2}" for i, k in enumerate(updates))
            await conn.execute(
                f"UPDATE todos SET {set_clause} WHERE id = $1 AND note_id = $3",
                todo_id, *updates.values(), note_id,
            )

        row = await conn.fetchrow("SELECT * FROM todos WHERE id = $1", todo_id)
        return {**dict(row), "id": str(row["id"]), "note_id": str(row["note_id"])}


@app.delete("/notes/{note_id}/todos/{todo_id}", status_code=204)
async def delete_todo(
    note_id: str,
    todo_id: str,
    claims: dict = Depends(validate_token),
    pool: asyncpg.Pool = Depends(get_db),
):
    user_id = claims["sub"]
    async with pool.acquire() as conn:
        note = await fetch_note_full(conn, note_id, user_id)
        if note is None:
            raise HTTPException(404, "Note not found")
        if note["permission"] == "view":
            raise HTTPException(403, "Read-only access")
        await conn.execute(
            "DELETE FROM todos WHERE id = $1 AND note_id = $2", todo_id, note_id
        )


# ── Sharing ───────────────────────────────────────────────────────────────────

@app.post("/notes/{note_id}/share", status_code=201)
async def share_note(
    note_id: str,
    body: ShareCreate,
    claims: dict = Depends(validate_token),
    pool: asyncpg.Pool = Depends(get_db),
):
    user_id = claims["sub"]
    async with pool.acquire() as conn:
        note = await conn.fetchrow("SELECT owner_id FROM notes WHERE id = $1", note_id)
        if note is None or note["owner_id"] != user_id:
            raise HTTPException(403, "Only the owner can share a note")

        if body.permission not in ("view", "edit"):
            raise HTTPException(400, "permission must be 'view' or 'edit'")

        try:
            row = await conn.fetchrow(
                """
                INSERT INTO note_shares (note_id, shared_with_id, shared_with_email, permission)
                VALUES ($1,
                  (SELECT uid FROM users WHERE email = $2),
                  $2, $3)
                ON CONFLICT (note_id, shared_with_id) DO UPDATE SET permission = EXCLUDED.permission
                RETURNING *
                """,
                note_id, body.shared_with_email, body.permission,
            )
        except asyncpg.PostgresError as exc:
            log.warning("Share failed: %s", exc)
            raise HTTPException(400, f"Could not share: {exc}")

        log.info("Note %s shared with %s (%s)", note_id, body.shared_with_email, body.permission)
        return {**dict(row), "id": str(row["id"]), "note_id": str(row["note_id"])}


@app.delete("/notes/{note_id}/share/{shared_with_id}", status_code=204)
async def unshare_note(
    note_id: str,
    shared_with_id: str,
    claims: dict = Depends(validate_token),
    pool: asyncpg.Pool = Depends(get_db),
):
    user_id = claims["sub"]
    async with pool.acquire() as conn:
        note = await conn.fetchrow("SELECT owner_id FROM notes WHERE id = $1", note_id)
        if note is None or note["owner_id"] != user_id:
            raise HTTPException(403, "Only the owner can remove shares")
        await conn.execute(
            "DELETE FROM note_shares WHERE note_id = $1 AND shared_with_id = $2",
            note_id, shared_with_id,
        )


# ── Admin: Provisioning ────────────────────────────────────────────────────────
# These endpoints are called by the FuzzyPeanut provisioning service (in auth/).
# They MUST be internal-only — do not expose them through nginx.

@app.post("/admin/provision-user", status_code=201)
async def provision_user(
    body: ProvisionUser,
    _: None = Depends(require_provisioning),
    pool: asyncpg.Pool = Depends(get_db),
):
    """
    Create a user record if needed. For Notes, there's nothing to provision
    up-front — notes are created on demand. We just log the user's arrival.
    Returns 201 on first call, 409 if already provisioned (both are success).
    """
    log.info("Provision user: %s (%s)", body.user_id, body.email)
    # Notes are lazily created; no per-user setup required.
    # Return 409 if called again to satisfy the idempotency contract.
    return {"status": "ok", "user_id": body.user_id}


@app.delete("/admin/provision-user/{user_id}", status_code=204)
async def deprovision_user(
    user_id: str,
    _: None = Depends(require_provisioning),
    pool: asyncpg.Pool = Depends(get_db),
):
    """Delete all notes, todos, and shares owned by this user."""
    async with pool.acquire() as conn:
        result = await conn.execute("DELETE FROM notes WHERE owner_id = $1", user_id)
        count = int(result.split()[-1])
        log.info("Deprovisioned user %s — deleted %d notes", user_id, count)
        # Also remove any shares they were a recipient of.
        await conn.execute("DELETE FROM note_shares WHERE shared_with_id = $1", user_id)
