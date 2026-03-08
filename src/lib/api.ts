/**
 * api.ts — Typed API client for the notes backend.
 *
 * Every method fetches a fresh token from the SDK before each request.
 * Throws an Error on any non-2xx response.
 */

import { getSDK } from '@fuzzypeanut/sdk';
import type {
  Note, Todo, NoteShare,
  CreateNoteInput, UpdateNoteInput,
  CreateTodoInput, UpdateTodoInput,
  ShareNoteInput,
} from './types';

// Injected at build time via Vite. In docker-compose, set VITE_NOTES_API_URL.
const BASE = import.meta.env.VITE_NOTES_API_URL ?? '';

async function headers(): Promise<Record<string, string>> {
  const token = await getSDK().auth.getToken();
  return {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
  };
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: await headers(),
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) {
    const msg = await res.text().catch(() => res.statusText);
    throw new Error(`${method} ${path} → ${res.status}: ${msg}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json() as T;
}

// ─── Notes ────────────────────────────────────────────────────────────────────

export const getNotes    = ()                              => request<Note[]>('GET', '/notes');
export const getNote     = (id: string)                    => request<Note>('GET', `/notes/${id}`);
export const createNote  = (data: CreateNoteInput)         => request<Note>('POST', '/notes', data);
export const updateNote  = (id: string, data: UpdateNoteInput) => request<Note>('PUT', `/notes/${id}`, data);
export const deleteNote  = (id: string)                    => request<void>('DELETE', `/notes/${id}`);

// ─── Todos ────────────────────────────────────────────────────────────────────

export const createTodo  = (noteId: string, data: CreateTodoInput) =>
  request<Todo>('POST', `/notes/${noteId}/todos`, data);
export const updateTodo  = (noteId: string, todoId: string, data: UpdateTodoInput) =>
  request<Todo>('PUT', `/notes/${noteId}/todos/${todoId}`, data);
export const deleteTodo  = (noteId: string, todoId: string) =>
  request<void>('DELETE', `/notes/${noteId}/todos/${todoId}`);

// ─── Sharing ──────────────────────────────────────────────────────────────────

export const shareNote   = (noteId: string, data: ShareNoteInput) =>
  request<NoteShare>('POST', `/notes/${noteId}/share`, data);
export const unshareNote = (noteId: string, shareId: string) =>
  request<void>('DELETE', `/notes/${noteId}/share/${shareId}`);
