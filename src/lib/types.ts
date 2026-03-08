// ─── Domain types shared between frontend and backend ─────────────────────────

export interface Note {
  id: string;
  title: string;
  content: string;
  color: string;       // CSS hex color, e.g. "#ffffff"
  pinned: boolean;
  archived: boolean;
  owner_id: string;    // Authentik uid of the note owner
  created_at: string;  // ISO 8601
  updated_at: string;
  todos: Todo[];
  shares: NoteShare[];
  /** Your relationship to this note — set by the API per-request. */
  permission: 'owner' | 'edit' | 'view';
}

export interface Todo {
  id: string;
  note_id: string;
  text: string;
  checked: boolean;
  position: number;
}

export interface NoteShare {
  id: string;
  note_id: string;
  shared_with_id: string;
  shared_with_email: string;
  permission: 'view' | 'edit';
}

// ─── API request/response shapes ───────────────────────────────────────────────

export type CreateNoteInput = Pick<Note, 'title' | 'content' | 'color' | 'pinned'>;
export type UpdateNoteInput = Partial<Pick<Note, 'title' | 'content' | 'color' | 'pinned' | 'archived'>>;

export interface CreateTodoInput { text: string; position?: number; }
export interface UpdateTodoInput { text?: string; checked?: boolean; position?: number; }

export interface ShareNoteInput {
  shared_with_email: string;
  permission: 'view' | 'edit';
}
