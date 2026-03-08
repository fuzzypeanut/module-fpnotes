import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Build as an ES library — the shell loads remoteEntry.js via dynamic import.
// Svelte is bundled in (not externalized) so each module carries its own runtime.
export default defineConfig({
  plugins: [svelte()],
  build: {
    lib: {
      entry: 'src/index.ts',
      name: 'FPNotes',
      fileName: () => 'remoteEntry.js',
      formats: ['es'],
    },
  },
});
