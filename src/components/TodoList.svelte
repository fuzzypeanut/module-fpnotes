<script lang="ts">
  import type { Todo } from '../lib/types';
  import * as api from '../lib/api';

  let {
    noteId,
    todos,
    readonly = false,
    on_change,
  }: {
    noteId: string;
    todos: Todo[];
    readonly?: boolean;
    on_change: (todos: Todo[]) => void;
  } = $props();

  let newText = $state('');

  async function toggle(todo: Todo) {
    if (readonly) return;
    const updated = await api.updateTodo(noteId, todo.id, { checked: !todo.checked });
    on_change(todos.map(t => t.id === todo.id ? updated : t));
  }

  async function addTodo() {
    const text = newText.trim();
    if (!text) return;
    const created = await api.createTodo(noteId, { text, position: todos.length });
    on_change([...todos, created]);
    newText = '';
  }

  async function deleteTodo(id: string) {
    await api.deleteTodo(noteId, id);
    on_change(todos.filter(t => t.id !== id));
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); addTodo(); }
  }
</script>

<div class="todo-list">
  {#each todos as todo (todo.id)}
    <div class="todo-row" class:checked={todo.checked}>
      <button
        class="checkbox"
        onclick={() => toggle(todo)}
        disabled={readonly}
        aria-label={todo.checked ? 'Uncheck' : 'Check'}
      >
        {#if todo.checked}
          <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
            <path d="M2 6l3 3 5-5" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        {/if}
      </button>
      <span class="todo-text">{todo.text}</span>
      {#if !readonly}
        <button class="delete-todo" onclick={() => deleteTodo(todo.id)} aria-label="Remove item">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><path d="M18 6L6 18M6 6l12 12"/></svg>
        </button>
      {/if}
    </div>
  {/each}

  {#if !readonly}
    <div class="new-todo-row">
      <span class="add-icon">+</span>
      <input
        class="new-todo-input"
        type="text"
        placeholder="List item"
        bind:value={newText}
        onkeydown={onKeydown}
      />
    </div>
  {/if}
</div>

<style>
  .todo-list { display: flex; flex-direction: column; gap: 2px; }

  .todo-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
    border-radius: 4px;
  }

  .todo-row:hover .delete-todo { opacity: 1; }

  .checkbox {
    width: 18px;
    height: 18px;
    border: 2px solid var(--border, #e5e2de);
    border-radius: 4px;
    background: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: border-color 0.15s, background 0.15s;
    color: var(--primary, #7c3aed);
    padding: 0;
  }

  .checked .checkbox { background: var(--primary, #7c3aed); border-color: var(--primary, #7c3aed); color: #fff; }
  .todo-text { flex: 1; font-size: 0.88rem; color: var(--text, #1a1714); }
  .checked .todo-text { text-decoration: line-through; color: var(--text-muted, #6b6560); }

  .delete-todo {
    opacity: 0;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted, #6b6560);
    padding: 2px;
    display: flex;
    align-items: center;
    transition: opacity 0.15s;
    flex-shrink: 0;
  }

  .new-todo-row { display: flex; align-items: center; gap: 8px; padding: 4px 0; }
  .add-icon { width: 18px; text-align: center; color: var(--text-muted, #6b6560); font-size: 1rem; flex-shrink: 0; }
  .new-todo-input {
    flex: 1;
    background: none;
    border: none;
    font-size: 0.88rem;
    color: var(--text, #1a1714);
    outline: none;
  }

  .new-todo-input::placeholder { color: var(--text-muted, #6b6560); }
</style>
