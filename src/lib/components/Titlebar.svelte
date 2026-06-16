<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { isDarkMode, showSettings } from '$lib/store';

  const appWindow = getCurrentWindow();

  export let filename: string = '';

  async function minimize() {
    await appWindow.minimize();
  }

  async function toggleMaximize() {
    await appWindow.toggleMaximize();
  }

  async function close() {
    await appWindow.close();
  }

  function toggleTheme() {
    $isDarkMode = !$isDarkMode;
    if ($isDarkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }

  function openSettings() {
    $showSettings = !$showSettings;
  }
</script>

<div data-tauri-drag-region class="h-[40px] w-full flex items-center justify-between pl-4 select-none shrink-0 titlebar-modern relative">
  <div class="titlebar-inner pointer-events-none" class:hidden={!$isDarkMode}></div>
  <div class="noise-overlay" class:hidden={!$isDarkMode}></div>

  <!-- Left Side (App Name) -->
  <div data-tauri-drag-region class="flex items-center gap-3 z-10 relative">
    <img src="/player.png" alt="App Icon" class="w-5 h-5 object-contain pointer-events-none" />
    <span class="font-semibold text-[14px] tracking-tight pointer-events-none">Universal Video Cutter</span>
  </div>

  <!-- Center (Filename) -->
  <div data-tauri-drag-region class="absolute inset-0 flex items-center justify-center z-10">
    {#if filename}
      <span class="text-[13px] opacity-80 truncate max-w-[300px] pointer-events-none">{filename}</span>
    {/if}
  </div>

  <!-- Right Side (Controls) -->
  <div class="flex items-center justify-end z-10 h-full">
    <!-- Theme Toggle -->
    <button aria-label="Toggle Theme" on:click={toggleTheme} class="titlebar-btn mr-1" title={$isDarkMode ? 'Switch to Light Mode' : 'Switch to Dark Mode'}>
      {#if $isDarkMode}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"></circle><line x1="12" y1="1" x2="12" y2="3"></line><line x1="12" y1="21" x2="12" y2="23"></line><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line><line x1="1" y1="12" x2="3" y2="12"></line><line x1="21" y1="12" x2="23" y2="12"></line><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line></svg>
      {:else}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path></svg>
      {/if}
    </button>

    <!-- Settings -->
    <button aria-label="Settings" on:click={openSettings} class="titlebar-btn mr-3" class:active={$showSettings} title="Settings">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
    </button>

    <!-- Window Controls Separator -->
    <div class="flex items-center h-full border-l border-white/20">
      <button aria-label="Minimize" on:click={minimize} class="wc-btn" title="Minimize">
        <svg width="14" height="1" viewBox="0 0 14 1" fill="currentColor"><rect width="14" height="1"/></svg>
      </button>
      <button aria-label="Maximize" on:click={toggleMaximize} class="wc-btn" title="Maximize">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="1" y="1" width="10" height="10"/></svg>
      </button>
      <button aria-label="Close" on:click={close} class="wc-btn wc-close" title="Close">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M1 1l10 10m0-10L1 11"/></svg>
      </button>
    </div>
  </div>
</div>

<style>
  .titlebar-btn {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--titlebar-text);
    opacity: 0.8;
    border: 1px solid transparent;
    transition: all 120ms ease;
  }
  .titlebar-btn:hover {
    background: var(--titlebar-hover);
    opacity: 1;
  }
  .titlebar-btn:active {
    transform: scale(0.92);
  }
  .titlebar-btn.active {
    background: rgba(255, 255, 255, 0.25);
    opacity: 1;
  }

  .wc-btn {
    width: 46px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--titlebar-text);
    opacity: 0.8;
    transition: all 120ms ease;
  }
  .wc-btn:hover {
    background: var(--titlebar-hover);
    opacity: 1;
  }
  .wc-btn:active {
    background: var(--titlebar-hover);
  }
  .wc-close:hover {
    background: var(--danger) !important;
    color: #fff !important;
  }
</style>
