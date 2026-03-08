/**
 * src/index.ts — Module entry point.
 *
 * This file is the only thing the shell imports. It must export a default
 * object that implements the FPModule interface from @fuzzypeanut/sdk:
 *   { mount, unmount, onActive?, onInactive?, onPropsChanged? }
 *
 * The shell calls mount() when the user navigates to /notes and
 * unmount() when they leave. Everything else is internal to the module.
 */

import type { FPModule } from '@fuzzypeanut/sdk';
import App from './App.svelte';

interface AppInstance {
  unmount(): void;
}

const module: FPModule = {
  mount(target: HTMLElement): AppInstance {
    // mount() returns a Svelte component instance.
    // Svelte 5's createRoot returns an object with an unmount() method.
    const instance = new (App as unknown as new (options: { target: HTMLElement }) => AppInstance)({
      target,
    });
    return instance;
  },

  unmount(instance: unknown): void {
    (instance as AppInstance).unmount();
  },

  onActive(instance: unknown): void {
    // Called when the user navigates back to /notes.
    // Tell the app to refresh its data.
    (instance as { refresh?: () => void }).refresh?.();
  },
};

export default module;
