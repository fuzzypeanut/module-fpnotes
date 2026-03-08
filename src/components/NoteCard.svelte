<script lang="ts">
  import type { Note } from '../lib/types';

  let {
    note,
    on_open,
    on_archive,
    on_delete,
  }: {
    note: Note;
    on_open: (note: Note) => void;
    on_archive: (id: string) => void;
    on_delete: (id: string) => void;
  } = $props();

  // Dim the card background slightly in dark mode (injected via CSS var).
  // The note.color is always a light hex — we apply it as a tint.
  let cardStyle = $derived(
    note.color !== '#ffffff' ? `background: ${note.color};` : ''
  );

  let preview = $derived(note.content.slice(0, 180));
  let checkedCount = $derived(note.todos.filter(t => t.checked).length);
  let totalCount   = $derived(note.todos.length);

  function stopProp(e: MouseEvent, fn: () => void) {
    e.stopPropagation();
    fn();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="card"
  style={cardStyle}
  onclick={() => on_open(note)}
>
  {#if note.pinned}
    <div class="pin-badge" title="Pinned">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M5 5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v3a2 2 0 0 1-.586 1.414L16 12v7l-4 2-4-2v-7L4.586 9.414A2 2 0 0 1 4 8V5z"/></svg>
    </div>
  {/if}

  {#if note.title}
    <h3 class="card-title">{note.title}</h3>
  {/if}

  {#if note.todos.length > 0}
    <!-- Todo-style note: show first 3 items -->
    <div class="card-todos">
      {#each note.todos.slice(0, 3) as todo}
        <div class="card-todo" class:checked={todo.checked}>
          <span class="todo-box">{todo.checked ? '✓' : ''}</span>
          <span>{todo.text}</span>
        </div>
      {/each}
      {#if totalCount > 3}
        <div class="card-more">+{totalCount - 3} more</div>
      {/if}
    </div>
    {#if totalCount > 0}
      <div class="todo-progress">
        <div class="todo-bar" style="width: {(checkedCount / totalCount) * 100}%"></div>
      </div>
    {/if}
  {:else if preview}
    <p class="card-content">{preview}{note.content.length > 180 ? '…' : ''}</p>
  {/if}

  {#if note.permission !== 'owner'}
    <div class="shared-badge">Shared</div>
  {/if}

  <!-- Action buttons (shown on hover via CSS) -->
  <div class="card-actions">
    <button
      class="action-btn"
      title="Archive"
      onclick={(e) => stopProp(e, () => on_archive(note.id))}
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="21 8 21 21 3 21 3 8"/><rect x="1" y="3" width="22" height="5"/><line x1="10" y1="12" x2="14" y2="12"/></svg>
    </button>
    {#if note.permission === 'owner'}
      <button
        class="action-btn danger"
        title="Delete"
        onclick={(e) => stopProp(e, () => on_delete(note.id))}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/><path d="M9 6V4h6v2"/></svg>
      </button>
    {/if}
  </div>
</div>

<style>
  .card {
    position: relative;
    background: var(--surface, #fff);
    border: 1px solid var(--border, #e5e2de);
    border-radius: 10px;
    padding: 14px 16px;
    cursor: pointer;
    transition: box-shadow 0.15s, transform 0.1s;
    min-height: 80px;
  }

  .card:hover { box-shadow: 0 2px 12px rgba(0,0,0,0.1); transform: translateY(-1px); }

  .pin-badge {
    position: absolute;
    top: 10px;
    right: 10px;
    color: var(--text-muted, #6b6560);
  }

  .card-title {
    font-size: 0.9rem;
    font-weight: 700;
    margin-bottom: 6px;
    color: var(--text, #1a1714);
  }

  .card-content {
    font-size: 0.82rem;
    color: var(--text-muted, #6b6560);
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .card-todos { display: flex; flex-direction: column; gap: 4px; margin-bottom: 8px; }
  .card-todo { display: flex; align-items: center; gap: 6px; font-size: 0.82rem; color: var(--text-muted, #6b6560); }
  .card-todo.checked { text-decoration: line-through; opacity: 0.5; }
  .todo-box { width: 14px; height: 14px; border: 1.5px solid currentColor; border-radius: 3px; flex-shrink: 0; font-size: 0.65rem; display: flex; align-items: center; justify-content: center; }
  .card-more { font-size: 0.75rem; color: var(--text-faint, #b0a89e); }

  .todo-progress { height: 3px; background: var(--border, #e5e2de); border-radius: 2px; margin-top: 4px; }
  .todo-bar { height: 100%; background: var(--primary, #7c3aed); border-radius: 2px; transition: width 0.3s; }

  .shared-badge {
    display: inline-block;
    margin-top: 8px;
    font-size: 0.7rem;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--primary-light, #ede9fe);
    color: var(--primary-dark, #5b21b6);
  }

  .card-actions {
    position: absolute;
    bottom: 8px;
    right: 8px;
    display: none;
    gap: 4px;
  }

  .card:hover .card-actions { display: flex; }

  .action-btn {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    background: rgba(255,255,255,0.9);
    border: 1px solid var(--border, #e5e2de);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted, #6b6560);
    transition: background 0.1s;
  }

  .action-btn:hover { background: var(--surface-warm, #f5f3f0); }
  .action-btn.danger:hover { background: #fee2e2; color: #dc2626; }
</style>
