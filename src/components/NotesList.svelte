<script lang="ts">
  import type { Note } from '../lib/types';
  import NoteCard from './NoteCard.svelte';

  let {
    pinned,
    unpinned,
    on_open,
    on_archive,
    on_delete,
  }: {
    pinned: Note[];
    unpinned: Note[];
    on_open: (note: Note) => void;
    on_archive: (id: string) => void;
    on_delete: (id: string) => void;
  } = $props();
</script>

{#if pinned.length === 0 && unpinned.length === 0}
  <div class="empty">
    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14,2 14,8 20,8"/>
    </svg>
    <p>No notes yet. Hit the + button to start.</p>
  </div>
{:else}
  {#if pinned.length > 0}
    <div class="section-label">Pinned</div>
    <div class="notes-grid">
      {#each pinned as note (note.id)}
        <NoteCard {note} {on_open} {on_archive} {on_delete} />
      {/each}
    </div>
  {/if}

  {#if unpinned.length > 0}
    {#if pinned.length > 0}
      <div class="section-label">Other</div>
    {/if}
    <div class="notes-grid">
      {#each unpinned as note (note.id)}
        <NoteCard {note} {on_open} {on_archive} {on_delete} />
      {/each}
    </div>
  {/if}
{/if}

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 80px 0;
    color: var(--text-muted, #6b6560);
    font-size: 0.9rem;
  }

  .section-label {
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--text-muted, #6b6560);
    margin-bottom: 12px;
    margin-top: 8px;
  }

  .notes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 12px;
    margin-bottom: 28px;
  }
</style>
