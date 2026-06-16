<script lang="ts">
  import './app.css';
  import Titlebar from '$lib/components/Titlebar.svelte';
  import { onMount } from 'svelte';
  
  onMount(() => {
    // Default to dark mode as requested by spec
    document.documentElement.classList.add('dark');
  });
</script>

<div class="flex flex-col h-screen overflow-hidden bg-bg">
  <Titlebar />
  <main class="flex-1 overflow-hidden relative">
    <slot />
  </main>
</div>
