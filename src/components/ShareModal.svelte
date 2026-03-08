<script lang="ts">
  import { getSDK } from '@fuzzypeanut/sdk';
  import type { Note, NoteShare } from '../lib/types';
  import * as api from '../lib/api';

  let {
    note,
    on_close,
    on_update,
  }: {
    note: Note;
    on_close: () => void;
    on_update: (note: Note) => void;
  } = $props();

  const sdk = getSDK();

  let shareEmail  = $state('');
  let permission  = $state<'view' | 'edit'>('view');
  let sharing     = $state(false);
  let shareError  = $state<string | null>(null);

  // ── Contact picker integration ─────────────────────────────────────────────
  // If the contacts module is installed, offer a "Pick from contacts" button.
  // Gracefully absent when contacts module isn't running.
  let hasContacts = $derived(sdk.registry.hasModule('module-contacts'));

  let unsubPicked: (() => void) | null = null;

  function pickContact() {
    // Emit the contacts:pick event. The contacts module opens its picker UI
    // and fires contacts:picked back on the bus with the selected contact.
    const returnEvent = 'notes:contact-picked';
    unsubPicked = sdk.events.on(returnEvent, (payload: unknown) => {
      const p = payload as { contacts: Array<{ email?: string }> };
      const email = p.contacts[0]?.email ?? '';
      shareEmail = email;
      unsubPicked?.();
      unsubPicked = null;
    });
    sdk.events.emit('contacts:pick', { returnEvent, multiple: false });
  }

  // ── Share action ───────────────────────────────────────────────────────────
  async function share() {
    const email = shareEmail.trim();
    if (!email) return;
    sharing = true;
    shareError = null;
    try {
      const updated = await api.shareNote(note.id, { shared_with_email: email, permission });
      // Optimistically update the note's shares list.
      const updatedNote = { ...note, shares: [...note.shares, updated] };
      on_update(updatedNote);
      shareEmail = '';

      // Fire the SDK event so the recipient (if they're online) gets a
      // real-time notification via the event bus.
      sdk.events.emit('notes:shared', { noteId: note.id, sharedWithEmail: email });
    } catch (e) {
      shareError = e instanceof Error ? e.message : 'Share failed';
    } finally {
      sharing = false;
    }
  }

  async function unshare(sharedWithId: string) {
    await api.unshareNote(note.id, sharedWithId);
    const updatedNote = { ...note, shares: note.shares.filter(s => s.id !== sharedWithId) };
    on_update(updatedNote);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="overlay" onclick={on_close}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h3>Share note</h3>
      <button class="close-btn" onclick={on_close}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><path d="M18 6L6 18M6 6l12 12"/></svg>
      </button>
    </div>

    <div class="share-form">
      <div class="email-row">
        <input
          class="email-input"
          type="email"
          placeholder="Email address"
          bind:value={shareEmail}
        />
        {#if hasContacts}
          <button class="contacts-btn" onclick={pickContact} title="Pick from contacts">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
          </button>
        {/if}
      </div>

      <div class="perm-row">
        <label>
          <input type="radio" bind:group={permission} value="view" />
          Can view
        </label>
        <label>
          <input type="radio" bind:group={permission} value="edit" />
          Can edit
        </label>
      </div>

      {#if shareError}
        <p class="share-error">{shareError}</p>
      {/if}

      <button class="share-btn" onclick={share} disabled={sharing || !shareEmail.trim()}>
        {sharing ? 'Sharing…' : 'Share'}
      </button>
    </div>

    {#if note.shares.length > 0}
      <div class="share-list">
        <div class="share-list-label">Shared with</div>
        {#each note.shares as share}
          <div class="share-row">
            <span class="share-email">{share.shared_with_email}</span>
            <span class="share-perm">{share.permission}</span>
            <button class="remove-btn" onclick={() => unshare(share.id)}>Remove</button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .modal {
    background: var(--surface, #fff);
    border-radius: 12px;
    width: 100%;
    max-width: 420px;
    padding: 24px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.2);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
  }

  .modal-header h3 { font-size: 1rem; font-weight: 700; }

  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted, #6b6560);
    display: flex;
    align-items: center;
    padding: 4px;
  }

  .share-form { display: flex; flex-direction: column; gap: 12px; }

  .email-row { display: flex; gap: 8px; }
  .email-input {
    flex: 1;
    padding: 9px 12px;
    border: 1px solid var(--border, #e5e2de);
    border-radius: 8px;
    font-size: 0.875rem;
    background: var(--bg, #faf9f7);
    color: var(--text, #1a1714);
    outline: none;
  }
  .email-input:focus { border-color: var(--primary, #7c3aed); }

  .contacts-btn {
    width: 38px;
    height: 38px;
    border: 1px solid var(--border, #e5e2de);
    border-radius: 8px;
    background: var(--bg, #faf9f7);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted, #6b6560);
    flex-shrink: 0;
  }

  .contacts-btn:hover { background: var(--surface-warm, #f5f3f0); }

  .perm-row { display: flex; gap: 16px; }
  .perm-row label { display: flex; align-items: center; gap: 6px; font-size: 0.875rem; cursor: pointer; }

  .share-error { font-size: 0.82rem; color: #dc2626; }

  .share-btn {
    padding: 9px 20px;
    background: var(--primary, #7c3aed);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    align-self: flex-end;
    transition: background 0.15s;
  }

  .share-btn:hover:not(:disabled) { background: var(--primary-dark, #6d28d9); }
  .share-btn:disabled { opacity: 0.5; cursor: default; }

  .share-list { margin-top: 20px; border-top: 1px solid var(--border, #e5e2de); padding-top: 16px; }
  .share-list-label { font-size: 0.72rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.08em; color: var(--text-muted, #6b6560); margin-bottom: 10px; }

  .share-row { display: flex; align-items: center; gap: 10px; padding: 6px 0; }
  .share-email { flex: 1; font-size: 0.875rem; }
  .share-perm { font-size: 0.78rem; color: var(--text-muted, #6b6560); background: var(--bg, #faf9f7); padding: 2px 7px; border-radius: 4px; }
  .remove-btn { background: none; border: none; cursor: pointer; font-size: 0.78rem; color: #dc2626; padding: 2px 6px; border-radius: 4px; }
  .remove-btn:hover { background: #fee2e2; }
</style>
