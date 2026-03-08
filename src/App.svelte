<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getSDK } from '@fuzzypeanut/sdk';
  import type { Note } from './lib/types';
  import * as api from './lib/api';
  import NotesList from './components/NotesList.svelte';
  import NoteEditor from './components/NoteEditor.svelte';

  // ── State ──────────────────────────────────────────────────────────────────
  let notes     = $state<Note[]>([]);
  let loading   = $state(true);
  let error     = $state<string | null>(null);
  let activeNote = $state<Note | null>(null);
  let searchQuery = $state('');

  // ── SDK setup ──────────────────────────────────────────────────────────────
  const sdk = getSDK();

  // Subscribe to the notes:shared event so we can refresh when someone shares
  // a note with us (emitted by another user's session via the event bus).
  let unsubscribe: (() => void) | null = null;

  onMount(() => {
    loadNotes();
    unsubscribe = sdk.events.on('notes:shared', () => loadNotes());
  });

  onDestroy(() => {
    unsubscribe?.();
  });

  // Exposed so onActive() in index.ts can call it when the user navigates back.
  export function refresh() { loadNotes(); }

  // ── Data loading ──────────────────────────────────────────────────────────
  async function loadNotes() {
    loading = true;
    error = null;
    try {
      notes = await api.getNotes();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load notes';
    } finally {
      loading = false;
    }
  }

  // ── Derived views ──────────────────────────────────────────────────────────
  let filtered = $derived(
    notes.filter(n =>
      !n.archived &&
      (searchQuery === '' ||
        n.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        n.content.toLowerCase().includes(searchQuery.toLowerCase()))
    )
  );

  let pinned   = $derived(filtered.filter(n => n.pinned));
  let unpinned = $derived(filtered.filter(n => !n.pinned));

  // ── Actions ────────────────────────────────────────────────────────────────
  async function handleCreate() {
    const note = await api.createNote({ title: '', content: '', color: '#ffffff', pinned: false });
    notes = [note, ...notes];
    activeNote = note;
  }

  async function handleSave(updated: Note) {
    notes = notes.map(n => n.id === updated.id ? updated : n);
    activeNote = null;
  }

  async function handleDelete(id: string) {
    await api.deleteNote(id);
    notes = notes.filter(n => n.id !== id);
    if (activeNote?.id === id) activeNote = null;
  }

  async function handleArchive(id: string) {
    const updated = await api.updateNote(id, { archived: true });
    notes = notes.map(n => n.id === id ? updated : n);
    if (activeNote?.id === id) activeNote = null;
  }
</script>

<div class="notes-app">
  <header class="notes-header">
    <div class="header-left">
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14,2 14,8 20,8"/>
      </svg>
      <span>Notes</span>
    </div>
    <div class="search-wrap">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
      </svg>
      <input
        class="search-input"
        type="search"
        placeholder="Search notes…"
        bind:value={searchQuery}
      />
    </div>
  </header>

  <div class="notes-body">
    {#if loading}
      <div class="state-msg">Loading…</div>
    {:else if error}
      <div class="state-msg error">{error}</div>
    {:else}
      <NotesList
        {pinned}
        {unpinned}
        on_open={(note) => { activeNote = note; }}
        on_archive={handleArchive}
        on_delete={handleDelete}
      />
    {/if}
  </div>

  <!-- Floating "new note" button -->
  <button class="fab" onclick={handleCreate} aria-label="New note">
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
      <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
    </svg>
  </button>

  <!-- Note editor modal -->
  {#if activeNote}
    <NoteEditor
      note={activeNote}
      on_save={handleSave}
      on_close={() => { activeNote = null; }}
      on_delete={handleDelete}
    />
  {/if}
</div>

<style>
  .notes-app {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg, #faf9f7);
    font-family: inherit;
    position: relative;
  }

  .notes-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 14px 20px;
    background: var(--surface, #fff);
    border-bottom: 1px solid var(--border, #e5e2de);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 700;
    font-size: 1rem;
    color: var(--text, #1a1714);
  }

  .search-wrap {
    position: relative;
    flex: 1;
    max-width: 400px;
  }

  .search-icon {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-muted, #6b6560);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 8px 12px 8px 34px;
    border: 1px solid var(--border, #e5e2de);
    border-radius: 8px;
    background: var(--bg, #faf9f7);
    font-size: 0.875rem;
    color: var(--text, #1a1714);
    outline: none;
  }

  .search-input:focus { border-color: var(--primary, #7c3aed); }

  .notes-body {
    flex: 1;
    overflow-y: auto;
    padding: 24px 20px;
  }

  .state-msg {
    text-align: center;
    color: var(--text-muted, #6b6560);
    padding: 60px 0;
    font-size: 0.9rem;
  }

  .state-msg.error { color: #dc2626; }

  .fab {
    position: fixed;
    bottom: 28px;
    right: 28px;
    width: 52px;
    height: 52px;
    border-radius: 50%;
    background: var(--primary, #7c3aed);
    color: #fff;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px rgba(0,0,0,0.2);
    transition: transform 0.15s, background 0.15s;
  }

  .fab:hover { background: var(--primary-dark, #6d28d9); transform: scale(1.05); }
</style>
