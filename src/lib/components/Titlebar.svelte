<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { isDarkMode, showSettings, openVideoRequest, pasteVideoRequest, videoFilePath, closeVideoSession } from '$lib/store';
  import { saveProject } from '$lib/project';
  import { message } from '@tauri-apps/plugin-dialog';
  import {
    House,
    FolderOpen,
    ClipboardText,
    FloppyDisk,
    Sun,
    Moon,
    GearSix,
    X,
    FilmStrip
  } from 'phosphor-svelte';

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
  }

  function openSettings() {
    $showSettings = !$showSettings;
  }

  function triggerOpen() {
    $openVideoRequest = Date.now();
  }

  function triggerPaste() {
    $pasteVideoRequest = Date.now();
  }

  async function handleGoHome() {
    if (!$videoFilePath) return;
    await closeVideoSession();
  }

  async function handleSaveProject() {
    if (!$videoFilePath) {
      await message('Please load a video first before saving a project.', { title: 'No Video Loaded', kind: 'warning' });
      return;
    }
    try {
      const path = await saveProject();
      if (path) {
        await message(`Your project was saved to:\n\n${path}`, { title: 'Project Saved', kind: 'info' });
      }
    } catch (e) {
      await message(`${e}`, { title: 'Save Failed', kind: 'error' });
    }
  }
</script>

<div class="px-2 py-2 w-full shrink-0">
  <div data-tauri-drag-region class="titlebar-menu h-[42px] w-full flex items-center justify-between relative overflow-hidden backdrop-blur-md z-10">
    
    <!-- Left Side (App Name & Icon - Clickable to go home) -->
    <div data-tauri-drag-region class="flex items-center gap-2.5 pl-3.5 z-10 w-[240px] shrink-0">
      <button 
        type="button"
        class="flex items-center gap-2.5 cursor-pointer hover:opacity-85 transition-opacity text-left bg-transparent border-0 p-0 focus:outline-none"
        on:click={handleGoHome}
        title="Universal Video Cutter - Click to go Home"
      >
        <img src="/player.png" alt="App Icon" class="w-[18px] h-[18px] rounded-[4px] shadow-sm select-none pointer-events-none object-contain" />
        <span class="font-brand text-[13.5px] font-normal tracking-wide pointer-events-none text-[var(--text-primary)] truncate">
          Universal Video Cutter
        </span>
      </button>
    </div>

    <!-- Center (Home, Actions & Filename) -->
    <div data-tauri-drag-region class="flex-1 flex items-center justify-center z-10 h-full">
      <div class="flex items-center gap-2">
        
        <!-- Home Button -->
        <button 
          class="titlebar-btn tooltip-host" 
          class:active={!$videoFilePath} 
          on:click={handleGoHome} 
          data-tooltip={!$videoFilePath ? "Home Screen (Active)" : "Home (Close Active Video)"}
          aria-label="Home"
        >
          <House size={18} weight={!$videoFilePath ? 'fill' : 'regular'} />
        </button>

        <div class="h-4 w-px bg-[var(--border-base)] mx-1"></div>

        <!-- Action buttons -->
        <div class="flex items-center gap-1">
          <!-- Open File (Video or Project) -->
          <button class="titlebar-btn tooltip-host" on:click={triggerOpen} data-tooltip="Open Video or Project (Ctrl+O)" aria-label="Open File">
            <FolderOpen size={18} />
          </button>
          
          <!-- Paste Path -->
          <button class="titlebar-btn tooltip-host" on:click={triggerPaste} data-tooltip="Paste Video Path (Ctrl+V)" aria-label="Paste Path">
            <ClipboardText size={18} />
          </button>

          <div class="h-4 w-px bg-[var(--border-base)] mx-1"></div>

          <!-- Save Project As -->
          <button 
            class="titlebar-btn tooltip-host" 
            on:click={handleSaveProject} 
            disabled={!$videoFilePath}
            class:opacity-40={!$videoFilePath}
            class:cursor-not-allowed={!$videoFilePath}
            data-tooltip={$videoFilePath ? "Save Project (Ctrl+S)" : "Save Project (No Video Loaded)"}
            aria-label="Save Project"
          >
            <FloppyDisk size={18} />
          </button>
        </div>

        {#if filename}
          <div class="h-4 w-px bg-[var(--border-base)] mx-1"></div>
          <div class="flex items-center gap-1.5 pl-2.5 pr-1.5 py-0.5 rounded-full bg-[var(--bg-elevated)] border border-[var(--border-base)] max-w-[220px]">
            <FilmStrip size={13} class="text-[var(--accent)] shrink-0" />
            <span class="text-[11px] text-[var(--text-secondary)] truncate font-medium">{filename}</span>
            <button 
              type="button"
              class="w-4 h-4 rounded-full flex items-center justify-center text-[var(--text-muted)] hover:text-white hover:bg-[var(--danger)] transition-all shrink-0 ml-0.5"
              on:click={handleGoHome}
              title="Close video"
              aria-label="Close video"
            >
              <X size={10} weight="bold" />
            </button>
          </div>
        {/if}
      </div>
    </div>

    <!-- Right Side (Controls) -->
    <div class="flex items-center justify-end z-10 h-full w-[240px]">
      
      <!-- App Settings / Theme -->
      <div class="flex items-center gap-1 pr-3">
        <button aria-label="Toggle Theme" on:click={toggleTheme} class="titlebar-btn relative tooltip-host" data-tooltip={$isDarkMode ? 'Switch to Light Mode' : 'Switch to Dark Mode'}>
          <div class="absolute inset-0 flex items-center justify-center transition-all duration-300" class:opacity-0={!$isDarkMode} class:-rotate-90={!$isDarkMode}>
            <Sun size={18} />
          </div>
          <div class="absolute inset-0 flex items-center justify-center transition-all duration-300" class:opacity-0={$isDarkMode} class:rotate-90={$isDarkMode}>
            <Moon size={18} />
          </div>
        </button>
        <button aria-label="Settings" on:click={openSettings} class="titlebar-btn tooltip-host" class:active={$showSettings} data-tooltip="Settings">
          <GearSix size={18} color={$showSettings ? 'var(--accent)' : 'currentColor'} weight={$showSettings ? 'fill' : 'regular'} />
        </button>
      </div>

      <!-- Window Controls (Spectral Chrome) -->
      <div class="flex items-center h-full border-l border-[var(--border-subtle)] px-2 gap-1">
        <button aria-label="Minimize" on:click={minimize} class="titlebar-btn group tooltip-host" data-tooltip="Minimize">
          <div class="wc-minimize group-hover:bg-[var(--text-primary)]"></div>
        </button>
        <button aria-label="Maximize" on:click={toggleMaximize} class="titlebar-btn group tooltip-host" data-tooltip="Maximize">
          <svg class="w-[14px] h-[14px] text-[var(--text-secondary)] group-hover:text-[var(--text-primary)] transition-colors" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M4.5 4.5H13.5V13.5" stroke="currentColor" stroke-width="1.5"/>
            <path d="M11.5 2.5H2.5V11.5" stroke="currentColor" stroke-width="1.5"/>
          </svg>
        </button>
        <button aria-label="Close" on:click={close} class="titlebar-btn wc-close group tooltip-host" data-tooltip="Close">
          <div class="w-full h-full rounded-full flex items-center justify-center group-hover:bg-[var(--danger)] transition-all duration-200">
            <X size={16} class="text-[var(--text-secondary)] group-hover:text-white transition-colors" weight="bold" />
          </div>
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .wc-minimize {
    width: 12px;
    height: 1.5px;
    background-color: var(--text-secondary);
    transition: background-color 0.2s;
  }
</style>
