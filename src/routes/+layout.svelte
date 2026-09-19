<script lang="ts">
  import './app.css';
  import Titlebar from '$lib/components/Titlebar.svelte';
  import { isDarkMode, videoFilePath } from '$lib/store';
  import { browser } from '$app/environment';
  import { saveProject } from '$lib/project';
  
  $: if (browser) {
    if ($isDarkMode) {
      document.documentElement.classList.add('dark');
      document.documentElement.classList.remove('light');
    } else {
      document.documentElement.classList.remove('dark');
      document.documentElement.classList.add('light');
    }
  }

  $: filename = $videoFilePath ? $videoFilePath.split('\\').pop()?.split('/').pop() || '' : '';

  async function handleKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key === 's') {
      event.preventDefault();
      try {
        await saveProject();
      } catch (e) {
        console.error("Save failed:", e);
      }
    }
    
    // Open devtools on F12 or Ctrl+Shift+I
    if (event.key === 'F12' || (event.ctrlKey && event.shiftKey && event.key.toLowerCase() === 'i')) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('open_devtools');
      } catch (e) {
        console.error("Failed to open devtools:", e);
      }
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="flex flex-col h-screen w-screen bg-transparent overflow-hidden">
  <div class="flex flex-col flex-1 overflow-hidden bg-[var(--bg-deep)] rounded-[var(--radius-xl)] relative ring-1 ring-black/5 dark:ring-white/10">
    <Titlebar {filename} />
    <main class="min-h-0 flex-1 flex flex-col relative bg-[var(--bg-base)]">
      <slot />
    </main>
  </div>
</div>
