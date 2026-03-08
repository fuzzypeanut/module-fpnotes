<script lang="ts">
  import type { Note } from '../lib/types';
  import * as api from '../lib/api';
  import TodoList from './TodoList.svelte';
  import ShareModal from './ShareModal.svelte';

  let {
    note,
    on_save,
    on_close,
    on_delete,
  }: {
    note: Note;
    on_save: (updated: Note) => void;
    on_close: () => void;
    on_delete: (id: string) => void;
  } = $props();

  // ── Editable state ─────────────────────────────────────────────────────────
  let title     = $state(note.title);
  let content   = $state(note.content);
  let color     = $state(note.color);
  let pinned    = $state(note.pinned);
  let todos     = $state([...note.todos]);
  let saving    = $state(false);
  let showShare = $state(false);

  // Readonly if user only has view permission on a shared note.
  let readonly = $derived(note.permission === 'view');

  const NOTE_COLORS = [
    '#ffffff', '#fef9c3', '#d1fae5', '#dbeafe', '#ede9fe',
    '#fce7f3', '#ffedd5', '#f1f5f9',
  ];

  async function save() {
    if (readonly) { on_close(); return; }
    saving = true;
    try {
      const updated = await api.updateNote(note.id, { title, content, color, pinned });
      // Merge in the current todos (they're saved individually by TodoList).
      on_save({ ...updated, todos, shares: note.shares });
    } finally {
      saving = false;
    }
  }

  async function togglePin() {
    pinned = !pinned;
  }

  function handleTodosChange(updated: typeof todos) {
    todos = updated;
  }

  function handleShareUpdate(updated: Note) {
    // Propagate share list changes back to parent without closing the editor.
    note = updated;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="overlay" onclick={save}>
  <div
    class="editor"
    style={color !== '#ffffff' ? `background: ${color};` : ''}
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Title -->
    <input
      class="title-input"
      type="text"
      placeholder="Title"
      bind:value={title}
      disabled={readonly}
    />

    <!-- Content or todos -->
    {#if note.todos.length > 0 || todos.length > 0}
      <TodoList
        noteId={note.id}
        {todos}
        {readonly}
        on_change={handleTodosChange}
      />
    {:else}
      <textarea
        class="content-input"
        placeholder="Take a note…"
        bind:value={content}
        disabled={readonly}
        rows="6"
      ></textarea>
    {/if}

    <!-- Toolbar -->
    <div class="editor-toolbar">
      <!-- Color picker -->
      {#if !readonly}
        <div class="color-picker">
          {#each NOTE_COLORS as c}
            <button
              class="color-swatch"
              class:active={color === c}
              style="background: {c};"
              onclick={() => { color = c; }}
              aria-label="Set color {c}"
            ></button>
          {/each}
        </div>
      {/if}

      <div class="toolbar-actions">
        <!-- Pin -->
        {#if !readonly}
          <button
            class="tool-btn"
            class:active={pinned}
            title={pinned ? 'Unpin' : 'Pin'}
            onclick={togglePin}
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill={pinned ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2"><path d="M5 5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v3a2 2 0 0 1-.586 1.414L16 12v7l-4 2-4-2v-7L4.586 9.414A2 2 0 0 1 4 8V5z"/></svg>
          </button>
        {/if}

        <!-- Share (owner only) -->
        {#if note.permission === 'owner'}
          <button class="tool-btn" title="Share" onclick={() => { showShare = true; }}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>
            {#if note.shares.length > 0}
              <span class="share-count">{note.shares.length}</span>
            {/if}
          </button>
        {/if}

        <!-- Delete (owner only) -->
        {#if note.permission === 'owner'}
          <button class="tool-btn danger" title="Delete" onclick={() => on_delete(note.id)}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/><path d="M9 6V4h6v2"/></svg>
          </button>
        {/if}

        <button class="close-btn" onclick={save} disabled={saving}>
          {saving ? 'Saving…' : 'Close'}
        </button>
      </div>
    </div>
  </div>

  {#if showShare}
    <ShareModal
      {note}
      on_close={() => { showShare = false; }}
      on_update={handleShareUpdate}
    />
  {/if}
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .editor {
    background: var(--surface, #fff);
    border-radius: 12px;
    width: 100%;
    max-width: 540px;
    padding: 20px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.2);
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-height: 85vh;
    overflow-y: auto;
  }

  .title-input {
    font-size: 1rem;
    font-weight: 700;
    border: none;
    background: none;
    outline: none;
    width: 100%;
    color: var(--text, #1a1714);
  }

  .title-input::placeholder { color: var(--text-muted, #6b6560); font-weight: 400; }

  .content-input {
    width: 100%;
    border: none;
    background: none;
    outline: none;
    resize: none;
    font-size: 0.875rem;
    color: var(--text, #1a1714);
    line-height: 1.6;
    font-family: inherit;
  }

  .content-input::placeholder { color: var(--text-muted, #6b6560); }

  .editor-toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    border-top: 1px solid var(--border, #e5e2de);
    padding-top: 12px;
    margin-top: 4px;
  }

  .color-picker { display: flex; gap: 5px; flex-wrap: wrap; flex: 1; }

  .color-swatch {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    transition: border-color 0.1s;
    box-shadow: 0 0 0 1px rgba(0,0,0,0.15);
  }

  .color-swatch.active { border-color: var(--primary, #7c3aed); }

  .toolbar-actions { display: flex; align-items: center; gap: 4px; margin-left: auto; }

  .tool-btn {
    width: 30px;
    height: 30px;
    border: none;
    background: none;
    border-radius: 6px;
    cursor: pointer;
    color: var(--text-muted, #6b6560);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    transition: background 0.15s;
  }

  .tool-btn:hover { background: var(--bg, #faf9f7); }
  .tool-btn.active { color: var(--primary, #7c3aed); }
  .tool-btn.danger:hover { color: #dc2626; background: #fee2e2; }

  .share-count {
    position: absolute;
    top: 2px;
    right: 2px;
    width: 14px;
    height: 14px;
    background: var(--primary, #7c3aed);
    color: #fff;
    font-size: 0.6rem;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
  }

  .close-btn {
    padding: 6px 14px;
    background: var(--primary, #7c3aed);
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .close-btn:hover:not(:disabled) { background: var(--primary-dark, #6d28d9); }
  .close-btn:disabled { opacity: 0.5; cursor: default; }
</style>
